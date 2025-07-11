// Declare the submodule dsl
pub mod dsl;

// Importations standard
use std::path::Path;
use syn::File;
use std::collections::HashMap;
use log::{info, debug, warn};

/// Severity level of a vulnerability
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Severity {
    /// High severity vulnerability that must be fixed immediately
    High,
    /// Medium severity vulnerability that should be fixed
    Medium,
    /// Low severity vulnerability or non-recommended practice
    Low,
    /// Information that could be useful but does not represent a direct risk
    Informational,
}

/// Location of a vulnerability in the source code
#[derive(Debug, Clone)]
pub struct Location {
    /// File path
    pub file: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

/// Finding of a vulnerability
#[derive(Debug, Clone)]
pub struct Finding {
    /// Description of the vulnerability
    pub description: String,
    /// Severity level of the vulnerability
    pub severity: Severity,
    /// Location of the vulnerability in the source code
    pub location: Location,
    /// Code snippet containing the vulnerability (optional)
    pub code_snippet: Option<String>,
}

/// Custom result type for analyzer operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub mod rules {
    // TODO: Implement rules
}

/// Result of an analysis
#[derive(Debug)]
pub struct AnalysisResult {
    /// Findings found during the analysis
    pub findings: Vec<Finding>,
    /// Statistics of the analysis
    pub stats: AnalysisStats,
}

/// Statistics of an analysis
#[derive(Debug, Default)]
pub struct AnalysisStats {
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Number of rules executed
    pub rules_executed: usize,
    /// Total analysis time in milliseconds
    pub total_time_ms: u64,
    /// Breakdown of findings by severity
    pub findings_by_severity: HashMap<Severity, usize>,
}

/// Options for analysis configuration
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    /// Severities to ignore
    pub ignore_severities: Vec<Severity>,
    /// Path to custom templates
    pub custom_templates_path: Option<String>,
    /// Whether to generate AST JSON
    pub generate_ast: bool,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            ignore_severities: Vec::new(),
            custom_templates_path: None,
            generate_ast: false,
        }
    }
}

/// Main analyzer
pub struct Analyzer {
    /// Configuration options
    options: AnalysisOptions,
}

impl Analyzer {
    /// Creates a new analyzer with default options
    pub fn new() -> Self {
        Self {
            options: AnalysisOptions::default(),
        }
    }
    
    /// Creates a new analyzer with custom options
    pub fn with_options(options: AnalysisOptions) -> Self {
        Self { options }
    }
    
    /// Analyzes a Rust file
    pub fn analyze_file(&self, file_path: &str, ast: &File) -> Result<Vec<Finding>> {
        debug!("Analyzing file: {}", file_path);
        
        let mut findings = Vec::new();
        
        // TODO: Implement rules
        
        debug!("Analysis completed for {}: {} findings", file_path, findings.len());
        Ok(findings)
    }

    /// Analyzes multiple Rust files
    pub fn analyze_files(&self, files: &[(std::path::PathBuf, File)]) -> Result<AnalysisResult> {
        info!("Starting analysis of {} files", files.len());
        
        let start_time = std::time::Instant::now();
        let mut stats = AnalysisStats::default();
        stats.files_analyzed = files.len();
        
        let mut all_findings = Vec::new();
        
        for (path, ast) in files {
            let file_path = path.to_string_lossy().to_string();
            match self.analyze_file(&file_path, ast) {
                Ok(mut findings) => {
                    // Filter findings by severity
                    findings.retain(|f| !self.options.ignore_severities.contains(&f.severity));
                    
                    // Update statistics
                    for finding in &findings {
                        *stats.findings_by_severity.entry(finding.severity.clone()).or_insert(0) += 1;
                    }
                    
                    all_findings.extend(findings);
                },
                Err(e) => {
                    warn!("Error analyzing {}: {}", file_path, e);
                }
            }
        }
        
        stats.total_time_ms = start_time.elapsed().as_millis() as u64;
        
        info!("Analysis completed: {} findings in {}ms", all_findings.len(), stats.total_time_ms);
        
        Ok(AnalysisResult {
            findings: all_findings,
            stats,
        })
    }
}

/// Creates an analyzer with default options
pub fn create_analyzer() -> Analyzer {
    Analyzer::new()
}

/// Creates an analyzer with custom options
pub fn create_analyzer_with_options(options: AnalysisOptions) -> Analyzer {
    Analyzer::with_options(options)
}