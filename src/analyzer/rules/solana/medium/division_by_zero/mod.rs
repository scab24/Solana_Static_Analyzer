use log::debug;
use std::sync::Arc;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::{Rule, Severity};

// Import our specific filters
mod filters;
use filters::DivisionByZeroFilters;

/// Crea la regla para detectar divisiones sin verificación de cero
pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-division-by-zero")
        .severity(Severity::Medium)
        .title("Division Without Zero Check")
        .description("Detects division operations without zero verification")
        .recommendations(vec![
            "Add explicit zero checks before division operations: if divisor == 0 { return Err(...) }",
            "Use checked division methods: checked_div() which returns Option<T>",
            "Implement proper error handling for division by zero cases",
            "Consider using safe arithmetic operations provided by Anchor or custom error types",
            "Validate input parameters at the beginning of instruction handlers"
        ])
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing unsafe divisions");
            
            AstQuery::new(ast)
                .functions()
                .has_unsafe_divisions()
        })
        .build()
}
