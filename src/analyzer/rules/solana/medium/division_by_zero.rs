use std::sync::Arc;
use log::debug;
use syn::{File, Expr, ExprBinary, BinOp, visit::{self, Visit}};
use syn::spanned::Spanned;

use crate::analyzer::{Finding, Severity, Location};
use crate::analyzer::engine::{Rule, RuleType};
use crate::analyzer::dsl::{RuleBuilder};

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-division-by-zero")
        .title("Division Without Zero Check")
        .description("Detect division operations without zero verification")
        .severity(Severity::Medium)
        .rule_type(RuleType::Solana)
        // Tags for classification
        .tag("arithmetic")
        .tag("panic")
        .tag("division")
        .tag("security")
        // References to documentation
        .reference(".")
        .reference("https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html")
        // Define the query to find divisions without verification
        .query(|ast| {
            debug!("Executing division by zero detector");
            
            // Create the visitor to find dangerous divisions
            let mut visitor = DivisionByZeroVisitor {
                findings: Vec::new(),
                file: ast,
                safe_variables: std::collections::HashMap::new(),
            };
            
            // Visit the AST
            visitor.visit_file(ast);
            
            // Return the findings
            visitor.findings
        })
        .build()
}

/// Visitor that finds division operations without zero verification
struct DivisionByZeroVisitor<'ast> {
    /// List of findings found
    findings: Vec<Finding>,
    /// AST file being analyzed
    file: &'ast File,
    /// Map of variables that we know are different from zero
    safe_variables: std::collections::HashMap<String, bool>,
}

impl<'ast> Visit<'ast> for DivisionByZeroVisitor<'ast> {
    /// Visit local assignments (let x = 5;)
    fn visit_local(&mut self, local: &'ast syn::Local) {
        // Verify if it's a simple assignment with a literal value
        if let Some(init) = &local.init {
            if let syn::Pat::Ident(pat_ident) = &local.pat {
                let var_name = pat_ident.ident.to_string();
                
                // Verify if the assigned value is a safe literal (different from zero)
                if let syn::Expr::Lit(lit_expr) = &*init.expr {
                    match &lit_expr.lit {
                        syn::Lit::Int(int_lit) => {
                            let value = int_lit.base10_digits();
                            if value != "0" {
                                debug!("Variable {} initialized with safe literal: {}", var_name, value);
                                self.safe_variables.insert(var_name, true);
                            }
                        },
                        syn::Lit::Float(float_lit) => {
                            let value = float_lit.base10_digits();
                            if value != "0" && value != "0.0" {
                                debug!("Variable {} initialized with safe literal: {}", var_name, value);
                                self.safe_variables.insert(var_name, true);
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        
        // Continue visiting
        visit::visit_local(self, local);
    }
    
    /// Visit binary expressions (like arithmetic operations)
    fn visit_expr_binary(&mut self, expr: &'ast ExprBinary) {
        // Verify if it's a division operation
        if matches!(expr.op, BinOp::Div(_)) {
            debug!("Found division operation");
            
            let divisor = &expr.right;
            
            // Verify if the divisor is potentially zero or unverified
            if self.is_potentially_zero_or_unverified(divisor) {
                debug!("Dangerous division detected");
                
                // Create the finding
                let finding = Finding {
                    description: format!(
                        "Division operation detected without zero verification. The divisor '{}' could be zero, causing panic",
                        self.expr_to_string(divisor)
                    ),
                    severity: Severity::Medium,
                    location: Location {
                        file: "archivo.rs".to_string(),
                        line: 0,
                        column: 0,
                    },
                    code_snippet: Some(format!(
                        "{} / {}", 
                        self.expr_to_string(&expr.left),
                        self.expr_to_string(&expr.right)
                    )),
                };
                
                self.findings.push(finding);
            }
        }
        
        // Continue visiting sub-expressions
        visit::visit_expr_binary(self, expr);
    }
    
    /// Visit expressions with operators (like /=)
    fn visit_expr_assign(&mut self, expr: &'ast syn::ExprAssign) {
        // Verify if it's an assignment with division by looking at the right side
        if let Expr::Binary(binary_expr) = &*expr.right {
            if matches!(binary_expr.op, BinOp::Div(_)) {
                debug!("Found assignment with division");
                
                // The divisor is the right side of the binary operation
                let divisor = &binary_expr.right;
                
                if self.is_potentially_zero_or_unverified(divisor) {
                    debug!("Dangerous assignment with division detected");
                    
                    let finding = Finding {
                        description: format!(
                            "Assignment with division detected without zero verification. The divisor '{}' could be zero",
                            self.expr_to_string(divisor)
                        ),
                        severity: Severity::Medium,
                        location: Location {
                            file: "archivo.rs".to_string(),
                            line: 0,
                            column: 0,
                        },
                        code_snippet: Some(format!(
                            "{} = {} / {}", 
                            self.expr_to_string(&expr.left),
                            self.expr_to_string(&binary_expr.left),
                            self.expr_to_string(&binary_expr.right)
                        )),
                    };
                    
                    self.findings.push(finding);
                }
            }
        }
        
        // Continue visiting
        visit::visit_expr_assign(self, expr);
    }
    
    /// Visit method calls to detect division methods
    fn visit_expr_method_call(&mut self, expr: &'ast syn::ExprMethodCall) {
        let method_name = expr.method.to_string();
        
        // Verify if it's a division method
        if method_name == "div" || method_name == "checked_div" {
            debug!("Found division method call: {}", method_name);
            
            if method_name == "checked_div" {
                let finding = Finding {
                    description: format!(
                        "Checked_div usage detected. Ensure the Option result is handled correctly"
                    ),
                    severity: Severity::Low,
                    location: Location {
                        file: "archivo.rs".to_string(),
                        line: 0,
                        column: 0,
                    },
                    code_snippet: Some(format!("x.checked_div(y)")),
                };
                
                self.findings.push(finding);
            }
        }
        
        // Continue visiting
        visit::visit_expr_method_call(self, expr);
    }
}

impl<'ast> DivisionByZeroVisitor<'ast> {
    /// Determines if an expression is potentially zero or unverified
    fn is_potentially_zero_or_unverified(&self, expr: &Expr) -> bool {
        match expr {
            // Literals
            Expr::Lit(lit) => {
                match &lit.lit {
                    syn::Lit::Int(int_lit) => {
                        let value = int_lit.base10_digits();
                        debug!("Literal integer divisor found: {}", value);
                        value == "0"
                    }
                    syn::Lit::Float(float_lit) => {
                        // If it's literally 0.0, it's dangerous and if it's different from 0.0, it's safe
                        let value = float_lit.base10_digits();
                        debug!("Literal float divisor found: {}", value);
                        value == "0.0" || value == "0"
                    }
                    _ => false,
                }
            }
            
            Expr::Path(path) => {
                if let Some(ident) = path.path.get_ident() {
                    let var_name = ident.to_string();
                    
                    // Verify if the variable is in our map of safe variables
                    if self.safe_variables.contains_key(&var_name) {
                        debug!("Variable {} detected as divisor - SAFE (known non-zero value)", var_name);
                        return false; 
                    }
                }
                
                debug!("Variable detected as divisor - requires verification");
                true 
            }
            
            Expr::Call(_) => {
                debug!("Function call as divisor - potentially dangerous");
                true
            }
            
            Expr::Field(_) => {
                debug!("Field access as divisor - potentially dangerous");
                true
            }
            
            Expr::Binary(binary) => {
                match binary.op {
                    BinOp::Sub(_) => true,
                    _ => false,
                }
            }
            
            _ => false,
        }
    }
    
    /// Converts an expression to string for display in the message
    fn expr_to_string(&self, expr: &Expr) -> String {
        match expr {
            Expr::Lit(lit) => {
                match &lit.lit {
                    syn::Lit::Int(int_lit) => int_lit.base10_digits().to_string(),
                    syn::Lit::Float(float_lit) => float_lit.base10_digits().to_string(),
                    syn::Lit::Str(str_lit) => format!("\"{}\"", str_lit.value()),
                    syn::Lit::Bool(bool_lit) => bool_lit.value.to_string(),
                    _ => "literal".to_string(),
                }
            }
            Expr::Path(path) => {
                if let Some(ident) = path.path.get_ident() {
                    ident.to_string()
                } else {
                    "path".to_string()
                }
            }
            Expr::Field(field) => {
                format!("field_access")
            }
            Expr::Call(_) => "function_call()".to_string(),
            Expr::MethodCall(method) => {
                format!("method_call.{}()", method.method)
            }
            Expr::Binary(binary) => {
                format!("binary_operation")
            }
            _ => "expression".to_string(),
        }
    }
}
