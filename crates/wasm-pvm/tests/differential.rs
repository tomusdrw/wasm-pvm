//! Differential tests: compile WAT fixtures through both legacy and LLVM backends,
//! verify both succeed and compare structural properties.
//!
//! These tests require the `llvm-backend` feature to be enabled so both
//! pipelines are available simultaneously.

#![cfg(feature = "llvm-backend")]

use wasm_pvm::translate::{WasmModule, compile_legacy, compile_via_llvm};

/// Read a WAT fixture file and parse to WASM bytes.
fn load_wat_fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../tests/fixtures/wat/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let wat = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    wat::parse_str(&wat).unwrap_or_else(|e| panic!("parse WAT {name}: {e}"))
}

/// Compile a WAT fixture through both backends and compare.
fn differential_compile(fixture_name: &str) {
    let wasm = load_wat_fixture(fixture_name);
    let module = WasmModule::parse(&wasm)
        .unwrap_or_else(|e| panic!("{fixture_name}: WasmModule::parse failed: {e}"));

    let legacy = compile_legacy(&module)
        .unwrap_or_else(|e| panic!("{fixture_name}: legacy backend failed: {e}"));
    let llvm = compile_via_llvm(&module)
        .unwrap_or_else(|e| panic!("{fixture_name}: LLVM backend failed: {e}"));

    // Structural comparisons (instruction sequences will differ)
    assert_eq!(
        legacy.heap_pages(),
        llvm.heap_pages(),
        "{fixture_name}: heap_pages mismatch"
    );
    assert_eq!(
        legacy.rw_data(),
        llvm.rw_data(),
        "{fixture_name}: rw_data mismatch (data segments / globals)"
    );

    // ro_data contains the dispatch table — both should have the same number of entries
    // (8 bytes per entry: 4-byte jump_ref + 4-byte type_idx)
    let legacy_entries = legacy.ro_data().len() / 8;
    let llvm_entries = llvm.ro_data().len() / 8;
    assert_eq!(
        legacy_entries, llvm_entries,
        "{fixture_name}: dispatch table entry count mismatch (legacy={legacy_entries}, llvm={llvm_entries})"
    );

    // Both should produce non-empty instruction sequences
    assert!(
        !legacy.code().instructions().is_empty(),
        "{fixture_name}: legacy produced empty code"
    );
    assert!(
        !llvm.code().instructions().is_empty(),
        "{fixture_name}: LLVM produced empty code"
    );
}

// ============================================================================
// Layer 1 — Core fixtures (must all pass)
// ============================================================================

#[test]
fn diff_add() {
    differential_compile("add.jam.wat");
}

#[test]
fn diff_factorial() {
    differential_compile("factorial.jam.wat");
}

#[test]
fn diff_fibonacci() {
    differential_compile("fibonacci.jam.wat");
}

#[test]
fn diff_gcd() {
    differential_compile("gcd.jam.wat");
}

#[test]
fn diff_start_section() {
    differential_compile("start-section.jam.wat");
}

// ============================================================================
// Layer 2 — Intermediate fixtures
// ============================================================================

#[test]
fn diff_bit_ops() {
    differential_compile("bit-ops.jam.wat");
}

#[test]
fn diff_block_br_test() {
    differential_compile("block-br-test.jam.wat");
}

#[test]
fn diff_block_result() {
    differential_compile("block-result.jam.wat");
}

#[test]
fn diff_br_table() {
    differential_compile("br-table.jam.wat");
}

#[test]
fn diff_br_table_spill_test() {
    differential_compile("br-table-spill-test.jam.wat");
}

#[test]
fn diff_call() {
    differential_compile("call.jam.wat");
}

#[test]
fn diff_call_indirect() {
    differential_compile("call-indirect.jam.wat");
}

#[test]
fn diff_call_indirect_test() {
    differential_compile("call-indirect-test.jam.wat");
}

#[test]
fn diff_compare_test() {
    differential_compile("compare-test.jam.wat");
}

#[test]
fn diff_computed_addr_test() {
    differential_compile("computed-addr-test.jam.wat");
}

#[test]
fn diff_div() {
    differential_compile("div.jam.wat");
}

#[test]
fn diff_entry_points() {
    differential_compile("entry-points.jam.wat");
}

#[test]
fn diff_i64_ops() {
    differential_compile("i64-ops.jam.wat");
}

#[test]
fn diff_is_prime() {
    differential_compile("is-prime.jam.wat");
}

#[test]
fn diff_life_init_test() {
    differential_compile("life-init-test.jam.wat");
}

#[test]
fn diff_life_simple() {
    differential_compile("life-simple.jam.wat");
}

#[test]
fn diff_loop_if_pattern() {
    differential_compile("loop-if-pattern.jam.wat");
}

#[test]
fn diff_loop_memory_test() {
    differential_compile("loop-memory-test.jam.wat");
}

#[test]
fn diff_loop_offset_store_test() {
    differential_compile("loop-offset-store-test.jam.wat");
}

#[test]
fn diff_many_locals() {
    differential_compile("many-locals.jam.wat");
}

#[test]
fn diff_many_locals_call_test() {
    differential_compile("many-locals-call-test.jam.wat");
}

#[test]
fn diff_memory_grow_test() {
    differential_compile("memory-grow-test.jam.wat");
}

#[test]
fn diff_memory_loop_test() {
    differential_compile("memory-loop-test.jam.wat");
}

#[test]
fn diff_nested_calls() {
    differential_compile("nested-calls.jam.wat");
}

#[test]
fn diff_nested_loop_lt_test() {
    differential_compile("nested-loop-lt-test.jam.wat");
}

#[test]
fn diff_nested_loop_test() {
    differential_compile("nested-loop-test.jam.wat");
}

#[test]
fn diff_nested_memory_4locals() {
    differential_compile("nested-memory-4locals.jam.wat");
}

#[test]
fn diff_nested_memory_6locals() {
    differential_compile("nested-memory-6locals.jam.wat");
}

#[test]
fn diff_nested_memory_11locals() {
    differential_compile("nested-memory-11locals.jam.wat");
}

#[test]
fn diff_recursive() {
    differential_compile("recursive.jam.wat");
}

#[test]
fn diff_rotate() {
    differential_compile("rotate.jam.wat");
}

#[test]
fn diff_select_loop_test() {
    differential_compile("select-loop-test.jam.wat");
}

#[test]
fn diff_simple_memory_test() {
    differential_compile("simple-memory-test.jam.wat");
}

#[test]
fn diff_spilled_local_store_test() {
    differential_compile("spilled-local-store-test.jam.wat");
}

#[test]
fn diff_stack_overflow() {
    differential_compile("stack-overflow.jam.wat");
}

#[test]
fn diff_stack_overflow_trap() {
    differential_compile("stack-overflow-trap.jam.wat");
}

#[test]
fn diff_stack_test() {
    differential_compile("stack-test.jam.wat");
}

#[test]
fn diff_store_offset_test() {
    differential_compile("store-offset-test.jam.wat");
}
