use log::debug;
use std::sync::Arc;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::{Rule, Severity};

// Import our specific filters
mod filters;
use filters::DuplicateMutableAccountsFilters;

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("duplicate-mutable-accounts")
        .severity(Severity::Medium)
        .title("Duplicate Mutable Accounts")
        .description("Detects account structs with multiple mutable references to the same account type, which can lead to unexpected behavior")
        .recommendations(vec![
            "Add constraints to ensure accounts are different: #[account(constraint = account1.key() != account2.key())]",
            "Use a single mutable account reference instead of multiple ones when possible",
            "Implement explicit validation in your instruction handler to prevent the same account being passed multiple times",
            "Consider using Anchor's constraint system to enforce account uniqueness at the framework level"
        ])
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing duplicate mutable accounts");
            
            AstQuery::new(ast)
                .structs()
                .derives_accounts()
                .has_duplicate_mutable_accounts()
        })
        .build()
}
