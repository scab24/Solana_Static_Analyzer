pub mod json;
pub mod parser;

pub use parser::{parse_rust_code, parse_rust_file, process_directory};

pub use json::ast_to_json;

use anyhow::Result;
use log::debug;
use std::path::Path;

pub fn process_rust_file(path: &Path) -> Result<String> {
    debug!("Processing Rust file: {}", path.display());
    let ast = parser::parse_rust_file(path)?;
    let json = json::ast_to_json(&ast)?;
    Ok(json)
}
