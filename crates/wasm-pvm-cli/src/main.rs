use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use wasm_pvm::{CompileOptions, ImportAction};

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
            help = "Import map file mapping import names to actions (trap, nop, ecalli:N)"
        )]
        imports: Option<PathBuf>,

        #[arg(
            short,
            long,
            help = "Adapter WAT file whose exports replace matching imports"
        )]
        adapter: Option<PathBuf>,
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
                let content = fs::read_to_string(&adapter_path)
                    .with_context(|| format!("Failed to read adapter {}", adapter_path.display()))?;
                Some(content)
            } else {
                None
            };

            let options = CompileOptions {
                import_map,
                adapter: adapter_wat,
                metadata: metadata.into_bytes(),
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
/// some_func = ecalli:5
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
        } else if let Some(rest) = action_str.strip_prefix("ecalli:") {
            // Parse "ecalli:N" or "ecalli:N:ptr"
            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            let idx_str = parts[0].trim();
            let index: u32 = idx_str.parse().with_context(|| {
                format!(
                    "{}:{}: invalid ecalli index '{idx_str}'",
                    path.display(),
                    line_num + 1
                )
            })?;
            let ptr_params = parts.get(1).is_some_and(|s| s.trim() == "ptr");
            ImportAction::Ecalli { index, ptr_params }
        } else {
            anyhow::bail!(
                "{}:{}: unknown action '{action_str}', expected 'trap', 'nop', or 'ecalli:N[:ptr]'",
                path.display(),
                line_num + 1
            );
        };

        map.insert(name, action);
    }

    Ok(map)
}
