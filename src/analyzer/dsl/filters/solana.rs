use log::{debug, trace};
use syn::{Meta, Fields, Attribute, ExprBinary, ExprMacro};
use syn::visit::{self, Visit};
use quote::ToTokens;
use std::collections::HashMap;
use crate::analyzer::dsl::query::{AstQuery, NodeData, AstNode};

/// This trait extends the basic AST query functionality
pub trait SolanaFilters<'a> {
    /// Filter for structs that derive the Accounts trait (Anchor pattern)
    fn derives_accounts(self) -> AstQuery<'a>;
    
    /// Filter for structs with duplicate mutable accounts (SOLANA-001)
    fn has_duplicate_mutable_accounts(self) -> AstQuery<'a>;
    
    /// Filter structs that have missing signer checks
    fn has_missing_signer_checks(self) -> AstQuery<'a>;

    /// Filter structs/functions that have owner checks
    fn has_owner_check(self) -> AstQuery<'a>;

    /// Filter functions that are Anchor program instructions
    fn anchor_instructions(self) -> AstQuery<'a>;

    /// Filter functions that have unsafe division operations
    fn has_unsafe_divisions(self) -> AstQuery<'a>;

    /// Filter for public functions only
    fn public_functions(self) -> AstQuery<'a>;

    /// Filter functions that don't return Result<T> (missing error handling)
    fn missing_error_handling(self) -> AstQuery<'a>;
}

impl<'a> SolanaFilters<'a> for AstQuery<'a> {
    fn derives_accounts(self) -> AstQuery<'a> {
        debug!("Filtering structs that derive Accounts (Anchor pattern)");
        let mut new_results = Vec::new();
        
        for node in self.results() {
            if let NodeData::Struct(struct_item) = &node.data {
                // Check if the struct derives Accounts
                for attr in &struct_item.attrs {
                    if let Meta::List(meta_list) = &attr.meta {
                        if meta_list.path.is_ident("derive") {
                            let tokens_str = meta_list.tokens.to_string();
                            if tokens_str.contains("Accounts") {
                                trace!("Found struct deriving Accounts: {}", struct_item.ident);
                                new_results.push(node.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        AstQuery::from_nodes(new_results)
    }

    fn has_duplicate_mutable_accounts(self) -> AstQuery<'a> {
        debug!("Filtering structs with duplicate mutable accounts (SOLANA-001)");
        let mut new_results = Vec::new();
        
        for node in self.results() {
            if let NodeData::Struct(struct_item) = &node.data {
                let mut mutable_account_count = 0;
                let mut has_constraints = false;
                
                // Check if struct has fields
                if let Fields::Named(fields) = &struct_item.fields {
                    // Check each field for mutable accounts
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
                                       tokens_str.contains("bump") {
                                        has_field_constraint = true;
                                        has_constraints = true;
                                    }
                                }
                            }
                        }
                        
                        // Count mutable accounts
                        if is_mutable {
                            mutable_account_count += 1;
                            if !has_field_constraint {
                                trace!("Found mutable account without constraints: {:?}", field.ident);
                            }
                        }
                    }
                }
                
                // If we have 2+ mutable accounts without proper constraints, it's vulnerable
                if mutable_account_count >= 2 && !has_constraints {
                    trace!("SOLANA-001: Found struct '{}' with {} mutable accounts without constraints", 
                           struct_item.ident, mutable_account_count);
                    new_results.push(node.clone());
                }
            }
        }
        
        AstQuery::from_nodes(new_results)
    }

    fn has_missing_signer_checks(self) -> AstQuery<'a> {
        debug!("Filtering structs with missing signer checks");
        
        let mut new_results = Vec::new();
        
        for node in self.results() {
            if let NodeData::Struct(item_struct) = &node.data {
                let mut has_missing_signer = false;
                
                if let Fields::Named(fields) = &item_struct.fields {
                    for field in &fields.named {
                        // Check if field name suggests it should be a signer
                        if let Some(field_name) = &field.ident {
                            let name = field_name.to_string();
                            if name.contains("authority") || name.contains("user") || name.contains("owner") {
                                // Check if it has signer constraint
                                let mut has_signer_constraint = false;
                                for attr in &field.attrs {
                                    if let Meta::List(meta_list) = &attr.meta {
                                        if meta_list.path.is_ident("account") {
                                            let tokens_str = meta_list.tokens.to_string();
                                            if tokens_str.contains("signer") {
                                                has_signer_constraint = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                                
                                if !has_signer_constraint {
                                    has_missing_signer = true;
                                    trace!("Found field '{}' that should be a signer but isn't", name);
                                    break;
                                }
                            }
                        }
                    }
                }
                
                if has_missing_signer {
                    new_results.push(node.clone());
                }
            }
        }
        
        AstQuery::from_nodes(new_results)
    }

    /// Filter structs/functions that have owner checks
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

    /// Filter functions that are Anchor program instructions
    fn anchor_instructions(self) -> AstQuery<'a> {
        debug!("Filtering Anchor instruction functions");
        let mut new_results = Vec::new();

        for node in self.results() {
            match node.data {
                NodeData::Function(func) => {
                    // Check if function is public and has Context parameter
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
                    // Check if function is public and has Context parameter
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

    fn has_unsafe_divisions(self) -> AstQuery<'a> {
        debug!("Filtering functions with unsafe division operations");
        
        let mut new_results = Vec::new();
        
        for node in self.results() {
            match &node.data {
                NodeData::Function(func) => {
                    let mut finder = UnsafeDivisionFinder {
                        found: false,
                        safe_variables: std::collections::HashMap::new(),
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
                        safe_variables: std::collections::HashMap::new(),
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

    fn public_functions(self) -> AstQuery<'a> {
        debug!("Filtering for public functions only");
        
        let mut new_results = Vec::new();
        
        for node in self.results() {
            match &node.data {
                NodeData::Function(func) => {
                    debug!("Checking function: {} with visibility: {:?}", func.sig.ident, func.vis);
                    // Check if function has pub visibility
                    if matches!(func.vis, syn::Visibility::Public(_)) {
                        debug!("Found public function: {}", func.sig.ident);
                        new_results.push(node.clone());
                    } else {
                        debug!("Function {} is not public, skipping", func.sig.ident);
                    }
                }
                NodeData::ImplFunction(func) => {
                    debug!("Checking impl function: {} with visibility: {:?}", func.sig.ident, func.vis);
                    // Check if function has pub visibility
                    if matches!(func.vis, syn::Visibility::Public(_)) {
                        debug!("Found public impl function: {}", func.sig.ident);
                        new_results.push(node.clone());
                    } else {
                        debug!("Impl function {} is not public, skipping", func.sig.ident);
                    }
                }
                _ => {}
            }
        }
        
        debug!("Found {} public functions", new_results.len());
        AstQuery::from_nodes(new_results)
    }

    fn missing_error_handling(self) -> AstQuery<'a> {
        debug!("Filtering functions with missing error handling (not returning Result<T>)");
        
        let mut new_results = Vec::new();
        
        for node in self.results() {
            match &node.data {
                NodeData::Function(func) => {
                    // Check if return type is NOT Result<T>
                    let returns_result = match &func.sig.output {
                        syn::ReturnType::Default => {
                            debug!("Function {} has no return type (returns ())", func.sig.ident);
                            false
                        },
                        syn::ReturnType::Type(_, ty) => {
                            // Convert type to string and check if it contains "Result"
                            let type_str = quote::ToTokens::to_token_stream(ty).to_string();
                            debug!("Function {} returns: {}", func.sig.ident, type_str);
                            type_str.contains("Result")
                        }
                    };
                    
                    if !returns_result {
                        debug!("Found function without Result return type: {}", func.sig.ident);
                        new_results.push(node.clone());
                    } else {
                        debug!("Function {} returns Result, skipping", func.sig.ident);
                    }
                }
                NodeData::ImplFunction(func) => {
                    // Check if return type is NOT Result<T>
                    let returns_result = match &func.sig.output {
                        syn::ReturnType::Default => {
                            debug!("Impl function {} has no return type (returns ())", func.sig.ident);
                            false
                        },
                        syn::ReturnType::Type(_, ty) => {
                            // Convert type to string and check if it contains "Result"
                            let type_str = quote::ToTokens::to_token_stream(ty).to_string();
                            debug!("Impl function {} returns: {}", func.sig.ident, type_str);
                            type_str.contains("Result")
                        }
                    };
                    
                    if !returns_result {
                        debug!("Found impl function without Result return type: {}", func.sig.ident);
                        new_results.push(node.clone());
                    } else {
                        debug!("Impl function {} returns Result, skipping", func.sig.ident);
                    }
                }
                _ => {}
            }
        }
        
        debug!("Found {} functions with missing error handling", new_results.len());
        AstQuery::from_nodes(new_results)
    }
}

/// Helper visitor to find owner checks in function bodies
struct OwnerCheckFinder {
    found: bool,
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
        
        // Continue visiting sub-expressions
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
        
        // Continue visiting sub-expressions
        visit::visit_expr_macro(self, mac);
    }
}

/// Helper visitor to find unsafe division operations
struct UnsafeDivisionFinder {
    found: bool,
    safe_variables: std::collections::HashMap<String, bool>,
}

impl<'ast> Visit<'ast> for UnsafeDivisionFinder {
    /// Visit local assignments (let x = 5;)
    fn visit_local(&mut self, local: &'ast syn::Local) {
        // Check if it's a simple assignment with a literal value
        if let Some(init) = &local.init {
            if let syn::Pat::Ident(pat_ident) = &local.pat {
                let var_name = pat_ident.ident.to_string();

                // Check if the assigned value is a safe literal (non-zero)
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

        // Continue visiting
        visit::visit_local(self, local);
    }

    /// Visit binary expressions (arithmetic operations..)
    fn visit_expr_binary(&mut self, expr: &'ast syn::ExprBinary) {
        // Check if it's a division operation
        if matches!(expr.op, syn::BinOp::Div(_)) {
            let divisor = &expr.right;

            // Check if the divisor is potentially zero or unverified
            if self.is_potentially_dangerous(divisor) {
                self.found = true;
                trace!("Found unsafe division operation");
            }
        }

        // Continue visiting sub-expressions
        visit::visit_expr_binary(self, expr);
    }
}

impl UnsafeDivisionFinder {
    /// Determines if an expression is potentially dangerous for division
    fn is_potentially_dangerous(&self, expr: &syn::Expr) -> bool {
        match expr {
            // Literals
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

                    // Check if the variable is in our map of safe variables
                    if self.safe_variables.contains_key(&var_name) {
                        return false;
                    }
                }

                // Variable detected as divisor - requires verification
                true
            }

            syn::Expr::Call(_) => true,
            syn::Expr::Field(_) => true,
            syn::Expr::Binary(binary) => matches!(binary.op, syn::BinOp::Sub(_)),
            _ => false,
        }
    }
}
