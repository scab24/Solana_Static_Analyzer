use crate::analyzer::dsl::{AstQuery, RuleBuilder};
use crate::analyzer::{Rule, Severity};
use std::sync::Arc;
use log::debug;

mod filters;

#[cfg(test)]
mod test;

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("missing-signer-check")
        .title("Missing Signer Check")
        .description("Detects Anchor account fields that may need signer verification")
        .severity(Severity::High)
        .dsl_query(|ast, file_path, span_extractor| {
            debug!("Analyzing missing signer checks using DSL with specialized filters");
            
            AstQuery::new(ast)
                .structs()
                .derives_accounts()
                .filter(|node| {
                    if let crate::analyzer::dsl::query::NodeData::Struct(item_struct) = &node.data {
                        filters::has_missing_signer_checks(item_struct)
                    } else {
                        false
                    }
                })
        })
        .build()
}
