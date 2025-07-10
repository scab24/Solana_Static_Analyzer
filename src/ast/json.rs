use anyhow::Result;
use syn::File;
use log::debug;

/// Convert an AST to JSON
pub fn ast_to_json(ast: &File) -> Result<String> {
    debug!("Converting AST to JSON");
    let json_string = syn_serde::json::to_string_pretty(ast);
    Ok(json_string)
}
