mod codegen;
pub mod memory_layout;
mod stack;
pub mod wasm_module;

use crate::pvm::Instruction;
use crate::{Error, Result, SpiProgram};

pub use codegen::CompileContext;
pub use wasm_module::WasmModule;

const ENTRY_HEADER_SIZE: usize = 10;

pub fn compile(wasm: &[u8]) -> Result<SpiProgram> {
    let module = WasmModule::parse(wasm)?;

    #[cfg(feature = "llvm-backend")]
    return compile_via_llvm(&module);

    #[cfg(not(feature = "llvm-backend"))]
    return compile_legacy(&module);
}

#[cfg(feature = "llvm-backend")]
pub fn compile_via_llvm(module: &WasmModule) -> Result<SpiProgram> {
    use crate::llvm_backend::{self, LoweringContext};
    use crate::llvm_frontend;
    use inkwell::context::Context;

    // Phase 1: WASM → LLVM IR
    let context = Context::create();
    let llvm_module = llvm_frontend::translate_wasm_to_llvm(&context, module)?;

    // Phase 2: Build lowering context
    let ctx = LoweringContext {
        wasm_memory_base: module.wasm_memory_base,
        num_globals: module.globals.len(),
        function_signatures: module.function_signatures.clone(),
        type_signatures: module.type_signatures.clone(),
        function_table: module.function_table.clone(),
        num_imported_funcs: module.num_imported_funcs as usize,
        initial_memory_pages: module.memory_limits.initial_pages,
        max_memory_pages: module.max_memory_pages,
        stack_size: memory_layout::DEFAULT_STACK_SIZE,
    };

    // Phase 3: LLVM IR → PVM bytecode for each function
    let mut all_instructions: Vec<Instruction> = Vec::new();
    let mut all_call_fixups: Vec<(usize, codegen::CallFixup)> = Vec::new();
    let mut all_indirect_call_fixups: Vec<(usize, codegen::IndirectCallFixup)> = Vec::new();
    let mut function_offsets: Vec<usize> = Vec::new();

    // Entry header (same as legacy)
    all_instructions.push(Instruction::Jump { offset: 0 });
    if module.has_secondary_entry {
        all_instructions.push(Instruction::Jump { offset: 0 });
    } else {
        all_instructions.push(Instruction::Trap);
        all_instructions.push(Instruction::Fallthrough);
        all_instructions.push(Instruction::Fallthrough);
        all_instructions.push(Instruction::Fallthrough);
        all_instructions.push(Instruction::Fallthrough);
    }

    let entry_header_bytes: usize = all_instructions.iter().map(|i| i.encode().len()).sum();
    debug_assert_eq!(
        entry_header_bytes, ENTRY_HEADER_SIZE,
        "Entry header must be exactly 10 bytes"
    );

    for local_func_idx in 0..module.functions.len() {
        let global_func_idx = module.num_imported_funcs as usize + local_func_idx;
        let fn_name = format!("wasm_func_{global_func_idx}");
        let llvm_func = llvm_module
            .get_function(&fn_name)
            .ok_or_else(|| Error::Internal(format!("missing LLVM function: {fn_name}")))?;

        let is_main = local_func_idx == module.main_func_local_idx;
        let is_secondary = module.secondary_entry_local_idx == Some(local_func_idx);
        let is_entry = is_main || is_secondary;

        let entry_returns_ptr_len = if is_main {
            module.main_returns_ptr_len
        } else if is_secondary {
            module.secondary_returns_ptr_len
        } else {
            false
        };

        // Determine result globals for entry functions using legacy convention.
        let result_globals = if is_entry && !entry_returns_ptr_len {
            match (module.result_ptr_global, module.result_len_global) {
                (Some(ptr), Some(len)) => Some((ptr, len)),
                _ => None,
            }
        } else {
            None
        };

        let func_start_offset: usize = all_instructions.iter().map(|i| i.encode().len()).sum();
        function_offsets.push(func_start_offset);

        // If entry function and there's a start function, call it first.
        if let Some(start_local_idx) = module.start_func_local_idx.filter(|_| is_entry) {
            // Save r7 and r8 to stack.
            all_instructions.push(Instruction::AddImm64 {
                dst: codegen::STACK_PTR_REG,
                src: codegen::STACK_PTR_REG,
                value: -16,
            });
            all_instructions.push(Instruction::StoreIndU64 {
                base: codegen::STACK_PTR_REG,
                src: codegen::ARGS_PTR_REG,
                offset: 0,
            });
            all_instructions.push(Instruction::StoreIndU64 {
                base: codegen::STACK_PTR_REG,
                src: codegen::ARGS_LEN_REG,
                offset: 8,
            });

            // Call start function.
            let current_instr_idx = all_instructions.len();
            all_instructions.push(Instruction::LoadImm64 {
                reg: codegen::RETURN_ADDR_REG,
                value: 0,
            });
            all_instructions.push(Instruction::Jump { offset: 0 });

            all_call_fixups.push((
                current_instr_idx,
                codegen::CallFixup {
                    target_func: start_local_idx as u32,
                    return_addr_instr: 0,
                    jump_instr: 1,
                },
            ));

            // Restore r7 and r8.
            all_instructions.push(Instruction::LoadIndU64 {
                dst: codegen::ARGS_PTR_REG,
                base: codegen::STACK_PTR_REG,
                offset: 0,
            });
            all_instructions.push(Instruction::LoadIndU64 {
                dst: codegen::ARGS_LEN_REG,
                base: codegen::STACK_PTR_REG,
                offset: 8,
            });
            all_instructions.push(Instruction::AddImm64 {
                dst: codegen::STACK_PTR_REG,
                src: codegen::STACK_PTR_REG,
                value: 16,
            });
        }

        let translation = llvm_backend::lower_function(
            llvm_func,
            &ctx,
            is_entry,
            global_func_idx,
            result_globals,
        )?;

        let instr_base = all_instructions.len();
        for fixup in translation.call_fixups {
            all_call_fixups.push((
                instr_base,
                codegen::CallFixup {
                    return_addr_instr: fixup.return_addr_instr,
                    jump_instr: fixup.jump_instr,
                    target_func: fixup.target_func,
                },
            ));
        }
        for fixup in translation.indirect_call_fixups {
            all_indirect_call_fixups.push((
                instr_base,
                codegen::IndirectCallFixup {
                    return_addr_instr: fixup.return_addr_instr,
                    jump_ind_instr: fixup.jump_ind_instr,
                },
            ));
        }

        all_instructions.extend(translation.instructions);
    }

    // Phase 4: Resolve call fixups and build jump table.
    let (jump_table, func_entry_jump_table_base) = resolve_call_fixups(
        &mut all_instructions,
        &all_call_fixups,
        &all_indirect_call_fixups,
        &function_offsets,
    )?;

    // Patch entry header jumps.
    let main_offset = function_offsets[module.main_func_local_idx] as i32;
    if let Instruction::Jump { offset } = &mut all_instructions[0] {
        *offset = main_offset;
    }

    if let Some(secondary_idx) = module.secondary_entry_local_idx {
        let secondary_offset = function_offsets[secondary_idx] as i32 - 5;
        if let Instruction::Jump { offset } = &mut all_instructions[1] {
            *offset = secondary_offset;
        }
    }

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

    let blob = crate::pvm::ProgramBlob::new(all_instructions).with_jump_table(jump_table);
    let rw_data_section = build_rw_data(
        &module.data_segments,
        &module.global_init_values,
        module.memory_limits.initial_pages,
        module.wasm_memory_base,
    );

    Ok(SpiProgram::new(blob)
        .with_heap_pages(module.heap_pages)
        .with_ro_data(ro_data)
        .with_rw_data(rw_data_section))
}

pub fn compile_legacy(module: &WasmModule) -> Result<SpiProgram> {
    let mut all_instructions: Vec<Instruction> = Vec::new();
    let mut all_call_fixups: Vec<(usize, codegen::CallFixup)> = Vec::new();
    let mut all_indirect_call_fixups: Vec<(usize, codegen::IndirectCallFixup)> = Vec::new();
    let mut function_offsets: Vec<usize> = Vec::new();

    all_instructions.push(Instruction::Jump { offset: 0 });

    if module.has_secondary_entry {
        all_instructions.push(Instruction::Jump { offset: 0 });
    } else {
        all_instructions.push(Instruction::Trap);
        all_instructions.push(Instruction::Fallthrough);
        all_instructions.push(Instruction::Fallthrough);
        all_instructions.push(Instruction::Fallthrough);
        all_instructions.push(Instruction::Fallthrough);
    }

    let entry_header_bytes: usize = all_instructions.iter().map(|i| i.encode().len()).sum();
    debug_assert_eq!(
        entry_header_bytes, ENTRY_HEADER_SIZE,
        "Entry header must be exactly 10 bytes"
    );

    for (local_func_idx, func) in module.functions.iter().enumerate() {
        let global_func_idx = module.num_imported_funcs as usize + local_func_idx;

        let (num_params, has_return) = module
            .function_signatures
            .get(global_func_idx)
            .copied()
            .unwrap_or((0, false));

        let is_main = local_func_idx == module.main_func_local_idx;
        let is_secondary_entry = module.secondary_entry_local_idx == Some(local_func_idx);
        let is_entry_func = is_main || is_secondary_entry;

        let entry_returns_ptr_len = if is_main {
            module.main_returns_ptr_len
        } else if is_secondary_entry {
            module.secondary_returns_ptr_len
        } else {
            false
        };

        let ctx = CompileContext {
            num_params,
            num_locals: 0,
            num_globals: module.globals.len(),
            result_ptr_global: if is_entry_func && !entry_returns_ptr_len {
                module.result_ptr_global
            } else {
                None
            },
            result_len_global: if is_entry_func && !entry_returns_ptr_len {
                module.result_len_global
            } else {
                None
            },
            is_main: is_entry_func,
            has_return,
            entry_returns_ptr_len,
            function_offsets: vec![],
            function_signatures: module.function_signatures.clone(),
            func_idx: global_func_idx,
            function_table: module.function_table.clone(),
            type_signatures: module.type_signatures.clone(),
            num_imported_funcs: module.num_imported_funcs as usize,
            imported_func_names: module.imported_func_names.clone(),
            stack_size: memory_layout::DEFAULT_STACK_SIZE,
            initial_memory_pages: module.memory_limits.initial_pages,
            max_memory_pages: module.max_memory_pages,
            wasm_memory_base: module.wasm_memory_base,
        };

        let func_start_offset: usize = all_instructions.iter().map(|i| i.encode().len()).sum();
        function_offsets.push(func_start_offset);

        // If this is an entry function and we have a start function, execute it first.
        if let Some(start_local_idx) = module.start_func_local_idx.filter(|_| is_entry_func) {
            // Save r7 and r8 to stack
            all_instructions.push(Instruction::AddImm64 {
                dst: codegen::STACK_PTR_REG,
                src: codegen::STACK_PTR_REG,
                value: -16,
            });
            all_instructions.push(Instruction::StoreIndU64 {
                base: codegen::STACK_PTR_REG,
                src: codegen::ARGS_PTR_REG,
                offset: 0,
            });
            all_instructions.push(Instruction::StoreIndU64 {
                base: codegen::STACK_PTR_REG,
                src: codegen::ARGS_LEN_REG,
                offset: 8,
            });

            // Call start function
            let current_instr_idx = all_instructions.len();
            all_instructions.push(Instruction::LoadImm64 {
                reg: codegen::RETURN_ADDR_REG,
                value: 0,
            });
            all_instructions.push(Instruction::Jump { offset: 0 });

            let call_fixup = codegen::CallFixup {
                target_func: start_local_idx as u32,
                return_addr_instr: 0,
                jump_instr: 1,
            };
            all_call_fixups.push((current_instr_idx, call_fixup));

            // Restore r7 and r8 from stack
            all_instructions.push(Instruction::LoadIndU64 {
                dst: codegen::ARGS_PTR_REG,
                base: codegen::STACK_PTR_REG,
                offset: 0,
            });
            all_instructions.push(Instruction::LoadIndU64 {
                dst: codegen::ARGS_LEN_REG,
                base: codegen::STACK_PTR_REG,
                offset: 8,
            });
            all_instructions.push(Instruction::AddImm64 {
                dst: codegen::STACK_PTR_REG,
                src: codegen::STACK_PTR_REG,
                value: 16,
            });
        }

        let translation = codegen::translate_function(func, &ctx)?;

        let instr_base = all_instructions.len();
        for fixup in translation.call_fixups {
            all_call_fixups.push((instr_base, fixup));
        }
        for fixup in translation.indirect_call_fixups {
            all_indirect_call_fixups.push((instr_base, fixup));
        }

        all_instructions.extend(translation.instructions);
    }

    let (jump_table, func_entry_jump_table_base) = resolve_call_fixups(
        &mut all_instructions,
        &all_call_fixups,
        &all_indirect_call_fixups,
        &function_offsets,
    )?;

    let main_offset = function_offsets[module.main_func_local_idx] as i32;
    if let Instruction::Jump { offset } = &mut all_instructions[0] {
        *offset = main_offset;
    }

    if let Some(secondary_idx) = module.secondary_entry_local_idx {
        let secondary_offset = function_offsets[secondary_idx] as i32 - 5;
        if let Instruction::Jump { offset } = &mut all_instructions[1] {
            *offset = secondary_offset;
        }
    }

    // Build dispatch table for call_indirect
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
    let blob = crate::pvm::ProgramBlob::new(all_instructions).with_jump_table(jump_table);

    let _ = module.max_memory_pages; // silence unused variable warning
    let rw_data_section = build_rw_data(
        &module.data_segments,
        &module.global_init_values,
        module.memory_limits.initial_pages,
        module.wasm_memory_base,
    );

    Ok(SpiProgram::new(blob)
        .with_heap_pages(module.heap_pages)
        .with_ro_data(ro_data)
        .with_rw_data(rw_data_section))
}

/// Build the `rw_data` section from WASM data segments and global initializers.
pub(crate) fn build_rw_data(
    data_segments: &[wasm_module::DataSegment],
    global_init_values: &[i32],
    initial_memory_pages: u32,
    wasm_memory_base: i32,
) -> Vec<u8> {
    // Calculate the minimum size needed for globals
    // +1 for the compiler-managed memory size global
    let globals_end = (global_init_values.len() + 1) * 4;

    // Calculate the size needed for data segments
    let wasm_to_rw_offset = wasm_memory_base as u32 - 0x30000;

    let data_end = data_segments
        .iter()
        .map(|seg| wasm_to_rw_offset + seg.offset + seg.data.len() as u32)
        .max()
        .unwrap_or(0) as usize;

    let total_size = globals_end.max(data_end);

    if total_size == 0 {
        return Vec::new();
    }

    let mut rw_data = vec![0u8; total_size];

    // Initialize user globals
    for (i, &value) in global_init_values.iter().enumerate() {
        let offset = i * 4;
        if offset + 4 <= rw_data.len() {
            rw_data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }

    // Initialize compiler-managed memory size global (right after user globals)
    let mem_size_offset = global_init_values.len() * 4;
    if mem_size_offset + 4 <= rw_data.len() {
        rw_data[mem_size_offset..mem_size_offset + 4]
            .copy_from_slice(&initial_memory_pages.to_le_bytes());
    }

    // Copy data segments to their WASM memory locations
    for seg in data_segments {
        let rw_offset = (wasm_to_rw_offset + seg.offset) as usize;
        if rw_offset + seg.data.len() <= rw_data.len() {
            rw_data[rw_offset..rw_offset + seg.data.len()].copy_from_slice(&seg.data);
        }
    }

    rw_data
}

fn resolve_call_fixups(
    instructions: &mut [Instruction],
    call_fixups: &[(usize, codegen::CallFixup)],
    indirect_call_fixups: &[(usize, codegen::IndirectCallFixup)],
    function_offsets: &[usize],
) -> Result<(Vec<u32>, usize)> {
    let mut jump_table: Vec<u32> = Vec::new();

    for (instr_base, fixup) in call_fixups {
        let target_offset = function_offsets
            .get(fixup.target_func as usize)
            .ok_or_else(|| {
                Error::Unsupported(format!("call to unknown function {}", fixup.target_func))
            })?;

        let return_addr_idx = instr_base + fixup.return_addr_instr;
        let jump_idx = instr_base + fixup.jump_instr;

        let return_addr_offset: usize = instructions[..=jump_idx]
            .iter()
            .map(|i| i.encode().len())
            .sum();

        let jump_table_index = jump_table.len();
        jump_table.push(return_addr_offset as u32);

        let jump_table_address = (jump_table_index as u64 + 1) * 2;

        if let Instruction::LoadImm64 { value, .. } = &mut instructions[return_addr_idx] {
            *value = jump_table_address;
        }

        let jump_start_offset: usize = instructions[..jump_idx]
            .iter()
            .map(|i| i.encode().len())
            .sum();
        let relative_offset = (*target_offset as i32) - (jump_start_offset as i32);

        if let Instruction::Jump { offset } = &mut instructions[jump_idx] {
            *offset = relative_offset;
        }
    }

    for (instr_base, fixup) in indirect_call_fixups {
        let return_addr_idx = instr_base + fixup.return_addr_instr;
        let jump_ind_idx = instr_base + fixup.jump_ind_instr;

        let return_addr_offset: usize = instructions[..=jump_ind_idx]
            .iter()
            .map(|i| i.encode().len())
            .sum();

        let jump_table_index = jump_table.len();
        jump_table.push(return_addr_offset as u32);

        let jump_table_address = (jump_table_index as u64 + 1) * 2;

        if let Instruction::LoadImm64 { value, .. } = &mut instructions[return_addr_idx] {
            *value = jump_table_address;
        }
    }

    let func_entry_base = jump_table.len();
    for &offset in function_offsets {
        jump_table.push(offset as u32);
    }

    Ok((jump_table, func_entry_base))
}
