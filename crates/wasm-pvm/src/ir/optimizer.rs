use super::IrInstruction;

/// Perform peephole optimizations on the IR.
///
/// This pass runs after the initial translation from WASM to IR and before
/// codegen. It applies simple pattern matching to improve code quality.
///
/// Current optimizations:
/// - `local.set $x; local.get $x` -> `local.tee $x`
///   (This avoids a pop/push cycle and keeps the value on the stack)
pub fn optimize(ir: &mut Vec<IrInstruction>) {
    let mut i = 0;
    while i < ir.len().saturating_sub(1) {
        if let (IrInstruction::LocalSet(set_idx), IrInstruction::LocalGet(get_idx)) =
            (&ir[i], &ir[i + 1])
            && set_idx == get_idx
        {
            // Replace `local.set $x; local.get $x` with `local.tee $x`
            ir[i] = IrInstruction::LocalTee(*set_idx);
            ir.remove(i + 1);
            // Don't increment i, check this new instruction against next
            continue;
        }
        i += 1;
    }
}
