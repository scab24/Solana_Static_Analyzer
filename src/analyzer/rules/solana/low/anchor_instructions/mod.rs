use log::debug;
use std::sync::Arc;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::{Rule, Severity};

// Import our specific filters
mod filters;
use filters::AnchorInstructionsFilters;

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("anchor-instructions")
        .severity(Severity::Low)
        .title("Anchor Instructions Detection")
        .description("Detects functions that are Anchor program instructions (public functions with Context parameter)")
        .recommendations(vec![
            "Ensure all instruction handlers return Result<()> for proper error handling",
            "Add proper account validation using constraints in your Context struct",
            "Consider adding access control checks at the beginning of instruction handlers",
            "Use #[access_control] attribute for complex authorization logic",
            "Document instruction parameters and expected account states"
        ])
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing Anchor instructions");
            
            AstQuery::new(ast)
                .functions()                           
                .anchor_instructions()                 
        })
        .build()
}
