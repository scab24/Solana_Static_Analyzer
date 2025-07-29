use log::debug;
use std::sync::Arc;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::{Rule, Severity};

// Import our specific filters
mod filters;
use filters::MissingSignerCheckFilters;

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("missing-signer-check")
        .severity(Severity::High)
        .title("Missing Signer Check")
        .description("Detects Anchor instructions that don't properly verify signer permissions")
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing missing signer checks");
            
            AstQuery::new(ast)
                .structs()
                .derives_accounts()                    
                .has_missing_signer_checks()           
        })
        .build()
}
