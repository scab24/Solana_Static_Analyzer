use log::{debug, trace};
use crate::analyzer::dsl::query::{AstQuery, NodeData};

pub trait UnsafeCodeFilters<'a> {
    fn uses_unsafe(self) -> AstQuery<'a>;
}

impl<'a> UnsafeCodeFilters<'a> for AstQuery<'a> {
    fn uses_unsafe(self) -> AstQuery<'a> {
        debug!("Filtering functions that use unsafe code");
        let mut new_results = Vec::new();

        for node in self.results() {
            match node.data {
                NodeData::Function(func) => {
                    let uses_unsafe = func.sig.unsafety.is_some() || has_unsafe_in_block(&func.block);
                    if uses_unsafe {
                        trace!("Found unsafe code in function: {}", func.sig.ident);
                        new_results.push(node.clone());
                    }
                }
                NodeData::ImplFunction(func) => {
                    let uses_unsafe = func.sig.unsafety.is_some() || has_unsafe_in_block(&func.block);
                    if uses_unsafe {
                        trace!("Found unsafe code in impl function: {}", func.sig.ident);
                        new_results.push(node.clone());
                    }
                }
                _ => {}
            }
        }

        AstQuery::from_nodes(new_results)
    }
}

/// Helper function to check if a block contains unsafe code
fn has_unsafe_in_block(block: &syn::Block) -> bool {
    use syn::visit::Visit;
    
    struct UnsafeVisitor {
        found_unsafe: bool,
    }
    
    impl<'ast> Visit<'ast> for UnsafeVisitor {
        fn visit_expr_unsafe(&mut self, _node: &'ast syn::ExprUnsafe) {
            self.found_unsafe = true;
        }
    }
    
    let mut visitor = UnsafeVisitor { found_unsafe: false };
    visitor.visit_block(block);
    visitor.found_unsafe
}
