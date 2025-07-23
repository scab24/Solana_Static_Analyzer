use log::debug;
use std::sync::Arc;
use syn::spanned::Spanned;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::dsl::filters::SolanaFilters;
use crate::analyzer::{Finding, Location, Rule, Severity};

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("duplicate-mutable-accounts")
        .severity(Severity::Medium)
        .title("Duplicate Mutable Accounts")
        .description("Detects when an Anchor instruction has multiple mutable accounts that could reference the same account")
        .dsl_rule(|file, file_path| {
            debug!("Analyzing AST for duplicate mutable accounts using DSL");
            
            let vulnerable_structs = AstQuery::new(file)
                .structs()
                .derives_accounts()
                .has_duplicate_mutable_accounts();
            
            let mut findings = Vec::new();
            for node in vulnerable_structs.nodes() {
                let location = node.location(file_path);
                let finding = Finding {
                    description: format!(
                        "Struct '{}' has multiple mutable accounts without proper constraints. \
                        This could allow the same account to be passed multiple times, \
                        leading to unexpected behavior or vulnerabilities.",
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