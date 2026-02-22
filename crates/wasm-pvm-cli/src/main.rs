use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use wasm_pvm::{CompileOptions, ImportAction, OptimizationFlags};

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

        #[arg(long, help = "Disable register allocation (r5/r6 for long-lived values)")]
        no_register_alloc: bool,
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
                },
            };

            let spi =
                wasm_pvm::compile_with_options(&wasm, &options).context("Compilation failed")?;
            let encoded = spi.encode();
            fs::write(&output, &encoded)
                .with_context(|| format!("Failed to write output to {}", output.display()))?;
            println!(
                "Compiled {} -> {} ({} bytes)",
                input.display(),
                output.display(),
                encoded.len()
            );
        }
    }

    Ok(())
}

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
