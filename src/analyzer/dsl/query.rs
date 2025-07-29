use log::{debug, trace};
use std::fmt;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Block, Expr, File, Item, ItemEnum, ItemFn, ItemStruct, Stmt};

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
    /// Impl Function (function inside impl block)
    ImplFunction(&'a syn::ImplItemFn),
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

    /// Create a new node from an impl function
    pub fn from_impl_function(func: &'a syn::ImplItemFn) -> Self {
        Self {
            node_type: NodeType::Function,
            data: NodeData::ImplFunction(func),
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
            NodeData::ImplFunction(func) => format!("fn {}(...)", func.sig.ident),
            NodeData::Struct(struct_item) => format!("struct {}", struct_item.ident),
            NodeData::Enum(enum_item) => format!("enum {}", enum_item.ident),
            NodeData::Block(_) => "{ ... }".to_string(),
            NodeData::Expression(_) => "...".to_string(),
            _ => "...".to_string(),
        }
    }

    /// Get the underlying AST node that implements Spanned for use with SpanExtractor
    pub fn get_spanned_node(&self) -> Option<&dyn syn::spanned::Spanned> {
        use syn::spanned::Spanned;
        
        match &self.data {
            NodeData::Function(func) => Some(func as &dyn Spanned),
            NodeData::ImplFunction(func) => Some(func as &dyn Spanned),
            NodeData::Struct(struct_item) => Some(struct_item as &dyn Spanned),
            NodeData::Enum(enum_item) => Some(enum_item as &dyn Spanned),
            NodeData::Block(block) => Some(block as &dyn Spanned),
            NodeData::Expression(expr) => Some(expr as &dyn Spanned),
            NodeData::File(file) => Some(file as &dyn Spanned),
            NodeData::Other => None,
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

    /// Create a new query from a list of nodes
    pub fn from_nodes(nodes: Vec<AstNode<'a>>) -> Self {
        Self {
            results: nodes,
        }
    }

    /// Create a new query from a node
    pub fn from_node(node: &AstNode<'a>) -> Self {
        Self {
            results: vec![node.clone()],
        }
    }

    /// Returns a mutable reference to the results for internal use
    pub(crate) fn results_mut(&mut self) -> &mut Vec<AstNode<'a>> {
        &mut self.results
    }

    /// Returns the results of the query
    pub fn results(&self) -> &[AstNode<'a>] {
        &self.results
    }

    /// Returns the nodes found by the query (alias for results)
    pub fn nodes(&self) -> &[AstNode<'a>] {
        self.results()
    }

    /// Filter functions
    pub fn functions(self) -> Self {
        debug!("Searching for functions recursively in all modules");
        let mut new_results = Vec::new();

        for node in self.results {
            match node.data {
                NodeData::File(file) => {
                    // Search for functions recursively in the file
                    Self::extract_functions_recursive(&file.items, &mut new_results);
                }
                // Other cases
                _ => {}
            }
        }

        Self {
            results: new_results,
        }
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
                }
                // Other cases
                _ => {}
            }
        }

        Self {
            results: new_results,
        }
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

        Self {
            results: new_results,
        }
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
                }
                NodeData::ImplFunction(func) => {
                    if func.sig.unsafety.is_some() {
                        trace!("Found unsafe impl function: {}", func.sig.ident);
                        new_results.push(node);
                    }
                }
                NodeData::Block(block) => {
                    // Search for unsafe blocks
                    for stmt in &block.stmts {
                        if let Stmt::Expr(Expr::Unsafe(_), _) = stmt {
                            trace!("Found unsafe block");
                            new_results.push(node);
                            break;
                        }
                    }
                }
                // Other cases as needed
                _ => {}
            }
        }

        Self {
            results: new_results,
        }
    }

    /// Filter for structs that derive the Accounts trait
    pub fn derives_accounts(self) -> Self {
        debug!("Filtering structs that derive Accounts");
        let mut new_results = Vec::new();
        
        for node in self.results {
            if let NodeData::Struct(struct_item) = &node.data {
                // Check if the struct derives Accounts
                for attr in &struct_item.attrs {
                    if let syn::Meta::List(meta_list) = &attr.meta {
                        if meta_list.path.is_ident("derive") {
                            let tokens_str = meta_list.tokens.to_string();
                            if tokens_str.contains("Accounts") {
                                trace!("Found struct deriving Accounts: {}", struct_item.ident);
                                new_results.push(node);
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        Self {
            results: new_results,
        }
    }

    /// Filter for public functions only
    pub fn public_functions(self) -> Self {
        debug!("Filtering for public functions only");
        
        let mut new_results = Vec::new();
        
        for node in self.results {
            match &node.data {
                NodeData::Function(func) => {
                    // Check if function has pub visibility
                    if matches!(func.vis, syn::Visibility::Public(_)) {
                        trace!("Found public function: {}", func.sig.ident);
                        new_results.push(node);
                    }
                }
                NodeData::ImplFunction(func) => {
                    // Check if function has pub visibility
                    if matches!(func.vis, syn::Visibility::Public(_)) {
                        trace!("Found public impl function: {}", func.sig.ident);
                        new_results.push(node);
                    }
                }
                _ => {}
            }
        }
        
        Self {
            results: new_results,
        }
    }

    /// Search for calls to a specific function
    pub fn calls_to(self, function_name: &str) -> Self {
        debug!("Searching for calls to: {}", function_name);
        let mut new_results = Vec::new();

        for node in self.results {
            let found_call = match node.data {
                NodeData::Function(func) => {
                    Self::has_function_call(function_name, |finder| finder.visit_item_fn(func))
                }
                NodeData::ImplFunction(func) => {
                    Self::has_function_call(function_name, |finder| finder.visit_impl_item_fn(func))
                }
                NodeData::Block(block) => {
                    Self::has_function_call(function_name, |finder| finder.visit_block(block))
                }
                _ => false,
            };

            if found_call {
                trace!("Found call to {} in {}", function_name, node.name());
                new_results.push(node);
            }
        }

        Self {
            results: new_results,
        }
    }

    /// Helper function to check if a function call exists
    fn has_function_call<F>(function_name: &str, visit_fn: F) -> bool
    where
        F: FnOnce(&mut CallFinder),
    {
        let mut call_finder = CallFinder {
            target_function: function_name.to_string(),
            found: false,
        };
        visit_fn(&mut call_finder);
        call_finder.found
    }

    /// Apply a custom predicate
    pub fn filter<F>(self, predicate: F) -> Self
    where
        F: Fn(&AstNode<'a>) -> bool,
    {
        debug!("Applying custom predicate");
        let new_results = self
            .results
            .into_iter()
            .filter(|node| predicate(node))
            .collect();

        Self {
            results: new_results,
        }
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

        // @todo => Simple implementation
        let new_results = self
            .results
            .into_iter()
            .filter(|node| other_results.contains(node))
            .collect();

        Self {
            results: new_results,
        }
    }

    /// Negate the query (NOT operator)
    pub fn not(self) -> Self {
        debug!("Negating query - returning empty result (placeholder implementation)");
        // @todo => Implement proper negation logic

        Self {
            results: Vec::new(),
        }
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

        self.results
            .into_iter()
            .map(|node| {
                let description = match &node.name {
                    Some(name) => format!("{} in '{}'", message, name),
                    None => message.to_string(),
                };

                Finding {
                    description,
                    severity: severity.clone(),
                    location: Self::create_fallback_location(file_path),
                    code_snippet: Some(node.snippet()),
                }
            })
            .collect()
    }

    /// Convert query results to findings with precise locations using SpanExtractor
    /// This is the preferred method for dsl_query rules
    pub fn to_findings_with_span_extractor(
        self, 
        severity: Severity, 
        title: &str,
        description: &str,
        file_path: &str,
        span_extractor: &crate::analyzer::span_utils::SpanExtractor
    ) -> Vec<Finding> {
        debug!("Converting {} results to findings with precise locations", self.results.len());

        self.results
            .into_iter()
            .map(|node| {
                // Use SpanExtractor for precise location and snippet
                let (location, code_snippet) = if let Some(spanned_node) = node.get_spanned_node() {
                    (
                        span_extractor.extract_location(spanned_node),
                        span_extractor.extract_snippet(spanned_node)
                    )
                } else {
                    // Fallback for nodes without span info
                    (Self::create_fallback_location(file_path), node.snippet())
                };

                // Create descriptive message based on node name
                let finding_description = match &node.name {
                    Some(name) => format!(
                        "{} in '{}'. {}", 
                        title, 
                        name, 
                        description
                    ),
                    None => format!("{}: {}", title, description),
                };

                Finding {
                    description: finding_description,
                    severity: severity.clone(),
                    location,
                    code_snippet: Some(code_snippet),
                }
            })
            .collect()
    }

    /// Helper function to create a fallback location for nodes without span info
    fn create_fallback_location(file_path: &str) -> crate::analyzer::Location {
        crate::analyzer::Location {
            file: file_path.to_string(),
            line: 1,
            column: None,
            end_line: None,
            end_column: None,
        }
    }

    /// Helper function to recursively extract functions from items (including nested modules)
    fn extract_functions_recursive<'b>(items: &'b [syn::Item], results: &mut Vec<AstNode<'b>>) {
        for item in items {
            match item {
                syn::Item::Fn(func) => {
                    trace!("Found function: {}", func.sig.ident);
                    results.push(AstNode::from_function(func));
                }
                syn::Item::Mod(module) => {
                    debug!("Searching in module: {}", module.ident);
                    // Check if module has inline content (not external file)
                    if let Some((_, items)) = &module.content {
                        // Recursively search in the module
                        Self::extract_functions_recursive(items, results);
                    }
                }
                syn::Item::Impl(impl_block) => {
                    debug!("Searching in impl block");
                    // Search for functions in impl blocks
                    for impl_item in &impl_block.items {
                        if let syn::ImplItem::Fn(func) = impl_item {
                            trace!("Found impl function: {}", func.sig.ident);
                            results.push(AstNode::from_impl_function(func));
                        }
                    }
                }
                _ => {
                    // Other items (structs, enums..)
                }
            }
        }
    }
}

/// Helper visitor to find calls to specific functions
struct CallFinder {
    target_function: String,
    found: bool,
}

impl<'ast> Visit<'ast> for CallFinder {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        // Check if this is a call to our target function
        if let syn::Expr::Path(path) = &*call.func {
            if let Some(ident) = path.path.get_ident() {
                if ident.to_string() == self.target_function {
                    self.found = true;
                    trace!("Found call to target function: {}", self.target_function);
                }
            }
        }
        
        // Continue visiting sub-expressions
        visit::visit_expr_call(self, call);
    }
    
    fn visit_expr_method_call(&mut self, method_call: &'ast syn::ExprMethodCall) {
        // Check if this is a method call to our target function
        if method_call.method.to_string() == self.target_function {
            self.found = true;
            trace!("Found method call to target function: {}", self.target_function);
        }
        
        // Continue visiting sub-expressions
        visit::visit_expr_method_call(self, method_call);
    }
}