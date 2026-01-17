use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

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
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input, output } => {
            let wasm = read_wasm(&input)?;
            let spi = wasm_pvm::compile(&wasm).context("Compilation failed")?;
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
