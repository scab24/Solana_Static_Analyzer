# Rust Solana Analyzer - DSL Documentation

This comprehensive documentation explains how the Domain Specific Language (DSL) of the Solana static analyzer works, how to write new rules, and how to extend its functionality.

## Table of Contents

1. [General Architecture](#general-architecture)
2. [SpanExtractor Integration](#spanextractor-integration)
3. [File `query.rs` - DSL Core](#file-queryrs---dsl-core)
4. [File `builders.rs` - Rule Builder](#file-buildersrs---rule-builder)
5. [Modular Rule Filters - Specific Filters](#modular-rule-filters---specific-filters)
6. [How to Write a New Rule](#how-to-write-a-new-rule)

---

## General Architecture

The analyzer's DSL consists of three main components:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   query.rs      │    │   builders.rs   │    │ rules/[rule]/   │
│                 │    │                 │    │                 │
│ • AstQuery      │◄───┤ • RuleBuilder   │◄───┤ • mod.rs        │
│ • AstNode       │    │ • Fluent API    │    │ • filters.rs    │
│ • NodeData      │    │ • Integration   │    │ • Specific      │
│ • Generic       │    │                 │    │   Helpers       │
│   Helpers       │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
        ▲                       ▲                       ▲
        │                       │                       │
        └───────────────────────┼───────────────────────┘
                                │
                    ┌─────────────────┐
                    │   Modular       │
                    │ Vulnerability   │
                    │    Rules        │
                    │                 │
                    │ high/unsafe_code│
                    │ medium/division │
                    │ low/error_handle│
                    └─────────────────┘
```

### Workflow

1. **Parsing**: `query.rs` parses the Rust AST using `syn`
2. **Filtering**: Applies Solana-specific filters
3. **Construction**: `builders.rs` facilitates rule creation
4. **Detection**: Rules detect vulnerability patterns
5. **Location Extraction**: `SpanExtractor` provides precise locations
6. **Reporting**: Generates findings with exact locations and code snippets

---

## SpanExtractor Integration

The analyzer includes **precise location extraction** through the `SpanExtractor` system, which provides exact file locations and automatic code snippet generation for all vulnerability findings.

### Key Features

#### **🎯 Precise Locations**
- Exact locations like `src/lib.rs:42:15`
- Line numbers, column positions, and end positions

#### **📝 Automatic Code Snippets**
- **Function signatures**: `pub fn initialize(ctx: Context<Initialize>) -> Result<()>`
- **Struct definitions**: `pub struct TransferFunds`
- **Code context**: Surrounding lines for better understanding

#### **🔗 AST Integration**
- **`get_spanned_node()`**: Method to get `syn::spanned::Spanned` objects
- **`to_findings_with_span_extractor()`**: Method for creating findings
- **Automatic integration**: Works seamlessly with all DSL queries

### SpanExtractor Methods

#### `SpanExtractor::new(source_code: String, file_path: String)`
```rust
let extractor = SpanExtractor::new(
    source_code.to_string(),
    "src/lib.rs".to_string()
);
```
**Usage**: Create a new extractor for a specific file.

#### `span_to_location(&self, span: Span) -> Location`
```rust
let location = extractor.span_to_location(node_span);
```
**Usage**: Convert a `syn::Span` to precise `Location`.

#### `extract_snippet(&self, spanned: &dyn Spanned) -> String`
```rust
let snippet = extractor.extract_snippet(&function_node);
```
**Usage**: Extract code snippet from any `Spanned` object.

#### `extract_context(&self, span: Span, context_lines: usize) -> String`
```rust
let context = extractor.extract_context(span, 2);
```
**Usage**: Extract code with surrounding context lines.

### Integration with DSL

#### **New AstNode Method: `get_spanned_node()`**
```rust
let spanned_node = ast_node.get_spanned_node();
if let Some(spanned) = spanned_node {
    let location = span_extractor.span_to_location(spanned.span());
    let snippet = span_extractor.extract_snippet(spanned);
}
```
**Purpose**: Get the underlying `syn` node that implements `Spanned` trait.

#### **New Preferred Method: `to_findings_with_span_extractor()`**
```rust
let findings = AstQuery::new(ast)
    .functions()
    .uses_unsafe()
    .to_findings_with_span_extractor(
        Severity::High,
        "Unsafe Code Detection",
        "Detects unsafe code blocks that could lead to memory safety issues",
        file_path,
        &span_extractor
    );
```

**Parameters**:
- `severity: Severity` - Vulnerability severity
- `title: &str` - Short title for the finding
- `description: &str` - Detailed description
- `file_path: &str` - Source file path
- `span_extractor: &SpanExtractor` - Extractor instance

**Returns**: `Vec<Finding>` with precise locations and code snippets.

### Best Practices with SpanExtractor

#### **Standard Usage Pattern**
```rust
.dsl_query(|ast, file_path, span_extractor| {
    AstQuery::new(ast)
        .functions()
        .uses_unsafe()
        .to_findings_with_span_extractor(
            Severity::High,
            "Unsafe Code Detection",
            "Detects unsafe code blocks that could lead to memory safety issues",
            file_path,
            span_extractor
        )
})
```

**Key Points**:
- Always use the `span_extractor` parameter provided to `dsl_query()`
- Provide clear, descriptive titles and detailed descriptions
- The method automatically extracts precise locations and code snippets

---

## File `query.rs` - DSL Core

This is the heart of the DSL. It contains the fundamental data structures and operators for querying the AST.

### Main Structures

#### `NodeType` - AST Node Types
```rust
pub enum NodeType {
    File,        // Complete file
    Function,    // Function (normal or impl)
    Struct,      // Structure
    Enum,        // Enumeration
    Block,       // Code block
    Expression,  // Expression
    Other,       // Other elements
}
```

#### `NodeData<'a>` - Node Data
```rust
pub enum NodeData<'a> {
    File(&'a File),                    // syn::File
    Function(&'a ItemFn),              // Normal function
    ImplFunction(&'a syn::ImplItemFn), // Impl function (NEW)
    Struct(&'a ItemStruct),            // Structure
    Enum(&'a ItemEnum),                // Enumeration
    Block(&'a Block),                  // Block
    Expression(&'a Expr),              // Expression
    Other,                             // Others
}
```

**⚠️ Important**: The distinction between `Function` and `ImplFunction` is crucial for supporting Anchor projects, where functions are inside `impl` blocks.

#### `AstNode<'a>` - AST Node
```rust
pub struct AstNode<'a> {
    pub node_type: NodeType,    // Logical node type
    pub data: NodeData<'a>,     // Node-specific data
    pub name: Option<String>,   // Name (if applicable)
}
```

**Main Methods:**

##### `from_file(file: &File)` - Create File Node
```rust
let file_node = AstNode::from_file(&ast);
```
**Usage**: Create a node representing the entire file.

##### `from_function(func: &ItemFn)` - Create Normal Function Node
```rust
let func_node = AstNode::from_function(&function_item);
```
**Usage**: For functions defined directly in the file (`fn my_function() {}`).

##### `from_impl_function(func: &ImplItemFn)` - Create Impl Function Node
```rust
let impl_func_node = AstNode::from_impl_function(&impl_function);
```
**Usage**: For functions inside `impl` blocks (Anchor pattern).

##### `from_struct(struct_item: &ItemStruct)` - Create Structure Node
```rust
let struct_node = AstNode::from_struct(&struct_item);
```
**Usage**: For data structures.

##### `node_type()` - Get Node Type
```rust
let node_type: NodeType = node.node_type();
match node_type {
    NodeType::Function => println!("It's a function"),
    NodeType::Struct => println!("It's a structure"),
    _ => {}
}
```

##### `name()` - Get Node Name
```rust
let name: String = node.name();
println!("Name: {}", name);  // "Name: my_function"
```
**Note**: Returns "unnamed" if the node has no name.

##### `snippet()` - Get Code Snippet
```rust
let code_snippet: String = node.snippet();
// Examples of output:
// "fn initialize(...)"
// "struct MyAccount"
// "enum MyEnum"
```

##### `get_spanned_node()` - Get Spanned Node (NEW)
```rust
let spanned_node = node.get_spanned_node();
if let Some(spanned) = spanned_node {
    let span = spanned.span();
    // Use with SpanExtractor for precise locations
}
```
**Usage**: Get the underlying `syn` node that implements `Spanned` trait for use with `SpanExtractor`.

**Returns**: `Option<&dyn syn::spanned::Spanned>`
- `Some(spanned)` - For nodes with span information (functions, structs, etc.)
- `None` - For nodes without span information

#### `AstQuery<'a>` - Query Constructor
```rust
pub struct AstQuery<'a> {
    results: Vec<AstNode<'a>>,  // Nodes that match the query
}
```

### DSL Operators

#### Basic Filtering Operators

##### `functions()` - Filter Functions
```rust
// Extracts ALL functions (normal + impl)
let query = AstQuery::new(ast)
    .functions();  // Finds functions anywhere
```

**Internal Implementation:**
- Searches for `syn::Item::Fn` (normal functions)
- Searches for `syn::Item::Impl` → `syn::ImplItem::Fn` (impl functions)
- Recursively searches in nested modules

##### `structs()` - Filter Structures
```rust
let query = AstQuery::new(ast)
    .structs();  // Finds all structures
```

##### `with_name(name: &str)` - Filter by Name
```rust
let query = AstQuery::new(ast)
    .functions()
    .with_name("initialize");  // Only "initialize" function
```

#### Code Analysis Operators

##### `uses_unsafe()` - Detect Unsafe Code
```rust
let query = AstQuery::new(ast)
    .functions()
    .uses_unsafe();  // Functions with 'unsafe' or unsafe blocks
```

**Detects:**
- Functions marked as `unsafe fn`
- `unsafe { ... }` blocks inside functions
- Both normal and impl functions

##### `calls_to(function_name: &str)` - Detect Calls
```rust
let query = AstQuery::new(ast)
    .functions()
    .calls_to("panic");  // Functions that call panic!()
```

**Detects:**
- Function calls: `function_name()`
- Method calls: `obj.method_name()`
- Uses visitor pattern to traverse the AST

#### Logical Operators

##### `or(other: AstQuery)` - OR Operator
```rust
let unsafe_or_panic = query1.or(query2);  // Combines results
```

##### `and(other: AstQuery)` - AND Operator
```rust
let intersection = query1.and(query2);  // Only common elements
```

##### `not()` - NOT Operator
```rust
let negated = query.not();  // Inverts the query
```

#### Result Operators

##### `exists()` - Check Existence
```rust
if query.exists() {
    // Results were found
}
```

##### `count()` - Count Results
```rust
let num_functions = query.count();
```

##### `collect()` - Get Results
```rust
let nodes: Vec<AstNode> = query.collect();
```

##### `to_findings_with_span_extractor()` - Convert with Precise Locations
```rust
let findings = query.to_findings_with_span_extractor(
    Severity::High,
    "Unsafe Code Detection",
    "Detects unsafe code blocks that could lead to memory safety issues",
    "src/lib.rs",
    &span_extractor
);
```

**Parameters**:
- `severity: Severity` - Vulnerability severity (High/Medium/Low)
- `title: &str` - Short, descriptive title for the finding
- `description: &str` - Detailed explanation of the vulnerability
- `file_path: &str` - Path to the source file being analyzed
- `span_extractor: &SpanExtractor` - Extractor for precise location information

**Returns**: `Vec<Finding>` with:
- Exact file locations (line:column)
- Automatic code snippets
- Professional descriptions

##### `filter<F>(predicate: F)` - Custom Filter
```rust
let custom_filtered = AstQuery::new(ast)
    .functions()
    .filter(|node| {
        // Custom filtering logic
        node.name().contains("unsafe")
    });
```

**Functionality:**
- Allows applying custom predicates
- Takes a function that returns `bool`
- Useful for complex filtering logic

##### `from_nodes(nodes: Vec<AstNode>)` - Create from Nodes
```rust
let custom_nodes = vec![node1, node2, node3];
let query = AstQuery::from_nodes(custom_nodes);
```

##### `from_node(node: &AstNode)` - Create from Single Node
```rust
let single_query = AstQuery::from_node(&my_node);
```

##### `results_mut()` - Mutable Reference to Results
```rust
// For internal DSL use
let results = query.results_mut();  // &mut Vec<AstNode>
```

##### `results()` - Reference to Results
```rust
let results = query.results();  // &[AstNode]
```

##### `nodes()` - Alias for Results
```rust
let nodes = query.nodes();  // Same as results()
```

### Recursive Parsing Function

#### `extract_functions_recursive()` - Main Parser
```rust
fn extract_functions_recursive<'b>(
    items: &'b [syn::Item], 
    results: &mut Vec<AstNode<'b>>
)
```

**Functionality:**
1. **Normal functions**: `syn::Item::Fn` → `AstNode::from_function()`
2. **Modules**: `syn::Item::Mod` → Recursion in content
3. **Impl blocks**: `syn::Item::Impl` → `AstNode::from_impl_function()`

**Example of parsed code:**
```rust
// 1. Normal function
pub fn normal_function() { }  // ← Detected

// 2. Module with functions
pub mod my_module {
    pub fn nested_function() { }  // ← Detected recursively
}

// 3. Impl block (Anchor pattern)
impl MyProgram {
    pub fn instruction_function() { }  // ← Detected as ImplFunction
}
```

---

## File `builders.rs` - Rule Builder

This file provides a fluent API for creating analysis rules without manually implementing the `Rule` trait.

### Main Structure

#### `RuleBuilder` - Fluent Constructor
```rust
pub struct RuleBuilder {
    id: String,                    // Unique rule ID
    title: String,                 // Descriptive title
    description: String,           // Detailed description
    severity: Severity,            // Severity (High/Medium/Low)
    rule_type: RuleType,          // Type (Solana/Rust/General)
    query_builder: Option<Box<dyn Fn(&File, &str, &SpanExtractor) -> Vec<Finding> + Send + Sync>>, // Analysis function with SpanExtractor
    references: Vec<String>,       // Documentation references
    tags: Vec<String>,            // Classification tags
    enabled: bool,                // Enabled by default
}
```

### Builder Methods

#### Basic Configuration

##### `new()` - Create Builder
```rust
let rule = RuleBuilder::new()
    .id("solana-unsafe-code")
    .title("Unsafe Code Detection")
    .description("Detects unsafe code blocks and functions")
    .severity(Severity::High);
```

##### `id(id: &str)` - Set ID
```rust
.id("my-custom-rule")  // Unique rule ID
```

##### `title(title: &str)` - Set Title
```rust
.title("My Custom Security Rule")
```

##### `description(description: &str)` - Set Description
```rust
.description("This rule detects a specific vulnerability pattern")
```

##### `severity(severity: Severity)` - Set Severity
```rust
.severity(Severity::High)     // Critical
.severity(Severity::Medium)   // Medium
.severity(Severity::Low)      // Low
```

#### Logic Implementation

##### `visitor_rule<F>(rule_fn: F)` - Rule with Visitor
```rust
.visitor_rule(|ast: &syn::File| -> Vec<Finding> {
    // Manual implementation using visitor pattern
    let mut findings = Vec::new();
    // ... detection logic
    findings
})
```

##### `dsl_rule<F>(rule_fn: F)` - Rule with DSL
```rust
.dsl_rule(|ast: &syn::File, file_path: &str| -> Vec<Finding> {
    // Use DSL for detection
    AstQuery::new(ast)
        .functions()
        .uses_unsafe()
        .to_findings(Severity::High, "Unsafe code detected", file_path)
})
```

##### `dsl_query<F>(dsl_builder: F)` - DSL Query
```rust
.dsl_query(|ast: &syn::File, file_path: &str, span_extractor: &SpanExtractor| -> Vec<Finding> {
    // Use DSL with SpanExtractor for precise locations
    AstQuery::new(ast)
        .functions()
        .uses_unsafe()
        .to_findings_with_span_extractor(
            Severity::High,
            "Unsafe Code Detection",
            "Detects unsafe code blocks that could lead to memory safety issues",
            file_path,
            span_extractor
        )
})
```

**Parameters**:
- `ast: &syn::File` - The parsed AST of the source file
- `file_path: &str` - Path to the source file being analyzed
- `span_extractor: &SpanExtractor` - Extractor for precise location information

**Returns**: `Vec<Finding>` with precise locations and code snippets.

#### Metadata and Classification

##### `reference(reference: &str)` - Add Reference
```rust
.reference("https://docs.solana.com/security")
```

##### `references(refs: Vec<&str>)` - Multiple References
```rust
.references(vec![
    "https://docs.solana.com/security",
    "https://github.com/solana-labs/solana/security"
])
```

##### `tag(tag: &str)` - Add Tag
```rust
.tag("security")
.tag("unsafe")
.tag("solana")
```

##### `tags(tags: Vec<&str>)` - Multiple Tags
```rust
.tags(vec!["security", "unsafe", "critical"])
```

##### `enabled(enabled: bool)` - Enable/Disable
```rust
.enabled(true)   // Enabled by default
.enabled(false)  // Disabled by default
```



#### Final Construction

##### `build()` - Build Rule
```rust
let rule: Arc<dyn Rule> = builder.build();
```

**Internal Process:**
1. Validates all required fields are present
2. Creates a `RustRule` with the provided logic
3. Returns an `Arc<dyn Rule>` for use in the engine

### Complete Example

```rust
pub fn create_unsafe_code_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-unsafe-code")
        .title("Unsafe Code Detection")
        .description("Detects unsafe code blocks and functions that could lead to memory safety issues")
        .severity(Severity::High)
        .rule_type(RuleType::Solana)
        .dsl_query(|ast: &syn::File, file_path: &str, span_extractor: &SpanExtractor| -> Vec<Finding> {
            AstQuery::new(ast)
                .functions()
                .uses_unsafe()
                .to_findings_with_span_extractor(
                    Severity::High,
                    "Unsafe Code Detection",
                    "Detects unsafe code blocks and functions that could lead to memory safety issues in Solana programs",
                    file_path,
                    span_extractor
                )
        })
        .references(vec![
            "https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html",
            "https://docs.solana.com/developing/programming-model/overview"
        ])
        .tags(vec!["security", "unsafe", "memory-safety"])
        .enabled(true)
        .build()
}
```

---

## Modular Rule Filters - Specific Filters

The analyzer uses a **modular architecture** where each vulnerability rule has its own specific filters. This approach provides better maintainability, scalability, and clarity compared to a centralized filter system.

### Architecture Overview

```
src/analyzer/rules/solana/
├── high/
│   ├── unsafe_code/
│   │   ├── mod.rs (rule implementation)
│   │   └── filters.rs (specific filters)
│   └── missing_signer_check/
│       ├── mod.rs
│       └── filters.rs
├── medium/
│   ├── division_by_zero/
│   │   ├── mod.rs
│   │   └── filters.rs
│   └── duplicate_mutable_accounts/
│       ├── mod.rs
│       └── filters.rs
└── low/
    └── missing_error_handling/
        ├── mod.rs
        └── filters.rs
```

### Generic Helpers (in `query.rs`)

These are **generic helpers** available to all rules:

```rust
.functions()         // All functions
.structs()          // All structures  
.with_name(name)    // Filter by name
.uses_unsafe()      // Functions/blocks with unsafe code
.derives_accounts() // Structs with #[derive(Accounts)]
.public_functions() // Only public functions
.calls_to(name)     // Calls to specific function
.filter(predicate)  // Custom predicate
.or()/.and()/.not() // Logical operators
.exists()/.count()  // Result queries
.to_findings_with_span_extractor() // Convert to findings
```

### Specific Filters (Modularized by Rule)

Each rule defines its own **specific filters** using trait extensions:

#### Example: `unsafe_code/filters.rs`
```rust
use crate::analyzer::dsl::query::AstQuery;

pub trait UnsafeCodeFilters<'a> {
    fn uses_unsafe(self) -> AstQuery<'a>;
}

impl<'a> UnsafeCodeFilters<'a> for AstQuery<'a> {
    fn uses_unsafe(self) -> AstQuery<'a> {
        // Implementation specific to unsafe code detection
        self.filter(|node| {
            // Custom logic for this vulnerability
        })
    }
}
```

#### Example: `duplicate_mutable_accounts/filters.rs`
```rust
use crate::analyzer::dsl::query::AstQuery;

pub trait DuplicateMutableAccountsFilters<'a> {
    fn has_duplicate_mutable_accounts(self) -> AstQuery<'a>;
}

impl<'a> DuplicateMutableAccountsFilters<'a> for AstQuery<'a> {
    fn has_duplicate_mutable_accounts(self) -> AstQuery<'a> {
        // Implementation specific to duplicate mutable accounts
        self.filter(|node| {
            // Custom logic for this vulnerability
        })
    }
}
```

### Rule Implementation Pattern

Each rule follows this **unified pattern**:

```rust
// In mod.rs of each rule
use crate::analyzer::dsl::query::AstQuery;
use crate::analyzer::dsl::builders::RuleBuilder;
use crate::analyzer::{Rule, Severity};
use std::sync::Arc;

// Import the specific filters for this rule
mod filters;
use filters::SpecificFilters; // Trait name varies per rule

pub fn create_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-rule-name")
        .title("Rule Title")
        .description("Rule description")
        .severity(Severity::High) // or Medium/Low
        .dsl_query(|ast, file_path, span_extractor| {
            AstQuery::new(ast)
                .functions()                    // Generic helper
                .specific_filter()              // Specific helper
                .to_findings_with_span_extractor(
                    Severity::High,
                    "Rule Title",
                    "Detailed description",
                    file_path,
                    span_extractor
                )
        })
        .build()
}
```

### Benefits of Modular Architecture

1. **Maintainability**: Each vulnerability is self-contained
2. **Scalability**: Easy to add new rules without affecting existing ones
3. **Clarity**: Clear separation between generic and specific logic
4. **Performance**: No unused filters loaded
5. **Encapsulation**: Rule-specific logic stays with the rule

### How to Create a New Rule

1. **Create the directory structure:**
   ```
   src/analyzer/rules/solana/[severity]/[rule_name]/
   ├── mod.rs
   └── filters.rs
   ```

2. **Implement specific filters** in `filters.rs`
3. **Implement the rule** in `mod.rs` using the pattern above
4. **Register the rule** in the parent `mod.rs`

### Real Examples from Current Codebase

#### High Severity: `unsafe_code`
```rust
// Uses generic helper from query.rs
AstQuery::new(ast)
    .functions()
    .uses_unsafe()  
```

#### Medium Severity: `duplicate_mutable_accounts`
```rust
// Uses specific filter from the rule's filters.rs
AstQuery::new(ast)
    .structs()
    .derives_accounts()                    // Generic helper
    .has_duplicate_mutable_accounts()      // Specific filter
```

#### High Severity: `missing_signer_check`
```rust
// Uses specific filter with anchor-syn for advanced analysis
AstQuery::new(ast)
    .structs()
    .derives_accounts()                    // Generic helper
    .has_missing_signer_checks()           // Specific filter
```

---

## Internal Helpers (Visitors)

The DSL uses several internal visitors to detect specific patterns. These are helpers that implement the `syn` Visitor pattern to efficiently traverse the AST.

### `UnsafeDivisionFinder` - Unsafe Division Detector

```rust
struct UnsafeDivisionFinder {
    found: bool,
    safe_variables: HashMap<String, bool>,  // Variables marked as safe
}
```

**Functionality:**
1. **Safe variable tracking**: Identifies variables assigned to non-zero literals
2. **Division detection**: Finds `/` and `%` operators
3. **Danger assessment**: Determines if the divisor is potentially unsafe

**Main Methods:**

#### `visit_local(&mut self, local: &syn::Local)`
```rust
// Detects and tracks assignments like:
let safe_divisor = 100;        // ← Marked as safe (non-zero literal)
let unsafe_divisor = user_input; // ← Not marked (unknown value)
let zero_divisor = 0;          // ← Not marked (zero literal)
```

#### `visit_expr_binary(&mut self, expr: &syn::ExprBinary)`
```rust
// Detects division operations:
let result = amount / divisor;    // ← Analyzes if 'divisor' is safe
let remainder = value % modulo;   // ← Also analyzes modulo
```

#### `is_potentially_dangerous(&self, expr: &syn::Expr) -> bool`
```rust
// Evaluates different expression types:
let zero = 0;              // → true (dangerous)
let safe = 100;            // → false (safe)
let variable = x;          // → true if not in safe_variables
let call = get_value();    // → true (unknown result)
let field = obj.field;     // → true (unknown value)
```

### `OwnerCheckFinder` - Owner Check Detector

```rust
struct OwnerCheckFinder {
    found: bool,
}
```

**Functionality:** Detects owner checks in Solana/Anchor code.

**Main Methods:**

#### `visit_expr_binary(&mut self, binary: &syn::ExprBinary)`
```rust
// Detects comparisons involving "owner":
if account.owner == program_id { }           // ← Detected
if ctx.accounts.token.owner == &spl_token::id() { } // ← Detected
if owner_key != expected_owner { }           // ← Detected
```

#### `visit_expr_macro(&mut self, mac: &syn::ExprMacro)`
```rust
// Detects verification macros with "owner":
require!(account.owner == program_id);       // ← Detected
assert_eq!(token.owner, expected_owner);     // ← Detected
assert!(ctx.accounts.mint.owner == &spl_token::id()); // ← Detected
```

### `CallFinder` - Function Call Detector

```rust
struct CallFinder {
    target_function: String,  // Target function to find
    found: bool,
}
```

**Functionality:** Searches for specific function or method calls (but NOT macros).

**Main Methods:**

#### `visit_expr_call(&mut self, call: &syn::ExprCall)`
```rust
// Detects direct function calls:
dangerous_function();         // ← If target_function = "dangerous_function"
some_function();              // ← If target_function = "some_function"
// Note: Does NOT detect macros like panic!() or assert!()
```

#### `visit_expr_method_call(&mut self, method_call: &syn::ExprMethodCall)`
```rust
// Detects method calls:
result.unwrap();             // ← If target_function = "unwrap"
value.dangerous_method();    // ← If target_function = "dangerous_method"
token.transfer();            // ← If target_function = "transfer"
```

**Important Limitation:** `CallFinder` currently does NOT support macro detection. For detecting macros like `panic!()`, `assert!()`, etc., a custom visitor with `visit_expr_macro()` would be needed.

### Example of Internal Usage

```rust
// Inside has_unsafe_divisions():
let mut finder = UnsafeDivisionFinder {
    found: false,
    safe_variables: HashMap::new(),
};

// The visitor traverses the entire function AST
syn::visit::visit_item_fn(&mut finder, func);

// If unsafe divisions were found, include the function in results
if finder.found {
    new_results.push(node.clone());
}
```

**Advantages of the Visitor Pattern:**
1. **Efficiency**: Traverses the AST only once
2. **Completeness**: Doesn't miss nested nodes
3. **Flexibility**: Easy to extend for new patterns
4. **Reusability**: Visitors can be combined

---

## How to Write a New Rule

### Step 1: Define the Rule

Create a file in the appropriate severity folder:
- `src/analyzer/rules/solana/high/` - High severity
- `src/analyzer/rules/solana/medium/` - Medium severity  
- `src/analyzer/rules/solana/low/` - Low severity

### Step 2: Implement the Logic

#### Option A: Use DSL (Recommended)
```rust
use crate::analyzer::dsl::builders::RuleBuilder;
use crate::analyzer::dsl::query::AstQuery;
use crate::analyzer::dsl::filters::solana::SolanaFilters;
use crate::analyzer::{Severity, engine::{Rule, RuleType}};
use std::sync::Arc;

pub fn create_my_custom_rule() -> Arc<dyn Rule> {
    RuleBuilder::new()
        .id("solana-my-custom-rule")
        .title("My Custom Security Rule")
        .description("Detects a specific vulnerability pattern")
        .severity(Severity::Medium)
        .rule_type(RuleType::Solana)
        .dsl_query(|ast: &syn::File| -> AstQuery {
            // Use DSL to define the query
            AstQuery::new(ast)
                .functions()
                .public_functions()
                .calls_to("dangerous_function")
        })
        .references(vec![
            "https://docs.solana.com/security"
        ])
        .tags(vec!["security", "solana"])
        .build()
}
```

### Step 3: Register the Rule

In `src/analyzer/rules/solana/[severity]/mod.rs`:
```rust
mod my_custom_rule;

pub use my_custom_rule::create_my_custom_rule;
```

In `src/analyzer/rules/solana/mod.rs`:
```rust
// In the register_builtin_rules function
engine.register_rule(high::create_my_custom_rule());
```
