use crate::analyzer::dsl::{AstQuery, RuleBuilder};
use crate::analyzer::dsl::filters::SolanaFilters;
use crate::analyzer::engine::{Rule, RuleType};
use crate::analyzer::{Finding, Location, Severity};
use log::debug;
use std::sync::Arc;

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-missing-error-handling")
        .severity(Severity::Low)
        .title("Missing Error Handling in Public Functions")
        .description("Detects public functions that don't return Result<T> and may fail silently. In Solana contracts, proper error handling is essential for security and debugging.")
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing missing error handling using DSL");
            
            AstQuery::new(ast)
                .functions()
                .missing_error_handling()
        })
        .build()
}
