use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use log::{info, error};

/// Parse a Rust file and return the AST
pub fn parse_rust_file(path: &Path) -> Result<syn::File> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read file {}", path.display()))?;

    parse_rust_code(&content).with_context(|| format!("Failed to parse file {}", path.display()))
}

/// Parse a string of Rust code and return the AST
pub fn parse_rust_code(content: &str) -> Result<syn::File> {
    syn::parse_str::<syn::File>(content).map_err(|e| anyhow::anyhow!("Failed to parse Rust code: {}", e))
}

/// Process a directory and return a vector of (path, AST) pairs
pub fn process_directory(dir_path: &Path) -> Result<Vec<(PathBuf, syn::File)>> {
    let mut results = Vec::new();

    for entry in WalkDir::new(dir_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok()) {
        
        let path = entry.path();

        // Only process Rust files
        if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            match parse_rust_file(path) {
                Ok(ast) => {
                    info!("Successfully parsed file {}", path.display());
                    results.push((path.to_path_buf(), ast))
                },
                Err(e) => error!("Failed to parse file {}: {}", path.display(), e),
            }
        }
    }
    info!("Processed {} Rust files", results.len());
    Ok(results)
}
