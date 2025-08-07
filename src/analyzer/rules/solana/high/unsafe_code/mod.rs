use log::debug;
use std::sync::Arc;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::{Rule, Severity};
use crate::analyzer::engine::RuleType;

// Import our specific filters
mod filters;
use filters::UnsafeCodeFilters;

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-unsafe-code")
        .title("Unsafe Code Usage")
        .description("Using unsafe code in Solana programs can lead to security vulnerabilities")
        .severity(Severity::High)
        .rule_type(RuleType::Solana)
        .tag("security")
        .tag("unsafe")
        .reference(".")
        .reference("https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html")
        .recommendations(vec![
            "Avoid using unsafe code in Solana programs unless absolutely necessary",
            "If unsafe is required, thoroughly document why it's needed and ensure all invariants are maintained",
            "Consider using safe alternatives like checked arithmetic operations"
        ])
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing unsafe code");
            
            AstQuery::new(ast)
                .functions()                           
                .uses_unsafe()                         
        })
        .build()
}
