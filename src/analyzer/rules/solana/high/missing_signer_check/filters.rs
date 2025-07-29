use log::{debug, trace, warn};
use syn::{Fields, Meta};
use crate::analyzer::dsl::query::{AstQuery, NodeData};

use anchor_syn::{AccountsStruct, AccountField, Ty as AnchorTy};
use syn1 as anchor_syn_base;
use quote;

/// Using anchor-syn
/// @todo
pub trait MissingSignerCheckFilters<'a> {
    fn has_missing_signer_checks(self) -> AstQuery<'a>;
}

impl<'a> MissingSignerCheckFilters<'a> for AstQuery<'a> {
    fn has_missing_signer_checks(self) -> AstQuery<'a> {
        debug!("Filtering structs with missing signer checks using hybrid analysis");
        
        let mut new_results = Vec::new();
        
        for node in self.results() {
            if let NodeData::Struct(item_struct) = &node.data {
                let has_missing_signer = analyze_missing_signer_checks_with_defaults(item_struct);
                
                if has_missing_signer {
                    new_results.push(node.clone());
                }
            }
        }
        
        AstQuery::from_nodes(new_results)
    }
}

fn analyze_missing_signer_checks_with_defaults(item_struct: &syn::ItemStruct) -> bool {
    let default_keywords = ["authority", "signer", "owner", "admin"];
    analyze_missing_signer_checks_hybrid(item_struct, &default_keywords)
}

/// ## Parameters
/// - `item_struct`: The struct to analyze (from syn)
/// - `keywords`: Field name keywords to check for signer requirements

pub fn analyze_missing_signer_checks_hybrid(
    item_struct: &syn::ItemStruct,
    keywords: &[&str],
) -> bool {
    debug!("Starting anchor-syn semantic analysis for missing signer checks");
    debug!("Keywords to check: {:?}", keywords);
    
    if !is_anchor_accounts_struct(item_struct) {
        debug!("Struct '{}' is not an Anchor Accounts struct, skipping", item_struct.ident);
        return false;
    }
    
    match try_anchor_syn_analysis(item_struct, keywords) {
        Ok(has_missing_checks) => {
            debug!("Anchor-syn analysis completed successfully");
            has_missing_checks
        }
        Err(e) => {
            warn!("Anchor-syn analysis failed: {}, returning false", e);
            false
        }
    }
}

fn try_anchor_syn_analysis(
    item_struct: &syn::ItemStruct,
    keywords: &[&str],
) -> Result<bool, String> {
    debug!("Attempting anchor-syn semantic analysis");
    
    let syn1_tokens = quote::quote! { #item_struct };
    let syn1_struct: anchor_syn_base::ItemStruct = syn1::parse2(syn1_tokens)
        .map_err(|e| format!("Failed to convert to syn1 format: {}", e))?;
    
    let accounts_struct = AccountsStruct::new(syn1_struct, Vec::new(), None);
    
    debug!("Successfully parsed struct with anchor-syn");
    
    for field in &accounts_struct.fields {
        if should_check_field_for_signer(&field, keywords) {
            if !has_signer_constraint_semantic(&field) {
                debug!("Found missing signer constraint on field");
                return Ok(true);
            }
        }
    }
    
    debug!("All relevant fields have proper signer constraints");
    Ok(false)
}

fn should_check_field_for_signer(field: &AccountField, keywords: &[&str]) -> bool {
    match field {
        AccountField::Field(field_data) => {
            let field_name = field_data.ident.to_string();
            
            let matches_keyword = keywords.iter().any(|&keyword| {
                field_name.to_lowercase().contains(&keyword.to_lowercase())
            });
            
            if matches_keyword {
                debug!("Field '{}' matches signer check keywords", field_name);
                return true;
            }
            
            match &field_data.ty {
                anchor_syn::Ty::AccountInfo => {
                    debug!("Field '{}' is AccountInfo type, checking for signer constraint", field_name);
                    true
                }
                anchor_syn::Ty::UncheckedAccount => {
                    debug!("Field '{}' is UncheckedAccount type, checking for signer constraint", field_name);
                    true
                }
                _ => false,
            }
        }
        AccountField::CompositeField(_) => {
            debug!("Skipping CompositeField for signer constraint analysis");
            false
        }
    }
}

fn has_signer_constraint_semantic(field: &AccountField) -> bool {
    match field {
        AccountField::Field(field_data) => {
            let field_name = field_data.ident.to_string();
            
            match &field_data.ty {
                anchor_syn::Ty::Signer => {
                    debug!("Field '{}' is Signer type - has implicit signer constraint", field_name);
                    return true;
                }
                _ => {}
            }
            
            if field_data.constraints.signer.is_some() {
                debug!("Found explicit signer constraint on field '{}'", field_name);
                return true;
            }
            
            debug!("Field '{}' lacks signer constraint", field_name);
            false
        }
        AccountField::CompositeField(_) => {
            debug!("CompositeField assumed safe for signer constraint analysis");
            true
        }
    }
}

fn is_anchor_accounts_struct(item_struct: &syn::ItemStruct) -> bool {
    for attr in &item_struct.attrs {
        if let Meta::List(meta_list) = &attr.meta {
            if meta_list.path.is_ident("derive") {
                let tokens_str = meta_list.tokens.to_string();
                if tokens_str.contains("Accounts") {
                    trace!("Found Accounts derive on struct '{}'", item_struct.ident);
                    return true;
                }
            }
        }
    }
    
    false
}
