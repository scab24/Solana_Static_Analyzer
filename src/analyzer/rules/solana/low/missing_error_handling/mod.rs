use log::debug;
use std::sync::Arc;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::{Rule, Severity};
use crate::analyzer::engine::RuleType;

// Import our specific filters
mod filters;
use filters::MissingErrorHandlingFilters;

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-missing-error-handling")
        .severity(Severity::Low)
        .title("Missing Error Handling in Public Functions")
        .description("Detects public functions that don't return Result<T> and may fail silently. In Solana contracts, proper error handling is essential for security and debugging.")
        .recommendations(vec![
            "Change function return type to Result<T, YourErrorType> to handle potential failures",
            "Use Anchor's Result<()> for instruction handlers to properly propagate errors",
            "Implement custom error types using #[error_code] for better error reporting",
            "Add proper error handling with ? operator or explicit error returns",
            "Consider using anchor_lang::Result for Anchor-specific error handling"
        ])
        .rule_type(RuleType::Solana)
        .tag("error-handling")
        .tag("best-practices")
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing missing error handling");
            
            AstQuery::new(ast)
                .functions()                           
                .missing_error_handling()              
        })
        .build()
}
