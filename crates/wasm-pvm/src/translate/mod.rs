// Address calculations and jump offsets often require wrapping/truncation.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

pub mod adapter_merge;
pub use crate::memory_layout;
pub mod stats;
pub mod wasm_module;

use std::collections::BTreeMap;

use crate::pvm::Instruction;
use crate::{Error, Result, SpiProgram};

pub use wasm_module::WasmModule;

/// Action to take when a WASM import is called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportAction {
    /// Emit a trap (unreachable) instruction.
    Trap,
    /// Emit a no-op (return 0 for functions with return values).
    Nop,
    /// Emit a PVM `ecalli` instruction with the given index.
    /// Arguments are loaded into data registers (r7-r12), return value from r7.
    Ecalli(u32),
}

/// Flags to enable/disable individual compiler optimizations.
/// All optimizations are enabled by default.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct OptimizationFlags {
    /// Run LLVM optimization passes (mem2reg, instcombine, simplifycfg, gvn, dce).
    /// When false, also disables inlining (all LLVM passes are skipped).
    pub llvm_passes: bool,
    /// Run peephole optimizer (fallthrough removal, dead code elimination).
    pub peephole: bool,
    /// Enable per-block register cache (store-load forwarding).
    pub register_cache: bool,
    /// Fuse `ICmp` + Branch into a single PVM branch instruction.
    pub icmp_branch_fusion: bool,
    /// Only save/restore callee-saved registers (r9-r12) that are actually used.
    pub shrink_wrap_callee_saves: bool,
    /// Eliminate SP-relative stores whose target offset is never loaded from.
    pub dead_store_elimination: bool,
    /// Skip redundant `LoadImm`/`LoadImm64` when the register already holds the constant.
    pub constant_propagation: bool,
    /// Inline small functions at the LLVM IR level to eliminate call overhead.
    pub inlining: bool,
    /// Propagate register cache across single-predecessor block boundaries.
    pub cross_block_cache: bool,
    /// Allocate long-lived SSA values to physical registers (r5, r6) across block boundaries.
    pub register_allocation: bool,
    /// Eliminate unconditional jumps to the immediately following block (fallthrough).
    pub fallthrough_jumps: bool,
    /// Lower the minimum-use threshold for register allocation candidates from 2 to 1.
    /// Captures more values (e.g. two-branch if-else patterns) at the cost of slightly
    /// more `MoveReg` traffic in small leaf functions.
    pub aggressive_register_allocation: bool,
    /// Allocate r5/r6 (`abi::SCRATCH1`/`SCRATCH2`) in all functions that don't
    /// clobber them (no bulk memory ops, no funnel shifts). In non-leaf functions,
    /// spill/reload around calls is handled automatically.
    pub allocate_scratch_regs: bool,
    /// Allocate r7/r8 (`RETURN_VALUE_REG`/`ARGS_LEN_REG`) in all functions.
    /// These are caller-saved and idle after the prologue; in non-leaf functions,
    /// they are invalidated after calls via arity-aware predicate.
    pub allocate_caller_saved_regs: bool,
    /// Skip stack stores at definition for register-allocated values (lazy spill).
    /// Values are only written to the stack when required (call clobber, return,
    /// phi reads, eviction). Requires `register_allocation` to be effective.
    pub lazy_spill: bool,
    /// Max LLVM IR instructions for a function to be inlineable.
    /// Functions exceeding this are marked `noinline`. `None` uses LLVM's
    /// default (225). Default: `Some(5)` — only tiny helpers are inlined.
    /// Only effective when `inlining` is `true`.
    pub inline_threshold: Option<u32>,
    /// Skip the zero-extension mask (`zext i32→i64` / `and x, 0xFFFFFFFF`) on
    /// values consumed exclusively as memory-access addresses. For any wasm
    /// memory smaller than 2 GB, sign- and zero-extension agree on every
    /// valid address and both forms trap on every invalid one, so the
    /// 2-instruction `shl 32; shr 32` pair per address is pure overhead.
    /// Caveat: programs that intentionally rely on wrapping i32 address
    /// arithmetic (well-defined in WASM, never emitted by LLVM or
    /// `AssemblyScript` for valid pointers) trap instead of wrapping.
    /// Automatically disabled when max memory ≥ 2 GB.
    pub address_mask_elision: bool,
    /// Recognize compiler-builtins libcalls (`__multi3`, `__udivti3`) by name
    /// and replace their bodies with hand-crafted PVM-friendly IR.
    /// `__multi3` collapses to a `Mul64` + `MulUpperUU` + a few adds.
    /// `__udivti3` dispatches on `(a_hi | b_hi) == 0`: fast path is native
    /// `DivU64`, slow path forwards to the original `specialized_div_rem`
    /// callee via the same stack-frame setup. Recognition is name-based and
    /// silently skips when the WASM `name` custom section is stripped or the
    /// function shape differs.
    pub libcall_recognition: bool,
    /// Run LLVM's `mergefunc` pass at the end of the optimization pipeline.
    /// Merges functions with byte-identical bodies into a single definition
    /// and replaces duplicates with thunks. Targets rustc monomorphizations
    /// (`quicksort`, `scale_info::TypeInfo::type_info` etc.) where many type
    /// parameters share a body.
    pub mergefunc: bool,
}

impl Default for OptimizationFlags {
    fn default() -> Self {
        Self {
            llvm_passes: true,
            peephole: true,
            register_cache: true,
            icmp_branch_fusion: true,
            address_mask_elision: true,
            shrink_wrap_callee_saves: true,
            dead_store_elimination: true,
            constant_propagation: true,
            inlining: true,
            cross_block_cache: true,
            register_allocation: true,
            fallthrough_jumps: true,
            aggressive_register_allocation: true,
            allocate_scratch_regs: true,
            allocate_caller_saved_regs: true,
            lazy_spill: true,
            inline_threshold: Some(5),
            libcall_recognition: true,
            mergefunc: true,
        }
    }
}

impl OptimizationFlags {
    /// All optional optimizations disabled. Used for correctness differential
    /// testing — running the same fixture twice (default vs `all_disabled`) and
    /// comparing results catches miscompiles that any optimization introduces.
    ///
    /// `llvm_passes` stays enabled because the PVM backend cannot lower
    /// `alloca`/unpromoted SSA — disabling LLVM passes (including `mem2reg`)
    /// would make non-trivial WASM fail to compile, not just run slower.
    /// `inline_threshold` is unchanged (it only matters when `inlining` is on).
    #[must_use]
    pub fn all_disabled() -> Self {
        Self {
            llvm_passes: true,
            peephole: false,
            register_cache: false,
            icmp_branch_fusion: false,
            address_mask_elision: false,
            shrink_wrap_callee_saves: false,
            dead_store_elimination: false,
            constant_propagation: false,
            inlining: false,
            cross_block_cache: false,
            register_allocation: false,
            fallthrough_jumps: false,
            aggressive_register_allocation: false,
            allocate_scratch_regs: false,
            allocate_caller_saved_regs: false,
            lazy_spill: false,
            inline_threshold: Some(5),
            libcall_recognition: false,
            mergefunc: false,
        }
    }
}

/// Options for compilation.
#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    /// Mapping from import function names to actions.
    /// When provided, all imports (except known intrinsics like `host_call_N` and `pvm_ptr`)
    /// must have a mapping or compilation will fail with `UnresolvedImport`.
    pub import_map: Option<BTreeMap<String, ImportAction>>,
    /// WAT source for an adapter module whose exports replace matching main imports.
    /// Applied before the text-based import map, so the two compose.
    pub adapter: Option<String>,
    /// Metadata blob to prepend to the SPI output.
    /// Typically contains the source filename and compiler version.
    pub metadata: Vec<u8>,
    /// Optimization flags controlling which compiler passes are enabled.
    pub optimizations: OptimizationFlags,
    /// Override the maximum memory pages (memory.grow ceiling).
    /// When set, this takes precedence over both the WASM-declared max and the compiler default.
    pub max_memory_pages: Option<u32>,
    /// When true, replace every f32/f64 operator with a runtime trap instead of
    /// failing compilation. Useful for diagnosing what other unsupported features
    /// a WASM module uses past the float wall. JAMs run normally if execution
    /// never reaches a float operator; otherwise they trap deterministically.
    pub trap_floats: bool,
}

// Re-export register constants from abi module
pub use crate::abi::{ARGS_LEN_REG, ARGS_PTR_REG, RETURN_ADDR_REG, STACK_PTR_REG};

// ── Call fixup types (shared with LLVM backend) ──

#[derive(Debug, Clone)]
pub struct CallFixup {
    pub return_addr_instr: usize,
    pub jump_instr: usize,
    pub target_func: u32,
}

#[derive(Debug, Clone)]
pub struct IndirectCallFixup {
    pub return_addr_instr: usize,
    // For `LoadImmJumpInd`, this equals `return_addr_instr`.
    pub jump_ind_instr: usize,
}

/// `RO_DATA` region size is 64KB (0x10000 to 0x1FFFF)
const RO_DATA_SIZE: usize = 64 * 1024;

/// Check if an import name is a known compiler intrinsic (`host_call_N` variants, `pvm_ptr`).
fn is_known_intrinsic(name: &str) -> bool {
    name == "pvm_ptr" || crate::abi::parse_host_call_variant(name).is_some()
}

/// Default mappings applied when no explicit import map is provided.
const DEFAULT_MAPPINGS: &[&str] = &["abort"];

pub fn compile(wasm: &[u8]) -> Result<SpiProgram> {
    compile_with_options(wasm, &CompileOptions::default())
}

pub fn compile_with_options(wasm: &[u8], options: &CompileOptions) -> Result<SpiProgram> {
    let (program, _) = compile_with_stats(wasm, options)?;
    Ok(program)
}

pub fn compile_with_stats(
    wasm: &[u8],
    options: &CompileOptions,
) -> Result<(SpiProgram, stats::CompileStats)> {
    // Apply adapter merge if provided (produces a new WASM binary with fewer imports).
    let merged_wasm;
    let wasm = if let Some(adapter_wat) = &options.adapter {
        merged_wasm = adapter_merge::merge_adapter(wasm, adapter_wat)?;
        &merged_wasm
    } else {
        wasm
    };

    let mut module = WasmModule::parse(wasm)?;

    // Apply max_memory_pages override if provided.
    if let Some(max_pages) = options.max_memory_pages {
        module.max_memory_pages = max_pages.max(module.memory_limits.initial_pages);
    }

    // Validate imports and collect resolutions.
    let mut import_resolutions = Vec::new();
    for name in &module.imported_func_names {
        if is_known_intrinsic(name) {
            let action = if name == "pvm_ptr" || name == "host_call_r8" {
                "intrinsic"
            } else {
                "ecalli"
            };
            import_resolutions.push(stats::ImportResolution {
                name: name.clone(),
                action: action.to_string(),
            });
            continue;
        }
        if let Some(import_map) = &options.import_map {
            if let Some(action) = import_map.get(name) {
                let action_str = match action {
                    ImportAction::Trap => "trap".to_string(),
                    ImportAction::Nop => "nop".to_string(),
                    ImportAction::Ecalli(idx) => format!("ecalli:{idx}"),
                };
                import_resolutions.push(stats::ImportResolution {
                    name: name.clone(),
                    action: action_str,
                });
                continue;
            }
        } else if DEFAULT_MAPPINGS.contains(&name.as_str()) {
            import_resolutions.push(stats::ImportResolution {
                name: name.clone(),
                action: "trap (default)".to_string(),
            });
            continue;
        }
        return Err(Error::UnresolvedImport(format!(
            "import '{name}' has no mapping. Provide a mapping via --imports or add it to the import map."
        )));
    }

    let active_data_segments = module
        .data_segments
        .iter()
        .filter(|s| s.offset.is_some())
        .count();
    let passive_data_segments = module
        .data_segments
        .iter()
        .filter(|s| s.offset.is_none())
        .count();
    let globals_region_bytes = memory_layout::globals_region_size(
        &module.global_widths,
        passive_data_segments,
        module.needs_memory_size_global,
    );

    let result = compile_via_llvm(&module, options)?;

    let spi_blob_bytes = result.program.encode().len();

    let compile_stats = stats::CompileStats {
        local_functions: module.functions.len(),
        imported_functions: module.num_imported_funcs as usize,
        globals: module.globals.len(),
        active_data_segments,
        passive_data_segments,
        function_table_entries: module.function_table.len(),
        initial_memory_pages: module.memory_limits.initial_pages,
        max_memory_pages: module.max_memory_pages,
        wasm_declared_max_pages: module.memory_limits.max_pages,
        import_resolutions,
        wasm_memory_base: module.wasm_memory_base,
        globals_region_bytes,
        ro_data_bytes: result.program.ro_data().len(),
        rw_data_bytes: result.program.rw_data().len(),
        heap_pages: result.program.heap_pages(),
        stack_size: memory_layout::DEFAULT_STACK_SIZE,
        pvm_instructions: result.pvm_instructions,
        code_bytes: result.code_bytes,
        jump_table_entries: result.jump_table_entries,
        spi_blob_bytes,
        functions: result.function_stats,
    };

    Ok((result.program, compile_stats))
}

/// Internal result of `compile_via_llvm`, carrying both the program and stats.
struct CompilationOutput {
    program: SpiProgram,
    function_stats: Vec<stats::FunctionStats>,
    pvm_instructions: usize,
    code_bytes: usize,
    jump_table_entries: usize,
}

fn compile_via_llvm(module: &WasmModule, options: &CompileOptions) -> Result<CompilationOutput> {
    use crate::llvm_backend::{self, LoweringContext};
    use crate::llvm_frontend;
    use inkwell::context::Context;

    // Phase 1: WASM → LLVM IR
    let context = Context::create();
    let llvm_module = llvm_frontend::translate_wasm_to_llvm(
        &context,
        module,
        options.optimizations.llvm_passes,
        options.optimizations.inlining,
        options.optimizations.inline_threshold,
        options.trap_floats,
        options.optimizations.libcall_recognition,
        options.optimizations.mergefunc,
    )?;

    // Calculate RO_DATA offsets and lengths for passive data segments
    let mut data_segment_offsets = std::collections::BTreeMap::new();
    let mut data_segment_lengths = std::collections::BTreeMap::new();
    let mut current_ro_offset = if module.function_table.is_empty() {
        1 // dummy byte if no function table
    } else {
        module.function_table.len() * 8 // jump_ref + type_idx per entry
    };

    let mut data_segment_length_addrs = std::collections::BTreeMap::new();
    let mut passive_ordinal = 0usize;

    for (idx, seg) in module.data_segments.iter().enumerate() {
        if seg.offset.is_none() {
            // Check that segment fits within RO_DATA region
            if current_ro_offset + seg.data.len() > RO_DATA_SIZE {
                return Err(Error::Internal(format!(
                    "passive data segment {} (size {}) would overflow RO_DATA region ({} bytes used of {})",
                    idx,
                    seg.data.len(),
                    current_ro_offset,
                    RO_DATA_SIZE
                )));
            }
            data_segment_offsets.insert(idx as u32, current_ro_offset as u32);
            data_segment_lengths.insert(idx as u32, seg.data.len() as u32);
            data_segment_length_addrs.insert(
                idx as u32,
                memory_layout::data_segment_length_offset(
                    &module.global_widths,
                    passive_ordinal,
                    module.needs_memory_size_global,
                ),
            );
            current_ro_offset += seg.data.len();
            passive_ordinal += 1;
        }
    }

    // Phase 2: Build lowering context
    let param_overflow_base = memory_layout::compute_param_overflow_base(
        &module.global_widths,
        passive_ordinal,
        module.needs_memory_size_global,
    );
    let ctx = LoweringContext {
        wasm_memory_base: module.wasm_memory_base,
        num_globals: module.globals.len(),
        has_memory_size_global: module.needs_memory_size_global,
        global_offsets: module.global_offsets.clone(),
        global_widths: module.global_widths.clone(),
        param_overflow_base,
        param_overflow_reserved: module.needs_param_overflow,
        function_signatures: module.function_signatures.clone(),
        type_signatures: module.type_signatures.clone(),
        function_table: module.function_table.clone(),
        num_imported_funcs: module.num_imported_funcs as usize,
        imported_func_names: module.imported_func_names.clone(),
        initial_memory_pages: module.memory_limits.initial_pages,
        max_memory_pages: module.max_memory_pages,
        stack_size: memory_layout::DEFAULT_STACK_SIZE,
        data_segment_offsets,
        data_segment_lengths,
        data_segment_length_addrs,
        wasm_import_map: options.import_map.clone(),
        optimizations: options.optimizations.clone(),
    };

    // Phase 3: LLVM IR → PVM bytecode for each function
    let mut all_instructions: Vec<Instruction> = Vec::new();
    let mut all_call_fixups: Vec<(usize, CallFixup)> = Vec::new();
    let mut all_indirect_call_fixups: Vec<(usize, IndirectCallFixup)> = Vec::new();
    let mut function_offsets: Vec<usize> = vec![0; module.functions.len()];
    let mut next_call_return_idx: usize = 0;
    let mut function_stats: Vec<stats::FunctionStats> = Vec::with_capacity(module.functions.len());

    // Entry header: Jump to main (PC=0) + Trap or secondary Jump (PC=5).
    // When there's no secondary entry, we omit the Fallthrough padding (6 bytes instead of 10).
    // Width-stable jumps: the secondary entry point is *defined* at pc=5
    // (right after a 5-byte jump), and these offsets are patched after layout.
    all_instructions.push(Instruction::JumpFixed { offset: 0 });
    if module.has_secondary_entry {
        all_instructions.push(Instruction::JumpFixed { offset: 0 });
    } else {
        all_instructions.push(Instruction::Trap);
    }

    // Build emission order: main first, then secondary (if any), then remaining in index order.
    // This places main immediately after the entry header, minimizing the entry Jump distance.
    let mut emission_order: Vec<usize> = Vec::with_capacity(module.functions.len());
    emission_order.push(module.main_func_local_idx);
    if let Some(secondary_idx) = module.secondary_entry_local_idx
        && secondary_idx != module.main_func_local_idx
    {
        emission_order.push(secondary_idx);
    }
    for idx in 0..module.functions.len() {
        if idx != module.main_func_local_idx && module.secondary_entry_local_idx != Some(idx) {
            emission_order.push(idx);
        }
    }

    // Running total of `all_instructions.iter().map(|i| i.encode().len()).sum()`.
    // Updated incrementally each time we append instructions so per-function
    // `func_start_offset` lookups stay O(1) — re-summing on every iteration
    // was an O(N²) shape that made compile times unbounded on real-world
    // modules (see issue #225). Seed from the entry header pushed above so
    // the first function's offset is correct.
    let mut current_code_bytes: usize = all_instructions.iter().map(|i| i.encode().len()).sum();

    for &local_func_idx in &emission_order {
        let global_func_idx = module.num_imported_funcs as usize + local_func_idx;
        let fn_name = format!("wasm_func_{global_func_idx}");
        let llvm_func = llvm_module
            .get_function(&fn_name)
            .ok_or_else(|| Error::Internal(format!("missing LLVM function: {fn_name}")))?;

        let is_main = local_func_idx == module.main_func_local_idx;
        let is_secondary = module.secondary_entry_local_idx == Some(local_func_idx);
        let is_entry = is_main || is_secondary;

        function_offsets[local_func_idx] = current_code_bytes;
        let func_emission_start = all_instructions.len();

        // If entry function and there's a start function, call it first.
        if let Some(start_local_idx) = module.start_func_local_idx.filter(|_| is_entry) {
            // Save r7 and r8 to stack.
            all_instructions.push(Instruction::AddImm64 {
                dst: STACK_PTR_REG,
                src: STACK_PTR_REG,
                value: -16,
            });
            all_instructions.push(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: ARGS_PTR_REG,
                offset: 0,
            });
            all_instructions.push(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: ARGS_LEN_REG,
                offset: 8,
            });

            // Call start function using LoadImmJump (combined load + jump).
            let call_return_addr = ((next_call_return_idx + 1) * 2) as i32;
            next_call_return_idx += 1;
            let current_instr_idx = all_instructions.len();
            all_instructions.push(Instruction::LoadImmJump {
                reg: RETURN_ADDR_REG,
                value: call_return_addr,
                offset: 0, // patched during fixup resolution
            });

            all_call_fixups.push((
                current_instr_idx,
                CallFixup {
                    target_func: start_local_idx as u32,
                    return_addr_instr: 0,
                    jump_instr: 0, // same instruction for LoadImmJump
                },
            ));

            // Restore r7 and r8.
            all_instructions.push(Instruction::LoadIndU64 {
                dst: ARGS_PTR_REG,
                base: STACK_PTR_REG,
                offset: 0,
            });
            all_instructions.push(Instruction::LoadIndU64 {
                dst: ARGS_LEN_REG,
                base: STACK_PTR_REG,
                offset: 8,
            });
            all_instructions.push(Instruction::AddImm64 {
                dst: STACK_PTR_REG,
                src: STACK_PTR_REG,
                value: 16,
            });
        }

        let display_name = module.local_function_display_name(local_func_idx);
        let translation = llvm_backend::lower_function(
            llvm_func,
            &ctx,
            is_entry,
            global_func_idx,
            &display_name,
            next_call_return_idx,
        )?;
        next_call_return_idx += translation.num_call_returns;

        let instr_base = all_instructions.len();
        for fixup in translation.call_fixups {
            all_call_fixups.push((
                instr_base,
                CallFixup {
                    return_addr_instr: fixup.return_addr_instr,
                    jump_instr: fixup.jump_instr,
                    target_func: fixup.target_func,
                },
            ));
        }
        for fixup in translation.indirect_call_fixups {
            all_indirect_call_fixups.push((
                instr_base,
                IndirectCallFixup {
                    return_addr_instr: fixup.return_addr_instr,
                    jump_ind_instr: fixup.jump_ind_instr,
                },
            ));
        }

        let ls = &translation.lowering_stats;
        function_stats.push(stats::FunctionStats {
            name: module.local_function_display_name(local_func_idx),
            index: local_func_idx,
            instruction_count: translation.instructions.len(),
            frame_size: ls.frame_size,
            is_leaf: ls.is_leaf,
            is_entry,
            regalloc: stats::FunctionRegAllocStats {
                total_values: ls.regalloc_total_values,
                allocated_values: ls.regalloc_allocated_values,
                registers_used: ls
                    .regalloc_registers_used
                    .iter()
                    .map(|r| format!("r{r}"))
                    .collect(),
                skipped_reason: ls.regalloc_skipped_reason.map(String::from),
                load_hits: ls.regalloc_load_hits,
                load_reloads: ls.regalloc_load_reloads,
                load_moves: ls.regalloc_load_moves,
                store_hits: ls.regalloc_store_hits,
                store_moves: ls.regalloc_store_moves,
            },
            pre_dse_instructions: ls.pre_dse_instructions,
            pre_peephole_instructions: ls.pre_peephole_instructions,
        });

        all_instructions.extend(translation.instructions);

        // Update the running byte counter to cover everything emitted this
        // iteration (entry-header trampoline pushes + the lowered body).
        for ins in &all_instructions[func_emission_start..] {
            current_code_bytes += ins.encode().len();
        }
    }

    // Phase 3.5: patch the entry-header jumps. The header uses
    // `JumpFixed` (width-stable 4-byte offsets), so patching cannot change
    // the layout; `function_offsets` computed during emission stay valid.
    // Narrow offsets to i32 with explicit bounds checks: a code section larger
    // than i32::MAX should surface as an error, not silently wrap into a
    // mispatched entry jump (mirrors the call-fixup path's handling).
    let main_offset = i32::try_from(function_offsets[module.main_func_local_idx])
        .map_err(|_| Error::Internal("main entry offset exceeds i32 range".to_string()))?;
    if let Instruction::JumpFixed { offset } = &mut all_instructions[0] {
        *offset = main_offset;
    }
    if let Some(secondary_idx) = module.secondary_entry_local_idx {
        // Offset is relative to the second header instruction, which starts
        // right after the first 5-byte JumpFixed (the pc=5 ABI entry).
        let i0_len = i32::try_from(all_instructions[0].encode().len())
            .map_err(|_| Error::Internal("entry-header length exceeds i32 range".to_string()))?;
        let secondary_offset = i32::try_from(function_offsets[secondary_idx])
            .map_err(|_| Error::Internal("secondary entry offset exceeds i32 range".to_string()))?
            - i0_len;
        if let Instruction::JumpFixed { offset } = &mut all_instructions[1] {
            *offset = secondary_offset;
        }
    }

    // Phase 4: Resolve call fixups and build jump table.
    let (jump_table, func_entry_jump_table_base) = resolve_call_fixups(
        &mut all_instructions,
        &all_call_fixups,
        &all_indirect_call_fixups,
        &function_offsets,
    )?;

    // The prefix-sum / running-counter optimisation (#225) relies on the
    // Phase-4 patches leaving every instruction's encoded byte length
    // unchanged — `LoadImmJump.offset` is a fixed 4-byte field
    // (`encode_one_reg_one_imm_one_off_fixed`) precisely so link-time call
    // patching stays width-stable. Branch/Jump offsets ARE variable-length,
    // but they are finalized earlier (intra-function relaxation in
    // `resolve_fixups`, entry-header relaxation in Phase 3.5). If a future
    // change patches a variable-length immediate after Phase 3.5,
    // `function_offsets` and `jump_table` silently desync; catch that here
    // instead of producing a corrupt JAM.
    debug_assert_eq!(
        all_instructions
            .iter()
            .map(|i| i.encode().len())
            .sum::<usize>(),
        current_code_bytes,
        "post-patch instruction stream size differs from emission-time total — \
         a patched instruction's encoded length changed, invalidating \
         function_offsets / jump_table",
    );

    // Phase 5: Build dispatch table for call_indirect.
    let mut ro_data = vec![0u8];
    if !module.function_table.is_empty() {
        ro_data.clear();
        for &func_idx in &module.function_table {
            if func_idx == u32::MAX || (func_idx as usize) < module.num_imported_funcs as usize {
                ro_data.extend_from_slice(&u32::MAX.to_le_bytes());
                ro_data.extend_from_slice(&u32::MAX.to_le_bytes());
            } else {
                let local_func_idx = func_idx as usize - module.num_imported_funcs as usize;
                let jump_ref = 2 * (func_entry_jump_table_base + local_func_idx + 1) as u32;
                ro_data.extend_from_slice(&jump_ref.to_le_bytes());
                let type_idx = *module
                    .function_type_indices
                    .get(local_func_idx)
                    .unwrap_or(&u32::MAX);
                ro_data.extend_from_slice(&type_idx.to_le_bytes());
            }
        }
    }

    // Append passive data segments to RO_DATA.
    // NOTE: This loop must iterate data_segments in the same order as the offset
    // calculation loop above, since data_segment_offsets indices depend on it.
    for seg in &module.data_segments {
        if seg.offset.is_none() {
            ro_data.extend_from_slice(&seg.data);
        }
    }

    // Capture stats before moving instructions into the blob.
    let pvm_instructions = all_instructions.len();
    let code_bytes: usize = all_instructions.iter().map(|i| i.encode().len()).sum();
    let jump_table_entries = jump_table.len();

    let blob = crate::pvm::ProgramBlob::new(all_instructions).with_jump_table(jump_table);
    let rw_data_section = build_rw_data(
        &module.data_segments,
        &module.global_init_values,
        &module.global_widths,
        module.memory_limits.initial_pages,
        module.wasm_memory_base,
        &ctx.data_segment_length_addrs,
        &ctx.data_segment_lengths,
        module.needs_memory_size_global,
    )?;

    let heap_pages = calculate_heap_pages(
        rw_data_section.len(),
        module.wasm_memory_base,
        module.memory_limits.initial_pages,
    )?;

    let program = SpiProgram::new(blob)
        .with_heap_pages(heap_pages)
        .with_ro_data(ro_data)
        .with_rw_data(rw_data_section)
        .with_metadata(options.metadata.clone());

    Ok(CompilationOutput {
        program,
        function_stats,
        pvm_instructions,
        code_bytes,
        jump_table_entries,
    })
}

/// Calculate the number of 4KB PVM heap pages needed after `rw_data`.
///
/// `heap_pages` tells the runtime how many zero-initialized writable pages to allocate
/// immediately after the `rw_data` blob. This covers the initial WASM linear memory,
/// globals, and spilled locals that aren't already covered by `rw_data`.
///
/// By computing this **after** `build_rw_data()`, we use the actual (trimmed) `rw_data`
/// length instead of guessing with headroom.
///
/// We add 1 extra page beyond the exact initial memory requirement. This ensures that
/// the first `memory.grow` / sbrk allocation has a pre-allocated page available at the
/// boundary of the initial WASM memory. Without it, PVM-in-PVM execution fails because
/// the inner interpreter's page-fault handling at the exact heap boundary doesn't
/// correctly propagate through the outer PVM.
fn calculate_heap_pages(
    rw_data_len: usize,
    wasm_memory_base: i32,
    initial_pages: u32,
) -> Result<u16> {
    use wasm_module::MIN_INITIAL_WASM_PAGES;

    let initial_pages = initial_pages.max(MIN_INITIAL_WASM_PAGES);
    let wasm_memory_initial_end = wasm_memory_base as usize + (initial_pages as usize) * 64 * 1024;

    let total_bytes = wasm_memory_initial_end - memory_layout::GLOBAL_MEMORY_BASE as usize;
    let rw_pages = rw_data_len.div_ceil(4096);
    let total_pages = total_bytes.div_ceil(4096);
    let heap_pages = total_pages.saturating_sub(rw_pages) + 1;

    u16::try_from(heap_pages).map_err(|_| {
        Error::Internal(format!(
            "heap size {heap_pages} pages exceeds u16::MAX ({}) — module too large",
            u16::MAX
        ))
    })
}

/// Build the `rw_data` section from WASM data segments and global initializers.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_rw_data(
    data_segments: &[wasm_module::DataSegment],
    global_init_values: &[i64],
    global_widths: &[u32],
    initial_memory_pages: u32,
    wasm_memory_base: i32,
    data_segment_length_addrs: &std::collections::BTreeMap<u32, i32>,
    data_segment_lengths: &std::collections::BTreeMap<u32, u32>,
    has_memory_size_global: bool,
) -> Result<Vec<u8>> {
    // Internal invariant: `global_widths` and `global_init_values` are populated
    // together by `WasmModule::parse`. Surface a mismatch as a real error
    // rather than a `debug_assert!` panic that disappears in release.
    if global_init_values.len() != global_widths.len() {
        return Err(Error::Internal(format!(
            "build_rw_data: global_init_values.len()={} != global_widths.len()={} — parallel-array invariant violated",
            global_init_values.len(),
            global_widths.len()
        )));
    }
    // Calculate the minimum size needed for globals — optionally includes a
    // compiler-managed memory-size slot plus passive segment lengths.
    let num_passive_segments = data_segment_length_addrs.len();
    let globals_end = memory_layout::globals_region_size(
        global_widths,
        num_passive_segments,
        has_memory_size_global,
    );

    // Calculate the size needed for data segments
    let wasm_to_rw_offset = wasm_memory_base as u32 - 0x30000;

    let data_end = data_segments
        .iter()
        .filter_map(|seg| {
            seg.offset
                .map(|off| wasm_to_rw_offset + off + seg.data.len() as u32)
        })
        .max()
        .unwrap_or(0) as usize;

    let total_size = globals_end.max(data_end);

    if total_size == 0 {
        return Ok(Vec::new());
    }

    let mut rw_data = vec![0u8; total_size];

    // Layout: [mem_size_slot?][globals (per-global width)][passive_lens (4B each)][data_segments...]
    // The mem-size slot lives at rw_data offset 0 when emitted (4 bytes); user
    // globals follow, each occupying `global_widths[i]` bytes packed in
    // declaration order (4 B for i32/f32, 8 B for i64/f64).
    let mem_size_slot_bytes = if has_memory_size_global {
        memory_layout::MEM_SIZE_SLOT_BYTES
    } else {
        0
    };

    if has_memory_size_global && mem_size_slot_bytes <= rw_data.len() {
        rw_data[..4].copy_from_slice(&initial_memory_pages.to_le_bytes());
    }

    let mut offset = mem_size_slot_bytes;
    for (i, &value) in global_init_values.iter().enumerate() {
        // Validate that the width is one we can actually serialize from an i64
        // little-endian buffer (max 8 bytes). Reaching this with anything else
        // would mean `global_storage_width` returned a width the backend's
        // 4/8 dispatch can't service either — surface a real error.
        let width = match global_widths[i] {
            w @ (4 | 8) => w as usize,
            other => {
                return Err(Error::Internal(format!(
                    "build_rw_data: wasm_global_{i} has unsupported storage width {other} bytes (expected 4 or 8)"
                )));
            }
        };
        if offset + width <= rw_data.len() {
            // Take the low `width` bytes of the i64 little-endian encoding.
            // For i32 globals: low 4 bytes (top 4 are sign/zero-extension we
            // don't store). For i64 globals: all 8 bytes.
            let le = value.to_le_bytes();
            rw_data[offset..offset + width].copy_from_slice(&le[..width]);
        }
        offset += width;
    }

    // Initialize passive data segment effective lengths (right after memory size global).
    // These are used by memory.init for bounds checking and zeroed by data.drop.
    for (&seg_idx, &addr) in data_segment_length_addrs {
        if let Some(&length) = data_segment_lengths.get(&seg_idx) {
            // addr is absolute PVM address; convert to rw_data offset
            let rw_offset = (addr - memory_layout::GLOBAL_MEMORY_BASE) as usize;
            if rw_offset + 4 <= rw_data.len() {
                rw_data[rw_offset..rw_offset + 4].copy_from_slice(&length.to_le_bytes());
            }
        }
    }

    // Copy data segments to their WASM memory locations
    for seg in data_segments {
        if let Some(offset) = seg.offset {
            let rw_offset = (wasm_to_rw_offset + offset) as usize;
            if rw_offset + seg.data.len() <= rw_data.len() {
                rw_data[rw_offset..rw_offset + seg.data.len()].copy_from_slice(&seg.data);
            }
        }
    }

    // Trim trailing zeros to reduce SPI size. Heap pages are zero-initialized,
    // so omitted high-address zero bytes are semantically equivalent.
    if let Some(last_non_zero) = rw_data.iter().rposition(|&b| b != 0) {
        rw_data.truncate(last_non_zero + 1);
    } else {
        rw_data.clear();
    }

    Ok(rw_data)
}

/// Extract the pre-assigned jump-table index from a return-address load instruction.
///
/// Call return addresses are pre-assigned as `(idx + 1) * 2` at emission time.
/// This helper recovers `idx` so that `resolve_call_fixups` can write the byte
/// offset into the correct jump-table slot instead of appending in list order
/// (which would desync when a function mixes direct and indirect calls).
///
/// Direct calls use `LoadImmJump`, while indirect calls use either `LoadImm` (legacy
/// two-instruction sequence) or `LoadImmJumpInd` (combined return-addr load + jump).
fn return_addr_jump_table_idx(
    instructions: &[Instruction],
    return_addr_instr: usize,
) -> Result<usize> {
    let value = match instructions.get(return_addr_instr) {
        Some(
            Instruction::LoadImmJump { value, .. }
            | Instruction::LoadImm { value, .. }
            | Instruction::LoadImmJumpInd { value, .. },
        ) => Some(*value),
        _ => None,
    };
    match value {
        Some(v) if v > 0 && v % 2 == 0 => Ok((v as usize / 2) - 1),
        _ => Err(Error::Internal(format!(
            "expected LoadImmJump/LoadImm/LoadImmJumpInd((idx+1)*2) at return_addr_instr {return_addr_instr}, got {:?}",
            instructions.get(return_addr_instr)
        ))),
    }
}

fn resolve_call_fixups(
    instructions: &mut [Instruction],
    call_fixups: &[(usize, CallFixup)],
    indirect_call_fixups: &[(usize, IndirectCallFixup)],
    function_offsets: &[usize],
) -> Result<(Vec<u32>, usize)> {
    // Pre-compute a prefix sum of instruction byte sizes. `byte_prefix[i]` is
    // the sum of `instructions[0..i].encode().len()`. Patching `LoadImmJump`
    // only changes the `offset` field — a fixed 4-byte field per
    // `encode_one_reg_one_imm_one_off` — and `Jump.offset` is likewise fixed
    // 4 bytes, so these prefix sums stay valid throughout the loop below and
    // through the entry-header patches in `compile_via_llvm` afterwards.
    //
    // Before this precompute existed, the loop body re-summed
    // `instructions[..=jump_idx].iter().map(|i| i.encode().len()).sum()` per
    // fixup → O(N × M) where N = #fixups, M = #instructions. On Polkadot
    // runtimes that's ~10⁹ operations + Vec allocations (encode() returns a
    // fresh Vec each time just to count bytes), which made compile times
    // unbounded once the recent backend fixes let real-world modules reach
    // this point — see issue #225.
    let mut byte_prefix: Vec<usize> = Vec::with_capacity(instructions.len() + 1);
    byte_prefix.push(0);
    let mut acc: usize = 0;
    for ins in instructions.iter() {
        acc += ins.encode().len();
        byte_prefix.push(acc);
    }

    // Count total call-return entries by finding the maximum pre-assigned index.
    // Entries are written at their pre-assigned slot so mixed direct/indirect
    // call ordering within a function is preserved correctly.
    let mut num_call_returns: usize = 0;

    for (instr_base, fixup) in call_fixups {
        let idx = return_addr_jump_table_idx(instructions, instr_base + fixup.return_addr_instr)?;
        num_call_returns = num_call_returns.max(idx + 1);
    }
    for (instr_base, fixup) in indirect_call_fixups {
        let idx = return_addr_jump_table_idx(instructions, instr_base + fixup.return_addr_instr)?;
        num_call_returns = num_call_returns.max(idx + 1);
    }

    let mut jump_table: Vec<u32> = vec![0u32; num_call_returns];

    // Call return addresses (LoadImmJump/LoadImm/LoadImmJumpInd values) are pre-assigned at emission time,
    // so we only need to compute byte offsets for the jump table and patch Jump targets.
    // Write each entry at its pre-assigned index to keep values in sync.
    for (instr_base, fixup) in call_fixups {
        let target_offset = function_offsets
            .get(fixup.target_func as usize)
            .ok_or_else(|| {
                Error::Unsupported(format!("call to unknown function {}", fixup.target_func))
            })?;

        let jump_idx = instr_base + fixup.jump_instr;

        // Return address = byte offset after the LoadImmJump instruction.
        let return_addr_offset = byte_prefix[jump_idx + 1];

        let slot = return_addr_jump_table_idx(instructions, instr_base + fixup.return_addr_instr)?;
        jump_table[slot] = return_addr_offset as u32;

        // Verify pre-assigned jump table address matches actual index.
        let expected_addr = ((slot + 1) * 2) as i32;
        debug_assert!(
            matches!(&instructions[jump_idx], Instruction::LoadImmJump { value, .. } if *value == expected_addr),
            "pre-assigned jump table address mismatch: expected {expected_addr}, got {:?}",
            &instructions[jump_idx]
        );

        // Patch the offset field of LoadImmJump.
        let jump_start_offset = byte_prefix[jump_idx];
        let relative_offset = (*target_offset as i32) - (jump_start_offset as i32);

        if let Instruction::LoadImmJump { offset, .. } = &mut instructions[jump_idx] {
            *offset = relative_offset;
        }
    }

    for (instr_base, fixup) in indirect_call_fixups {
        let jump_ind_idx = instr_base + fixup.jump_ind_instr;

        let return_addr_offset = byte_prefix[jump_ind_idx + 1];

        let slot = return_addr_jump_table_idx(instructions, instr_base + fixup.return_addr_instr)?;
        jump_table[slot] = return_addr_offset as u32;
    }

    let func_entry_base = jump_table.len();
    for &offset in function_offsets {
        jump_table.push(offset as u32);
    }

    Ok((jump_table, func_entry_base))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::OptimizationFlags;
    use super::build_rw_data;
    use super::memory_layout;
    use super::wasm_module::DataSegment;

    #[test]
    fn all_disabled_turns_off_every_optional_optimization() {
        let f = OptimizationFlags::all_disabled();
        // `llvm_passes` must stay on — the PVM backend cannot lower alloca.
        assert!(f.llvm_passes, "llvm_passes must stay enabled");
        // Every other boolean must be false. If a new optional optimization
        // is added, update `all_disabled()` and this assertion together.
        assert!(!f.peephole);
        assert!(!f.register_cache);
        assert!(!f.icmp_branch_fusion);
        assert!(!f.shrink_wrap_callee_saves);
        assert!(!f.dead_store_elimination);
        assert!(!f.constant_propagation);
        assert!(!f.inlining);
        assert!(!f.cross_block_cache);
        assert!(!f.register_allocation);
        assert!(!f.fallthrough_jumps);
        assert!(!f.aggressive_register_allocation);
        assert!(!f.allocate_scratch_regs);
        assert!(!f.allocate_caller_saved_regs);
        assert!(!f.lazy_spill);
        assert!(!f.libcall_recognition);
        assert!(!f.mergefunc);
    }

    #[test]
    fn build_rw_data_trims_all_zero_tail_to_empty() {
        let rw = build_rw_data(
            &[],
            &[],
            &[],
            0,
            0x30000,
            &BTreeMap::new(),
            &BTreeMap::new(),
            false,
        )
        .expect("valid inputs");
        assert!(rw.is_empty());
    }

    #[test]
    fn build_rw_data_skips_mem_size_slot_when_not_needed() {
        // Program with no user globals, no passive segments, no memory ops:
        // rw_data must be empty — no 4-byte memory-size slot emitted.
        let rw = build_rw_data(
            &[],
            &[],
            &[],
            16,
            0x31000,
            &BTreeMap::new(),
            &BTreeMap::new(),
            false,
        )
        .expect("valid inputs");
        assert!(rw.is_empty());
    }

    #[test]
    fn build_rw_data_emits_mem_size_slot_when_needed() {
        // Same inputs but the module uses memory.grow: 4-byte slot carries initial pages.
        // Trailing zeros are trimmed, leaving a single non-zero low byte.
        let rw = build_rw_data(
            &[],
            &[],
            &[],
            16,
            0x31000,
            &BTreeMap::new(),
            &BTreeMap::new(),
            true,
        )
        .expect("valid inputs");
        assert_eq!(rw, vec![16]);
    }

    #[test]
    fn build_rw_data_places_mem_size_before_globals() {
        // Reorder invariant: mem-size at offset 0 (value = initial pages),
        // i32 user globals shift to offsets [4..4+N*4).
        let globals: [i64; 2] = [0x11, 0x22];
        let widths: [u32; 2] = [4, 4];
        let rw = build_rw_data(
            &[],
            &globals,
            &widths,
            1, // initial_memory_pages = 1 → mem-size slot byte 0 = 0x01
            0x3000C,
            &BTreeMap::new(),
            &BTreeMap::new(),
            true,
        )
        .expect("valid inputs");
        // rw_data[0..4]  = mem-size (le u32 = 1)           = [0x01, 0, 0, 0]
        // rw_data[4..8]  = globals[0] (le i32 = 0x11)      = [0x11, 0, 0, 0]
        // rw_data[8..12] = globals[1] (le i32 = 0x22)      = [0x22, 0, 0, 0]
        // Trailing zeros trimmed from the globals[1] tail.
        assert_eq!(rw, vec![0x01, 0, 0, 0, 0x11, 0, 0, 0, 0x22]);
    }

    #[test]
    fn build_rw_data_round_trips_full_i64_global() {
        // (global i64 (i64.const 0x1122_3344_5566_7788)) — confirm all 8 bytes
        // land in rw_data without truncation when the width says 8.
        let globals: [i64; 1] = [0x1122_3344_5566_7788];
        let widths: [u32; 1] = [8];
        let rw = build_rw_data(
            &[],
            &globals,
            &widths,
            0,
            0x30008,
            &BTreeMap::new(),
            &BTreeMap::new(),
            false,
        )
        .expect("valid inputs");
        // 8 little-endian bytes of 0x1122334455667788, no trimming needed.
        assert_eq!(rw, vec![0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11]);
    }

    #[test]
    fn build_rw_data_mixed_widths_pack_without_padding() {
        // Two i32s + one i64 + one i32, all with distinct low-byte signatures.
        // Verifies width-aware packing: i32 slots take 4 bytes, i64 takes 8.
        let globals: [i64; 4] = [0xAA, 0xBB, 0x1122_3344_5566_7788, 0xCC];
        let widths: [u32; 4] = [4, 4, 8, 4];
        let rw = build_rw_data(
            &[],
            &globals,
            &widths,
            0,
            0x30014, // 4 i32s + 1 i64 = 20 bytes from base
            &BTreeMap::new(),
            &BTreeMap::new(),
            false,
        )
        .expect("valid inputs");
        // [0..4]:  i32 0xAA = [AA, 0, 0, 0]
        // [4..8]:  i32 0xBB = [BB, 0, 0, 0]
        // [8..16]: i64 0x1122334455667788 = [88, 77, 66, 55, 44, 33, 22, 11]
        // [16..20]: i32 0xCC = [CC, 0, 0, 0]
        assert_eq!(
            rw,
            vec![
                0xAA, 0, 0, 0, 0xBB, 0, 0, 0, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0xCC,
            ]
        );
    }

    #[test]
    fn build_rw_data_preserves_internal_zeros_and_trims_trailing_zeros() {
        let data_segments = vec![DataSegment {
            offset: Some(0),
            data: vec![1, 0, 2, 0, 0],
        }];

        let rw = build_rw_data(
            &data_segments,
            &[],
            &[],
            0,
            0x30000,
            &BTreeMap::new(),
            &BTreeMap::new(),
            false,
        )
        .expect("valid inputs");

        assert_eq!(rw, vec![1, 0, 2]);
    }

    #[test]
    fn build_rw_data_keeps_non_zero_passive_length_bytes() {
        let mut addrs = BTreeMap::new();
        addrs.insert(0u32, memory_layout::GLOBAL_MEMORY_BASE + 4);
        let mut lengths = BTreeMap::new();
        lengths.insert(0u32, 7u32);

        let rw =
            build_rw_data(&[], &[], &[], 0, 0x30000, &addrs, &lengths, true).expect("valid inputs");

        assert_eq!(rw, vec![0, 0, 0, 0, 7]);
    }

    #[test]
    fn build_rw_data_rejects_mismatched_parallel_arrays() {
        // Parallel-array invariant violation: surfaces as Error::Internal,
        // not as a release-build slice panic.
        let result = build_rw_data(
            &[],
            &[0x11i64, 0x22i64],
            &[4u32], // length mismatch
            0,
            0x30008,
            &BTreeMap::new(),
            &BTreeMap::new(),
            false,
        );
        assert!(matches!(result, Err(crate::Error::Internal(_))));
    }

    #[test]
    fn build_rw_data_rejects_unsupported_global_width() {
        // Width 16 (v128) is rejected even though parse normally filters it out
        // first — defense in depth against a bypassed rejection guard.
        let result = build_rw_data(
            &[],
            &[0x11i64],
            &[16u32],
            0,
            0x30010,
            &BTreeMap::new(),
            &BTreeMap::new(),
            false,
        );
        assert!(matches!(result, Err(crate::Error::Internal(_))));
    }

    // ── calculate_heap_pages tests ──

    #[test]
    fn heap_pages_with_empty_rw_data_equals_total_pages_plus_one() {
        // wasm_memory_base = 0x31000 (typical with few globals), initial_pages = 0 (clamped to 16)
        // end = 0x31000 + 16*64*1024 = 0x31000 + 0x100000 = 0x131000
        // total_bytes = 0x131000 - 0x30000 = 0x101000 = 1052672
        // total_pages = ceil(1052672 / 4096) = 257
        // rw_pages = 0, heap_pages = 257 + 1 = 258
        let pages = super::calculate_heap_pages(0, 0x31000, 0).unwrap();
        assert_eq!(pages, 258);
    }

    #[test]
    fn heap_pages_reduced_by_rw_data_pages() {
        // Same scenario but with 8192 bytes of rw_data (2 pages)
        let pages_no_rw = super::calculate_heap_pages(0, 0x31000, 0).unwrap();
        let pages_with_rw = super::calculate_heap_pages(8192, 0x31000, 0).unwrap();
        assert_eq!(pages_no_rw - pages_with_rw, 2);
    }

    #[test]
    fn heap_pages_saturates_at_one_for_large_rw_data() {
        // rw_data that covers more than total_pages still gets +1 headroom
        let pages = super::calculate_heap_pages(2 * 1024 * 1024, 0x31000, 0).unwrap();
        assert_eq!(pages, 1);
    }

    #[test]
    fn heap_pages_respects_initial_pages() {
        // initial_pages = 32 (larger than MIN_INITIAL_WASM_PAGES=16)
        // end = 0x31000 + 32*64*1024 = 0x31000 + 0x200000 = 0x231000
        // total_bytes = 0x231000 - 0x30000 = 0x201000
        // total_pages = ceil(0x201000 / 4096) = 513
        // heap_pages = 513 + 1 = 514
        let pages = super::calculate_heap_pages(0, 0x31000, 32).unwrap();
        assert_eq!(pages, 514);
    }
}
