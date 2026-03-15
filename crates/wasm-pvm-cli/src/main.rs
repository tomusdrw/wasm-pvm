// Formatting helpers use `as f64` for display purposes — precision loss is acceptable.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use wasm_pvm::{CompileOptions, CompileStats, ImportAction, OptimizationFlags};

const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "wasm-pvm")]
#[command(about = "WASM to PVM (PolkaVM) recompiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compile {
        #[arg(help = "Input WASM or WAT file")]
        input: PathBuf,

        #[arg(short, long, help = "Output SPI file")]
        output: PathBuf,

        #[arg(
            short,
            long,
            help = "Import map file mapping import names to actions (trap, nop)"
        )]
        imports: Option<PathBuf>,

        #[arg(
            short,
            long,
            help = "Adapter WAT file whose exports replace matching imports"
        )]
        adapter: Option<PathBuf>,

        #[arg(long, help = "Show per-function details and optimization stats")]
        verbose: bool,

        #[arg(long, help = "Output stats as JSON instead of text")]
        json: bool,

        #[arg(long, help = "Disable LLVM optimization passes")]
        no_llvm_passes: bool,

        #[arg(long, help = "Disable peephole optimizer")]
        no_peephole: bool,

        #[arg(long, help = "Disable register cache (store-load forwarding)")]
        no_register_cache: bool,

        #[arg(long, help = "Disable ICmp+Branch fusion")]
        no_icmp_fusion: bool,

        #[arg(long, help = "Disable callee-save shrink wrapping")]
        no_shrink_wrap: bool,

        #[arg(long, help = "Disable dead store elimination")]
        no_dead_store_elim: bool,

        #[arg(
            long,
            help = "Disable constant propagation (redundant LoadImm elimination)"
        )]
        no_const_prop: bool,

        #[arg(long, help = "Disable LLVM function inlining")]
        no_inline: bool,

        #[arg(long, help = "Disable cross-block register cache propagation")]
        no_cross_block_cache: bool,

        #[arg(
            long,
            help = "Disable register allocation (r5/r6 for long-lived values)"
        )]
        no_register_alloc: bool,

        #[arg(long, help = "Disable dead function elimination")]
        no_dead_function_elim: bool,

        #[arg(long, help = "Disable fallthrough jump elimination")]
        no_fallthrough_jumps: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            imports,
            adapter,
            verbose,
            json,
            no_llvm_passes,
            no_peephole,
            no_register_cache,
            no_icmp_fusion,
            no_shrink_wrap,
            no_dead_store_elim,
            no_const_prop,
            no_inline,
            no_cross_block_cache,
            no_register_alloc,
            no_dead_function_elim,
            no_fallthrough_jumps,
        } => {
            let wasm = read_wasm(&input)?;

            let filename = input
                .file_name()
                .map_or_else(|| input.to_string_lossy(), |f| f.to_string_lossy());
            let metadata = format!("{filename} (wasm-pvm {COMPILER_VERSION})");

            let import_map = if let Some(imports_path) = imports {
                Some(parse_import_map(&imports_path)?)
            } else {
                None
            };

            let adapter_wat = if let Some(adapter_path) = adapter {
                let content = fs::read_to_string(&adapter_path).with_context(|| {
                    format!("Failed to read adapter {}", adapter_path.display())
                })?;
                Some(content)
            } else {
                None
            };

            let options = CompileOptions {
                import_map,
                adapter: adapter_wat,
                metadata: metadata.into_bytes(),
                optimizations: OptimizationFlags {
                    llvm_passes: !no_llvm_passes,
                    peephole: !no_peephole,
                    register_cache: !no_register_cache,
                    icmp_branch_fusion: !no_icmp_fusion,
                    shrink_wrap_callee_saves: !no_shrink_wrap,
                    dead_store_elimination: !no_dead_store_elim,
                    constant_propagation: !no_const_prop,
                    inlining: !no_inline,
                    cross_block_cache: !no_cross_block_cache,
                    register_allocation: !no_register_alloc,
                    dead_function_elimination: !no_dead_function_elim,
                    fallthrough_jumps: !no_fallthrough_jumps,
                },
            };

            let start = Instant::now();
            let (spi, stats) =
                wasm_pvm::compile_with_stats(&wasm, &options).context("Compilation failed")?;
            let elapsed = start.elapsed();

            let encoded = spi.encode();
            fs::write(&output, &encoded)
                .with_context(|| format!("Failed to write output to {}", output.display()))?;

            if json {
                print_json(
                    &stats,
                    &input,
                    &output,
                    verbose,
                    elapsed.as_millis() as u64,
                );
            } else {
                print_text(
                    &stats,
                    &input,
                    &output,
                    verbose,
                    elapsed.as_millis() as u64,
                );
            }
        }
    }

    Ok(())
}

// ── Text output (Style C) ──

fn section(title: &str) {
    let dash_count = 43usize.saturating_sub(4 + title.len());
    let dashes: String = "\u{2500}".repeat(dash_count);
    println!("\u{2500}\u{2500} {title} {dashes}");
}

fn row(label: &str, value: impl std::fmt::Display) {
    println!("  {label:<17}{value}");
}

fn format_memory_size(pages: u32, page_size_kb: u32) -> String {
    let total_kb = u64::from(pages) * u64::from(page_size_kb);
    if total_kb >= 1024 {
        format!("{} MB", total_kb / 1024)
    } else {
        format!("{total_kb} KB")
    }
}

fn print_text(stats: &CompileStats, input: &Path, output: &Path, verbose: bool, ms: u64) {
    println!("wasm-pvm v{COMPILER_VERSION}");
    println!();

    // ── Input ──
    section("Input");
    row("Source", input.display());
    let total_funcs = stats.local_functions + stats.imported_functions;
    row(
        "Functions",
        format!(
            "{total_funcs} total ({} local, {} imported)",
            stats.local_functions, stats.imported_functions
        ),
    );
    row("Globals", stats.globals);
    let total_segments = stats.active_data_segments + stats.passive_data_segments;
    row(
        "Data segments",
        format!(
            "{total_segments} ({} active, {} passive)",
            stats.active_data_segments, stats.passive_data_segments
        ),
    );
    if stats.function_table_entries > 0 {
        row(
            "Function table",
            format!("{} entries", stats.function_table_entries),
        );
    }
    row(
        "Initial memory",
        format!(
            "{} pages ({})",
            stats.initial_memory_pages,
            format_memory_size(stats.initial_memory_pages, 64)
        ),
    );
    row(
        "Max memory",
        format!(
            "{} pages ({})",
            stats.max_memory_pages,
            format_memory_size(stats.max_memory_pages, 64)
        ),
    );
    if !stats.import_resolutions.is_empty() {
        let imports_str: Vec<String> = stats
            .import_resolutions
            .iter()
            .map(|r| format!("{} \u{2192} {}", r.name, r.action))
            .collect();
        row("Imports", imports_str.join(", "));
    }
    println!();

    // ── Memory Layout ──
    section("Memory Layout");
    row(
        "WASM memory base",
        format!("0x{:X}", stats.wasm_memory_base),
    );
    row(
        "Globals region",
        format!("{} bytes", stats.globals_region_bytes),
    );
    row("RO data", format!("{} bytes", stats.ro_data_bytes));
    row("RW data", format!("{} bytes", stats.rw_data_bytes));
    row("Heap pages", stats.heap_pages);
    row("Stack size", format!("{} bytes", stats.stack_size));
    println!();

    // ── Output ──
    section("Output");
    row("Destination", output.display());
    row("PVM instructions", format_number(stats.pvm_instructions));
    row("Code size", format!("{} bytes", format_number(stats.code_bytes)));
    row(
        "Jump table",
        format!(
            "{} entries ({} bytes)",
            stats.jump_table_entries,
            stats.jump_table_entries * 4
        ),
    );
    if stats.dead_functions_eliminated > 0 {
        row("Dead funcs elim", stats.dead_functions_eliminated);
    }
    row(
        "SPI blob size",
        format!("{} bytes", format_number(stats.spi_blob_bytes)),
    );

    // ── Verbose: per-function + optimization stats ──
    if verbose {
        println!();
        print_verbose_text(stats);
    }

    println!();
    println!("Compiled in {ms}ms");
}

fn print_verbose_text(stats: &CompileStats) {
    section("Functions");
    for f in &stats.functions {
        if f.is_dead {
            println!(
                "  #{:<4} {:<20} DEAD",
                f.index,
                truncate(&f.name, 20),
            );
            continue;
        }
        let kind = if f.is_leaf { "leaf" } else { "calls" };
        let entry_marker = if f.is_entry { " [entry]" } else { "" };
        let regalloc_info = if f.regalloc.allocated_values > 0 {
            format!(
                "regalloc: {}/{} \u{2192} {{{}}}",
                f.regalloc.allocated_values,
                f.regalloc.total_values,
                f.regalloc.registers_used.join(", ")
            )
        } else if let Some(reason) = &f.regalloc.skipped_reason {
            format!("regalloc: skipped ({reason})")
        } else {
            String::new()
        };
        println!(
            "  #{:<4} {:<20} {:>4} instrs  frame={:<5} {:<5}{} {}",
            f.index,
            truncate(&f.name, 20),
            f.instruction_count,
            format!("{}B", f.frame_size),
            kind,
            entry_marker,
            regalloc_info,
        );
    }

    // ── Optimization impact (aggregate) ──
    let total_pre_dse: usize = stats.functions.iter().map(|f| f.pre_dse_instructions).sum();
    let total_pre_peep: usize = stats
        .functions
        .iter()
        .map(|f| f.pre_peephole_instructions)
        .sum();
    let total_final: usize = stats
        .functions
        .iter()
        .filter(|f| !f.is_dead)
        .map(|f| f.instruction_count)
        .sum();

    let total_load_hits: usize = stats.functions.iter().map(|f| f.regalloc.load_hits).sum();
    let total_load_reloads: usize = stats.functions.iter().map(|f| f.regalloc.load_reloads).sum();
    let total_load_moves: usize = stats.functions.iter().map(|f| f.regalloc.load_moves).sum();
    let total_store_hits: usize = stats.functions.iter().map(|f| f.regalloc.store_hits).sum();
    let total_store_moves: usize = stats.functions.iter().map(|f| f.regalloc.store_moves).sum();

    let has_dse = total_pre_dse > 0 && total_pre_dse != total_pre_peep;
    let has_peephole = total_pre_peep > 0 && total_pre_peep != total_final;
    let has_regalloc =
        total_load_hits > 0 || total_load_reloads > 0 || total_store_hits > 0;

    if has_dse || has_peephole || has_regalloc {
        println!();
        section("Optimizations");
        if has_dse {
            let pct = reduction_pct(total_pre_dse, total_pre_peep);
            row(
                "Dead store elim",
                format!("{total_pre_dse} \u{2192} {total_pre_peep} instrs ({pct})"),
            );
        }
        if has_peephole {
            let pct = reduction_pct(total_pre_peep, total_final);
            row(
                "Peephole",
                format!("{total_pre_peep} \u{2192} {total_final} instrs ({pct})"),
            );
        }
        if has_regalloc {
            row(
                "Regalloc usage",
                format!(
                    "load_hits={total_load_hits} reloads={total_load_reloads} \
                     moves={total_load_moves} store_hits={total_store_hits} \
                     store_moves={total_store_moves}"
                ),
            );
        }
    }
}

fn reduction_pct(before: usize, after: usize) -> String {
    if before == 0 {
        return String::new();
    }
    let pct = (1.0 - after as f64 / before as f64) * 100.0;
    format!("-{pct:.1}%")
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        let millions = n as f64 / 1_000_000.0;
        format!("{millions:.1}M")
    } else if n >= 10_000 {
        let thousands = n as f64 / 1000.0;
        format!("{thousands:.1}K")
    } else {
        n.to_string()
    }
}

// ── JSON output ──

fn print_json(stats: &CompileStats, input: &Path, output: &Path, verbose: bool, ms: u64) {
    let mut imports_arr = Vec::new();
    for r in &stats.import_resolutions {
        imports_arr.push(serde_json::json!({
            "name": r.name,
            "action": r.action,
        }));
    }

    let mut obj = serde_json::json!({
        "version": COMPILER_VERSION,
        "compile_time_ms": ms,
        "input": {
            "source": input.display().to_string(),
            "functions": {
                "total": stats.local_functions + stats.imported_functions,
                "local": stats.local_functions,
                "imported": stats.imported_functions,
            },
            "globals": stats.globals,
            "data_segments": {
                "total": stats.active_data_segments + stats.passive_data_segments,
                "active": stats.active_data_segments,
                "passive": stats.passive_data_segments,
            },
            "function_table_entries": stats.function_table_entries,
            "memory": {
                "initial_pages": stats.initial_memory_pages,
                "max_pages": stats.max_memory_pages,
            },
            "imports": imports_arr,
        },
        "memory_layout": {
            "wasm_memory_base": format!("0x{:X}", stats.wasm_memory_base),
            "globals_region_bytes": stats.globals_region_bytes,
            "ro_data_bytes": stats.ro_data_bytes,
            "rw_data_bytes": stats.rw_data_bytes,
            "heap_pages": stats.heap_pages,
            "stack_bytes": stats.stack_size,
        },
        "output": {
            "destination": output.display().to_string(),
            "pvm_instructions": stats.pvm_instructions,
            "code_bytes": stats.code_bytes,
            "jump_table_entries": stats.jump_table_entries,
            "jump_table_bytes": stats.jump_table_entries * 4,
            "dead_functions_eliminated": stats.dead_functions_eliminated,
            "spi_blob_bytes": stats.spi_blob_bytes,
        },
    });

    if verbose {
        let functions: Vec<serde_json::Value> = stats
            .functions
            .iter()
            .map(|f| {
                serde_json::json!({
                    "name": f.name,
                    "index": f.index,
                    "instruction_count": f.instruction_count,
                    "frame_size": f.frame_size,
                    "is_leaf": f.is_leaf,
                    "is_entry": f.is_entry,
                    "is_dead": f.is_dead,
                    "pre_dse_instructions": f.pre_dse_instructions,
                    "pre_peephole_instructions": f.pre_peephole_instructions,
                    "regalloc": {
                        "total_values": f.regalloc.total_values,
                        "allocated_values": f.regalloc.allocated_values,
                        "registers_used": f.regalloc.registers_used,
                        "skipped_reason": f.regalloc.skipped_reason,
                        "load_hits": f.regalloc.load_hits,
                        "load_reloads": f.regalloc.load_reloads,
                        "load_moves": f.regalloc.load_moves,
                        "store_hits": f.regalloc.store_hits,
                        "store_moves": f.regalloc.store_moves,
                    },
                })
            })
            .collect();

        obj["functions"] = serde_json::Value::Array(functions);
    }

    println!("{}", serde_json::to_string_pretty(&obj).expect("JSON serialization failed"));
}

// ── Helpers ──

fn read_wasm(path: &PathBuf) -> Result<Vec<u8>> {
    let contents = fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;

    if path.extension().is_some_and(|e| e == "wat") {
        wat::parse_bytes(&contents)
            .map(std::borrow::Cow::into_owned)
            .map_err(|e| anyhow::anyhow!("WAT parse error: {e}"))
    } else {
        Ok(contents)
    }
}

/// Parse an import map file.
///
/// Format (one mapping per line):
/// ```text
/// # Comments start with #
/// abort = trap
/// console.log = nop
/// ```
fn parse_import_map(path: &PathBuf) -> Result<HashMap<String, ImportAction>> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let mut map = HashMap::new();

    for (line_num, line) in contents.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (name, action) = line.split_once('=').ok_or_else(|| {
            anyhow::anyhow!(
                "{}:{}: invalid format, expected 'name = action'",
                path.display(),
                line_num + 1
            )
        })?;

        let name = name.trim().to_string();
        let action_str = action.trim();

        let action = if action_str == "trap" {
            ImportAction::Trap
        } else if action_str == "nop" {
            ImportAction::Nop
        } else {
            anyhow::bail!(
                "{}:{}: unknown action '{action_str}', expected 'trap' or 'nop'",
                path.display(),
                line_num + 1
            );
        };

        map.insert(name, action);
    }

    Ok(map)
}
