use log::debug;
use std::sync::Arc;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::dsl::filters::SolanaFilters;
use crate::analyzer::{Finding, Location, Rule, Severity};

/// Crea la regla para detectar divisiones sin verificación de cero
pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-division-by-zero")
        .severity(Severity::Medium)
        .title("Division Without Zero Check")
        .description("Detects division operations without zero verification")
        .dsl_rule(|file, file_path| {
            debug!("Analyzing AST for unsafe divisions using DSL");
            
            let vulnerable_functions = AstQuery::new(file)
                .functions()                
                .has_unsafe_divisions();    
            
            let mut findings = Vec::new();
            for node in vulnerable_functions.nodes() {
                let location = node.location(file_path);
                let finding = Finding {
                    description: format!(
                        "Function '{}' contains division operations without zero verification. \
                        This could cause panic if divisor is zero. Consider using checked_div() \
                        or adding explicit zero checks.",
                        node.name()
                    ),
                    severity: Severity::Medium,
                    location,
                    code_snippet: Some(node.snippet()),
                };
                findings.push(finding);
            }
            
            findings
        })
        .build()
}
