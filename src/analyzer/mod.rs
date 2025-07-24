// Declare submodules
pub mod dsl;
pub mod engine;
pub mod rules;
pub mod reporting;
pub mod span_utils;

// Standard imports
use anyhow::Context;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use syn::File;

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
    pub column: Option<usize>,
    /// End line number (1-indexed)
    pub end_line: Option<usize>,
    /// End column number (1-indexed)
    pub end_column: Option<usize>,
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

pub use engine::{
    Rule, RuleEngine, RuleEngineConfig, RuleType, create_rule_engine,
    create_rule_engine_with_config,
};

/// Creates an analyzer with default options
pub fn create_analyzer() -> Analyzer {
    Analyzer::new()
}

/// Creates an analyzer with custom options
pub fn create_analyzer_with_options(options: AnalysisOptions) -> Analyzer {
    Analyzer::with_options(options)
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

/// Options for analysis
#[derive(Debug, Clone, Default)]
pub struct AnalysisOptions {
    /// Whether to generate AST JSON files
    pub generate_ast: bool,

    /// Path to custom templates
    pub custom_templates_path: Option<String>,

    /// Severities to ignore
    pub ignore_severities: Vec<Severity>,

    /// Rule IDs to ignore
    pub ignore_rules: Vec<String>,

    /// Rule types to include
    pub include_rule_types: Vec<RuleType>,
}

/// Analyzer for Solana contracts
pub struct Analyzer {
    /// Options for analysis
    options: AnalysisOptions,

    /// Rule engine
    rule_engine: RuleEngine,
}

impl Analyzer {
    /// Creates a new analyzer with default options
    pub fn new() -> Self {
        Self {
            options: AnalysisOptions::default(),
            rule_engine: create_rule_engine(),
        }
    }

    /// Creates a new analyzer with the given options
    pub fn with_options(options: AnalysisOptions) -> Self {
        // Convert analysis options to rule engine config
        let config = RuleEngineConfig {
            custom_templates_path: options.custom_templates_path.clone(),
            ignore_severities: options.ignore_severities.clone(),
            ignore_rules: options.ignore_rules.clone(),
            include_rule_types: options.include_rule_types.clone(),
        };

        let mut rule_engine = create_rule_engine_with_config(config);

        // Load built-in rules
        if let Err(e) = rule_engine.load_builtin_rules() {
            warn!("Failed to load built-in rules: {}", e);
        }

        // Load custom rules if specified
        if let Some(templates_path) = &options.custom_templates_path {
            let path = Path::new(templates_path);
            if path.exists() && path.is_dir() {
                if let Err(e) = rule_engine.load_yaml_rules(path) {
                    warn!("Failed to load YAML rules from {}: {}", path.display(), e);
                }
            } else {
                warn!(
                    "Custom templates path does not exist or is not a directory: {}",
                    path.display()
                );
            }
        }

        Self {
            options,
            rule_engine,
        }
    }

    /// Analyzes a single file
    pub fn analyze_file(&self, file_path: &str, ast: &File) -> Result<Vec<Finding>> {
        debug!("Analyzing file: {}", file_path);

        // Read source code for precise locations
        let source_code = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read source code from {}", file_path))?;

        // Execute rules on the AST with source code for precise locations
        let findings = self
            .rule_engine
            .execute_rules(ast, file_path, &source_code)
            .with_context(|| format!("Failed to execute rules on {}", file_path))?;

        debug!("Found {} issues in {}", findings.len(), file_path);

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
                        *stats
                            .findings_by_severity
                            .entry(finding.severity.clone())
                            .or_insert(0) += 1;
                    }

                    all_findings.extend(findings);
                }
                Err(e) => {
                    warn!("Error analyzing {}: {}", file_path, e);
                }
            }
        }

        stats.total_time_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "Analysis completed: {} findings in {}ms",
            all_findings.len(),
            stats.total_time_ms
        );

        Ok(AnalysisResult {
            findings: all_findings,
            stats,
        })
    }
}
