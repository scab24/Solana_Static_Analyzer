use crate::analyzer::dsl::{AstQuery, RuleBuilder};
use crate::analyzer::dsl::filters::solana::SolanaFilters;
use crate::analyzer::{Finding, Location, Severity};
use log::debug;
use std::sync::Arc;

pub fn create_rule() -> Arc<dyn crate::analyzer::engine::Rule> {
    RuleBuilder::new()
        .id("solana-missing-error-handling")
        .severity(Severity::Low)
        .title("Missing Error Handling in Public Functions")
        .description("Detects public functions that don't return Result<T> and may fail silently. In Solana contracts, proper error handling is essential for security and debugging.")
        .dsl_rule(|file, file_path| {
            debug!("Analyzing AST for missing error handling using DSL");
            
            let vulnerable_functions = AstQuery::new(file)
                .functions()
                .public_functions()
                .missing_error_handling();
            
            let mut findings = Vec::new();
            for node in vulnerable_functions.nodes() {
                let location = node.location(file_path);
                let finding = Finding {
                    description: format!(
                        "Public function '{}' does not return Result<T> and may fail silently. \
                        Consider returning Result<()> or Result<T> to handle potential errors explicitly. \
                        This is especially important in Solana contracts where silent failures can lead to security issues.",
                        node.name()
                    ),
                    severity: Severity::Low,
                    location,
                    code_snippet: Some(node.snippet()),
                };
                findings.push(finding);
            }
            
            findings
        })
        .build()
}
