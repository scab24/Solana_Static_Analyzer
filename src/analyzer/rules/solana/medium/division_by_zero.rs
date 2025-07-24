use log::debug;
use std::sync::Arc;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::dsl::filters::SolanaFilters;
use crate::analyzer::engine::{Rule, RuleType};
use crate::analyzer::{Finding, Location, Severity};

/// Crea la regla para detectar divisiones sin verificación de cero
pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-division-by-zero")
        .severity(Severity::Medium)
        .title("Division Without Zero Check")
        .description("Detects division operations without zero verification")
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing unsafe divisions using DSL");
            
            AstQuery::new(ast)
                .functions()
                .has_unsafe_divisions()
        })
        .build()
}
