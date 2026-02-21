// LLVM IR frontend: translates WASM operators → LLVM IR via inkwell.

mod function_builder;

pub use function_builder::WasmToLlvm;

use inkwell::context::Context;
use inkwell::module::Module;

use crate::Result;
use crate::translate::wasm_module::WasmModule;

/// Translate a parsed WASM module into an LLVM IR module.
///
/// Creates an LLVM context-scoped module with all functions and globals,
/// then optionally runs LLVM optimization passes (mem2reg, instcombine, simplifycfg, gvn, dce).
pub fn translate_wasm_to_llvm<'ctx>(
    context: &'ctx Context,
    wasm_module: &WasmModule,
    run_llvm_passes: bool,
    run_inlining: bool,
) -> Result<Module<'ctx>> {
    let translator = WasmToLlvm::new(context, "wasm_module");
    translator.translate_module(wasm_module, run_llvm_passes, run_inlining)
}
