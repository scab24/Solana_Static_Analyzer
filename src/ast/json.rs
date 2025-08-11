use anyhow::Result;
use log::debug;
use syn::File;

/// Convert an AST to JSON
pub fn ast_to_json(ast: &File) -> String {
    debug!("Converting AST to JSON");
    let json_string = syn_serde::json::to_string_pretty(ast);
    json_string
}
