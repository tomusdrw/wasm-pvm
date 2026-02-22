//! Test with the actual AssemblyScript-generated nested-repro WAT

use wasm_pvm::test_harness::*;

#[test]
#[ignore = "Requires examples-as build artifacts (npm run build:nested-repro)"]
fn test_as_nested_repro() {
    let wat = std::fs::read_to_string("../../examples-as/build/nested-repro.wat")
        .expect("Failed to read nested-repro.wat");

    let program = compile_wat(&wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    println!("\n=== AssemblyScript nested-repro ===");
    println!("Generated {} instructions:", instructions.len());

    // Find the main function area (approximately)
    // Looking for patterns related to the nested if-result
    for (i, instr) in instructions.iter().enumerate() {
        println!("{i:4}: {instr:?}");
    }

    assert!(!instructions.is_empty());
}
