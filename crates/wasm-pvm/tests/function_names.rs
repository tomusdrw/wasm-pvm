//! Tests for friendly function names in `CompileStats`.
//!
//! `FunctionStats.name` should reflect the WASM `name` custom section, with
//! the export name and synthetic `wasm_func_<global_idx>` placeholder as
//! fallbacks. This is what the CLI's `--verbose` and `--json` output show.

use wasm_pvm::test_harness::wat_to_wasm;
use wasm_pvm::{CompileOptions, compile_with_stats};

/// `$identifier` in WAT becomes a name-section entry and should be picked up
/// by `FunctionStats.name`. Export names are an independent fallback.
#[test]
fn name_section_identifier_used_for_function_stats() {
    let wat = r#"
        (module
            (func $helper (result i32)
                i32.const 7
            )
            (func $entry (export "main") (param i32 i32) (result i64)
                i64.const 17179869184
            )
        )
    "#;
    let wasm = wat_to_wasm(wat).expect("WAT should parse");
    let (_program, stats) =
        compile_with_stats(&wasm, &CompileOptions::default()).expect("compilation should succeed");

    let names: Vec<&str> = stats.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(
        names.contains(&"helper"),
        "expected `helper` in function stats names, got {names:?}"
    );
    assert!(
        names.contains(&"entry"),
        "expected `entry` (name section wins over export alias `main`), got {names:?}"
    );
    assert!(
        !names.iter().any(|n| n.starts_with("wasm_func_")),
        "no synthetic placeholder should leak through when names exist, got {names:?}"
    );
}

/// When a function has only an export name (no `$identifier`), the display
/// name falls back to the export name.
#[test]
fn export_name_fallback_when_name_section_empty() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i64)
                i64.const 17179869184
            )
        )
    "#;
    let wasm = wat_to_wasm(wat).expect("WAT should parse");
    let (_program, stats) =
        compile_with_stats(&wasm, &CompileOptions::default()).expect("compilation should succeed");

    let entry = stats
        .functions
        .iter()
        .find(|f| f.is_entry)
        .expect("entry function must exist");
    assert_eq!(
        entry.name, "main",
        "entry stats should fall back to export name `main`"
    );
}

/// Functions with no name and no export still get the synthetic placeholder.
/// (Hard to construct in pure WAT — every local function generally ends up
/// either named via `$id` or unnamed and unexported when only referenced
/// internally.) An unnamed, unexported helper called by an exported entry
/// exercises the fallback path.
#[test]
fn unnamed_unexported_function_uses_synthetic_name() {
    let wat = r#"
        (module
            (func (result i32)
                i32.const 1
            )
            (func (export "main") (param i32 i32) (result i64)
                call 0
                drop
                i64.const 17179869184
            )
        )
    "#;
    let wasm = wat_to_wasm(wat).expect("WAT should parse");
    let (_program, stats) =
        compile_with_stats(&wasm, &CompileOptions::default()).expect("compilation should succeed");

    let helper = stats
        .functions
        .iter()
        .find(|f| f.index == 0)
        .expect("helper function must be present in stats");
    assert_eq!(
        helper.name, "wasm_func_0",
        "unnamed/unexported helper should use the synthetic placeholder"
    );
}
