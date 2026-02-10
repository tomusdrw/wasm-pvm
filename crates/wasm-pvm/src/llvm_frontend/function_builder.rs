use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::BuilderError;
use inkwell::context::Context;
use inkwell::intrinsics::Intrinsic;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, IntType};
use inkwell::values::{
    BasicMetadataValueEnum, FunctionValue, GlobalValue, IntValue, PhiValue, PointerValue,
};
use wasmparser::{FunctionBody, Operator};

use crate::translate::wasm_module::WasmModule;
use crate::{Error, Result};

fn llvm_err<T>(result: std::result::Result<T, BuilderError>) -> Result<T> {
    result.map_err(|e| Error::Internal(format!("LLVM builder error: {e:?}")))
}

/// WASM control flow frame, tracking LLVM basic blocks and phi nodes.
enum ControlFrame<'ctx> {
    Block {
        merge_bb: BasicBlock<'ctx>,
        result_phi: Option<PhiValue<'ctx>>,
        stack_depth: usize,
    },
    Loop {
        header_bb: BasicBlock<'ctx>,
        merge_bb: BasicBlock<'ctx>,
        stack_depth: usize,
    },
    If {
        else_bb: BasicBlock<'ctx>,
        merge_bb: BasicBlock<'ctx>,
        result_phi: Option<PhiValue<'ctx>>,
        stack_depth: usize,
        else_seen: bool,
    },
}

impl<'ctx> ControlFrame<'ctx> {
    fn merge_bb(&self) -> BasicBlock<'ctx> {
        match self {
            Self::Block { merge_bb, .. }
            | Self::Loop { merge_bb, .. }
            | Self::If { merge_bb, .. } => *merge_bb,
        }
    }

    /// The branch target for `br`: `merge_bb` for Block/If, `header_bb` for Loop.
    fn br_target(&self) -> BasicBlock<'ctx> {
        match self {
            Self::Block { merge_bb, .. } | Self::If { merge_bb, .. } => *merge_bb,
            Self::Loop { header_bb, .. } => *header_bb,
        }
    }

    fn result_phi(&self) -> Option<PhiValue<'ctx>> {
        match self {
            Self::Block { result_phi, .. } | Self::If { result_phi, .. } => *result_phi,
            Self::Loop { .. } => None,
        }
    }

    /// Whether a branch to this frame's target carries a result value.
    /// Block/If targets (merge) can have results; Loop targets (header) never do.
    fn br_has_result(&self) -> bool {
        match self {
            Self::Block { result_phi, .. } | Self::If { result_phi, .. } => result_phi.is_some(),
            Self::Loop { .. } => false,
        }
    }
}

pub struct WasmToLlvm<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,
    i1_type: IntType<'ctx>,
    i32_type: IntType<'ctx>,
    i64_type: IntType<'ctx>,

    // Module-level declarations
    functions: Vec<FunctionValue<'ctx>>,
    globals: Vec<GlobalValue<'ctx>>,

    // PVM intrinsic function declarations for memory operations
    pvm_intrinsics: PvmIntrinsics<'ctx>,

    // Type signatures for indirect calls: (num_params, num_results) per type index
    type_signatures: Vec<(usize, usize)>,

    // Per-function state (reset for each function)
    operand_stack: Vec<IntValue<'ctx>>,
    locals: Vec<PointerValue<'ctx>>,
    current_fn: Option<FunctionValue<'ctx>>,
    has_return: bool,
    control_stack: Vec<ControlFrame<'ctx>>,
    /// True when current position is after a terminator (unreachable code).
    unreachable: bool,
}

/// PVM-specific intrinsic function declarations for memory access.
/// These are recognized by name in the PVM backend and lowered to PVM instructions.
struct PvmIntrinsics<'ctx> {
    load_i32: FunctionValue<'ctx>,
    load_i64: FunctionValue<'ctx>,
    load_i8u: FunctionValue<'ctx>,
    load_i8s: FunctionValue<'ctx>,
    load_i16u: FunctionValue<'ctx>,
    load_i16s: FunctionValue<'ctx>,
    load_i32u_64: FunctionValue<'ctx>,
    load_i32s_64: FunctionValue<'ctx>,
    load_i8u_64: FunctionValue<'ctx>,
    load_i8s_64: FunctionValue<'ctx>,
    load_i16u_64: FunctionValue<'ctx>,
    load_i16s_64: FunctionValue<'ctx>,
    store_i32: FunctionValue<'ctx>,
    store_i64: FunctionValue<'ctx>,
    store_i8: FunctionValue<'ctx>,
    store_i16: FunctionValue<'ctx>,
    store_i8_64: FunctionValue<'ctx>,
    store_i16_64: FunctionValue<'ctx>,
    store_i32_64: FunctionValue<'ctx>,
    memory_size: FunctionValue<'ctx>,
    memory_grow: FunctionValue<'ctx>,
    memory_fill: FunctionValue<'ctx>,
    memory_copy: FunctionValue<'ctx>,
    call_indirect: FunctionValue<'ctx>,
}

impl<'ctx> WasmToLlvm<'ctx> {
    #[must_use]
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let pvm_intrinsics = Self::declare_pvm_intrinsics(context, &module);

        Self {
            context,
            module,
            builder,
            i1_type: context.bool_type(),
            i32_type: context.i32_type(),
            i64_type: context.i64_type(),
            pvm_intrinsics,
            type_signatures: Vec::new(),
            functions: Vec::new(),
            globals: Vec::new(),
            operand_stack: Vec::new(),
            locals: Vec::new(),
            current_fn: None,
            has_return: false,
            control_stack: Vec::new(),
            unreachable: false,
        }
    }

    fn declare_pvm_intrinsics(
        context: &'ctx Context,
        module: &Module<'ctx>,
    ) -> PvmIntrinsics<'ctx> {
        let i64_type = context.i64_type();
        let void_type = context.void_type();

        // Helper: (addr: i64) -> i64
        let load_sig = i64_type.fn_type(&[i64_type.into()], false);
        // Helper: (addr: i64, val: i64) -> void
        let store_sig = void_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        // () -> i64
        let nullary_sig = i64_type.fn_type(&[], false);
        // (pages: i64) -> i64
        let unary_sig = i64_type.fn_type(&[i64_type.into()], false);
        // (dst: i64, val: i64, len: i64) -> void
        let ternary_void_sig =
            void_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        // call_indirect: (type_idx: i64, table_idx: i64) -> i64 (varargs)
        let call_indirect_sig = i64_type.fn_type(
            &[i64_type.into(), i64_type.into()],
            true, // varargs
        );

        let decl = |name, sig| module.add_function(name, sig, None);

        PvmIntrinsics {
            load_i32: decl("__pvm_load_i32", load_sig),
            load_i64: decl("__pvm_load_i64", load_sig),
            load_i8u: decl("__pvm_load_i8u", load_sig),
            load_i8s: decl("__pvm_load_i8s", load_sig),
            load_i16u: decl("__pvm_load_i16u", load_sig),
            load_i16s: decl("__pvm_load_i16s", load_sig),
            load_i32u_64: decl("__pvm_load_i32u_64", load_sig),
            load_i32s_64: decl("__pvm_load_i32s_64", load_sig),
            load_i8u_64: decl("__pvm_load_i8u_64", load_sig),
            load_i8s_64: decl("__pvm_load_i8s_64", load_sig),
            load_i16u_64: decl("__pvm_load_i16u_64", load_sig),
            load_i16s_64: decl("__pvm_load_i16s_64", load_sig),
            store_i32: decl("__pvm_store_i32", store_sig),
            store_i64: decl("__pvm_store_i64", store_sig),
            store_i8: decl("__pvm_store_i8", store_sig),
            store_i16: decl("__pvm_store_i16", store_sig),
            store_i8_64: decl("__pvm_store_i8_64", store_sig),
            store_i16_64: decl("__pvm_store_i16_64", store_sig),
            store_i32_64: decl("__pvm_store_i32_64", store_sig),
            memory_size: decl("__pvm_memory_size", nullary_sig),
            memory_grow: decl("__pvm_memory_grow", unary_sig),
            memory_fill: decl("__pvm_memory_fill", ternary_void_sig),
            memory_copy: decl("__pvm_memory_copy", ternary_void_sig),
            call_indirect: decl("__pvm_call_indirect", call_indirect_sig),
        }
    }

    pub fn translate_module(mut self, wasm_module: &WasmModule) -> Result<Module<'ctx>> {
        self.declare_functions(wasm_module);
        self.declare_globals(wasm_module);
        self.type_signatures
            .clone_from(&wasm_module.type_signatures);

        for (local_idx, func_body) in wasm_module.functions.iter().enumerate() {
            let global_idx = wasm_module.num_imported_funcs as usize + local_idx;
            let func_value = self.functions[global_idx];
            let (num_params, has_return) = wasm_module.function_signatures[global_idx];
            self.translate_function(func_body, func_value, num_params, has_return)?;
        }

        self.run_mem2reg()?;

        self.module
            .verify()
            .map_err(|e| Error::Internal(format!("LLVM verify failed: {e}")))?;

        Ok(self.module)
    }

    fn declare_functions(&mut self, wasm_module: &WasmModule) {
        self.functions.clear();
        for &(num_params, has_return) in &wasm_module.function_signatures {
            let param_types: Vec<BasicMetadataTypeEnum> =
                (0..num_params).map(|_| self.i64_type.into()).collect();
            let fn_type = if has_return {
                self.i64_type.fn_type(&param_types, false)
            } else {
                self.context.void_type().fn_type(&param_types, false)
            };
            let idx = self.functions.len();
            let fn_val = self
                .module
                .add_function(&format!("wasm_func_{idx}"), fn_type, None);
            self.functions.push(fn_val);
        }
    }

    fn declare_globals(&mut self, wasm_module: &WasmModule) {
        self.globals.clear();
        for (idx, &init_value) in wasm_module.global_init_values.iter().enumerate() {
            let global = self
                .module
                .add_global(self.i64_type, None, &format!("wasm_global_{idx}"));
            global.set_initializer(&self.i64_type.const_int(init_value as u64, true));
            self.globals.push(global);
        }
    }

    fn translate_function(
        &mut self,
        func_body: &FunctionBody,
        func_value: FunctionValue<'ctx>,
        num_params: usize,
        has_return: bool,
    ) -> Result<()> {
        self.operand_stack.clear();
        self.locals.clear();
        self.control_stack.clear();
        self.current_fn = Some(func_value);
        self.has_return = has_return;
        self.unreachable = false;

        let entry_bb = self.context.append_basic_block(func_value, "entry");
        self.builder.position_at_end(entry_bb);

        // Count declared locals
        let locals_reader = func_body.get_locals_reader()?;
        let mut declared_locals = 0usize;
        for local in locals_reader {
            let (count, _ty) = local?;
            declared_locals += count as usize;
        }
        let total_locals = num_params + declared_locals;

        // Create alloca slots for all locals
        for i in 0..total_locals {
            let alloca = llvm_err(
                self.builder
                    .build_alloca(self.i64_type, &format!("local_{i}")),
            )?;
            self.locals.push(alloca);
        }

        // Store params into alloca slots
        for i in 0..num_params {
            let param = func_value
                .get_nth_param(i as u32)
                .ok_or_else(|| Error::Internal(format!("missing param {i}")))?
                .into_int_value();
            llvm_err(self.builder.build_store(self.locals[i], param))?;
        }

        // Zero-init declared locals
        let zero = self.i64_type.const_zero();
        for i in num_params..total_locals {
            llvm_err(self.builder.build_store(self.locals[i], zero))?;
        }

        // Push implicit function block frame.
        // The function body's final End pops this frame and emits the return.
        let fn_val = self.current_fn.unwrap();
        let merge_bb = self.context.append_basic_block(fn_val, "fn_return");
        let result_phi = if has_return {
            self.builder.position_at_end(merge_bb);
            let phi = llvm_err(self.builder.build_phi(self.i64_type, "fn_result"))?;
            self.builder.position_at_end(entry_bb);
            Some(phi)
        } else {
            None
        };
        self.control_stack.push(ControlFrame::Block {
            merge_bb,
            result_phi,
            stack_depth: 0,
        });

        // Translate operators
        let ops: Vec<Operator> = func_body
            .get_operators_reader()?
            .into_iter()
            .collect::<std::result::Result<_, _>>()?;

        for op in &ops {
            self.translate_operator(op)?;
        }

        // Verify function ends cleanly (the final End should have popped the fn frame)
        debug_assert!(
            self.control_stack.is_empty(),
            "control stack not empty after function translation"
        );

        Ok(())
    }

    fn translate_operator(&mut self, op: &Operator) -> Result<()> {
        // In unreachable code, only track control flow structure for End matching.
        if self.unreachable {
            match op {
                Operator::Block { .. } | Operator::Loop { .. } | Operator::If { .. } => {
                    // Push a dummy frame so End can pop it
                    let dummy_bb = self.builder.get_insert_block().unwrap();
                    self.control_stack.push(ControlFrame::Block {
                        merge_bb: dummy_bb,
                        result_phi: None,
                        stack_depth: self.operand_stack.len(),
                    });
                    return Ok(());
                }
                Operator::Else | Operator::End => {
                    // Else/End in dead code: handled by the arms below which check unreachable
                }
                _ => return Ok(()), // skip all other ops in dead code
            }
        }

        match op {
            // === Constants ===
            Operator::I32Const { value } => {
                self.push(self.i64_type.const_int(u64::from(*value as u32), false));
                Ok(())
            }
            Operator::I64Const { value } => {
                self.push(self.i64_type.const_int(*value as u64, false));
                Ok(())
            }

            // === Locals ===
            Operator::LocalGet { local_index } => {
                let idx = *local_index as usize;
                let val = llvm_err(self.builder.build_load(
                    self.i64_type,
                    self.locals[idx],
                    "local_get",
                ))?;
                self.push(val.into_int_value());
                Ok(())
            }
            Operator::LocalSet { local_index } => {
                let val = self.pop()?;
                llvm_err(
                    self.builder
                        .build_store(self.locals[*local_index as usize], val),
                )?;
                Ok(())
            }
            Operator::LocalTee { local_index } => {
                let val = self.peek()?;
                llvm_err(
                    self.builder
                        .build_store(self.locals[*local_index as usize], val),
                )?;
                Ok(())
            }

            // === Globals ===
            Operator::GlobalGet { global_index } => {
                let idx = *global_index as usize;
                let ptr = self.globals[idx].as_pointer_value();
                let val = llvm_err(self.builder.build_load(self.i64_type, ptr, "global_get"))?;
                self.push(val.into_int_value());
                Ok(())
            }
            Operator::GlobalSet { global_index } => {
                let val = self.pop()?;
                let ptr = self.globals[*global_index as usize].as_pointer_value();
                llvm_err(self.builder.build_store(ptr, val))?;
                Ok(())
            }

            // === i32 Arithmetic ===
            Operator::I32Add => self.i32_binop(|b, l, r| llvm_err(b.build_int_add(l, r, "i32add"))),
            Operator::I32Sub => self.i32_binop(|b, l, r| llvm_err(b.build_int_sub(l, r, "i32sub"))),
            Operator::I32Mul => self.i32_binop(|b, l, r| llvm_err(b.build_int_mul(l, r, "i32mul"))),
            Operator::I32DivU => {
                self.i32_binop(|b, l, r| llvm_err(b.build_int_unsigned_div(l, r, "i32divu")))
            }
            Operator::I32DivS => {
                self.i32_binop(|b, l, r| llvm_err(b.build_int_signed_div(l, r, "i32divs")))
            }
            Operator::I32RemU => {
                self.i32_binop(|b, l, r| llvm_err(b.build_int_unsigned_rem(l, r, "i32remu")))
            }
            Operator::I32RemS => {
                self.i32_binop(|b, l, r| llvm_err(b.build_int_signed_rem(l, r, "i32rems")))
            }

            // === i64 Arithmetic ===
            Operator::I64Add => self.i64_binop(|b, l, r| llvm_err(b.build_int_add(l, r, "i64add"))),
            Operator::I64Sub => self.i64_binop(|b, l, r| llvm_err(b.build_int_sub(l, r, "i64sub"))),
            Operator::I64Mul => self.i64_binop(|b, l, r| llvm_err(b.build_int_mul(l, r, "i64mul"))),
            Operator::I64DivU => {
                self.i64_binop(|b, l, r| llvm_err(b.build_int_unsigned_div(l, r, "i64divu")))
            }
            Operator::I64DivS => {
                self.i64_binop(|b, l, r| llvm_err(b.build_int_signed_div(l, r, "i64divs")))
            }
            Operator::I64RemU => {
                self.i64_binop(|b, l, r| llvm_err(b.build_int_unsigned_rem(l, r, "i64remu")))
            }
            Operator::I64RemS => {
                self.i64_binop(|b, l, r| llvm_err(b.build_int_signed_rem(l, r, "i64rems")))
            }

            // === i32 Bitwise ===
            Operator::I32And => self.i32_binop(|b, l, r| llvm_err(b.build_and(l, r, "i32and"))),
            Operator::I32Or => self.i32_binop(|b, l, r| llvm_err(b.build_or(l, r, "i32or"))),
            Operator::I32Xor => self.i32_binop(|b, l, r| llvm_err(b.build_xor(l, r, "i32xor"))),

            // === i64 Bitwise ===
            Operator::I64And => self.i64_binop(|b, l, r| llvm_err(b.build_and(l, r, "i64and"))),
            Operator::I64Or => self.i64_binop(|b, l, r| llvm_err(b.build_or(l, r, "i64or"))),
            Operator::I64Xor => self.i64_binop(|b, l, r| llvm_err(b.build_xor(l, r, "i64xor"))),

            // === i32 Shifts ===
            Operator::I32Shl => {
                self.i32_binop(|b, l, r| llvm_err(b.build_left_shift(l, r, "i32shl")))
            }
            Operator::I32ShrU => {
                self.i32_binop(|b, l, r| llvm_err(b.build_right_shift(l, r, false, "i32shru")))
            }
            Operator::I32ShrS => {
                self.i32_binop(|b, l, r| llvm_err(b.build_right_shift(l, r, true, "i32shrs")))
            }

            // === i64 Shifts ===
            Operator::I64Shl => {
                self.i64_binop(|b, l, r| llvm_err(b.build_left_shift(l, r, "i64shl")))
            }
            Operator::I64ShrU => {
                self.i64_binop(|b, l, r| llvm_err(b.build_right_shift(l, r, false, "i64shru")))
            }
            Operator::I64ShrS => {
                self.i64_binop(|b, l, r| llvm_err(b.build_right_shift(l, r, true, "i64shrs")))
            }

            // === i32 Rotations ===
            Operator::I32Rotl => self.i32_rotate("llvm.fshl"),
            Operator::I32Rotr => self.i32_rotate("llvm.fshr"),

            // === i64 Rotations ===
            Operator::I64Rotl => self.i64_rotate("llvm.fshl"),
            Operator::I64Rotr => self.i64_rotate("llvm.fshr"),

            // === i32 Bit counting ===
            Operator::I32Clz => self.i32_count_bits("llvm.ctlz", true),
            Operator::I32Ctz => self.i32_count_bits("llvm.cttz", true),
            Operator::I32Popcnt => self.i32_count_bits("llvm.ctpop", false),

            // === i64 Bit counting ===
            Operator::I64Clz => self.i64_count_bits("llvm.ctlz", true),
            Operator::I64Ctz => self.i64_count_bits("llvm.cttz", true),
            Operator::I64Popcnt => self.i64_count_bits("llvm.ctpop", false),

            // === i32 Comparisons ===
            Operator::I32Eqz => self.i32_eqz(),
            Operator::I32Eq => self.i32_cmp(IntPredicate::EQ),
            Operator::I32Ne => self.i32_cmp(IntPredicate::NE),
            Operator::I32LtU => self.i32_cmp(IntPredicate::ULT),
            Operator::I32LtS => self.i32_cmp(IntPredicate::SLT),
            Operator::I32GtU => self.i32_cmp(IntPredicate::UGT),
            Operator::I32GtS => self.i32_cmp(IntPredicate::SGT),
            Operator::I32LeU => self.i32_cmp(IntPredicate::ULE),
            Operator::I32LeS => self.i32_cmp(IntPredicate::SLE),
            Operator::I32GeU => self.i32_cmp(IntPredicate::UGE),
            Operator::I32GeS => self.i32_cmp(IntPredicate::SGE),

            // === i64 Comparisons ===
            Operator::I64Eqz => self.i64_eqz(),
            Operator::I64Eq => self.i64_cmp(IntPredicate::EQ),
            Operator::I64Ne => self.i64_cmp(IntPredicate::NE),
            Operator::I64LtU => self.i64_cmp(IntPredicate::ULT),
            Operator::I64LtS => self.i64_cmp(IntPredicate::SLT),
            Operator::I64GtU => self.i64_cmp(IntPredicate::UGT),
            Operator::I64GtS => self.i64_cmp(IntPredicate::SGT),
            Operator::I64LeU => self.i64_cmp(IntPredicate::ULE),
            Operator::I64LeS => self.i64_cmp(IntPredicate::SLE),
            Operator::I64GeU => self.i64_cmp(IntPredicate::UGE),
            Operator::I64GeS => self.i64_cmp(IntPredicate::SGE),

            // === Conversions ===
            Operator::I32WrapI64 => {
                let val = self.pop()?;
                let trunc = llvm_err(self.builder.build_int_truncate(val, self.i32_type, "wrap"))?;
                let ext = llvm_err(self.builder.build_int_z_extend(
                    trunc,
                    self.i64_type,
                    "wrap_ext",
                ))?;
                self.push(ext);
                Ok(())
            }
            Operator::I64ExtendI32S => {
                let val = self.pop()?;
                let trunc = llvm_err(self.builder.build_int_truncate(
                    val,
                    self.i32_type,
                    "ext_trunc",
                ))?;
                let ext = llvm_err(self.builder.build_int_s_extend(
                    trunc,
                    self.i64_type,
                    "i64extends",
                ))?;
                self.push(ext);
                Ok(())
            }
            Operator::I64ExtendI32U => {
                let val = self.pop()?;
                let trunc = llvm_err(self.builder.build_int_truncate(
                    val,
                    self.i32_type,
                    "ext_trunc",
                ))?;
                let ext = llvm_err(self.builder.build_int_z_extend(
                    trunc,
                    self.i64_type,
                    "i64extendu",
                ))?;
                self.push(ext);
                Ok(())
            }
            Operator::I32Extend8S => self.sign_extend_in_i32(8),
            Operator::I32Extend16S => self.sign_extend_in_i32(16),
            Operator::I64Extend8S => self.sign_extend_in_i64(8),
            Operator::I64Extend16S => self.sign_extend_in_i64(16),
            Operator::I64Extend32S => self.sign_extend_in_i64(32),

            // === Stack manipulation ===
            Operator::Drop => {
                self.pop()?;
                Ok(())
            }
            Operator::Select => {
                let cond = self.pop()?;
                let val2 = self.pop()?;
                let val1 = self.pop()?;
                let cond32 = llvm_err(self.builder.build_int_truncate(
                    cond,
                    self.i32_type,
                    "sel_cond",
                ))?;
                let cond_bool = llvm_err(self.builder.build_int_compare(
                    IntPredicate::NE,
                    cond32,
                    self.i32_type.const_zero(),
                    "sel_test",
                ))?;
                let result = llvm_err(self.builder.build_select(cond_bool, val1, val2, "select"))?;
                self.push(result.into_int_value());
                Ok(())
            }

            // === Control flow ===
            Operator::Nop => Ok(()),

            Operator::Unreachable => {
                if !self.unreachable {
                    llvm_err(self.builder.build_unreachable())?;
                    self.unreachable = true;
                }
                Ok(())
            }

            Operator::Return => {
                if !self.unreachable {
                    // Extract frame data before mutably borrowing self
                    let fn_phi = self.control_stack[0].result_phi();
                    let fn_merge = self.control_stack[0].merge_bb();
                    if let Some(phi) = fn_phi {
                        let val = self.pop()?;
                        let current_bb = self.builder.get_insert_block().unwrap();
                        phi.add_incoming(&[(&val, current_bb)]);
                    }
                    llvm_err(self.builder.build_unconditional_branch(fn_merge))?;
                    self.unreachable = true;
                }
                Ok(())
            }

            Operator::Block { blockty } => {
                if self.unreachable {
                    // In dead code, just track nesting for End matching
                    self.control_stack.push(ControlFrame::Block {
                        merge_bb: self.builder.get_insert_block().unwrap(),
                        result_phi: None,
                        stack_depth: self.operand_stack.len(),
                    });
                    return Ok(());
                }
                let has_result = !matches!(blockty, wasmparser::BlockType::Empty);
                let fn_val = self.current_fn.unwrap();
                let merge_bb = self.context.append_basic_block(fn_val, "block_merge");
                let result_phi = if has_result {
                    let current_bb = self.builder.get_insert_block().unwrap();
                    self.builder.position_at_end(merge_bb);
                    let phi = llvm_err(self.builder.build_phi(self.i64_type, "block_result"))?;
                    self.builder.position_at_end(current_bb);
                    Some(phi)
                } else {
                    None
                };
                self.control_stack.push(ControlFrame::Block {
                    merge_bb,
                    result_phi,
                    stack_depth: self.operand_stack.len(),
                });
                Ok(())
            }

            Operator::Loop { .. } => {
                if self.unreachable {
                    self.control_stack.push(ControlFrame::Loop {
                        header_bb: self.builder.get_insert_block().unwrap(),
                        merge_bb: self.builder.get_insert_block().unwrap(),
                        stack_depth: self.operand_stack.len(),
                    });
                    return Ok(());
                }
                let fn_val = self.current_fn.unwrap();
                let header_bb = self.context.append_basic_block(fn_val, "loop_header");
                let merge_bb = self.context.append_basic_block(fn_val, "loop_merge");
                // Branch from current block to loop header
                llvm_err(self.builder.build_unconditional_branch(header_bb))?;
                self.builder.position_at_end(header_bb);
                self.control_stack.push(ControlFrame::Loop {
                    header_bb,
                    merge_bb,
                    stack_depth: self.operand_stack.len(),
                });
                Ok(())
            }

            Operator::If { blockty } => {
                if self.unreachable {
                    self.control_stack.push(ControlFrame::If {
                        else_bb: self.builder.get_insert_block().unwrap(),
                        merge_bb: self.builder.get_insert_block().unwrap(),
                        result_phi: None,
                        stack_depth: self.operand_stack.len(),
                        else_seen: false,
                    });
                    return Ok(());
                }
                let has_result = !matches!(blockty, wasmparser::BlockType::Empty);
                let cond = self.pop()?;
                let cond32 = llvm_err(self.builder.build_int_truncate(
                    cond,
                    self.i32_type,
                    "if_cond",
                ))?;
                let cond_bool = llvm_err(self.builder.build_int_compare(
                    IntPredicate::NE,
                    cond32,
                    self.i32_type.const_zero(),
                    "if_test",
                ))?;

                let fn_val = self.current_fn.unwrap();
                let then_bb = self.context.append_basic_block(fn_val, "if_then");
                let else_bb = self.context.append_basic_block(fn_val, "if_else");
                let merge_bb = self.context.append_basic_block(fn_val, "if_merge");

                let result_phi = if has_result {
                    let current_bb = self.builder.get_insert_block().unwrap();
                    self.builder.position_at_end(merge_bb);
                    let phi = llvm_err(self.builder.build_phi(self.i64_type, "if_result"))?;
                    self.builder.position_at_end(current_bb);
                    Some(phi)
                } else {
                    None
                };

                llvm_err(
                    self.builder
                        .build_conditional_branch(cond_bool, then_bb, else_bb),
                )?;
                self.builder.position_at_end(then_bb);

                self.control_stack.push(ControlFrame::If {
                    else_bb,
                    merge_bb,
                    result_phi,
                    stack_depth: self.operand_stack.len(),
                    else_seen: false,
                });
                Ok(())
            }

            Operator::Else => {
                // Extract values from frame before any mutable borrow of self
                let frame = self
                    .control_stack
                    .last()
                    .ok_or_else(|| Error::Internal("Else without matching If".into()))?;
                let (merge, else_block, phi, depth) = if let ControlFrame::If {
                    else_bb,
                    merge_bb,
                    result_phi,
                    stack_depth,
                    ..
                } = frame
                {
                    (*merge_bb, *else_bb, *result_phi, *stack_depth)
                } else {
                    return Err(Error::Internal("Else without matching If frame".into()));
                };

                // Now mark else_seen
                if let Some(ControlFrame::If { else_seen, .. }) = self.control_stack.last_mut() {
                    *else_seen = true;
                }

                if !self.unreachable {
                    if let Some(phi) = phi {
                        let val = self.pop()?;
                        let current_bb = self.builder.get_insert_block().unwrap();
                        phi.add_incoming(&[(&val, current_bb)]);
                    }
                    llvm_err(self.builder.build_unconditional_branch(merge))?;
                }

                self.builder.position_at_end(else_block);
                self.operand_stack.truncate(depth);
                self.unreachable = false;
                Ok(())
            }

            Operator::End => {
                let frame = self
                    .control_stack
                    .pop()
                    .ok_or_else(|| Error::Internal("End without matching control frame".into()))?;

                match frame {
                    ControlFrame::Block {
                        merge_bb,
                        result_phi,
                        stack_depth,
                    } => {
                        if self.control_stack.is_empty() {
                            // Function-level End: branch to merge which has the return
                            if !self.unreachable {
                                if let Some(phi) = result_phi {
                                    let val = self.pop()?;
                                    let current_bb = self.builder.get_insert_block().unwrap();
                                    phi.add_incoming(&[(&val, current_bb)]);
                                }
                                llvm_err(self.builder.build_unconditional_branch(merge_bb))?;
                            }
                            // Position at merge block and emit actual return
                            self.builder.position_at_end(merge_bb);
                            if let Some(phi) = result_phi {
                                // Ensure phi has at least one incoming; if not, add undef
                                if phi.count_incoming() == 0 {
                                    let undef = self.i64_type.get_undef();
                                    phi.add_incoming(&[(&undef, merge_bb)]);
                                }
                                let ret_val = phi.as_basic_value().into_int_value();
                                llvm_err(self.builder.build_return(Some(&ret_val)))?;
                            } else {
                                llvm_err(self.builder.build_return(None))?;
                            }
                            self.unreachable = false;
                        } else {
                            // Nested block End
                            if !self.unreachable {
                                if let Some(phi) = result_phi {
                                    let val = self.pop()?;
                                    let current_bb = self.builder.get_insert_block().unwrap();
                                    phi.add_incoming(&[(&val, current_bb)]);
                                }
                                llvm_err(self.builder.build_unconditional_branch(merge_bb))?;
                            }
                            self.builder.position_at_end(merge_bb);
                            self.operand_stack.truncate(stack_depth);
                            if let Some(phi) = result_phi {
                                self.push(phi.as_basic_value().into_int_value());
                            }
                            self.unreachable = false;
                        }
                    }
                    ControlFrame::Loop {
                        merge_bb,
                        stack_depth,
                        ..
                    } => {
                        if !self.unreachable {
                            llvm_err(self.builder.build_unconditional_branch(merge_bb))?;
                        }
                        self.builder.position_at_end(merge_bb);
                        self.operand_stack.truncate(stack_depth);
                        self.unreachable = false;
                    }
                    ControlFrame::If {
                        else_bb,
                        merge_bb,
                        result_phi,
                        stack_depth,
                        else_seen,
                    } => {
                        if !self.unreachable {
                            if let Some(phi) = result_phi {
                                let val = self.pop()?;
                                let current_bb = self.builder.get_insert_block().unwrap();
                                phi.add_incoming(&[(&val, current_bb)]);
                            }
                            llvm_err(self.builder.build_unconditional_branch(merge_bb))?;
                        }

                        if !else_seen {
                            // No else branch: else_bb falls through to merge
                            self.builder.position_at_end(else_bb);
                            llvm_err(self.builder.build_unconditional_branch(merge_bb))?;
                        }

                        self.builder.position_at_end(merge_bb);
                        self.operand_stack.truncate(stack_depth);
                        if let Some(phi) = result_phi {
                            self.push(phi.as_basic_value().into_int_value());
                        }
                        self.unreachable = false;
                    }
                }
                Ok(())
            }

            Operator::Br { relative_depth } => {
                if !self.unreachable {
                    let depth = *relative_depth as usize;
                    let idx = self.control_stack.len() - 1 - depth;
                    let target_bb = self.control_stack[idx].br_target();
                    if self.control_stack[idx].br_has_result() {
                        let val = self.pop()?;
                        let current_bb = self.builder.get_insert_block().unwrap();
                        if let Some(phi) = self.control_stack[idx].result_phi() {
                            phi.add_incoming(&[(&val, current_bb)]);
                        }
                    }
                    llvm_err(self.builder.build_unconditional_branch(target_bb))?;
                    self.unreachable = true;
                }
                Ok(())
            }

            Operator::BrIf { relative_depth } => {
                if !self.unreachable {
                    let cond = self.pop()?;
                    let cond32 = llvm_err(self.builder.build_int_truncate(
                        cond,
                        self.i32_type,
                        "brif_c",
                    ))?;
                    let cond_bool = llvm_err(self.builder.build_int_compare(
                        IntPredicate::NE,
                        cond32,
                        self.i32_type.const_zero(),
                        "brif_test",
                    ))?;

                    let depth = *relative_depth as usize;
                    let idx = self.control_stack.len() - 1 - depth;
                    let target_bb = self.control_stack[idx].br_target();

                    // If the target block has a result, we need to pass it.
                    // For br_if, the value stays on the stack for the fallthrough path.
                    if self.control_stack[idx].br_has_result() {
                        let val = self.peek()?;
                        let current_bb = self.builder.get_insert_block().unwrap();
                        if let Some(phi) = self.control_stack[idx].result_phi() {
                            phi.add_incoming(&[(&val, current_bb)]);
                        }
                    }

                    let fn_val = self.current_fn.unwrap();
                    let continue_bb = self.context.append_basic_block(fn_val, "brif_cont");
                    llvm_err(self.builder.build_conditional_branch(
                        cond_bool,
                        target_bb,
                        continue_bb,
                    ))?;
                    self.builder.position_at_end(continue_bb);
                }
                Ok(())
            }

            Operator::BrTable { targets } => {
                if !self.unreachable {
                    let index = self.pop()?;
                    let index32 = llvm_err(self.builder.build_int_truncate(
                        index,
                        self.i32_type,
                        "brtbl_idx",
                    ))?;

                    let default_depth = targets.default() as usize;
                    let default_idx = self.control_stack.len() - 1 - default_depth;
                    let default_bb = self.control_stack[default_idx].br_target();

                    // Add phi incoming for default target
                    if self.control_stack[default_idx].br_has_result() {
                        let val = self.peek()?;
                        let current_bb = self.builder.get_insert_block().unwrap();
                        if let Some(phi) = self.control_stack[default_idx].result_phi() {
                            phi.add_incoming(&[(&val, current_bb)]);
                        }
                    }

                    let target_depths: Vec<u32> = targets
                        .targets()
                        .collect::<std::result::Result<Vec<_>, _>>()?;

                    // Add phi incomings for all non-default targets
                    let current_bb = self.builder.get_insert_block().unwrap();
                    for &depth in &target_depths {
                        let idx = self.control_stack.len() - 1 - depth as usize;
                        if self.control_stack[idx].br_has_result() {
                            let val = self.peek()?;
                            if let Some(phi) = self.control_stack[idx].result_phi() {
                                phi.add_incoming(&[(&val, current_bb)]);
                            }
                        }
                    }

                    let cases: Vec<(IntValue<'ctx>, BasicBlock<'ctx>)> = target_depths
                        .iter()
                        .enumerate()
                        .map(|(i, &depth)| {
                            let case_val = self.i32_type.const_int(i as u64, false);
                            let idx = self.control_stack.len() - 1 - depth as usize;
                            let bb = self.control_stack[idx].br_target();
                            (case_val, bb)
                        })
                        .collect();

                    llvm_err(self.builder.build_switch(index32, default_bb, &cases))?;
                    self.unreachable = true;
                }
                Ok(())
            }

            // === Memory loads ===
            Operator::I32Load { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i32, memarg.offset)
            }
            Operator::I64Load { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i64, memarg.offset)
            }
            Operator::I32Load8U { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i8u, memarg.offset)
            }
            Operator::I32Load8S { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i8s, memarg.offset)
            }
            Operator::I32Load16U { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i16u, memarg.offset)
            }
            Operator::I32Load16S { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i16s, memarg.offset)
            }
            Operator::I64Load8U { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i8u_64, memarg.offset)
            }
            Operator::I64Load8S { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i8s_64, memarg.offset)
            }
            Operator::I64Load16U { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i16u_64, memarg.offset)
            }
            Operator::I64Load16S { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i16s_64, memarg.offset)
            }
            Operator::I64Load32U { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i32u_64, memarg.offset)
            }
            Operator::I64Load32S { memarg } => {
                self.emit_load(self.pvm_intrinsics.load_i32s_64, memarg.offset)
            }

            // === Memory stores ===
            Operator::I32Store { memarg } => {
                self.emit_store(self.pvm_intrinsics.store_i32, memarg.offset)
            }
            Operator::I64Store { memarg } => {
                self.emit_store(self.pvm_intrinsics.store_i64, memarg.offset)
            }
            Operator::I32Store8 { memarg } => {
                self.emit_store(self.pvm_intrinsics.store_i8, memarg.offset)
            }
            Operator::I32Store16 { memarg } => {
                self.emit_store(self.pvm_intrinsics.store_i16, memarg.offset)
            }
            Operator::I64Store8 { memarg } => {
                self.emit_store(self.pvm_intrinsics.store_i8_64, memarg.offset)
            }
            Operator::I64Store16 { memarg } => {
                self.emit_store(self.pvm_intrinsics.store_i16_64, memarg.offset)
            }
            Operator::I64Store32 { memarg } => {
                self.emit_store(self.pvm_intrinsics.store_i32_64, memarg.offset)
            }

            // === Memory management ===
            Operator::MemorySize { .. } => {
                let result = llvm_err(self.builder.build_call(
                    self.pvm_intrinsics.memory_size,
                    &[],
                    "memsize",
                ))?;
                let val = result
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| Error::Internal("memory_size returned void".into()))?
                    .into_int_value();
                self.push(val);
                Ok(())
            }
            Operator::MemoryGrow { .. } => {
                let pages = self.pop()?;
                let result = llvm_err(self.builder.build_call(
                    self.pvm_intrinsics.memory_grow,
                    &[pages.into()],
                    "memgrow",
                ))?;
                let val = result
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| Error::Internal("memory_grow returned void".into()))?
                    .into_int_value();
                self.push(val);
                Ok(())
            }
            Operator::MemoryFill { .. } => {
                let len = self.pop()?;
                let val = self.pop()?;
                let dst = self.pop()?;
                llvm_err(self.builder.build_call(
                    self.pvm_intrinsics.memory_fill,
                    &[dst.into(), val.into(), len.into()],
                    "memfill",
                ))?;
                Ok(())
            }
            Operator::MemoryCopy { .. } => {
                let len = self.pop()?;
                let src = self.pop()?;
                let dst = self.pop()?;
                llvm_err(self.builder.build_call(
                    self.pvm_intrinsics.memory_copy,
                    &[dst.into(), src.into(), len.into()],
                    "memcopy",
                ))?;
                Ok(())
            }

            // === Calls ===
            Operator::Call { function_index } => {
                let target_fn = self.functions[*function_index as usize];
                let param_count = target_fn.count_params() as usize;
                let mut args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(param_count);
                for _ in 0..param_count {
                    args.push(self.pop()?.into());
                }
                args.reverse();
                let result = llvm_err(self.builder.build_call(target_fn, &args, "call"))?;
                if target_fn.get_type().get_return_type().is_some() {
                    let val = result
                        .try_as_basic_value()
                        .basic()
                        .ok_or_else(|| Error::Internal("call returned void unexpectedly".into()))?
                        .into_int_value();
                    self.push(val);
                }
                Ok(())
            }
            Operator::CallIndirect {
                type_index,
                table_index: _,
            } => {
                let (num_params, num_results) = self
                    .type_signatures
                    .get(*type_index as usize)
                    .copied()
                    .ok_or_else(|| Error::Internal(format!("unknown type index {type_index}")))?;
                // Pop the table entry index (on top of the args on the stack)
                let table_entry = self.pop()?;
                // Pop arguments in reverse order
                let mut fn_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(num_params);
                for _ in 0..num_params {
                    fn_args.push(self.pop()?.into());
                }
                fn_args.reverse();
                // Build intrinsic args: type_idx, table_entry, then function args
                let mut all_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(num_params + 2);
                all_args.push(
                    self.i64_type
                        .const_int(u64::from(*type_index), false)
                        .into(),
                );
                all_args.push(table_entry.into());
                all_args.extend(fn_args);
                let result = llvm_err(self.builder.build_call(
                    self.pvm_intrinsics.call_indirect,
                    &all_args,
                    "call_indirect",
                ))?;
                if num_results > 0 {
                    let val = result
                        .try_as_basic_value()
                        .basic()
                        .ok_or_else(|| Error::Internal("call_indirect returned void".into()))?
                        .into_int_value();
                    self.push(val);
                }
                Ok(())
            }

            // === Float stubs ===
            Operator::I32TruncSatF64U
            | Operator::I32TruncSatF64S
            | Operator::I32TruncSatF32U
            | Operator::I32TruncSatF32S
            | Operator::I64TruncSatF64U
            | Operator::I64TruncSatF64S
            | Operator::I64TruncSatF32U
            | Operator::I64TruncSatF32S => Err(Error::FloatNotSupported),

            _ => Err(Error::Unsupported(format!("{op:?}"))),
        }
    }

    // ── Stack helpers ──

    fn push(&mut self, val: IntValue<'ctx>) {
        self.operand_stack.push(val);
    }

    fn pop(&mut self) -> Result<IntValue<'ctx>> {
        self.operand_stack
            .pop()
            .ok_or_else(|| Error::Internal("operand stack underflow".into()))
    }

    fn peek(&self) -> Result<IntValue<'ctx>> {
        self.operand_stack
            .last()
            .copied()
            .ok_or_else(|| Error::Internal("operand stack underflow on peek".into()))
    }

    // ── Memory operation helpers ──

    fn emit_load(&mut self, intrinsic: FunctionValue<'ctx>, offset: u64) -> Result<()> {
        let addr = self.pop()?;
        let eff_addr = if offset == 0 {
            addr
        } else {
            let offset_val = self.i64_type.const_int(offset, false);
            llvm_err(self.builder.build_int_add(addr, offset_val, "load_addr"))?
        };
        let result = llvm_err(
            self.builder
                .build_call(intrinsic, &[eff_addr.into()], "load"),
        )?;
        let val = result
            .try_as_basic_value()
            .basic()
            .ok_or_else(|| Error::Internal("load intrinsic returned void".into()))?
            .into_int_value();
        self.push(val);
        Ok(())
    }

    fn emit_store(&mut self, intrinsic: FunctionValue<'ctx>, offset: u64) -> Result<()> {
        let val = self.pop()?;
        let addr = self.pop()?;
        let eff_addr = if offset == 0 {
            addr
        } else {
            let offset_val = self.i64_type.const_int(offset, false);
            llvm_err(self.builder.build_int_add(addr, offset_val, "store_addr"))?
        };
        llvm_err(
            self.builder
                .build_call(intrinsic, &[eff_addr.into(), val.into()], "store"),
        )?;
        Ok(())
    }

    // ── Binary operation helpers ──

    fn i32_binop<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(
            &inkwell::builder::Builder<'ctx>,
            IntValue<'ctx>,
            IntValue<'ctx>,
        ) -> Result<IntValue<'ctx>>,
    {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        let lhs32 = llvm_err(self.builder.build_int_truncate(lhs, self.i32_type, "l32"))?;
        let rhs32 = llvm_err(self.builder.build_int_truncate(rhs, self.i32_type, "r32"))?;
        let result32 = f(&self.builder, lhs32, rhs32)?;
        let result64 = llvm_err(
            self.builder
                .build_int_z_extend(result32, self.i64_type, "ext"),
        )?;
        self.push(result64);
        Ok(())
    }

    fn i64_binop<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(
            &inkwell::builder::Builder<'ctx>,
            IntValue<'ctx>,
            IntValue<'ctx>,
        ) -> Result<IntValue<'ctx>>,
    {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        let result = f(&self.builder, lhs, rhs)?;
        self.push(result);
        Ok(())
    }

    // ── Comparison helpers ──

    fn i32_cmp(&mut self, pred: IntPredicate) -> Result<()> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        let lhs32 = llvm_err(self.builder.build_int_truncate(lhs, self.i32_type, "cl"))?;
        let rhs32 = llvm_err(self.builder.build_int_truncate(rhs, self.i32_type, "cr"))?;
        let cmp = llvm_err(self.builder.build_int_compare(pred, lhs32, rhs32, "cmp"))?;
        let ext = llvm_err(
            self.builder
                .build_int_z_extend(cmp, self.i64_type, "cmpext"),
        )?;
        self.push(ext);
        Ok(())
    }

    fn i64_cmp(&mut self, pred: IntPredicate) -> Result<()> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        let cmp = llvm_err(self.builder.build_int_compare(pred, lhs, rhs, "cmp"))?;
        let ext = llvm_err(
            self.builder
                .build_int_z_extend(cmp, self.i64_type, "cmpext"),
        )?;
        self.push(ext);
        Ok(())
    }

    fn i32_eqz(&mut self) -> Result<()> {
        let val = self.pop()?;
        let val32 = llvm_err(self.builder.build_int_truncate(val, self.i32_type, "eqz_t"))?;
        let cmp = llvm_err(self.builder.build_int_compare(
            IntPredicate::EQ,
            val32,
            self.i32_type.const_zero(),
            "eqz",
        ))?;
        let ext = llvm_err(
            self.builder
                .build_int_z_extend(cmp, self.i64_type, "eqzext"),
        )?;
        self.push(ext);
        Ok(())
    }

    fn i64_eqz(&mut self) -> Result<()> {
        let val = self.pop()?;
        let cmp = llvm_err(self.builder.build_int_compare(
            IntPredicate::EQ,
            val,
            self.i64_type.const_zero(),
            "eqz",
        ))?;
        let ext = llvm_err(
            self.builder
                .build_int_z_extend(cmp, self.i64_type, "eqzext"),
        )?;
        self.push(ext);
        Ok(())
    }

    // ── Rotation helpers (via fshl/fshr intrinsics) ──

    fn i32_rotate(&mut self, intrinsic_name: &str) -> Result<()> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        let val32 = llvm_err(self.builder.build_int_truncate(lhs, self.i32_type, "rotv"))?;
        let amt32 = llvm_err(self.builder.build_int_truncate(rhs, self.i32_type, "rota"))?;
        let result32 =
            self.call_ternary_intrinsic(intrinsic_name, self.i32_type, val32, val32, amt32)?;
        let ext = llvm_err(
            self.builder
                .build_int_z_extend(result32, self.i64_type, "rotext"),
        )?;
        self.push(ext);
        Ok(())
    }

    fn i64_rotate(&mut self, intrinsic_name: &str) -> Result<()> {
        let amt = self.pop()?;
        let val = self.pop()?;
        let result = self.call_ternary_intrinsic(intrinsic_name, self.i64_type, val, val, amt)?;
        self.push(result);
        Ok(())
    }

    // ── Bit counting helpers (ctlz/cttz/ctpop) ──

    fn i32_count_bits(&mut self, intrinsic_name: &str, has_zero_poison_arg: bool) -> Result<()> {
        let val = self.pop()?;
        let val32 = llvm_err(self.builder.build_int_truncate(val, self.i32_type, "cnt_t"))?;
        let result32 = if has_zero_poison_arg {
            // ctlz/cttz take an extra i1 arg: is_zero_poison (false = defined for zero)
            self.call_intrinsic_with_bool(intrinsic_name, self.i32_type, val32, false)?
        } else {
            // ctpop has no extra arg
            self.call_unary_intrinsic(intrinsic_name, self.i32_type, val32)?
        };
        let ext = llvm_err(
            self.builder
                .build_int_z_extend(result32, self.i64_type, "cntext"),
        )?;
        self.push(ext);
        Ok(())
    }

    fn i64_count_bits(&mut self, intrinsic_name: &str, has_zero_poison_arg: bool) -> Result<()> {
        let val = self.pop()?;
        let result = if has_zero_poison_arg {
            self.call_intrinsic_with_bool(intrinsic_name, self.i64_type, val, false)?
        } else {
            self.call_unary_intrinsic(intrinsic_name, self.i64_type, val)?
        };
        self.push(result);
        Ok(())
    }

    // ── Sign extension helpers ──

    fn sign_extend_in_i32(&mut self, from_bits: u32) -> Result<()> {
        let val = self.pop()?;
        let narrow_type = self.context.custom_width_int_type(from_bits);
        let val32 = llvm_err(
            self.builder
                .build_int_truncate(val, self.i32_type, "se_t32"),
        )?;
        let narrow = llvm_err(
            self.builder
                .build_int_truncate(val32, narrow_type, "se_narrow"),
        )?;
        let extended = llvm_err(self.builder.build_int_s_extend(
            narrow,
            self.i32_type,
            "se_ext32",
        ))?;
        let result = llvm_err(self.builder.build_int_z_extend(
            extended,
            self.i64_type,
            "se_ext64",
        ))?;
        self.push(result);
        Ok(())
    }

    fn sign_extend_in_i64(&mut self, from_bits: u32) -> Result<()> {
        let val = self.pop()?;
        let narrow_type = self.context.custom_width_int_type(from_bits);
        let narrow = llvm_err(
            self.builder
                .build_int_truncate(val, narrow_type, "se_narrow"),
        )?;
        let result = llvm_err(
            self.builder
                .build_int_s_extend(narrow, self.i64_type, "se_ext"),
        )?;
        self.push(result);
        Ok(())
    }

    // ── Intrinsic call helpers ──

    fn call_unary_intrinsic(
        &self,
        name: &str,
        operand_type: IntType<'ctx>,
        val: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>> {
        let intrinsic = Intrinsic::find(name)
            .ok_or_else(|| Error::Internal(format!("intrinsic {name} not found")))?;
        let fn_val = intrinsic
            .get_declaration(&self.module, &[operand_type.into()])
            .ok_or_else(|| Error::Internal(format!("{name} declaration failed")))?;
        let result = llvm_err(self.builder.build_call(fn_val, &[val.into()], "intrinsic"))?;
        result
            .try_as_basic_value()
            .basic()
            .ok_or_else(|| Error::Internal(format!("{name} returned void")))
            .map(|v| v.into_int_value())
    }

    fn call_intrinsic_with_bool(
        &self,
        name: &str,
        operand_type: IntType<'ctx>,
        val: IntValue<'ctx>,
        bool_arg: bool,
    ) -> Result<IntValue<'ctx>> {
        let intrinsic = Intrinsic::find(name)
            .ok_or_else(|| Error::Internal(format!("intrinsic {name} not found")))?;
        let fn_val = intrinsic
            .get_declaration(&self.module, &[operand_type.into()])
            .ok_or_else(|| Error::Internal(format!("{name} declaration failed")))?;
        let bool_val: BasicMetadataValueEnum =
            self.i1_type.const_int(u64::from(bool_arg), false).into();
        let result = llvm_err(self.builder.build_call(
            fn_val,
            &[val.into(), bool_val],
            "intrinsic",
        ))?;
        result
            .try_as_basic_value()
            .basic()
            .ok_or_else(|| Error::Internal(format!("{name} returned void")))
            .map(|v| v.into_int_value())
    }

    fn call_ternary_intrinsic(
        &self,
        name: &str,
        operand_type: IntType<'ctx>,
        a: IntValue<'ctx>,
        b: IntValue<'ctx>,
        c: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>> {
        let intrinsic = Intrinsic::find(name)
            .ok_or_else(|| Error::Internal(format!("intrinsic {name} not found")))?;
        let fn_val = intrinsic
            .get_declaration(&self.module, &[operand_type.into()])
            .ok_or_else(|| Error::Internal(format!("{name} declaration failed")))?;
        let result = llvm_err(self.builder.build_call(
            fn_val,
            &[a.into(), b.into(), c.into()],
            "intrinsic",
        ))?;
        result
            .try_as_basic_value()
            .basic()
            .ok_or_else(|| Error::Internal(format!("{name} returned void")))
            .map(|v| v.into_int_value())
    }

    // ── Optimization passes ──

    fn run_mem2reg(&self) -> Result<()> {
        use inkwell::passes::PassBuilderOptions;
        use inkwell::targets::{InitializationConfig, Target, TargetMachine};

        Target::initialize_all(&InitializationConfig::default());
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple)
            .map_err(|e| Error::Internal(format!("LLVM target error: {e}")))?;
        let machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                inkwell::OptimizationLevel::None,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or_else(|| Error::Internal("failed to create target machine".into()))?;

        let opts = PassBuilderOptions::create();
        self.module
            .run_passes("mem2reg", &machine, opts)
            .map_err(|e| Error::Internal(format!("mem2reg pass failed: {e}")))?;

        Ok(())
    }
}
