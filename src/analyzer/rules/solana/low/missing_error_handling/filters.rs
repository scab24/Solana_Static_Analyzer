use log::{debug, trace};
use crate::analyzer::dsl::query::{AstQuery, NodeData};

pub trait MissingErrorHandlingFilters<'a> {
    fn missing_error_handling(self) -> AstQuery<'a>;
}

impl<'a> MissingErrorHandlingFilters<'a> for AstQuery<'a> {
    fn missing_error_handling(self) -> AstQuery<'a> {
        debug!("Filtering functions missing error handling");
        let mut new_results = Vec::new();

        for node in self.results() {
            match node.data {
                NodeData::Function(func) => {
                    let is_public = matches!(func.vis, syn::Visibility::Public(_));
                    let returns_result = returns_result_type(&func.sig.output);
                    
                    if is_public && !returns_result {
                        trace!("Found public function without Result return: {}", func.sig.ident);
                        new_results.push(node.clone());
                    }
                }
                NodeData::ImplFunction(func) => {
                    let is_public = matches!(func.vis, syn::Visibility::Public(_));
                    let returns_result = returns_result_type(&func.sig.output);
                    
                    if is_public && !returns_result {
                        trace!("Found public impl function without Result return: {}", func.sig.ident);
                        new_results.push(node.clone());
                    }
                }
                _ => {}
            }
        }

        AstQuery::from_nodes(new_results)
    }
}

/// Helper function to check if a function returns Result<T>
fn returns_result_type(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Type(_, ty) => {
            let type_str = format!("{:?}", ty);
            type_str.contains("Result")
        }
        syn::ReturnType::Default => false,
    }
}
