use proc_macro2::Span;
use syn::spanned::Spanned;
use crate::analyzer::Location;
use crate::analyzer::dsl::query::NodeData;

pub struct SpanExtractor {
    source_code: String,
    file_path: String,
}

impl SpanExtractor {
    pub fn new(source_code: String, file_path: String) -> Self {
        Self {
            source_code,
            file_path,
        }
    }

    /// Extract precise location from a span
    pub fn extract_location<T: Spanned + ?Sized>(&self, node: &T) -> Location {
        let span = node.span();
        self.span_to_location(span)
    }

    /// Convert a `proc_macro2::Span` to our Location struct
    pub fn span_to_location(&self, span: Span) -> Location {
        let start = span.start();
        let end = span.end();
        
        if start.line > 0 {
            Location {
                file: self.file_path.clone(),
                line: start.line,
                column: Some(start.column),
                end_line: Some(end.line),
                end_column: Some(end.column),
            }
        } else {
            Location {
                file: self.file_path.clone(),
                line: 1,
                column: None,
                end_line: None,
                end_column: None,
            }
        }
    }

    /// Extract a code snippet from the source code based on span
    pub fn extract_snippet<T: Spanned + ?Sized>(&self, node: &T) -> String {
        let span = node.span();
        self.span_to_snippet(span)
    }

    /// Convert a span to a code snippet
    pub fn span_to_snippet(&self, span: Span) -> String {
        let start = span.start();
        let end = span.end();
        
        if start.line == 0 || end.line == 0 {
            return "// Code snippet unavailable".to_string();
        }

        let lines: Vec<&str> = self.source_code.lines().collect();
        
        if start.line > lines.len() || end.line > lines.len() {
            return "// Code snippet out of bounds".to_string();
        }

        let start_line_idx = (start.line - 1).max(0);
        let end_line_idx = (end.line - 1).min(lines.len() - 1);

        if start_line_idx == end_line_idx {
            let line = lines[start_line_idx];
            if start.column < line.len() && end.column <= line.len() {
                line[start.column..end.column.min(line.len())].to_string()
            } else {
                line.to_string()
            }
        } else {
            let mut snippet = String::new();
            
            if start_line_idx < lines.len() {
                let first_line = lines[start_line_idx];
                if start.column < first_line.len() {
                    snippet.push_str(&first_line[start.column..]);
                } else {
                    snippet.push_str(first_line);
                }
                snippet.push('\n');
            }
            
            for line_idx in (start_line_idx + 1)..end_line_idx {
                if line_idx < lines.len() {
                    snippet.push_str(lines[line_idx]);
                    snippet.push('\n');
                }
            }
            
            if end_line_idx < lines.len() && end_line_idx != start_line_idx {
                let last_line = lines[end_line_idx];
                if end.column <= last_line.len() {
                    snippet.push_str(&last_line[..end.column]);
                } else {
                    snippet.push_str(last_line);
                }
            }
            
            snippet
        }
    }

    /// Get context around a span (includes surrounding lines)
    pub fn extract_context<T: Spanned>(&self, node: &T, context_lines: usize) -> String {
        let span = node.span();
        let start = span.start();
        let end = span.end();
        
        if start.line == 0 || end.line == 0 {
            return "// Context unavailable".to_string();
        }

        let lines: Vec<&str> = self.source_code.lines().collect();
        
        let context_start = (start.line.saturating_sub(context_lines + 1)).max(0);
        let context_end = (end.line + context_lines - 1).min(lines.len());
        
        let mut context = String::new();
        
        for (i, line_idx) in (context_start..context_end).enumerate() {
            let actual_line_num = line_idx + 1;
            let line = lines.get(line_idx).unwrap_or(&"");
            
            if actual_line_num >= start.line && actual_line_num <= end.line {
                context.push_str(&format!("→ {actual_line_num:3} | {line}\n"));
            } else {
                context.push_str(&format!("  {actual_line_num:3} | {line}\n"));
            }
        }
        
        context
    }

    /// Extract function signature or struct definition
    pub fn extract_definition_signature<T: Spanned>(&self, node: &T) -> String {
        let span = node.span();
        let start = span.start();
        
        if start.line == 0 {
            return "// Signature unavailable".to_string();
        }

        let lines: Vec<&str> = self.source_code.lines().collect();
        
        if start.line > lines.len() {
            return "// Signature out of bounds".to_string();
        }

        let def_line = lines[(start.line - 1).min(lines.len() - 1)];
        
        if def_line.trim_start().starts_with("pub fn") || def_line.trim_start().starts_with("fn") {
            let mut signature = def_line.trim().to_string();
            
            if !signature.ends_with('{') && !signature.ends_with(';') {
                for line_idx in start.line..lines.len().min(start.line + 3) {
                    if let Some(next_line) = lines.get(line_idx) {
                        signature.push(' ');
                        signature.push_str(next_line.trim());
                        if next_line.contains('{') || next_line.contains(';') {
                            break;
                        }
                    }
                }
            }
            
            if let Some(brace_pos) = signature.find('{') {
                signature = signature[..brace_pos].trim().to_string();
            }
            
            signature
        } else if def_line.trim_start().starts_with("struct") || def_line.trim_start().starts_with("pub struct") {
            let mut signature = def_line.trim().to_string();
            if let Some(brace_pos) = signature.find('{') {
                signature = signature[..brace_pos].trim().to_string();
            }
            signature
        } else {
            def_line.trim().to_string()
        }
    }
}

impl Location {
    pub fn new_precise(file: String, line: usize, column: Option<usize>, end_line: Option<usize>, end_column: Option<usize>) -> Self {
        Self {
            file,
            line,
            column,
            end_line,
            end_column,
        }
    }

    pub fn format_location(&self) -> String {
        match (&self.column, &self.end_line, &self.end_column) {
            (Some(col), Some(end_line), Some(end_col)) if end_line != &self.line => {
                format!("{}:{}:{}-{}:{}", self.file, self.line, col, end_line, end_col)
            }
            (Some(col), _, Some(end_col)) => {
                format!("{}:{}:{}-{}", self.file, self.line, col, end_col)
            }
            (Some(col), _, _) => {
                format!("{}:{}:{}", self.file, self.line, col)
            }
            _ => {
                format!("{}:{}", self.file, self.line)
            }
        }
    }
}

/// Extract span from `NodeData`
pub fn extract_span_from_node_data(node_data: &NodeData) -> Span {
    match node_data {
        NodeData::File(file) => file.span(),
        NodeData::Function(func) => func.span(),
        NodeData::ImplFunction(impl_func) => impl_func.span(),
        NodeData::Struct(struct_item) => struct_item.span(),
        NodeData::Enum(enum_item) => enum_item.span(),
        NodeData::Block(block) => block.span(),
        NodeData::Expression(expr) => expr.span(),
        NodeData::Other => Span::call_site(),
    }
}