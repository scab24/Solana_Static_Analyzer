use log::{debug, trace};
use crate::analyzer::dsl::query::{AstQuery, NodeData};

pub trait AnchorInstructionsFilters<'a> {
    fn anchor_instructions(self) -> AstQuery<'a>;
}

impl<'a> AnchorInstructionsFilters<'a> for AstQuery<'a> {
    fn anchor_instructions(self) -> AstQuery<'a> {
        debug!("Filtering Anchor instruction functions");
        let mut new_results = Vec::new();

        for node in self.results() {
            match node.data {
                NodeData::Function(func) => {
                    let is_anchor_instruction = matches!(func.vis, syn::Visibility::Public(_)) &&
                        func.sig.inputs.iter().any(|input| {
                            if let syn::FnArg::Typed(pat_type) = input {
                                let type_str = format!("{:?}", pat_type.ty);
                                type_str.contains("Context")
                            } else {
                                false
                            }
                        });

                    if is_anchor_instruction {
                        trace!("Found Anchor instruction: {}", func.sig.ident);
                        new_results.push(node.clone());
                    }
                }
                NodeData::ImplFunction(func) => {
                    let is_anchor_instruction = matches!(func.vis, syn::Visibility::Public(_)) &&
                        func.sig.inputs.iter().any(|input| {
                            if let syn::FnArg::Typed(pat_type) = input {
                                let type_str = format!("{:?}", pat_type.ty);
                                type_str.contains("Context")
                            } else {
                                false
                            }
                        });

                    if is_anchor_instruction {
                        trace!("Found Anchor instruction in impl: {}", func.sig.ident);
                        new_results.push(node.clone());
                    }
                }
                _ => {}
            }
        }

        AstQuery::from_nodes(new_results)
    }
}
