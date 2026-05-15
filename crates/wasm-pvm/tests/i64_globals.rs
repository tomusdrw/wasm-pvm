//! Regression tests for per-global-width handling.
//!
//! Prior to this fix, the LLVM-to-PVM lowering emitted `LoadU32`/`StoreU32`
//! unconditionally regardless of the WASM-declared global type, silently
//! dropping the top 32 bits of any `(global i64 ...)`. These tests assert
//! that i64 globals lower to the 64-bit access opcodes while i32 globals
//! keep using the narrower 32-bit opcodes (4-byte storage slot preserved).

use wasm_pvm::Opcode;
use wasm_pvm::test_harness::*;

#[test]
fn i64_global_get_emits_loadu64() {
    let wat = r#"
        (module
            (memory 1)
            (export "memory" (memory 0))
            (global $g (mut i64) (i64.const 0x1122334455667788))
            (func (export "main") (param i32 i32) (result i64)
                (global.get $g)
            )
        )
    "#;
    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::LoadU64),
        "(global.get on i64 global) must lower to LoadU64, got: {instructions:#?}"
    );
    assert_eq!(
        count_opcode(&instructions, Opcode::LoadU32),
        0,
        "i64 global must NOT emit LoadU32 (would truncate top 32 bits)"
    );
}

#[test]
fn i64_global_set_const_emits_storeimmu64() {
    // Small constant (fits in i32 sign-extend range) — should use StoreImmU64.
    let wat = r#"
        (module
            (memory 1)
            (export "memory" (memory 0))
            (global $g (mut i64) (i64.const 0))
            (func (export "main") (param i32 i32) (result i64)
                (global.set $g (i64.const 42))
                (i64.const 0)
            )
        )
    "#;
    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::StoreImmU64),
        "small i64 constant must lower to StoreImmU64, got: {instructions:#?}"
    );
    assert_eq!(
        count_opcode(&instructions, Opcode::StoreImmU32),
        0,
        "i64 global must NOT emit StoreImmU32"
    );
}

#[test]
fn i64_global_set_large_const_emits_storeu64() {
    // Large i64 constant (doesn't fit in i32 sign-extend range) — must use
    // LoadImm64 + StoreU64.
    let wat = r#"
        (module
            (memory 1)
            (export "memory" (memory 0))
            (global $g (mut i64) (i64.const 0))
            (func (export "main") (param i32 i32) (result i64)
                (global.set $g (i64.const 0x1122334455667788))
                (i64.const 0)
            )
        )
    "#;
    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::StoreU64),
        "large i64 constant store must lower to StoreU64, got: {instructions:#?}"
    );
    assert_eq!(
        count_opcode(&instructions, Opcode::StoreU32),
        0,
        "i64 global must NOT emit StoreU32 (would truncate)"
    );
    assert_eq!(
        count_opcode(&instructions, Opcode::StoreImmU32),
        0,
        "i64 global must NOT emit StoreImmU32"
    );
}

#[test]
fn i32_global_keeps_using_loadu32_storeu32() {
    // i32 globals stay at 4-byte storage and use the 32-bit access opcodes.
    // Split set and get across two functions so LLVM's intra-function
    // store→load forwarding can't fold them.
    let wat = r#"
        (module
            (memory 1)
            (export "memory" (memory 0))
            (global $g (mut i32) (i32.const 12345))
            (func $get_g (result i32)
                (global.get $g)
            )
            (func (export "main") (param i32 i32) (result i64)
                (global.set $g (local.get 0))
                (i64.extend_i32_u (call $get_g))
            )
        )
    "#;
    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::LoadU32),
        "i32 global.get must lower to LoadU32, got: {instructions:#?}"
    );
    assert!(
        has_opcode(&instructions, Opcode::StoreU32)
            || has_opcode(&instructions, Opcode::StoreImmU32),
        "i32 global.set must lower to StoreU32 / StoreImmU32, got: {instructions:#?}"
    );
    // 64-bit access opcodes must NOT appear for i32 globals (they'd waste 4 bytes
    // per slot and read garbage from adjacent storage if the layout shifted).
    assert_eq!(
        count_opcode(&instructions, Opcode::LoadU64),
        0,
        "i32 global access must NOT emit LoadU64"
    );
    assert_eq!(
        count_opcode(&instructions, Opcode::StoreU64),
        0,
        "i32 global access must NOT emit StoreU64"
    );
    assert_eq!(
        count_opcode(&instructions, Opcode::StoreImmU64),
        0,
        "i32 global access must NOT emit StoreImmU64"
    );
}

#[test]
fn mixed_i32_i64_globals_compile_with_distinct_opcodes() {
    // Smoke test: a module declaring both i32 and i64 globals must lower
    // each to its own width opcodes — confirms per-global widths and the
    // packing layout don't accidentally swap.
    let wat = r#"
        (module
            (memory 1)
            (export "memory" (memory 0))
            (global $a (mut i32) (i32.const 0xAABB))
            (global $b (mut i64) (i64.const 0x1122334455667788))
            (global $c (mut i32) (i32.const 0xCCDD))
            (func $get_a (result i32) (global.get $a))
            (func $get_b (result i64) (global.get $b))
            (func $get_c (result i32) (global.get $c))
            (func (export "main") (param i32 i32) (result i64)
                (drop (call $get_a))
                (drop (call $get_b))
                (drop (call $get_c))
                (i64.const 0)
            )
        )
    "#;
    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);
    assert!(
        has_opcode(&instructions, Opcode::LoadU32),
        "mixed module must emit LoadU32 for i32 globals"
    );
    assert!(
        has_opcode(&instructions, Opcode::LoadU64),
        "mixed module must emit LoadU64 for i64 globals"
    );
}

#[test]
fn unsupported_global_type_is_rejected() {
    // v128 globals must error out at parse time rather than silently miscompile.
    // Note: v128 requires the SIMD feature; wasmparser validation may reject it
    // first if SIMD is disabled in the validator config — either way, the
    // module must NOT compile.
    let wat = r#"
        (module
            (global $g (mut v128) (v128.const i32x4 0 0 0 0))
            (func (export "main") (param i32 i32) (result i64)
                (i64.const 0)
            )
        )
    "#;
    let result = compile_wat(wat);
    assert!(result.is_err(), "v128 global must be rejected");
}

#[test]
fn float_global_is_rejected() {
    // f32/f64 globals don't survive the integer-typed `global.get`/`global.set`
    // lowering (init silently zeroed, bits observable via `i32.reinterpret_f32`).
    // They must error at parse time, not slip through `--trap-floats`.
    let f32_wat = r#"
        (module
            (global $g (mut f32) (f32.const 1.5))
            (func (export "main") (param i32 i32) (result i64) (i64.const 0))
        )
    "#;
    assert!(
        compile_wat(f32_wat).is_err(),
        "f32 global must be rejected at parse time"
    );

    let f64_wat = r#"
        (module
            (global $g (mut f64) (f64.const 3.14))
            (func (export "main") (param i32 i32) (result i64) (i64.const 0))
        )
    "#;
    assert!(
        compile_wat(f64_wat).is_err(),
        "f64 global must be rejected at parse time"
    );
}

#[test]
fn non_const_global_init_is_rejected() {
    // Imported-global-based initializers (`global.get $imported`) are legal WASM
    // const-exprs but the compiler has no path to evaluate them, so they used
    // to silently fall through to a zero initializer — a footgun. Must error.
    let wat = r#"
        (module
            (import "env" "BASE" (global $base i32))
            (global $g (mut i32) (global.get $base))
            (func (export "main") (param i32 i32) (result i64) (i64.const 0))
        )
    "#;
    assert!(
        compile_wat(wat).is_err(),
        "global.get-based init expression must be rejected"
    );
}

#[test]
fn extended_const_init_is_rejected() {
    // Extended constant expressions (i32.add of two consts) are legal WASM
    // — wasmparser's EXTENDED_CONST feature is on by default. We can only
    // evaluate single literals, so a multi-operator init must error rather
    // than silently truncate to the first literal.
    let wat = r#"
        (module
            (global $g (mut i32) (i32.add (i32.const 5) (i32.const 7)))
            (func (export "main") (param i32 i32) (result i64) (i64.const 0))
        )
    "#;
    assert!(
        compile_wat(wat).is_err(),
        "extended-const init expression (i32.add) must be rejected"
    );
}
