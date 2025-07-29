use log::{debug, trace};
use syn::{Meta, Fields};
use crate::analyzer::dsl::query::{AstQuery, NodeData};

pub trait DuplicateMutableAccountsFilters<'a> {
    fn has_duplicate_mutable_accounts(self) -> AstQuery<'a>;
}

impl<'a> DuplicateMutableAccountsFilters<'a> for AstQuery<'a> {
    fn has_duplicate_mutable_accounts(self) -> AstQuery<'a> {
        debug!("Filtering structs with duplicate mutable accounts (SOLANA-001)");
        let mut new_results = Vec::new();
        
        for node in self.results() {
            if let NodeData::Struct(struct_item) = &node.data {
                let mut mutable_account_count = 0;
                let mut mutable_accounts_with_constraints = 0;
                
                // Check if struct has fields
                if let Fields::Named(fields) = &struct_item.fields {
                    // Check each field for mutable accounts
                    let mut all_constraints = Vec::new();
                    
                    // First pass: collect all constraints
                    for field in &fields.named {
                        for attr in &field.attrs {
                            if let Meta::List(meta_list) = &attr.meta {
                                if meta_list.path.is_ident("account") {
                                    let tokens_str = meta_list.tokens.to_string();
                                    if tokens_str.contains("constraint") {
                                        all_constraints.push(tokens_str.clone());
                                    }
                                }
                            }
                        }
                    }
                    
                    // Second pass: check mutable accounts
                    for field in &fields.named {
                        let mut is_mutable = false;
                        let mut has_field_constraint = false;
                        
                        // Check field attributes
                        for attr in &field.attrs {
                            if let Meta::List(meta_list) = &attr.meta {
                                if meta_list.path.is_ident("account") {
                                    let tokens_str = meta_list.tokens.to_string();
                                    
                                    // Check if it's mutable
                                    if tokens_str.contains("mut") {
                                        is_mutable = true;
                                    }
                                    
                                    // Check if it has constraints that prevent duplication
                                    if tokens_str.contains("constraint") || 
                                       tokens_str.contains("seeds") ||
                                       tokens_str.contains("bump") ||
                                       tokens_str.contains("!=") ||
                                       tokens_str.contains("key()") {
                                        has_field_constraint = true;
                                        trace!("Field {:?} has constraint that prevents duplication: {}", field.ident, tokens_str);
                                    }
                                }
                            }
                        }
                        
                        // Check if this field is referenced in any constraint
                        if is_mutable && !has_field_constraint {
                            if let Some(field_name) = &field.ident {
                                let field_name_str = field_name.to_string();
                                for constraint in &all_constraints {
                                    if constraint.contains(&field_name_str) && constraint.contains("!=") {
                                        has_field_constraint = true;
                                        trace!("Field {:?} is protected by bidirectional constraint: {}", field.ident, constraint);
                                        break;
                                    }
                                }
                            }
                        }
                        
                        // Count mutable accounts and track constraints
                        if is_mutable {
                            mutable_account_count += 1;
                            if has_field_constraint {
                                mutable_accounts_with_constraints += 1;
                            } else {
                                trace!("Found mutable account without constraints: {:?}", field.ident);
                            }
                        }
                    }
                }
                
                // If we have 2+ mutable accounts without proper constraints, it's vulnerable
                if mutable_account_count >= 2 && mutable_account_count != mutable_accounts_with_constraints {
                    trace!("SOLANA-001: Found struct '{}' with {} mutable accounts without constraints", 
                           struct_item.ident, mutable_account_count - mutable_accounts_with_constraints);
                    new_results.push(node.clone());
                }
            }
        }
        
        AstQuery::from_nodes(new_results)
    }
}
