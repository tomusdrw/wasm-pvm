// LLVM IR frontend: translates WASM operators → LLVM IR via inkwell.

mod function_builder;
mod libcall_recognition;

pub use function_builder::WasmToLlvm;
pub use libcall_recognition::LibcallKind;

use inkwell::context::Context;
use inkwell::module::Module;

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
#[allow(clippy::fn_params_excessive_bools)]
pub fn translate_wasm_to_llvm<'ctx>(
    context: &'ctx Context,
    wasm_module: &WasmModule,
    run_llvm_passes: bool,
    run_inlining: bool,
    inline_threshold: Option<u32>,
    trap_floats: bool,
    libcall_recognition: bool,
    run_mergefunc: bool,
) -> Result<Module<'ctx>> {
    let translator = WasmToLlvm::new(context, "wasm_module", trap_floats, libcall_recognition);
    translator.translate_module(
        wasm_module,
        run_llvm_passes,
        run_inlining,
        inline_threshold,
        run_mergefunc,
    )
}
