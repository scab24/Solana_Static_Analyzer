use crate::analyzer::dsl::{AstQuery, RuleBuilder};
use crate::analyzer::engine::{Rule, RuleType};
use crate::analyzer::Severity;
use log::debug;
use std::sync::Arc;

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
        .dsl_query(|ast, _file_path, _span_extractor| {
            debug!("Analyzing unsafe code using DSL");
            
            AstQuery::new(ast)
                .functions()
                .uses_unsafe()
        })
        .build()
}
