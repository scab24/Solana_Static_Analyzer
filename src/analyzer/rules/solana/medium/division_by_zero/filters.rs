use log::{debug, trace};
use syn::visit::{self, Visit};
use std::collections::HashMap;
use crate::analyzer::dsl::query::{AstQuery, NodeData};

pub trait DivisionByZeroFilters<'a> {
    fn has_unsafe_divisions(self) -> AstQuery<'a>;
}

impl<'a> DivisionByZeroFilters<'a> for AstQuery<'a> {
    fn has_unsafe_divisions(self) -> AstQuery<'a> {
        debug!("Filtering functions with unsafe division operations");
        
        let mut new_results = Vec::new();
        
        for node in self.results() {
            match &node.data {
                NodeData::Function(func) => {
                    let mut finder = UnsafeDivisionFinder {
                        found: false,
                        safe_variables: HashMap::new(),
                    };
                    
                    syn::visit::visit_block(&mut finder, &func.block);
                    
                    if finder.found {
                        trace!("Found function with unsafe divisions: {}", func.sig.ident);
                        new_results.push(node.clone());
                    }
                }
                NodeData::ImplFunction(func) => {
                    let mut finder = UnsafeDivisionFinder {
                        found: false,
                        safe_variables: HashMap::new(),
                    };
                    
                    syn::visit::visit_block(&mut finder, &func.block);
                    
                    if finder.found {
                        trace!("Found impl function with unsafe divisions: {}", func.sig.ident);
                        new_results.push(node.clone());
                    }
                }
                _ => {}
            }
        }
        
        AstQuery::from_nodes(new_results)
    }
}

/// Helper visitor to find unsafe division operations
struct UnsafeDivisionFinder {
    found: bool,
    safe_variables: HashMap<String, bool>,
}

impl<'ast> Visit<'ast> for UnsafeDivisionFinder {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            if let syn::Pat::Ident(pat_ident) = &local.pat {
                let var_name = pat_ident.ident.to_string();

                if let syn::Expr::Lit(lit_expr) = &*init.expr {
                    match &lit_expr.lit {
                        syn::Lit::Int(int_lit) => {
                            let value = int_lit.base10_digits();
                            if value != "0" {
                                self.safe_variables.insert(var_name, true);
                            }
                        }
                        syn::Lit::Float(float_lit) => {
                            let value = float_lit.base10_digits();
                            if value != "0" && value != "0.0" {
                                self.safe_variables.insert(var_name, true);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        visit::visit_local(self, local);
    }

    fn visit_expr_binary(&mut self, expr: &'ast syn::ExprBinary) {
        if matches!(expr.op, syn::BinOp::Div(_)) {
            let divisor = &expr.right;

            if self.is_potentially_dangerous(divisor) {
                self.found = true;
                trace!("Found unsafe division operation");
            }
        }

        visit::visit_expr_binary(self, expr);
    }
}

impl UnsafeDivisionFinder {
    fn is_potentially_dangerous(&self, expr: &syn::Expr) -> bool {
        match expr {
            syn::Expr::Lit(lit) => {
                match &lit.lit {
                    syn::Lit::Int(int_lit) => {
                        let value = int_lit.base10_digits();
                        value == "0"
                    }
                    syn::Lit::Float(float_lit) => {
                        let value = float_lit.base10_digits();
                        value == "0.0" || value == "0"
                    }
                    _ => false,
                }
            }

            syn::Expr::Path(path) => {
                if let Some(ident) = path.path.get_ident() {
                    let var_name = ident.to_string();

                    if self.safe_variables.contains_key(&var_name) {
                        return false;
                    }
                }

                true
            }

            syn::Expr::Call(_) => true,
            syn::Expr::Field(_) => true,
            syn::Expr::Binary(binary) => matches!(binary.op, syn::BinOp::Sub(_)),
            _ => false,
        }
    }
}
