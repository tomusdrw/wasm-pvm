//! Tests for float-operator diagnostics and `--trap-floats` mode.
//!
//! PVM has no floating-point instructions; the compiler rejects any f32/f64
//! operator by default. These tests verify two related behaviours:
//!
//! 1. **Diagnostics**: when compilation fails on an unsupported operator, the
//!    error message identifies the function (index + display name) and the
//!    byte offset of the operator within the function body.
//!
//! 2. **`--trap-floats`** (`CompileOptions::trap_floats` = true): every f32/f64
//!    operator is replaced with a runtime trap so the rest of the function (and
//!    the rest of the module) can still be compiled. This lets users find out
//!    what *other* unsupported features a module uses past the float wall.

use wasm_pvm::test_harness::*;
use wasm_pvm::{CompileOptions, Error, Opcode};

// ──────────────────────────────────────────────────────────────────────────
// Diagnostics: location info on failure
// ──────────────────────────────────────────────────────────────────────────

/// Compiling a module with `f64.add` should fail with a `Located` error that
/// names the function and the operator's byte offset. When the WAT source
/// gives the function an explicit identifier (`$name`), that identifier
/// becomes a name-section entry and the diagnostic prefers it over the export
/// alias — name section is the canonical debug name.
#[test]
fn float_op_fails_with_function_name_and_offset() {
    let wat = r#"
        (module
            (func $float_user (export "main") (param f64 f64) (result f64)
                local.get 0
                local.get 1
                f64.add
            )
        )
    "#;

    let err = compile_wat(wat)
        .err()
        .expect("expected compilation to fail on f64.add");
    let Error::Located {
        func_idx,
        func_name,
        op_offset,
        cause,
    } = err
    else {
        panic!("expected Error::Located, got something else");
    };

    assert_eq!(func_idx, 0, "only one function exists in this module");
    assert_eq!(
        func_name, "float_user",
        "name section ($float_user) should win over export alias (main)"
    );
    assert!(
        op_offset.is_some_and(|o| o > 0),
        "op offset should point into function body, got {op_offset:?}"
    );

    let msg = cause.to_string();
    assert!(
        msg.contains("Unsupported") || msg.contains("Float"),
        "inner error should describe the unsupported operator, got: {msg}"
    );
}

/// When the WAT has only an export name (no `$identifier`), the diagnostic
/// falls back to the export name.
#[test]
fn export_name_fallback_when_name_section_empty() {
    let wat = r#"
        (module
            (func (export "main") (param f64 f64) (result f64)
                local.get 0
                local.get 1
                f64.add
            )
        )
    "#;
    let err = compile_wat(wat)
        .err()
        .expect("expected compilation to fail on f64.add");
    let Error::Located { func_name, .. } = err else {
        panic!("expected Error::Located");
    };
    assert_eq!(func_name, "main");
}

/// Display string for a `Located` error must include the function name and
/// hex offset, so users can grep for the location in CI logs.
#[test]
fn located_error_display_includes_context() {
    let wat = r#"
        (module
            (func (export "main") (param f64 f64) (result f64)
                local.get 0
                local.get 1
                f64.add
            )
        )
    "#;

    let err = compile_wat(wat)
        .err()
        .expect("expected compilation to fail");
    let display = err.to_string();
    assert!(
        display.contains("'main'"),
        "display should mention function name, got: {display}"
    );
    assert!(
        display.contains("byte offset 0x"),
        "display should include hex byte offset, got: {display}"
    );
}

/// Non-float unsupported features (e.g. `data.drop`) should also receive a
/// `Located` wrapper — the diagnostic improvement isn't float-specific.
#[test]
fn non_float_unsupported_op_also_gets_location() {
    let wat = r#"
        (module
            (memory 1)
            (data $d0 "hello")
            (func (export "main") (result i32)
                data.drop $d0
                i32.const 0
            )
        )
    "#;

    let err = compile_wat(wat)
        .err()
        .expect("expected compilation to fail on data.drop");
    let Error::Located {
        func_name,
        op_offset,
        cause,
        ..
    } = err
    else {
        panic!("expected Error::Located, got: {err:?}");
    };
    assert_eq!(func_name, "main");
    assert!(
        op_offset.is_some(),
        "frontend errors carry a byte offset, got {op_offset:?}"
    );
    assert!(
        cause.to_string().contains("data.drop"),
        "inner error should mention data.drop, got: {cause}"
    );
}

/// Errors raised during PVM lowering (after LLVM IR has been built) must also
/// be wrapped in `Error::Located`, so users see which function rejected the
/// program. `op_offset` is `None` because the WASM byte offset is no longer
/// recoverable at that point — the diagnostic explicitly says "during PVM
/// lowering" instead.
///
/// Trigger: `host_call_5` is recognised by name in the backend, where it's
/// validated to require an `(result i64)` signature. Declaring it without one
/// fires an `Error::Unsupported` from `llvm_backend::calls::lower_host_call_variant`
/// — long after the frontend's operator-byte-offset wrapper has had a chance.
#[test]
fn backend_error_includes_function_context() {
    let wat = r#"
        (module
            (import "env" "host_call_5"
                (func $hc5 (param i64 i64 i64 i64 i64 i64)))
            (func $caller (export "main") (result i32)
                (call $hc5
                    (i64.const 1) (i64.const 2) (i64.const 3)
                    (i64.const 4) (i64.const 5) (i64.const 6))
                (i32.const 0)
            )
        )
    "#;

    let err = compile_wat(wat)
        .err()
        .expect("expected backend rejection of host_call_5 with no result");
    let Error::Located {
        func_name,
        op_offset,
        cause,
        ..
    } = err
    else {
        panic!("expected Error::Located, got: {err:?}");
    };
    assert_eq!(
        func_name, "caller",
        "func_name should be the WASM-aware display name, not the LLVM symbol"
    );
    assert!(
        op_offset.is_none(),
        "backend errors don't have a WASM byte offset, got {op_offset:?}"
    );
    assert!(
        cause.to_string().contains("host_call_5"),
        "inner error should mention host_call_5, got: {cause}"
    );

    // Display string must clearly indicate the lowering phase.
    let display = Error::Located {
        func_idx: 1,
        func_name: "caller".into(),
        op_offset: None,
        cause: Box::new(Error::Unsupported("synthetic".into())),
    }
    .to_string();
    assert!(
        display.contains("during PVM lowering"),
        "display should distinguish backend errors, got: {display}"
    );
}

/// Errors raised in adapter-merge — before any LLVM IR exists — must be
/// wrapped in `Error::AdapterMerge` with a context naming the offending
/// adapter element. The trigger here is a float constant inside an adapter
/// function body, which `encode_passthrough_operator` rejects.
#[test]
fn adapter_merge_error_includes_context() {
    let main_wat = r#"
        (module
            (import "env" "abort" (func $abort))
            (func (export "main") (result i32)
                (call $abort)
                (i32.const 0)
            )
        )
    "#;
    let adapter_wat = r#"
        (module
            (func (export "abort")
                f64.const 1.0
                drop
            )
        )
    "#;

    let opts = CompileOptions {
        adapter: Some(adapter_wat.to_string()),
        ..CompileOptions::default()
    };
    let err = compile_wat_with_options(main_wat, &opts)
        .err()
        .expect("expected adapter-merge rejection of f64.const");
    let Error::AdapterMerge { context, cause } = err else {
        panic!("expected Error::AdapterMerge, got: {err:?}");
    };
    assert!(
        context.contains("adapter func"),
        "context should identify the adapter function, got: {context}"
    );
    assert!(
        cause.to_string().contains("floating point"),
        "inner error should mention float rejection, got: {cause}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// --trap-floats: float ops compile to runtime traps
// ──────────────────────────────────────────────────────────────────────────

/// With `trap_floats: true`, `f64.add` no longer fails compilation. The module
/// produces valid PVM bytecode containing at least one `Trap` instruction
/// (from the float operator) plus the entry-header trap.
#[test]
fn trap_floats_lets_f64_add_compile() {
    let wat = r#"
        (module
            (func (export "main") (param i64) (result i64)
                f64.const 1.0
                f64.const 2.0
                f64.add
                drop
                local.get 0
            )
        )
    "#;

    let opts = CompileOptions {
        trap_floats: true,
        ..CompileOptions::default()
    };
    let program = compile_wat_with_options(wat, &opts).expect("trap-floats should compile");
    let instructions = extract_instructions(&program);

    let trap_count = count_opcode(&instructions, Opcode::Trap);
    // At minimum: the entry-header trap (placeholder when no secondary entry)
    // plus one trap per float operator we converted (3 here: two consts + add).
    // The exact count depends on optimizer DCE, but >= 2 is a solid lower bound:
    // entry-header trap + at least one float-op trap.
    assert!(
        trap_count >= 2,
        "expected at least 2 Trap instructions, got {trap_count} in {instructions:?}"
    );
}

/// Trap-floats should compile every float-flavoured operator we support, not
/// just the binary ones. This module exercises a const, a binop, a comparison,
/// a unary op, and a conversion in a single function — each must be replaced
/// with a trap without breaking operand-stack tracking.
#[test]
fn trap_floats_handles_all_operator_families() {
    let wat = r#"
        (module
            (func (export "main") (param i64) (result i64)
                ;; constant + binop
                f64.const 1.5
                f64.const 2.5
                f64.add
                drop
                ;; unary
                f32.const 3.0
                f32.sqrt
                drop
                ;; compare → i32
                f64.const 1.0
                f64.const 2.0
                f64.lt
                drop
                ;; conversion (i32 → f64 → i32)
                i32.const 42
                f64.convert_i32_s
                i32.trunc_f64_s
                drop
                ;; result
                local.get 0
            )
        )
    "#;

    let opts = CompileOptions {
        trap_floats: true,
        ..CompileOptions::default()
    };
    let program =
        compile_wat_with_options(wat, &opts).expect("trap-floats should handle all op families");
    let instructions = extract_instructions(&program);
    assert!(
        has_opcode(&instructions, Opcode::Trap),
        "expected a Trap from the float operators"
    );
}

/// A function with an f32/f64 result type should still compile cleanly under
/// `--trap-floats`. The function body traps before producing the (placeholder)
/// return value, so the LLVM IR for the function-end phi must remain valid.
#[test]
fn trap_floats_function_returning_float_compiles() {
    let wat = r#"
        (module
            (func (export "main") (param i64) (result i64)
                call $produce_float
                drop
                local.get 0
            )
            (func $produce_float (result f64)
                f64.const 3.14
            )
        )
    "#;

    let opts = CompileOptions {
        trap_floats: true,
        ..CompileOptions::default()
    };
    let program = compile_wat_with_options(wat, &opts)
        .expect("function returning f64 should compile with trap-floats");
    assert!(!extract_instructions(&program).is_empty());
}

/// A float operator inside one arm of an `if` should trap that arm but leave
/// the other arm reachable, and the merge-block phi must be satisfied. This
/// is the trickiest control-flow case for the trap-floats lowering — it caught
/// a bug during development where setting `self.unreachable = true` left the
/// result phi without an incoming branch from the trap path.
#[test]
fn trap_floats_inside_if_arm_compiles() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i64)
                local.get 0
                if (result i64)
                    f64.const 1.0
                    drop
                    i64.const 100
                else
                    i64.const 200
                end
            )
        )
    "#;

    let opts = CompileOptions {
        trap_floats: true,
        ..CompileOptions::default()
    };
    let program = compile_wat_with_options(wat, &opts)
        .expect("if-arm with float op should compile under trap-floats");
    let instructions = extract_instructions(&program);
    assert!(
        has_opcode(&instructions, Opcode::Trap),
        "expected a Trap for the float branch"
    );
}

/// Default mode (`trap_floats` = false) MUST still reject float ops. We don't
/// want trap-floats to leak into normal compilation by accident.
#[test]
fn default_mode_still_rejects_floats() {
    let wat = r#"
        (module
            (func (export "main") (result i64)
                f64.const 1.0
                drop
                i64.const 0
            )
        )
    "#;
    let opts = CompileOptions::default();
    assert!(!opts.trap_floats, "default must be trap_floats=false");
    let err = compile_wat_with_options(wat, &opts)
        .err()
        .expect("default should still fail");
    assert!(matches!(err, Error::Located { .. }));
}
