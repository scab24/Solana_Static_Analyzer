pub mod parser;
pub mod json;

pub use parser::{parse_rust_file, parse_rust_code, process_directory};

pub use json::ast_to_json;

use log::debug;
use std::path::Path;
use anyhow::Result;

pub fn process_rust_file(path: &Path) -> Result<String> {
    debug!("Processing Rust file: {}", path.display());
    let ast = parser::parse_rust_file(path)?;
    let json = json::ast_to_json(&ast)?;
    Ok(json)
}