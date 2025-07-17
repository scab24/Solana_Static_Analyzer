use std::fmt;
use log::{debug, trace};
use syn::{File, Item, ItemFn, ItemStruct, ItemEnum, Expr, Stmt, Block};
use syn::visit::{self, Visit};
use syn::spanned::Spanned;

use crate::analyzer::{Finding, Location, Severity};

/// Type of node in the AST
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// File
    File,
    /// Function
    Function,
    /// Struct
    Struct,
    /// Enum
    Enum,
    /// Block
    Block,
    /// Expression
    Expression,
    /// Other
    Other,
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeType::File => write!(f, "File"),
            NodeType::Function => write!(f, "Function"),
            NodeType::Struct => write!(f, "Struct"),
            NodeType::Enum => write!(f, "Enum"),
            NodeType::Block => write!(f, "Block"),
            NodeType::Expression => write!(f, "Expression"),
            NodeType::Other => write!(f, "Other"),
        }
    }
}

/// Data associated with an AST node
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeData<'a> {
    /// File
    File(&'a File),
    /// Function
    Function(&'a ItemFn),
    /// Struct
    Struct(&'a ItemStruct),
    /// Enum
    Enum(&'a ItemEnum),
    /// Block
    Block(&'a Block),
    /// Expression
    Expression(&'a Expr),
    /// Other
    Other,
}

/// Node of the AST with metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstNode<'a> {
    /// Node type
    pub node_type: NodeType,
    /// Node data
    pub data: NodeData<'a>,
    /// Node name (if applicable)
    pub name: Option<String>,
}

impl<'a> AstNode<'a> {
    /// Create a new node from a file
    pub fn from_file(file: &'a File) -> Self {
        Self {
            node_type: NodeType::File,
            data: NodeData::File(file),
            name: None,
        }
    }
    
    /// Create a new node from a function
    pub fn from_function(func: &'a ItemFn) -> Self {
        Self {
            node_type: NodeType::Function,
            data: NodeData::Function(func),
            name: Some(func.sig.ident.to_string()),
        }
    }
    
    /// Create a new node from a struct
    pub fn from_struct(struct_item: &'a ItemStruct) -> Self {
        Self {
            node_type: NodeType::Struct,
            data: NodeData::Struct(struct_item),
            name: Some(struct_item.ident.to_string()),
        }
    }
    
    /// Get the node type
    pub fn node_type(&self) -> NodeType {
        self.node_type.clone()
    }
    
    /// Get the node name (if exists)
    pub fn name(&self) -> String {
        self.name.clone().unwrap_or_else(|| "unnamed".to_string())
    }
    
    /// Get a code snippet of the node
    pub fn snippet(&self) -> String {
        match &self.data {
            NodeData::Function(func) => format!("fn {}(...)", func.sig.ident),
            NodeData::Struct(struct_item) => format!("struct {}", struct_item.ident),
            NodeData::Enum(enum_item) => format!("enum {}", enum_item.ident),
            NodeData::Block(_) => "{ ... }".to_string(),
            NodeData::Expression(_) => "...".to_string(),
            _ => "...".to_string(),
        }
    }
    
    /// Convert the node to a location in the file
    pub fn location(&self, file_path: &str) -> Location {
        //@todo => convert span to line/column
        Location {
            file: file_path.to_string(),
            line: 0,
            column: 0,
        }
    }
}

/// AST query
pub struct AstQuery<'a> {
    /// Query results
    results: Vec<AstNode<'a>>,
}

impl<'a> AstQuery<'a> {
    /// Create a new query from a file
    pub fn new(ast: &'a File) -> Self {
        Self {
            results: vec![AstNode::from_file(ast)],
        }
    }
    
    /// Create a new query from a node
    pub fn from_node(node: &AstNode<'a>) -> Self {
        Self {
            results: vec![node.clone()],
        }
    }
    
    /// Filter functions
    pub fn functions(self) -> Self {
        debug!("Searching for functions");
        let mut new_results = Vec::new();
        
        for node in self.results {
            match node.data {
                NodeData::File(file) => {
                    // Search for functions in the file
                    for item in &file.items {
                        if let Item::Fn(func) = item {
                            trace!("Found function: {}", func.sig.ident);
                            new_results.push(AstNode::from_function(func));
                        }
                    }
                },
                // Other cases as needed
                _ => {}
            }
        }
        
        Self { results: new_results }
    }
    
    /// Filter structs
    pub fn structs(self) -> Self {
        debug!("Searching for structs");
        let mut new_results = Vec::new();
        
        for node in self.results {
            match node.data {
                NodeData::File(file) => {
                    // Search for structs in the file
                    for item in &file.items {
                        if let Item::Struct(struct_item) = item {
                            trace!("Found struct: {}", struct_item.ident);
                            new_results.push(AstNode::from_struct(struct_item));
                        }
                    }
                },
                // Other cases as needed
                _ => {}
            }
        }
        
        Self { results: new_results }
    }
    
    /// Filter by name
    pub fn with_name(self, name: &str) -> Self {
        debug!("Filtering by name: {}", name);
        let mut new_results = Vec::new();
        
        for node in self.results {
            if let Some(node_name) = &node.name {
                if node_name == name {
                    trace!("Found node with name: {}", name);
                    new_results.push(node);
                }
            }
        }
        
        Self { results: new_results }
    }
    
    pub fn uses_unsafe(self) -> Self {
        debug!("Searching for unsafe code");
        let mut new_results = Vec::new();
        
        for node in self.results {
            match node.data {
                NodeData::Function(func) => {
                    if func.sig.unsafety.is_some() {
                        trace!("Found unsafe function: {}", func.sig.ident);
                        new_results.push(node);
                    }
                },
                NodeData::Block(block) => {
                    // Search for unsafe blocks
                    for stmt in &block.stmts {
                        if let Stmt::Expr(Expr::Unsafe(_), _) = stmt {
                            trace!("Found unsafe block");
                            new_results.push(node);
                            break;
                        }
                    }
                },
                // Other cases as needed
                _ => {}
            }
        }
        
        Self { results: new_results }
    }
    
    /// Search for calls to a specific function
    pub fn calls_to(self, function_name: &str) -> Self {
        debug!("Searching for calls to: {}", function_name);
        let mut new_results = Vec::new();
        
        // Implement logic to search for function calls
        //@todo
        
        Self { results: new_results }
    }
    
    /// Apply a custom predicate
    pub fn filter<F>(self, predicate: F) -> Self 
    where 
        F: Fn(&AstNode<'a>) -> bool 
    {
        debug!("Applying custom predicate");
        let new_results = self.results.into_iter()
            .filter(|node| predicate(node))
            .collect();
        
        Self { results: new_results }
    }
    
    /// Combine with another query (OR operator)
    pub fn or(mut self, other: Self) -> Self {
        debug!("Combining queries with OR");
        self.results.extend(other.results);
        self
    }
    
    /// Combine with another query (AND operator)
    pub fn and(self, other: Self) -> Self {
        debug!("Combining queries with AND");
        let other_results = other.results;
        
        // Simple implementation: keep only nodes that are in both queries
        // In a real implementation, this would be more sophisticated
        let new_results = self.results.into_iter()
            .filter(|node| other_results.contains(node))
            .collect();
        
        Self { results: new_results }
    }
    
    /// Negate the query (NOT operator)
    pub fn not(self) -> Self {
        debug!("Negating query");
        //@todo
        Self { results: Vec::new() }
    }
    
    /// Check if there are results
    pub fn exists(self) -> bool {
        !self.results.is_empty()
    }
    
    /// Get the number of results
    pub fn count(self) -> usize {
        self.results.len()
    }
    
    /// Collect the results
    pub fn collect(self) -> Vec<AstNode<'a>> {
        self.results
    }
    
    /// Convert the results to findings
    pub fn to_findings(self, severity: Severity, message: &str, file_path: &str) -> Vec<Finding> {
        debug!("Converting {} results to findings", self.results.len());
        
        self.results.into_iter()
            .map(|node| {
                let description = match &node.name {
                    Some(name) => format!("{} in '{}'", message, name),
                    None => message.to_string(),
                };
                
                Finding {
                    description,
                    severity: severity.clone(),
                    location: node.location(file_path),
                    code_snippet: Some(node.snippet()),
                }
            })
            .collect()
    }
}
