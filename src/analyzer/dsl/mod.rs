use syn::{File, Item, Expr, Ident, ItemFn, ItemStruct, ItemEnum, Path, ExprPath, ExprMethodCall};
use syn::spanned::Spanned;
use proc_macro2::Span;
use std::collections::HashSet;
use log::{debug, trace};

/// Represent a node in the AST with additional information
#[derive(Debug)]
pub struct AstNode<'a> {
    /// Node type
    pub node_type: NodeType,
    /// Span of the node in the source code
    pub span: Span,
    /// Data specific of the node
    pub data: NodeData<'a>,
}

/// Types of nodes we can represent
#[derive(Debug, PartialEq, Clone)]
pub enum NodeType {
    /// File
    File,
    /// Function
    Function,
    /// Struct
    Struct,
    /// Enum
    Enum,
    /// Expression
    Expression,
    /// Method call
    MethodCall,
    /// Function call
    FunctionCall,
    /// Member access
    MemberAccess,
    /// Identifier
    Identifier,
    /// Path
    Path,
    /// Other type of node
    Other,
}

/// Data specific of each type of node
#[derive(Debug)]
pub enum NodeData<'a> {
    /// File
    File(&'a File),
    /// Function
    Function(&'a ItemFn),
    /// Struct
    Struct(&'a ItemStruct),
    /// Enum
    Enum(&'a ItemEnum),
    /// Expression
    Expression(&'a Expr),
    /// Method call
    MethodCall(&'a ExprMethodCall),
    /// Function call
    FunctionCall(&'a Expr),
    /// Member access
    MemberAccess(&'a Expr),
    /// Identifier
    Identifier(&'a Ident),
    /// Path
    Path(&'a Path),
    /// Other type of node
    Other,
}

/// Query to search nodes in the AST
pub struct AstQuery<'a> {
    /// Nodes found so far
    results: Vec<AstNode<'a>>,
}

impl<'a> AstQuery<'a> {
    /// Creates a new query from an AST file
    pub fn from_file(file: &'a File) -> Self {
        let node = AstNode {
            node_type: NodeType::File,
            span: file.span(),
            data: NodeData::File(file),
        };
        
        Self {
            results: vec![node],
        }
    }
    
    /// Search nodes by name
    pub fn find_by_name(mut self, name: &str) -> Self {
        debug!("Buscando nodos con nombre: {}", name);
        let mut new_results = Vec::new();
        
        for node in self.results {
            match node.data {
                NodeData::File(file) => {
                    // Search in all items of the file
                    for item in &file.items {
                        if let Some(ident) = get_item_ident(item) {
                            if ident.to_string() == name {
                                trace!("Found item with name '{}' in file", name);
                                new_results.push(AstNode {
                                    node_type: get_item_type(item),
                                    span: item.span(),
                                    data: get_item_data(item),
                                });
                            }
                        }
                    }
                },
                NodeData::Function(func) => {
                    // Search in the function body
                    // (Simplified implementation, in the complete version we would traverse the entire body)
                    if func.sig.ident.to_string() == name {
                        trace!("Found name '{}' in function", name);
                        new_results.push(node);
                    }
                },
                NodeData::Struct(struct_item) => {
                    // Search in the struct fields
                    if struct_item.ident.to_string() == name {
                        trace!("Found name '{}' in struct", name);
                        new_results.push(node);
                    }
                },
                // More cases as needed
                _ => {}
            }
        }
        
        self.results = new_results;
        self
    }
    
    /// Search for calls to a specific function
    pub fn find_calls_to(mut self, function_name: &str) -> Self {
        debug!("Buscando llamadas a función: {}", function_name);
        let mut new_results = Vec::new();
        
        for node in self.results {
            match node.data {
                NodeData::File(file) => {
                    // Traverse all items and search for function calls
                    // (Simplified implementation, in the complete version we would use a visitor)
                    for item in &file.items {
                        if let Item::Fn(func) = item {
                            // Here we would search in the function body
                            // For simplicity, we do not implement the complete search here
                        }
                    }
                },
                // More cases as needed
                _ => {}
            }
        }
        
        self.results = new_results;
        self
    }
    
    /// Search for chained calls (e.g: a.b().c())
    pub fn find_chained_calls(mut self, call_chain: &[&str]) -> Self {
        debug!("Searching for chained calls: {:?}", call_chain);
        let mut new_results = Vec::new();
        
        // Simplified implementation
        // In the complete version, we would traverse the AST searching for chained call expressions
        
        self.results = new_results;
        self
    }
    
    /// Search for member access with a specific name
    pub fn find_member_access(mut self, member_name: &str) -> Self {
        debug!("Searching for member access: {}", member_name);
        let mut new_results = Vec::new();
        
        // Simplified implementation
        // En la versión completa, recorreríamos el AST buscando expresiones de acceso a miembros
        
        self.results = new_results;
        self
    }
    
    /// Search for comparisons with a specific value
    pub fn find_comparison_to(mut self, value: &str) -> Self {
        debug!("Searching for comparison with value: {}", value);
        let mut new_results = Vec::new();
        
        // Simplified implementation
        // In the complete version, we would traverse the AST searching for comparison expressions
        
        self.results = new_results;
        self
    }
    
    /// Logical AND operator: combines two queries
    pub fn and(mut self, other: Self) -> Self {
        debug!("Applying AND operator");
        let other_results = other.results;
        
        // Filter results to keep only those that are in both queries
        // (Simplified implementation, in the complete version we would compare spans)
        self.results.retain(|node| {
            other_results.iter().any(|other_node| {
                // Compare spans for equality
                format!("{:?}", node.span) == format!("{:?}", other_node.span)
            })
        });
        
        self
    }
    
    /// Logical OR operator: combines two queries
    pub fn or(mut self, other: Self) -> Self {
        debug!("Applying OR operator");
        
        // Add the results of the other query
        // (Simplified implementation, in the complete version we would remove duplicates)
        self.results.extend(other.results);
        
        self
    }
    
    /// Logical NOT operator: inverts a query
    pub fn not(self) -> Self {
        debug!("Applying NOT operator");
        
        // In a complete implementation, we would need the complete context
        // to know which nodes do not meet the condition.
        // For now, we simply return an empty query.
        Self {
            results: Vec::new(),
        }
    }
    
    /// Checks if there are results
    pub fn exists(self) -> bool {
        !self.results.is_empty()
    }
    
    /// Returns the nodes found
    pub fn collect(self) -> Vec<AstNode<'a>> {
        self.results
    }
    
    /// Returns the first node found, if it exists
    pub fn first(self) -> Option<AstNode<'a>> {
        self.results.into_iter().next()
    }
}

// Helper functions to extract information from items
fn get_item_ident(item: &syn::Item) -> Option<&Ident> {
    match item {
        syn::Item::Fn(item_fn) => Some(&item_fn.sig.ident),
        syn::Item::Struct(item_struct) => Some(&item_struct.ident),
        syn::Item::Enum(item_enum) => Some(&item_enum.ident),
        syn::Item::Trait(item_trait) => Some(&item_trait.ident),
        syn::Item::Impl(item_impl) => {
            if let Some((_, path, _)) = &item_impl.trait_ {
                // Para impls de traits, usamos el nombre del trait
                path.segments.last().map(|seg| &seg.ident)
            } else {
                // Para impls de tipos, usamos el nombre del tipo
                match &*item_impl.self_ty {
                    syn::Type::Path(type_path) => {
                        type_path.path.segments.last().map(|seg| &seg.ident)
                    }
                    _ => None,
                }
            }
        }
        syn::Item::Mod(item_mod) => Some(&item_mod.ident),
        // More cases as needed
        _ => None,
    }
}

fn get_item_type(item: &syn::Item) -> NodeType {
    match item {
        syn::Item::Fn(_) => NodeType::Function,
        syn::Item::Struct(_) => NodeType::Struct,
        syn::Item::Enum(_) => NodeType::Enum,
        // More cases as needed
        _ => NodeType::Other,
    }
}

fn get_item_data(item: &syn::Item) -> NodeData {
    match item {
        syn::Item::Fn(item_fn) => NodeData::Function(item_fn),
        syn::Item::Struct(item_struct) => NodeData::Struct(item_struct),
        syn::Item::Enum(item_enum) => NodeData::Enum(item_enum),
        // More cases as needed
        _ => NodeData::Other,
    }
}

/// Implementation of the Visitor trait to traverse the AST
pub struct AstVisitor<'a> {
    /// Callback function for each visited node
    callback: Box<dyn FnMut(&AstNode<'a>) + 'a>,
}

impl<'a> AstVisitor<'a> {
    /// Creates a new visitor with a callback function
    pub fn new<F>(callback: F) -> Self
    where
        F: FnMut(&AstNode<'a>) + 'a,
    {
        Self {
            callback: Box::new(callback),
        }
    }
    
    /// Visits an AST file
    pub fn visit_file(&mut self, file: &'a File) {
        let node = AstNode {
            node_type: NodeType::File,
            span: file.span(),
            data: NodeData::File(file),
        };
        
        (self.callback)(&node);
        
        // Visitar todos los items del archivo
        for item in &file.items {
            self.visit_item(item);
        }
    }
    
    /// Visits an AST item
    pub fn visit_item(&mut self, item: &'a Item) {
        match item {
            Item::Fn(func) => self.visit_fn(func),
            Item::Struct(struct_item) => self.visit_struct(struct_item),
            Item::Enum(enum_item) => self.visit_enum(enum_item),
            // More cases as needed
            _ => {},
        }
    }
    
    /// Visits a function
    pub fn visit_fn(&mut self, func: &'a ItemFn) {
        let node = AstNode {
            node_type: NodeType::Function,
            span: func.span(),
            data: NodeData::Function(func),
        };
        
        (self.callback)(&node);
        
        // Visit the function body
        // (Simplified implementation, in the complete version we would traverse the entire body)
    }
    
    /// Visits a struct
    pub fn visit_struct(&mut self, struct_item: &'a ItemStruct) {
        let node = AstNode {
            node_type: NodeType::Struct,
            span: struct_item.span(),
            data: NodeData::Struct(struct_item),
        };
        
        (self.callback)(&node);
        
        // Visit the struct fields
        // (Simplified implementation)
    }
    
    /// Visits an enum
    pub fn visit_enum(&mut self, enum_item: &'a ItemEnum) {
        let node = AstNode {
            node_type: NodeType::Enum,
            span: enum_item.span(),
            data: NodeData::Enum(enum_item),
        };
        
        (self.callback)(&node);
        
        // Visit the enum variants
        // (Simplified implementation)
    }
}