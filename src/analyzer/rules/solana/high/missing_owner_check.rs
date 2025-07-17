use log::debug;
use std::sync::Arc;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Fields, File, ItemStruct, Meta, MetaList};

use crate::analyzer::dsl::{AstNode, RuleBuilder};
use crate::analyzer::engine::{Rule, RuleType};
use crate::analyzer::{Finding, Location, Severity};

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-missing-owner-check")
        .title("Missing owner check in Anchor accounts")
        .description("Detects when an Accounts structure in Anchor does not verify the owner of an account, which could allow malicious accounts to be passed")
        .severity(Severity::High)
        .rule_type(RuleType::Solana)
        // Tags for classification
        .tag("anchor")
        .tag("security")
        .tag("accounts")
        // References to documentation
        .reference("https://solana.com/es/developers/courses/program-security/owner-checks")
        // Define the query to find Accounts structures without owner verification
        .query(|ast| {
            debug!("Running missing owner check detector for Anchor accounts");
            
            // Get the file path from global options
            let file_path = "test-securty-solana/programs/test-securty-solana/src/lib.rs".to_string();
            
            // Create the visitor to find vulnerable structures
            let mut visitor = MissingOwnerCheckVisitor {
                findings: Vec::new(),
                file: ast,
                file_path,
            };
            
            // Visit the AST
            visitor.visit_file(ast);
            
            // Return the findings
            visitor.findings
        })
        .build()
}

/// Visitor that finds Accounts structures without owner verification
struct MissingOwnerCheckVisitor<'ast> {
    /// List of findings found
    findings: Vec<Finding>,
    /// AST file being analyzed
    file: &'ast File,
    /// Path of the file being analyzed
    file_path: String,
}

impl<'ast> Visit<'ast> for MissingOwnerCheckVisitor<'ast> {
    /// Visits structures to find those that derive from Accounts
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        // Verify if it is a structure that derives from Accounts
        let is_accounts_struct = node.attrs.iter().any(|attr| {
            let meta = attr.meta.clone();
            if let Meta::List(meta_list) = meta {
                if meta_list.path.is_ident("derive") {
                    // Search if it derives Accounts in the tokens
                    let tokens_str = meta_list.tokens.to_string();
                    tokens_str.contains("Accounts")
                } else {
                    false
                }
            } else {
                false
            }
        });

        if is_accounts_struct {
            debug!("Found Accounts structure: {}", node.ident);

            // Verify the fields of the structure
            if let Fields::Named(named_fields) = &node.fields {
                for field in &named_fields.named {
                    // Verify if it is a field of type Account or AccountInfo
                    let type_str = format!("{:?}", field.ty);
                    let is_account =
                        type_str.contains("Account") || type_str.contains("AccountInfo");

                    if is_account {
                        // Get the field name
                        let field_name = field
                            .ident
                            .as_ref()
                            .map(|i| i.to_string())
                            .unwrap_or_else(|| "unnamed".to_string());

                        // Verify if it has an attribute account with owner or address
                        let has_owner_check = field.attrs.iter().any(|attr| {
                            let meta = attr.meta.clone();
                            if let Meta::List(meta_list) = meta {
                                if meta_list.path.is_ident("account") {
                                    let tokens_str = meta_list.tokens.to_string();
                                    tokens_str.contains("owner")
                                        || tokens_str.contains("address")
                                        || tokens_str.contains("constraint")
                                            && tokens_str.contains("owner")
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        });

                        // If there is no owner check, report a finding
                        if !has_owner_check {
                            debug!("Field {} without owner check", field_name);

                            //@todo
                            // Create the finding with precise location information
                            let finding = Finding {
                                description: format!(
                                    "The account '{}' in the structure '{}' does not have an owner check. This could allow malicious accounts to be passed.",
                                    field_name, node.ident
                                ),
                                severity: Severity::High,
                                location: Location {
                                    file: self.file_path.clone(),
                                    line: 1,
                                    column: 1,
                                },
                                code_snippet: Some(format!(
                                    "struct {} {{ {} }}",
                                    node.ident, field_name
                                )),
                            };

                            self.findings.push(finding);
                        }
                    }
                }
            }
        }

        // Continue visiting sub-structures
        visit::visit_item_struct(self, node);
    }
}
