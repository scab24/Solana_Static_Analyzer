use log::debug;
use std::sync::Arc;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::{Rule, Severity};

mod filters;
use filters::OwnerCheckFilters;

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("owner-check")
        .severity(Severity::Medium)
        .title("Owner Check Validation")
        .description("Detects structs that properly implement owner checks for account validation")
        .recommendations(vec![
            "Add explicit owner validation in your account struct using #[account(constraint = account.owner == expected_owner)] or similar patterns",
            "Use Anchor's built-in Account<'info, T> wrapper which automatically validates the account owner",
            "Implement manual owner checks in your instruction handler before processing the account",
            "Consider using Anchor's #[account(owner = program_id)] constraint for program-owned accounts"
        ])
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing owner checks");
            
            AstQuery::new(ast)
                .structs()
                .derives_accounts()                    
                .has_owner_check()                     
        })
        .build()
}
