use log::debug;
use std::sync::Arc;
use syn::spanned::Spanned;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::dsl::filters::SolanaFilters;
use crate::analyzer::{Finding, Location, Rule, Severity};

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("missing-signer-check")
        .severity(Severity::High)
        .title("Missing Signer Check")
        .description("Detects Anchor instructions that don't properly verify signer permissions")
        .dsl_rule(|file, file_path| {
            debug!("Analyzing AST for missing signer checks using DSL");
            
            let vulnerable_structs = AstQuery::new(file)
                .structs()                      
                .derives_accounts()             
                .has_missing_signer_checks();   
            
            let mut findings = Vec::new();
            for node in vulnerable_structs.nodes() {
                let location = node.location(file_path);
                let finding = Finding {
                    description: format!(
                        "Struct '{}' has authority/user accounts without proper signer verification. \
                        This could allow unauthorized users to execute instructions. \
                        Add #[account(signer)] constraint to authority fields.",
                        node.name()
                    ),
                    severity: Severity::High,
                    location,
                    code_snippet: Some(node.snippet()),
                };
                findings.push(finding);
            }
            
            findings
        })
        .build()
}
