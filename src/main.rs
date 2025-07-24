use anyhow::Result;
use clap::Parser;
use env_logger;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

mod analyzer;
mod ast;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Contract path or directory to analyze
    #[arg(short, long)]
    path: PathBuf,

    /// Custom templates path
    #[arg(short, long)]
    templates: Option<PathBuf>,

    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Severities to ignore (separated by commas: low,medium,high,informational)
    #[arg(short, long)]
    ignore: Option<String>,

    /// Rule IDs to ignore (separated by commas)
    #[arg(long)]
    ignore_rules: Option<String>,

    /// Generate AST JSON along with the report
    #[arg(long)]
    ast: bool,

    /// Analyze vulnerabilities
    #[arg(long)]
    analyze: bool,
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse arguments from command line
    let args = Cli::parse();
    debug!("CLI arguments: {:?}", args);

    // Verify that the path exists
    if !args.path.exists() {
        anyhow::bail!("Path {} does not exist", args.path.display());
    }

    // Verify that the path is a directory
    if !args.path.is_dir() {
        anyhow::bail!("Path {} is not a directory", args.path.display());
    }

    info!("Starting analysis on directory: {}", args.path.display());
    let results = ast::parser::process_directory(&args.path)?;
    info!("Found {} Rust files to analyze", results.len());

    if args.ast {
        for (path, ast) in &results {
            let json = ast::json::ast_to_json(&ast)?;
            let mut json_path = path.clone();
            json_path.set_extension("json");
            fs::write(json_path, json)?;
            info!("AST JSON generated for {}", path.display());
        }
    }

    // Analyze vulnerabilities if requested
    if args.analyze {
        info!("Analyzing vulnerabilities");

        // Create analysis options based on CLI arguments
        let mut options = analyzer::AnalysisOptions::default();
        options.generate_ast = args.ast;

        // Set default rule types to include
        options.include_rule_types = vec![
            analyzer::RuleType::Solana,
            analyzer::RuleType::Anchor,
            analyzer::RuleType::General,
        ];

        if let Some(templates) = &args.templates {
            options.custom_templates_path = Some(templates.to_string_lossy().to_string());
        }

        if let Some(ignore) = &args.ignore {
            // Parse severities to ignore
            for sev in ignore.split(',') {
                match sev.trim().to_lowercase().as_str() {
                    "high" => options.ignore_severities.push(analyzer::Severity::High),
                    "medium" => options.ignore_severities.push(analyzer::Severity::Medium),
                    "low" => options.ignore_severities.push(analyzer::Severity::Low),
                    "informational" => options
                        .ignore_severities
                        .push(analyzer::Severity::Informational),
                    _ => warn!("Unknown severity level: {}", sev),
                }
            }
        }

        if let Some(ignore_rules) = &args.ignore_rules {
            // Parse rule IDs to ignore
            for rule_id in ignore_rules.split(',') {
                options.ignore_rules.push(rule_id.trim().to_string());
            }
        }

        // Create analyzer and run analysis
        let analyzer = analyzer::create_analyzer_with_options(options);
        match analyzer.analyze_files(&results) {
            Ok(analysis_result) => {
                info!(
                    "Analysis completed: {} findings",
                    analysis_result.findings.len()
                );

                // Show summary of findings by severity
                let mut severity_counts = HashMap::new();
                for (severity, count) in &analysis_result.stats.findings_by_severity {
                    severity_counts.insert(severity, *count);
                }

                // Display in order of severity (High to Informational)
                for severity in &[
                    analyzer::Severity::High,
                    analyzer::Severity::Medium,
                    analyzer::Severity::Low,
                    analyzer::Severity::Informational,
                ] {
                    if let Some(count) = severity_counts.get(severity) {
                        info!("- {:?}: {}", severity, count);
                    }
                }

                // Save results to file if specified
                if let Some(output_path) = &args.output {
                    let report_generator = analyzer::reporting::ReportGenerator::new(
                        analysis_result.findings.clone(),
                        args.path.to_string_lossy().to_string(),
                    );
                    
                    let output_str = output_path.to_string_lossy();
                    if output_str.ends_with(".md") || output_str.ends_with(".markdown") {
                        // Generate Markdown report
                        match report_generator.save_markdown_report(&output_str) {
                            Ok(()) => info!("📄 Markdown report saved to: {}", output_path.display()),
                            Err(e) => error!("Failed to save report: {}", e),
                        }
                    } else {
                        // Default to Markdown with .md extension
                        let mut md_path = output_path.clone();
                        md_path.set_extension("md");
                        match report_generator.save_markdown_report(&md_path.to_string_lossy()) {
                            Ok(()) => info!("📄 Markdown report saved to: {}", md_path.display()),
                            Err(e) => error!("Failed to save report: {}", e),
                        }
                    }
                } else {
                    // Show findings in the console using logs
                    if analysis_result.findings.is_empty() {
                        info!("No vulnerabilities found");
                    } else {
                        info!("Found {} vulnerabilities:", analysis_result.findings.len());

                        // Group findings by severity for better readability
                        let mut findings_by_severity = HashMap::new();
                        for finding in &analysis_result.findings {
                            findings_by_severity
                                .entry(&finding.severity)
                                .or_insert_with(Vec::new)
                                .push(finding);
                        }

                        // Display findings in order of severity
                        let mut index = 1;
                        for severity in &[
                            analyzer::Severity::High,
                            analyzer::Severity::Medium,
                            analyzer::Severity::Low,
                            analyzer::Severity::Informational,
                        ] {
                            if let Some(findings) = findings_by_severity.get(severity) {
                                info!("----- {:?} Severity Findings -----", severity);

                                for finding in findings {
                                    info!(
                                        "{}.\t{} ({}:{})",
                                        index,
                                        finding.description,
                                        finding.location.file,
                                        finding.location.line
                                    );

                                    // Show code snippet if available
                                    if let Some(snippet) = &finding.code_snippet {
                                        debug!("    Code: {}", snippet);
                                    }

                                    index += 1;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error during analysis: {}", e);
            }
        }
    }

    info!("Analysis completed.");
    Ok(())
}
