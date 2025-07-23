use log::debug;
use std::sync::Arc;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, File, ItemFn, Stmt};

use crate::analyzer::dsl::{AstNode, RuleBuilder};
use crate::analyzer::engine::{Rule, RuleType};
use crate::analyzer::{Finding, Location, Severity};

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-unsafe-code")
        .title("Unsafe Code Usage")
        .description("Using unsafe code in Solana programs can lead to security vulnerabilities")
        .severity(Severity::High)
        .rule_type(RuleType::Solana)
        .tag("security")
        .tag("unsafe")
        .reference(".")
        .reference("https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html")
        .dsl_query(|ast| {
            debug!("Analyzing unsafe code using DSL");
            
            crate::analyzer::dsl::query::AstQuery::new(ast)
                .functions()      
                .uses_unsafe()     
        })
        .build()
}
