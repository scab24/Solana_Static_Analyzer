use log::debug;
use std::sync::Arc;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Attribute, Field, File, ItemStruct, Meta, MetaList};

use crate::analyzer::dsl::{AstNode, RuleBuilder};
use crate::analyzer::engine::{Rule, RuleType};
use crate::analyzer::{Finding, Location, Severity};

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-duplicate-mutable-accounts")
        .title("Duplicate Mutable Accounts Without Constraints")
        .description("Multiple mutable accounts without proper constraints can lead to security vulnerabilities")
        .severity(Severity::Medium)
        .rule_type(RuleType::Solana)
        // Add tags for classification
        .tag("security")
        .tag("accounts")
        // Add references to documentation
        .reference("https://solana.com/es/developers/courses/program-security/duplicate-mutable-accounts")
        // Define the query to find duplicate mutable accounts
        .query(|ast| {
            debug!("Verifying duplicate mutable accounts with the improved DSL");
            
            // Create a visitor to find duplicate mutable accounts
            let mut visitor = DuplicateMutableAccountsVisitor {
                findings: Vec::new(),
                file: ast,
                current_struct: None,
                has_accounts_derive: false,
                has_constraint: false,
                mutable_accounts: Vec::new(),
            };
            
            // Visit the AST
            visitor.visit_file(ast);
            
            visitor.findings
        })
        .build()
}

/// Visitor that finds duplicate mutable accounts without proper constraints
struct DuplicateMutableAccountsVisitor<'ast> {
    /// Findings found
    findings: Vec<Finding>,
    /// AST file being analyzed
    file: &'ast File,
    /// Current struct being analyzed
    current_struct: Option<String>,
    /// Indicates if the current struct derives Accounts
    has_accounts_derive: bool,
    /// Indicates if the current struct has constraints
    has_constraint: bool,
    /// List of mutable accounts found
    mutable_accounts: Vec<String>,
}

impl<'ast> Visit<'ast> for DuplicateMutableAccountsVisitor<'ast> {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        // Save the name of the current struct
        let struct_name = node.ident.to_string();
        self.current_struct = Some(struct_name.clone());

        // Reset the state for the new struct
        self.has_accounts_derive = false;
        self.has_constraint = false;
        self.mutable_accounts.clear();

        // Verify if the struct derives Accounts
        for attr in &node.attrs {
            let meta = attr.meta.clone();
            if let Meta::List(meta_list) = meta {
                if meta_list.path.is_ident("derive") {
                    // Verify if it derives Accounts in the tokens
                    let tokens_str = meta_list.tokens.to_string();
                    if tokens_str.contains("Accounts") {
                        self.has_accounts_derive = true;
                    }
                } else if meta_list.path.is_ident("account") {
                    // Verify if it has constraints
                    let tokens_str = meta_list.tokens.to_string();
                    if tokens_str.contains("constraint") {
                        self.has_constraint = true;
                        debug!("Found constraint in struct attribute: {}", tokens_str);
                    }
                }
            }
        }

        // If the structure derives Accounts, search for mutable fields
        if self.has_accounts_derive {
            // Verify if there are constraints at the field level
            let mut field_constraints = false;

            for field in &node.fields {
                // Verify if the field is mutable and if it has constraints
                let mut has_field_constraint = false;
                let is_mutable = field.attrs.iter().any(|attr| {
                    let meta = attr.meta.clone();
                    if let Meta::List(meta_list) = meta {
                        if meta_list.path.is_ident("account") {
                            let tokens_str = meta_list.tokens.to_string();
                            // Verify if it has constraint in the same attribute as mut
                            if tokens_str.contains("constraint") {
                                has_field_constraint = true;
                                field_constraints = true;
                                debug!("Found constraint in field attribute: {}", tokens_str);
                            }
                            // Verify "mut" in the tokens
                            return tokens_str.contains("mut");
                        }
                    }
                    false
                });

                // If the field is mutable, add it to the list (unless it has constraints)
                if is_mutable && !has_field_constraint {
                    if let Some(ident) = &field.ident {
                        self.mutable_accounts.push(ident.to_string());
                    }
                }
            }

            // If there are at least 2 mutable accounts and no constraints (neither at struct level nor field level), report the issue
            if self.mutable_accounts.len() >= 2 && !self.has_constraint && !field_constraints {
                let struct_name = self.current_struct.as_ref().unwrap();

                // Create a finding for each mutable account
                for account in &self.mutable_accounts {
                    let description = format!(
                        "Multiple mutable accounts detected without proper constraints in struct '{}'. Account: '{}'",
                        struct_name, account
                    );

                    self.findings.push(Finding {
                        description,
                        severity: Severity::Medium,
                        location: Location {
                            file: "file.rs".to_string(),
                            line: 0,
                            column: 0,
                        },
                        code_snippet: Some(format!(
                            "struct {} {{ ... {} ... }}",
                            struct_name, account
                        )),
                    });
                }
            }
        }

        visit::visit_item_struct(self, node);
    }
}
