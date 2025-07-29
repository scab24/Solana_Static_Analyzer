use log::debug;
use std::sync::Arc;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::{Rule, Severity};
use crate::analyzer::engine::RuleType;

// Import our specific filters
mod filters;
use filters::DuplicateMutableAccountsFilters;

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("duplicate-mutable-accounts")
        .severity(Severity::Medium)
        .title("Duplicate Mutable Accounts")
        .description("Detects when an Anchor instruction has multiple mutable accounts that could reference the same account")
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing duplicate mutable accounts using specialized filters");
            
            AstQuery::new(ast)
                .structs()
                .derives_accounts()                    
                .has_duplicate_mutable_accounts()  
        })
        .build()
}
