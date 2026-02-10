// LLVM IR frontend: translates WASM operators → LLVM IR via inkwell.

mod function_builder;

pub use function_builder::WasmToLlvm;

use inkwell::context::Context;
use inkwell::module::Module;

use crate::translate::wasm_module::WasmModule;
use crate::Result;

/// Translate a parsed WASM module into an LLVM IR module.
///
/// Creates an LLVM context-scoped module with all functions and globals,
/// then runs `mem2reg` to promote alloca-based locals to SSA form.
pub fn translate_wasm_to_llvm<'ctx>(
    context: &'ctx Context,
    wasm_module: &WasmModule,
) -> Result<Module<'ctx>> {
    let translator = WasmToLlvm::new(context, "wasm_module");
    translator.translate_module(wasm_module)
}
