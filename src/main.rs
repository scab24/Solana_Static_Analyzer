use anyhow::{Result, Context};
use clap::Parser;
use std::path::PathBuf;
use std::fs;
use log::{info, error, warn, debug, trace};
use env_logger;

mod ast;
mod analyzer;

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
    
    /// Severities to ignore (separated by commas: low,medium,high,critical)
    #[arg(short, long)]
    ignore: Option<String>,
    
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
                    "informational" => options.ignore_severities.push(analyzer::Severity::Informational),
                    _ => warn!("Unknown severity level: {}", sev),
                }
            }
        }
        
        // Create analyzer and run analysis
        let analyzer = analyzer::create_analyzer_with_options(options);
        match analyzer.analyze_files(&results) {
            Ok(analysis_result) => {
                info!("Analysis completed: {} findings", analysis_result.findings.len());
                
                // Show summary of findings by severity
                for (severity, count) in &analysis_result.stats.findings_by_severity {
                    info!("- {:?}: {}", severity, count);
                }
                
                // Save results to file if specified
                if let Some(output_path) = &args.output {
                    // TODO: Implement report generation
                    // For now, just show a message
                    info!("Report would be saved to: {}", output_path.display());
                } else {
                    // Show findings in the console using logs
                    if analysis_result.findings.is_empty() {
                        info!("No vulnerabilities found");
                    } else {
                        info!("Found {} vulnerabilities:", analysis_result.findings.len());
                        for (i, finding) in analysis_result.findings.iter().enumerate() {
                            info!("{}. [{:?}] {} ({}:{})", 
                                i + 1,
                                finding.severity,
                                finding.description,
                                finding.location.file,
                                finding.location.line
                            );
                            
                            // Show code snippet if available
                            if let Some(snippet) = &finding.code_snippet {
                                debug!("Code: {}", snippet);
                            }
                        }
                    }
                }
            },
            Err(e) => {
                error!("Error during analysis: {}", e);
            }
        }
    }
    
    info!("Analysis completed.");
    Ok(())
}