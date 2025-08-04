
use syn::{ItemStruct, Field, Attribute, Meta};
use quote::{quote, ToTokens};
use log::debug;
use anchor_syn::{AccountsStruct, AccountField};
use syn1;

/// Filter for structs that have missing signer checks using anchor-syn
pub fn has_missing_signer_checks(item_struct: &ItemStruct) -> bool {
    debug!("Checking struct '{}' for missing signer checks using anchor-syn", item_struct.ident);
    
    // Convert syn2 ItemStruct to syn1 format for anchor-syn compatibility
    match convert_to_anchor_struct(item_struct) {
        Ok(accounts_struct) => {
            debug!("Successfully parsed AccountsStruct with {} fields", accounts_struct.fields.len());
            
            // Analyze each field using the EXACT logic from the working implementation
            for anchor_field in &accounts_struct.fields {
                // Use AccountField::Field pattern matching like in the working version
                if let AccountField::Field(field) = anchor_field {
                    // Check for vulnerable field types: AccountInfo, UncheckedAccount, SystemAccount
                    if matches!(
                        field.ty,
                        anchor_syn::Ty::AccountInfo | anchor_syn::Ty::UncheckedAccount | anchor_syn::Ty::SystemAccount
                    ) && !field.constraints.is_signer()
                    {
                        debug!("Found vulnerable field that needs signer verification");
                        return true;
                    }
                }
            }
            false
        },
        Err(e) => {
            debug!("Failed to parse struct with anchor-syn: {}", e);
            // Fallback to basic syn analysis
            has_missing_signer_checks_fallback(item_struct)
        }
    }
}

/// Convert syn2 ItemStruct to anchor-syn AccountsStruct
fn convert_to_anchor_struct(item_struct: &ItemStruct) -> Result<AccountsStruct, String> {
    let struct_tokens = quote::quote! { #item_struct };
    let struct_str = struct_tokens.to_string();
    
    debug!("Converting struct to string: {}", struct_str);
    
    // Parse the string using syn1
    let syn1_struct: syn1::ItemStruct = syn1::parse_str(&struct_str)
        .map_err(|e| format!("Failed to parse struct string: {}\nSource: {}", e, struct_str))?;
    
    debug!("Successfully parsed syn1 struct with {} fields", 
           match &syn1_struct.fields {
               syn1::Fields::Named(fields) => fields.named.len(),
               syn1::Fields::Unnamed(fields) => fields.unnamed.len(),
               syn1::Fields::Unit => 0,
           });
    
    // Parse using accounts_parser::parse
    use anchor_syn::parser::accounts as accounts_parser;
    let accounts_struct = accounts_parser::parse(&syn1_struct)
        .map_err(|e| format!("Failed to parse with accounts_parser: {}\nStruct: {:?}", e, syn1_struct))?;
    
    debug!("Successfully created AccountsStruct with {} fields", accounts_struct.fields.len());
    
    Ok(accounts_struct)
}

/// Fallback analysis using basic syn when anchor-syn fails
fn has_missing_signer_checks_fallback(item_struct: &ItemStruct) -> bool {
    debug!("Using fallback syn analysis for struct '{}'", item_struct.ident);
    
    if let syn::Fields::Named(fields_named) = &item_struct.fields {
        for field in &fields_named.named {
            if let Some(field_name) = &field.ident {
                let field_type = quote::quote!(#field.ty).to_string();
                
                if field_needs_signer_check(field, &field_type) {
                    debug!("Found field '{}' that may need signer verification", field_name);
                    return true;
                }
            }
        }
    }
    
    false
}

/// Check if a specific field needs signer verification (fallback method)
fn field_needs_signer_check(field: &Field, field_type: &str) -> bool {
    if has_signer_constraint(&field.attrs) {
        return false;
    }
    
    field_type.contains("AccountInfo") || 
    field_type.contains("UncheckedAccount") ||
    field_type.contains("SystemAccount") ||
    (field_type.contains("Account") && !field_type.contains("AccountLoader"))
}

/// Check if field has signer constraint in attributes (syn2 compatible)
fn has_signer_constraint(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("account") {
            let tokens = attr.meta.to_token_stream().to_string();
            if tokens.contains("signer") {
                debug!("Found signer constraint in attribute: {}", tokens);
                return true;
            }
        }
    }
    false
}

