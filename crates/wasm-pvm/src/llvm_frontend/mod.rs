// LLVM IR frontend: translates WASM operators → LLVM IR via inkwell.

mod function_builder;

pub use function_builder::WasmToLlvm;

use inkwell::context::Context;
use inkwell::module::Module;

use std::collections::HashSet;

use crate::Result;
use crate::translate::wasm_module::WasmModule;

/// Translate a parsed WASM module into an LLVM IR module.
///
/// Creates an LLVM context-scoped module with all functions and globals,
/// then optionally runs LLVM optimization passes in three phases:
/// - Phase 1 (pre-inline cleanup): `mem2reg`, `instcombine`, `simplifycfg`
/// - Phase 2 (inlining, controlled by `run_inlining`): `cgscc(inline)` with default threshold 225
/// - Phase 3 (post-inline cleanup): `instcombine<max-iterations=2>`, `simplifycfg`, `gvn`, `dce`
///
/// `run_llvm_passes` gates the entire optimization pipeline (all three phases).
/// `run_inlining` enables/disables Phase 2 independently (requires `run_llvm_passes = true`).
/// `reachable_locals` when `Some`, limits translation to only those local function indices.
#[allow(clippy::implicit_hasher)]
pub fn translate_wasm_to_llvm<'ctx>(
    context: &'ctx Context,
    wasm_module: &WasmModule,
    run_llvm_passes: bool,
    run_inlining: bool,
    reachable_locals: Option<&HashSet<usize>>,
) -> Result<Module<'ctx>> {
    let translator = WasmToLlvm::new(context, "wasm_module");
    translator.translate_module(wasm_module, run_llvm_passes, run_inlining, reachable_locals)
}
