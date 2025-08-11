
use syn::{ItemStruct, Field, Attribute};
use quote::{quote, ToTokens};
use log::debug;
use anchor_syn::{AccountsStruct, AccountField};
use syn1;

/// Filter for structs that have missing signer checks using anchor-syn
pub fn has_missing_signer_checks(item_struct: &ItemStruct) -> bool {
    debug!("Checking struct '{}' for missing signer checks using anchor-syn", item_struct.ident);
    
    if !is_accounts_struct(item_struct) {
        debug!("Struct '{}' is not an Accounts struct, skipping", item_struct.ident);
        return false;
    }
    
    match convert_to_anchor_struct_optimized(item_struct) {
        Ok(accounts_struct) => {
            debug!("Successfully parsed AccountsStruct with {} fields", accounts_struct.fields.len());
            
            for anchor_field in &accounts_struct.fields {
                if let AccountField::Field(field) = anchor_field {
                    if matches!(
                        field.ty,
                        anchor_syn::Ty::AccountInfo | anchor_syn::Ty::UncheckedAccount | anchor_syn::Ty::SystemAccount
                    ) && !field.constraints.is_signer()
                    {
                        debug!("Found vulnerable field '{}' that needs signer verification", field.ident);
                        return true;
                    }
                }
            }
            false
        },
        Err(e) => {
            debug!("Failed to parse struct with anchor-syn: {e}, using fallback");
            // Fallback to basic syn analysis
            has_missing_signer_checks_fallback(item_struct)
        }
    }
}

fn is_accounts_struct(item_struct: &ItemStruct) -> bool {
    for attr in &item_struct.attrs {
        if attr.path().is_ident("derive") {
            let tokens = attr.meta.to_token_stream().to_string();
            if tokens.contains("Accounts") {
                debug!("Found Accounts derive on struct '{}'", item_struct.ident);
                return true;
            }
        }
    }
    false
}

fn convert_to_anchor_struct_optimized(item_struct: &ItemStruct) -> Result<AccountsStruct, String> {
    let struct_source = generate_clean_struct_source(item_struct);
    
    debug!("Generated clean struct source: {struct_source}");
    
    let syn1_struct: syn1::ItemStruct = syn1::parse_str(&struct_source)
        .map_err(|e| format!("Failed to parse clean struct source: {e}\nSource: {struct_source}"))?;
    
    debug!("Successfully parsed syn1 struct with {} fields", 
           match &syn1_struct.fields {
               syn1::Fields::Named(fields) => fields.named.len(),
               syn1::Fields::Unnamed(fields) => fields.unnamed.len(),
               syn1::Fields::Unit => 0,
           });
    
    // Parse using accounts_parser::parse
    use anchor_syn::parser::accounts as accounts_parser;
    let accounts_struct = accounts_parser::parse(&syn1_struct)
        .map_err(|e| format!("Failed to parse with accounts_parser: {e}\nStruct: {syn1_struct:?}"))?;
    
    debug!("Successfully created AccountsStruct with {} fields", accounts_struct.fields.len());
    
    Ok(accounts_struct)
}

fn generate_clean_struct_source(item_struct: &ItemStruct) -> String {
    let mut source = String::new();
    for attr in &item_struct.attrs {
        source.push_str(&format!("{}\n", quote!(#attr)));
    }
    
    let vis = &item_struct.vis;
    let ident = &item_struct.ident;
    let generics = &item_struct.generics;
    
    source.push_str(&format!("{} struct {}{} ", quote!(#vis), ident, quote!(#generics)));
    
    match &item_struct.fields {
        syn::Fields::Named(fields_named) => {
            source.push_str("{\n");
            for field in &fields_named.named {
                
                for attr in &field.attrs {
                    source.push_str(&format!("    {}\n", quote!(#attr)));
                }
                
                let vis = &field.vis;
                let ident = field.ident.as_ref().unwrap();
                let ty = &field.ty;
                source.push_str(&format!("    {} {}: {},\n", quote!(#vis), ident, quote!(#ty)));
            }
            source.push_str("}\n");
        },
        syn::Fields::Unnamed(fields_unnamed) => {
            source.push('(');
            for (i, field) in fields_unnamed.unnamed.iter().enumerate() {
                if i > 0 { source.push_str(", "); }
                source.push_str(&quote!(#field.ty).to_string());
            }
            source.push_str(");\n");
        },
        syn::Fields::Unit => {
            source.push_str(";\n");
        }
    }
    
    source
}

/// Fallback analysis using basic syn when anchor-syn fails
fn has_missing_signer_checks_fallback(item_struct: &ItemStruct) -> bool {
    debug!("Using fallback syn analysis for struct '{}'", item_struct.ident);
    
    if let syn::Fields::Named(fields_named) = &item_struct.fields {
        for field in &fields_named.named {
            if let Some(field_name) = &field.ident {
                let field_type = quote::quote!(#field.ty).to_string();
                
                if field_needs_signer_check(field, &field_type) {
                    debug!("Found field '{field_name}' that may need signer verification");
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
                debug!("Found signer constraint in attribute: {tokens}");
                return true;
            }
        }
    }
    false
}

