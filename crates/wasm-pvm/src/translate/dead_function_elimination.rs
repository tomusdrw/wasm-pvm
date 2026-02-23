// Dead function elimination: compute which local functions are reachable
// from entry points and the function table, so unreachable functions can
// be skipped during compilation.

use std::collections::{HashSet, VecDeque};

use wasmparser::Operator;

use super::wasm_module::WasmModule;
use crate::Result;

/// Compute the set of reachable local function indices.
///
/// Starts from entry points (main, secondary, start) and all functions
/// referenced in the element table, then follows `Call` and `RefFunc`
/// instructions transitively.  Modules containing `CallIndirect`
/// conservatively mark all table-referenced functions as reachable.
pub fn reachable_functions(module: &WasmModule) -> Result<HashSet<usize>> {
    let num_imports = module.num_imported_funcs as usize;
    let num_locals = module.functions.len();

    let mut reachable: HashSet<usize> = HashSet::new();
    let mut worklist: VecDeque<usize> = VecDeque::new();

    // Seed: entry points
    worklist.push_back(module.main_func_local_idx);
    if let Some(idx) = module.secondary_entry_local_idx {
        worklist.push_back(idx);
    }
    if let Some(idx) = module.start_func_local_idx {
        worklist.push_back(idx);
    }

    // Seed: all functions referenced in the element table (for call_indirect)
    for &global_idx in &module.function_table {
        if global_idx != u32::MAX
            && let Some(local_idx) = (global_idx as usize).checked_sub(num_imports)
            && local_idx < num_locals
        {
            worklist.push_back(local_idx);
        }
    }

    // BFS: follow direct calls transitively
    while let Some(local_idx) = worklist.pop_front() {
        if !reachable.insert(local_idx) {
            continue; // already visited
        }
        if local_idx >= num_locals {
            continue; // out of bounds guard
        }

        // Scan function body for Call and RefFunc operators
        let body = &module.functions[local_idx];
        let mut reader = body.get_operators_reader()?;
        while !reader.eof() {
            let op = reader.read()?;
            let target_global = match op {
                Operator::Call { function_index } | Operator::RefFunc { function_index } => {
                    Some(function_index as usize)
                }
                _ => None,
            };
            if let Some(global_idx) = target_global
                && let Some(called_local) = global_idx.checked_sub(num_imports)
                && called_local < num_locals
                && !reachable.contains(&called_local)
            {
                worklist.push_back(called_local);
            }
            // CallIndirect targets are already seeded from the function table above,
            // so we don't need additional handling here.
        }
    }

    tracing::debug!(
        "Dead function elimination: {}/{} local functions reachable",
        reachable.len(),
        num_locals
    );

    Ok(reachable)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: parse a WAT module and return the reachable set.
    fn reachable_from_wat(wat: &str) -> HashSet<usize> {
        let wasm = wat::parse_str(wat).expect("valid WAT");
        let module = WasmModule::parse(&wasm).expect("valid module");
        reachable_functions(&module).expect("analysis succeeds")
    }

    #[test]
    fn single_main_function() {
        let reachable = reachable_from_wat(
            r#"(module
                (func (export "main") (result i32) (i32.const 42))
            )"#,
        );
        assert_eq!(reachable.len(), 1);
        assert!(reachable.contains(&0));
    }

    #[test]
    fn dead_function_not_reachable() {
        let reachable = reachable_from_wat(
            r#"(module
                (func (export "main") (result i32) (i32.const 42))
                (func (result i32) (i32.const 99))
            )"#,
        );
        assert_eq!(reachable.len(), 1);
        assert!(reachable.contains(&0));
        assert!(!reachable.contains(&1));
    }

    #[test]
    fn direct_call_chain() {
        // main -> f1 -> f2; f3 is dead
        let reachable = reachable_from_wat(
            r#"(module
                (func $main (export "main") (result i32) (call $f1))
                (func $f1 (result i32) (call $f2))
                (func $f2 (result i32) (i32.const 1))
                (func $dead (result i32) (i32.const 2))
            )"#,
        );
        assert_eq!(reachable.len(), 3);
        assert!(reachable.contains(&0)); // main
        assert!(reachable.contains(&1)); // f1
        assert!(reachable.contains(&2)); // f2
        assert!(!reachable.contains(&3)); // dead
    }

    #[test]
    fn mutual_recursion() {
        let reachable = reachable_from_wat(
            r#"(module
                (func $main (export "main") (result i32) (call $a))
                (func $a (result i32) (call $b))
                (func $b (result i32) (call $a))
                (func $dead (result i32) (i32.const 0))
            )"#,
        );
        assert_eq!(reachable.len(), 3);
        assert!(reachable.contains(&0));
        assert!(reachable.contains(&1));
        assert!(reachable.contains(&2));
        assert!(!reachable.contains(&3));
    }

    #[test]
    fn table_keeps_functions_alive() {
        let reachable = reachable_from_wat(
            r#"(module
                (type $sig (func (result i32)))
                (func $main (export "main") (result i32) (i32.const 42))
                (func $in_table (result i32) (i32.const 1))
                (func $dead (result i32) (i32.const 2))
                (table 2 funcref)
                (elem (i32.const 0) $main $in_table)
            )"#,
        );
        assert!(reachable.contains(&0)); // main
        assert!(reachable.contains(&1)); // in_table
        assert!(!reachable.contains(&2)); // dead
    }

    #[test]
    fn start_function_reachable() {
        let reachable = reachable_from_wat(
            r#"(module
                (func $start (call $helper))
                (func $main (export "main") (result i32) (i32.const 0))
                (func $helper)
                (func $dead (result i32) (i32.const 99))
                (start $start)
            )"#,
        );
        assert!(reachable.contains(&0)); // start
        assert!(reachable.contains(&1)); // main
        assert!(reachable.contains(&2)); // helper (called by start)
        assert!(!reachable.contains(&3)); // dead
    }
}
