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
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing owner checks");
            
            AstQuery::new(ast)
                .structs()
                .derives_accounts()                    
                .has_owner_check()                     
        })
        .build()
}
