use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::fs;
use log::{info, error, warn, debug};

mod ast;

/// Alalyzer static tool for Solana/Anchor contracts in Rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Contract path or directory to analyze
    #[arg(short, long)]
    path: PathBuf,

    /// Custom templates path
    #[arg(short, long)]
    templates: Option<PathBuf>,
    
    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Severities to ignore (separated by commas: low,medium,high,critical)
    #[arg(short, long)]
    ignore: Option<String>,
    
    /// Generate AST JSON along with the report
    #[arg(long)]
    ast: bool,
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    
    // Parse arguments from command line
    let args = Cli::parse();
    debug!("CLI arguments: {:?}", args);

    // Verify that the path exists
    if !args.path.exists() {
        anyhow::bail!("Path {} does not exist", args.path.display());
    }

    // Verify that the path is a directory
    if !args.path.is_dir() {
        anyhow::bail!("Path {} is not a directory", args.path.display());
    }

    info!("Starting analysis on directory: {}", args.path.display());
    let results = ast::parser::process_directory(&args.path)?;
    info!("Found {} Rust files to analyze", results.len());
    if args.ast {
        for (path, ast) in &results {
            let json = ast::json::ast_to_json(&ast)?;
            let mut json_path = path.clone();
            json_path.set_extension("json");
            fs::write(json_path, json)?;
            info!("AST JSON generated for {}", path.display());
        }
    }
    // TODO: Main logic
    // 2. Analyze vulnerabilities
    // 3. Generate report
    
    info!("Analysis completed. Implementation pending.");
    Ok(())
}