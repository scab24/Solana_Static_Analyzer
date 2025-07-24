use log::debug;
use std::sync::Arc;
use syn::spanned::Spanned;

use crate::analyzer::dsl::{RuleBuilder, AstQuery};
use crate::analyzer::dsl::filters::SolanaFilters;
use crate::analyzer::engine::{Rule, RuleType};
use crate::analyzer::{Finding, Location, Severity};

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("missing-signer-check")
        .severity(Severity::High)
        .title("Missing Signer Check")
        .description("Detects Anchor instructions that don't properly verify signer permissions")
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing missing signer checks using DSL");
            
            AstQuery::new(ast)
                .structs()
                .derives_accounts()
                .has_missing_signer_checks()
        })
        .build()
}
