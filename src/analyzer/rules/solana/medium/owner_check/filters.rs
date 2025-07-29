use log::{debug, trace};
use syn::{Meta, Fields, ExprBinary, ExprMacro};
use syn::visit::{self, Visit};
use crate::analyzer::dsl::query::{AstQuery, NodeData};

pub trait OwnerCheckFilters<'a> {
    fn has_owner_check(self) -> AstQuery<'a>;
}

impl<'a> OwnerCheckFilters<'a> for AstQuery<'a> {
    fn has_owner_check(self) -> AstQuery<'a> {
        debug!("Filtering for owner checks");
        let mut new_results = Vec::new();

        for node in self.results() {
            match node.data {
                NodeData::Struct(struct_item) => {
                    if let Fields::Named(named_fields) = &struct_item.fields {
                        let has_owner_check = named_fields.named.iter().any(|field| {
                            field.attrs.iter().any(|attr| {
                                if let Meta::List(meta_list) = &attr.meta {
                                    if meta_list.path.is_ident("account") {
                                        let tokens_str = meta_list.tokens.to_string();
                                        tokens_str.contains("owner") || 
                                        tokens_str.contains("address") ||
                                        (tokens_str.contains("constraint") && tokens_str.contains("owner"))
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            })
                        });

                        if has_owner_check {
                            trace!("Found struct with owner check: {}", struct_item.ident);
                            new_results.push(node.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        AstQuery::from_nodes(new_results)
    }
}

/// Helper visitor to find owner checks in function bodies
pub struct OwnerCheckFinder {
    pub found: bool,
}

impl<'ast> Visit<'ast> for OwnerCheckFinder {
    fn visit_expr_binary(&mut self, binary: &'ast ExprBinary) {
        let left_str = format!("{:?}", binary.left);
        let right_str = format!("{:?}", binary.right);
        
        if (left_str.contains("owner") || right_str.contains("owner")) &&
           matches!(binary.op, syn::BinOp::Eq(_)) {
            self.found = true;
            trace!("Found owner check in binary expression");
        }
        
        visit::visit_expr_binary(self, binary);
    }
    
    fn visit_expr_macro(&mut self, mac: &'ast ExprMacro) {
        // Check for require! or assert! macros with owner checks
        if let Some(ident) = mac.mac.path.get_ident() {
            let macro_name = ident.to_string();
            if macro_name == "require" || macro_name == "assert" || macro_name == "assert_eq" {
                let tokens_str = mac.mac.tokens.to_string();
                if tokens_str.contains("owner") {
                    self.found = true;
                    trace!("Found owner check in {} macro", macro_name);
                }
            }
        }
        
        visit::visit_expr_macro(self, mac);
    }
}
