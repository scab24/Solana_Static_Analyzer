use log::debug;
use std::sync::Arc;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, File, ItemFn, Stmt};

use crate::analyzer::dsl::{AstNode, RuleBuilder};
use crate::analyzer::engine::{Rule, RuleType};
use crate::analyzer::{Finding, Location, Severity};

/// Create a rule that detects unsafe code
pub fn create_rule() -> Arc<dyn Rule> {
    // Use the RuleBuilder with the new features
    RuleBuilder::new()
        .id("solana-unsafe-code")
        .title("Unsafe Code Usage")
        .description("Using unsafe code in Solana programs can lead to security vulnerabilities")
        .severity(Severity::High)
        .rule_type(RuleType::Solana)
        // Add tags for classification
        .tag("security")
        .tag("unsafe")
        // Add references to documentation
        .reference(".")
        .reference("https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html")
        // Define the query to find unsafe code
        .query(|ast| {
            debug!("Verifying unsafe code with the improved DSL");

            // Create a visitor to find unsafe code
            let mut visitor = UnsafeVisitor {
                nodes: Vec::new(),
                file: ast,
            };

            // Visit the AST
            visitor.visit_file(ast);

            // Convert the nodes to findings
            let mut findings = Vec::new();

            for node in visitor.nodes {
                let name = node.name.as_deref().unwrap_or("unknown");

                // Create the descriptive message
                let description = format!(
                    "Unsafe code detected in {}: this can introduce security vulnerabilities",
                    name
                );

                // Create the finding
                let finding = Finding {
                    description: format!("{} [CRITICAL]", description),
                    severity: Severity::High,
                    location: Location {
                        file: "file.rs".to_string(),
                        line: 0,
                        column: 0,
                    },
                    code_snippet: Some("code".to_string()),
                };

                findings.push(finding);
            }

            findings
        })
        // The filter, message and transform functions are no longer needed
        // since the logic has been moved directly to the query function
        .build()
}

/// Visitor that finds unsafe code
struct UnsafeVisitor<'ast> {
    /// AST nodes containing unsafe code
    nodes: Vec<AstNode<'ast>>,
    /// AST file being analyzed
    file: &'ast File,
}

impl<'ast> Visit<'ast> for UnsafeVisitor<'ast> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // Verify if the function is unsafe
        if node.sig.unsafety.is_some() {
            // Create an AST node for the unsafe function
            let mut ast_node = AstNode {
                node_type: crate::analyzer::dsl::query::NodeType::Function,
                data: crate::analyzer::dsl::query::NodeData::Other,
                name: Some(format!("unsafe function {}", node.sig.ident)),
            };

            // Add the node to the list
            self.nodes.push(ast_node);
        }

        // Continue visiting the function body
        visit::visit_item_fn(self, node);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        // Create an AST node for the unsafe block
        let mut ast_node = AstNode {
            node_type: crate::analyzer::dsl::query::NodeType::Expression,
            data: crate::analyzer::dsl::query::NodeData::Other,
            name: Some("unsafe block".to_string()),
        };

        // Add the node to the list
        self.nodes.push(ast_node);

        // Continue visiting the unsafe block
        visit::visit_expr_unsafe(self, node);
    }

    fn visit_block(&mut self, node: &'ast syn::Block) {
        // Verify unsafe blocks in declarations
        for stmt in &node.stmts {
            if let Stmt::Expr(Expr::Unsafe(_), _) = stmt {
                // No need to do anything here, visit_expr_unsafe will handle this
            }
        }

        // Continue visiting the block
        visit::visit_block(self, node);
    }
}
