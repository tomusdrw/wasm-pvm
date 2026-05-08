//! Regression tests for `ControlFrame::Loop` `End` handling when the body
//! ends in unreachable state (e.g. `loop { return …; br 0 }`).
//!
//! When a loop body never falls through to its `End`, the loop's `merge_bb`
//! has zero predecessors. The compiler must keep its `unreachable` flag set
//! after the loop's `End` so that subsequent operators are correctly treated
//! as dead code; otherwise it ends up trying to emit reachable LLVM IR
//! against a phantom block and fails (in particular, the function-level
//! `End` calls `pop()` on an empty operand stack).
//!
//! The same bug also affects the dead-code dispatcher's "dummy" `Block` /
//! `If` frames whose `merge_bb` reuses the current — already terminated —
//! block: their matching `End`/`Else` handlers must not reposition there or
//! reset `unreachable=false`.

use wasm_pvm::test_harness::*;

/// Direct repro of the polkadot/hashbrown failure shape: an outer loop whose
/// body ends unreachable (`return … ; br 0`), immediately followed by a
/// second loop whose `End` propagates result types onto the (empty) operand
/// stack via the function-level `End`.
///
/// Before the fix this trips `Internal error: operand stack underflow`.
#[test]
fn unreachable_loop_followed_by_result_loop_compiles() {
    let wat = r"
        (module
            (func (result i32)
                (loop $L
                    (return (i32.const 42))
                    (br $L)
                )
                (loop $L2 (result i32)
                    (i32.const 0)
                )
            )
        )
    ";

    compile_wat(wat).expect("compilation should succeed after the fix");
}

/// The user-supplied minimal WAT from the bug report. The trailing
/// `unreachable` operator masks the bug (it re-flips `self.unreachable=true`
/// before the function-level `End`), so this case happens to compile even
/// without the fix — it's kept here as a permanent regression guard against
/// a regression that breaks the masking path.
#[test]
fn unreachable_loop_followed_by_explicit_unreachable_compiles() {
    let wat = r"
        (module
            (func (result i32) (local i32)
                (loop $L
                    (return (i32.const 42))
                    (br $L)
                )
                unreachable
            )
        )
    ";

    compile_wat(wat).expect("compilation should succeed");
}

/// Isolated regression for the dead-code-dispatcher dummy-frame fix
/// (independent of the `Loop` `End` fix): the top-level `unreachable`
/// terminates `entry_bb`, then the `block (result i32)` frame is pushed
/// as a dummy whose `merge_bb` is exactly that terminated `entry_bb`.
/// Without the dummy guard, the block's `End` repositions there and resets
/// `unreachable=false`, so the function-level `End` then tries to `pop()` a
/// phi value from an empty operand stack.
#[test]
fn dummy_block_frame_with_terminated_merge_compiles() {
    let wat = r"
        (module
            (func (result i32)
                unreachable
                (block (result i32)
                    unreachable
                )
            )
        )
    ";

    compile_wat(wat).expect("compilation should succeed");
}
