use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use log::{debug, info, warn};
use syn::File;

use crate::analyzer::dsl::AstQuery;
use crate::analyzer::{Finding, Severity};

/// Type of rule
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuleType {
    /// Rules specific to Solana
    Solana,
    /// Rules specific to Anchor framework
    Anchor,
    /// General Rust rules
    General,
}

/// A rule that can be applied to an AST
pub trait Rule: Send + Sync {
    /// Returns the unique ID of the rule
    fn id(&self) -> &str;

    /// Returns the title of the rule
    fn title(&self) -> &str;

    /// Returns the description of the rule
    fn description(&self) -> &str;

    /// Returns the severity of the rule
    fn severity(&self) -> Severity;

    /// Returns the type of the rule
    fn rule_type(&self) -> RuleType;

    /// Checks if the rule applies to the given AST
    fn check(&self, ast: &File, file_path: &str) -> Result<Vec<Finding>>;
}

/// Configuration for the rule engine
#[derive(Debug, Clone)]
pub struct RuleEngineConfig {
    /// Path to custom rule templates
    pub custom_templates_path: Option<String>,

    /// Severities to ignore
    pub ignore_severities: Vec<Severity>,

    /// Rule IDs to ignore
    pub ignore_rules: Vec<String>,

    /// Rule types to include
    pub include_rule_types: Vec<RuleType>,
}

impl Default for RuleEngineConfig {
    fn default() -> Self {
        Self {
            custom_templates_path: None,
            ignore_severities: Vec::new(),
            ignore_rules: Vec::new(),
            include_rule_types: vec![RuleType::Solana, RuleType::Anchor, RuleType::General],
        }
    }
}

/// Engine for loading and executing rules
pub struct RuleEngine {
    /// Rules loaded in the engine
    rules: Vec<Arc<dyn Rule>>,

    /// Configuration for the engine
    config: RuleEngineConfig,
}

impl RuleEngine {
    /// Creates a new rule engine with the given configuration
    pub fn new(config: RuleEngineConfig) -> Self {
        Self {
            rules: Vec::new(),
            config,
        }
    }

    /// Creates a new rule engine with default configuration
    pub fn default() -> Self {
        Self::new(RuleEngineConfig::default())
    }

    /// Loads built-in rules
    pub fn load_builtin_rules(&mut self) -> Result<()> {
        debug!("Loading built-in rules");

        // Register security templates
        if let Err(e) = crate::analyzer::rules::register_builtin_rules(self) {
            return Err(anyhow::anyhow!("Failed to register built-in rules: {}", e));
        }

        info!("Loaded {} built-in rules", self.rule_count());

        Ok(())
    }

    /// Loads rules from YAML templates
    pub fn load_yaml_rules(&mut self, templates_path: &Path) -> Result<()> {
        debug!("Loading YAML rules from {}", templates_path.display());

        //@todo => implement YAML rule loading
        info!("YAML rule loading not implemented yet");

        Ok(())
    }

    /// Adds a rule to the engine
    pub fn add_rule(&mut self, rule: Arc<dyn Rule>) {
        // Check if the rule should be ignored based on severity
        if self.config.ignore_severities.contains(&rule.severity()) {
            debug!(
                "Ignoring rule {} due to severity {:?}",
                rule.id(),
                rule.severity()
            );
            return;
        }

        // Check if the rule should be ignored based on ID
        if self.config.ignore_rules.contains(&rule.id().to_string()) {
            debug!("Ignoring rule {} due to ID match", rule.id());
            return;
        }

        // Check if the rule type is included
        if !self.config.include_rule_types.contains(&rule.rule_type()) {
            debug!(
                "Ignoring rule {} due to rule type {:?}",
                rule.id(),
                rule.rule_type()
            );
            return;
        }

        debug!("Adding rule: {}", rule.id());
        self.rules.push(rule);
    }

    /// Returns the number of rules loaded
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Executes all rules on the given AST
    pub fn execute_rules(&self, ast: &File, file_path: &str) -> Result<Vec<Finding>> {
        debug!("Executing {} rules on {}", self.rules.len(), file_path);

        let mut findings = Vec::new();

        for rule in &self.rules {
            match rule.check(ast, file_path) {
                Ok(rule_findings) => {
                    debug!("Rule {} found {} issues", rule.id(), rule_findings.len());
                    findings.extend(rule_findings);
                }
                Err(e) => {
                    warn!("Error executing rule {}: {}", rule.id(), e);
                }
            }
        }

        Ok(findings)
    }
}

pub struct RustRule {
    /// Unique ID of the rule
    id: String,

    /// Title of the rule
    title: String,

    /// Description of the rule
    description: String,

    /// Severity of the rule
    severity: Severity,

    /// Type of the rule
    rule_type: RuleType,

    /// Function that implements the rule check
    check_fn: Box<dyn Fn(&File, &str) -> Result<Vec<Finding>> + Send + Sync>,
}

impl RustRule {
    /// Creates a new rule with the given parameters
    pub fn new<F>(
        id: &str,
        title: &str,
        description: &str,
        severity: Severity,
        rule_type: RuleType,
        check_fn: F,
    ) -> Self
    where
        F: Fn(&File, &str) -> Result<Vec<Finding>> + Send + Sync + 'static,
    {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            severity,
            rule_type,
            check_fn: Box::new(check_fn),
        }
    }
}

impl Rule for RustRule {
    fn id(&self) -> &str {
        &self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn severity(&self) -> Severity {
        self.severity.clone()
    }

    fn rule_type(&self) -> RuleType {
        self.rule_type.clone()
    }

    fn check(&self, ast: &File, file_path: &str) -> Result<Vec<Finding>> {
        (self.check_fn)(ast, file_path)
    }
}

/// Helper function to create a rule engine with default configuration
pub fn create_rule_engine() -> RuleEngine {
    RuleEngine::default()
}

/// Helper function to create a rule engine with the given configuration
pub fn create_rule_engine_with_config(config: RuleEngineConfig) -> RuleEngine {
    RuleEngine::new(config)
}
