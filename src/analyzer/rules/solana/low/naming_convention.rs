use std::sync::Arc;
use log::debug;
use syn::{File, Item, ItemFn, ItemStruct, Ident, visit::{self, Visit}};
use syn::spanned::Spanned;

use crate::analyzer::{Finding, Severity, Location};
use crate::analyzer::engine::{Rule, RuleType};
use crate::analyzer::dsl::{RuleBuilder, AstNode};

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-naming-convention")
        .title("Naming Convention")
        .description("Following naming conventions improves code readability and maintainability")
        .severity(Severity::Low)
        .rule_type(RuleType::Solana)
        // Add tags for classification
        .tag("style")
        .tag("readability")
        .tag("naming")
        // Add references to documentation
        .reference("https://doc.rust-lang.org/1.0.0/style/style/naming/README.html")
        .reference("https://rust-lang.github.io/api-guidelines/naming.html")
        // Define the query to find naming convention issues
        .query(|ast| {
            debug!("Verifying naming conventions with the improved DSL");
            
            // Create a visitor to find naming convention issues
            let mut visitor = NamingConventionVisitor {
                nodes: Vec::new(),
                file: ast,
            };
            
            // Visit the AST
            visitor.visit_file(ast);
            
            // Convert the nodes to findings
            let mut findings = Vec::new();
            
            for node in visitor.nodes {
                let name = node.name.as_deref().unwrap_or("unknown");
                let description = if name.starts_with("function_") {
                    let function_name = name.strip_prefix("function_").unwrap_or(name);
                    format!("The function '{}' does not follow the recommended snake_case convention", function_name)
                } else if name.starts_with("struct_") {
                    let struct_name = name.strip_prefix("struct_").unwrap_or(name);
                    format!("The struct '{}' does not follow the recommended PascalCase convention", struct_name)
                } else {
                    format!("The identifier '{}' does not follow the recommended naming conventions", name)
                };
                
                // Create the finding
                let finding = Finding {
                    description: format!("{} [SUGGESTION]", description),
                    severity: Severity::Low,
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
        .enabled(true)
        .build()
}

struct NamingConventionVisitor<'ast> {
    /// AST nodes containing naming convention issues
    nodes: Vec<AstNode<'ast>>,
    /// AST file being analyzed
    file: &'ast File,
}

impl<'ast> Visit<'ast> for NamingConventionVisitor<'ast> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let name = node.sig.ident.to_string();
        
        // Verify if the name follows snake_case
        if !is_snake_case(&name) {
            // Create an AST node for the function with the incorrect name
            let mut ast_node = AstNode::from_function(node);
            // Overwrite the name to include the prefix
            ast_node.name = Some(format!("function_{}", name));
            
            // Add the node to the list
            self.nodes.push(ast_node);
        }
        
        // Continue visiting the function
        visit::visit_item_fn(self, node);
    }
    
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let name = node.ident.to_string();
        
        // Verify if the name follows PascalCase
        if !is_pascal_case(&name) {
            // Create an AST node for the struct with the incorrect name
            let mut ast_node = AstNode::from_struct(node);
            // Overwrite the name to include the prefix
            ast_node.name = Some(format!("struct_{}", name));
            
            // Add the node to the list
            self.nodes.push(ast_node);
        }
        
        // Continue visiting the struct
        visit::visit_item_struct(self, node);
    }
}

fn is_snake_case(s: &str) -> bool {
    !s.contains(char::is_uppercase) && !s.contains('-')
}

fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() || !s.chars().next().unwrap().is_uppercase() {
        return false;
    }
    
    !s.contains('_') && !s.contains('-')
}
