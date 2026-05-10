//! Regression tests for #219 — phi-copy resolution with more values than the
//! available temp-register pool.
//!
//! Before the fix, both the legacy and lazy-spill phi-copy paths bailed with
//! `Unsupported: too many phi values for available temp registers` when a
//! join block needed to resolve more than 5 phi nodes from a single edge.
//! This caused real-world runtimes (asset-hub-kusama, asset-hub-polkadot,
//! bridge-hub-polkadot) to fail compilation under `--trap-floats`.
//!
//! The fix replaces the bail with a slot-based parallel-move resolver
//! that uses only TEMP1/TEMP2 and handles arbitrary numbers of copies,
//! including cycles introduced by loop-header phi swaps.

use wasm_pvm::CompileOptions;
use wasm_pvm::test_harness::*;
use wasm_pvm::translate::OptimizationFlags;

/// 8 locals diverge across both arms of an if/else: the then-arm reads from
/// memory (slot-based phi sources), the else-arm uses constants
/// (constant-copies path). After mem2reg the merge has 8 phi nodes for one
/// edge — past the 5-temp-register threshold.
const WAT_MANY_PHI_IF_ELSE: &str = r#"
    (module
        (memory 1)
        (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
            (local $a i64) (local $b i64) (local $c i64) (local $d i64)
            (local $e i64) (local $f i64) (local $g i64) (local $h i64)

            (if (i32.gt_s (local.get $args_len) (i32.const 0))
                (then
                    (local.set $a (i64.load (local.get $args_ptr)))
                    (local.set $b (i64.load offset=8 (local.get $args_ptr)))
                    (local.set $c (i64.load offset=16 (local.get $args_ptr)))
                    (local.set $d (i64.load offset=24 (local.get $args_ptr)))
                    (local.set $e (i64.load offset=32 (local.get $args_ptr)))
                    (local.set $f (i64.load offset=40 (local.get $args_ptr)))
                    (local.set $g (i64.load offset=48 (local.get $args_ptr)))
                    (local.set $h (i64.load offset=56 (local.get $args_ptr)))
                )
                (else
                    (local.set $a (i64.const 1))
                    (local.set $b (i64.const 2))
                    (local.set $c (i64.const 3))
                    (local.set $d (i64.const 4))
                    (local.set $e (i64.const 5))
                    (local.set $f (i64.const 6))
                    (local.set $g (i64.const 7))
                    (local.set $h (i64.const 8))
                )
            )

            ;; Re-store all values so LLVM keeps the locals live past the merge.
            (i64.store (local.get $args_ptr) (local.get $a))
            (i64.store offset=8  (local.get $args_ptr) (local.get $b))
            (i64.store offset=16 (local.get $args_ptr) (local.get $c))
            (i64.store offset=24 (local.get $args_ptr) (local.get $d))
            (i64.store offset=32 (local.get $args_ptr) (local.get $e))
            (i64.store offset=40 (local.get $args_ptr) (local.get $f))
            (i64.store offset=48 (local.get $args_ptr) (local.get $g))
            (i64.store offset=56 (local.get $args_ptr) (local.get $h))

            ;; Return packed (ptr=0, len=64).
            (i64.const 274877906944)
        )
    )
"#;

#[test]
fn many_phi_values_at_if_else_merge_compiles() {
    // Sanity check: verify the WAT actually produces >5 phi nodes at the
    // merge block. If LLVM ever optimises this into fewer phis, the test
    // stops exercising the parallel-move code path and should be reworked.
    let ir = dump_llvm_ir(WAT_MANY_PHI_IF_ELSE).expect("LLVM IR dump");
    let phi_count = ir.matches("= phi ").count();
    assert!(
        phi_count >= 6,
        "expected >=6 phi nodes after LLVM passes, got {phi_count}; \
         WAT no longer exercises the parallel-move resolver"
    );

    let program =
        compile_wat(WAT_MANY_PHI_IF_ELSE).expect("compilation should succeed after the fix");
    let instructions = extract_instructions(&program);
    assert!(!instructions.is_empty());
}

#[test]
fn many_phi_values_at_if_else_merge_compiles_without_lazy_spill() {
    // Disable lazy spill so the legacy phi-copy path is exercised.
    let opts = OptimizationFlags {
        lazy_spill: false,
        ..OptimizationFlags::default()
    };

    let program = compile_wat_with_options(
        WAT_MANY_PHI_IF_ELSE,
        &CompileOptions {
            optimizations: opts,
            ..CompileOptions::default()
        },
    )
    .expect("compilation should succeed after the fix");
    let instructions = extract_instructions(&program);
    assert!(!instructions.is_empty());
}

/// A loop body that cyclically rotates 6 locals (a←b, b←c, c←d, d←e, e←f,
/// f←a). After mem2reg the back-edge phi copies form a 6-cycle, exceeding
/// the 5-temp-register threshold and exercising the parallel-move cycle
/// resolver.
const WAT_MANY_PHI_LOOP_CYCLE: &str = r#"
    (module
        (memory 1)
        (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
            (local $a i64) (local $b i64) (local $c i64)
            (local $d i64) (local $e i64) (local $f i64)
            (local $i i32)
            (local $tmp_a i64) (local $tmp_b i64) (local $tmp_c i64)
            (local $tmp_d i64) (local $tmp_e i64) (local $tmp_f i64)

            (local.set $a (i64.load        (local.get $args_ptr)))
            (local.set $b (i64.load offset=8  (local.get $args_ptr)))
            (local.set $c (i64.load offset=16 (local.get $args_ptr)))
            (local.set $d (i64.load offset=24 (local.get $args_ptr)))
            (local.set $e (i64.load offset=32 (local.get $args_ptr)))
            (local.set $f (i64.load offset=40 (local.get $args_ptr)))
            (local.set $i (i32.const 0))

            (loop $L
                ;; Snapshot all 6 values before the rotation so each new value
                ;; reads the original (not the just-written) local.
                (local.set $tmp_a (local.get $b))
                (local.set $tmp_b (local.get $c))
                (local.set $tmp_c (local.get $d))
                (local.set $tmp_d (local.get $e))
                (local.set $tmp_e (local.get $f))
                (local.set $tmp_f (local.get $a))

                (local.set $a (local.get $tmp_a))
                (local.set $b (local.get $tmp_b))
                (local.set $c (local.get $tmp_c))
                (local.set $d (local.get $tmp_d))
                (local.set $e (local.get $tmp_e))
                (local.set $f (local.get $tmp_f))

                (local.set $i (i32.add (local.get $i) (i32.const 1)))
                (br_if $L (i32.lt_s (local.get $i) (local.get $args_len)))
            )

            ;; Persist final state so LLVM cannot eliminate the loop body.
            (i64.store        (local.get $args_ptr) (local.get $a))
            (i64.store offset=8  (local.get $args_ptr) (local.get $b))
            (i64.store offset=16 (local.get $args_ptr) (local.get $c))
            (i64.store offset=24 (local.get $args_ptr) (local.get $d))
            (i64.store offset=32 (local.get $args_ptr) (local.get $e))
            (i64.store offset=40 (local.get $args_ptr) (local.get $f))

            ;; Return packed (ptr=0, len=48).
            (i64.const 206158430208)
        )
    )
"#;

#[test]
fn many_phi_values_with_loop_cycle_compiles() {
    let ir = dump_llvm_ir(WAT_MANY_PHI_LOOP_CYCLE).expect("LLVM IR dump");
    let phi_count = ir.matches("= phi ").count();
    assert!(
        phi_count >= 6,
        "expected >=6 phi nodes after LLVM passes, got {phi_count}; \
         WAT no longer exercises the parallel-move resolver"
    );

    let program =
        compile_wat(WAT_MANY_PHI_LOOP_CYCLE).expect("compilation should succeed after the fix");
    let instructions = extract_instructions(&program);
    assert!(!instructions.is_empty());
}

#[test]
fn many_phi_values_with_loop_cycle_compiles_without_lazy_spill() {
    let opts = OptimizationFlags {
        lazy_spill: false,
        ..OptimizationFlags::default()
    };

    let program = compile_wat_with_options(
        WAT_MANY_PHI_LOOP_CYCLE,
        &CompileOptions {
            optimizations: opts,
            ..CompileOptions::default()
        },
    )
    .expect("compilation should succeed after the fix");
    let instructions = extract_instructions(&program);
    assert!(!instructions.is_empty());
}

/// Three independent swap-pairs in a single loop body. After mem2reg the
/// back-edge phi copies form **three disjoint 2-cycles** rather than one
/// long cycle, exercising the parallel-move resolver's ability to find and
/// emit each cycle independently. Plus the loop counter, this is 7 phis on
/// one edge.
const WAT_MANY_PHI_DISJOINT_CYCLES: &str = r#"
    (module
        (memory 1)
        (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
            (local $a i64) (local $b i64)
            (local $c i64) (local $d i64)
            (local $e i64) (local $f i64)
            (local $i i32)
            (local $tmp i64)

            (local.set $a (i64.load        (local.get $args_ptr)))
            (local.set $b (i64.load offset=8  (local.get $args_ptr)))
            (local.set $c (i64.load offset=16 (local.get $args_ptr)))
            (local.set $d (i64.load offset=24 (local.get $args_ptr)))
            (local.set $e (i64.load offset=32 (local.get $args_ptr)))
            (local.set $f (i64.load offset=40 (local.get $args_ptr)))
            (local.set $i (i32.const 0))

            (loop $L
                ;; Swap a, b
                (local.set $tmp (local.get $a))
                (local.set $a (local.get $b))
                (local.set $b (local.get $tmp))

                ;; Swap c, d
                (local.set $tmp (local.get $c))
                (local.set $c (local.get $d))
                (local.set $d (local.get $tmp))

                ;; Swap e, f
                (local.set $tmp (local.get $e))
                (local.set $e (local.get $f))
                (local.set $f (local.get $tmp))

                (local.set $i (i32.add (local.get $i) (i32.const 1)))
                (br_if $L (i32.lt_s (local.get $i) (local.get $args_len)))
            )

            ;; Persist final state so LLVM cannot fold the loop body away.
            (i64.store        (local.get $args_ptr) (local.get $a))
            (i64.store offset=8  (local.get $args_ptr) (local.get $b))
            (i64.store offset=16 (local.get $args_ptr) (local.get $c))
            (i64.store offset=24 (local.get $args_ptr) (local.get $d))
            (i64.store offset=32 (local.get $args_ptr) (local.get $e))
            (i64.store offset=40 (local.get $args_ptr) (local.get $f))

            ;; Return packed (ptr=0, len=48).
            (i64.const 206158430208)
        )
    )
"#;

#[test]
fn many_phi_values_with_disjoint_cycles_compiles() {
    let ir = dump_llvm_ir(WAT_MANY_PHI_DISJOINT_CYCLES).expect("LLVM IR dump");
    let phi_count = ir.matches("= phi ").count();
    assert!(
        phi_count >= 6,
        "expected >=6 phi nodes after LLVM passes, got {phi_count}; \
         WAT no longer exercises the parallel-move resolver"
    );

    let program = compile_wat(WAT_MANY_PHI_DISJOINT_CYCLES)
        .expect("compilation should succeed after the fix");
    let instructions = extract_instructions(&program);
    assert!(!instructions.is_empty());
}

#[test]
fn many_phi_values_with_disjoint_cycles_compiles_without_lazy_spill() {
    let opts = OptimizationFlags {
        lazy_spill: false,
        ..OptimizationFlags::default()
    };

    let program = compile_wat_with_options(
        WAT_MANY_PHI_DISJOINT_CYCLES,
        &CompileOptions {
            optimizations: opts,
            ..CompileOptions::default()
        },
    )
    .expect("compilation should succeed after the fix");
    let instructions = extract_instructions(&program);
    assert!(!instructions.is_empty());
}
