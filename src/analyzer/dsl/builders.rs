use log::{debug, info};
use std::marker::PhantomData;
use std::sync::Arc;
use syn::File;

use crate::analyzer::dsl::query::{AstNode, AstQuery};
use crate::analyzer::engine::{Rule, RuleType, RustRule};
use crate::analyzer::{Finding, Severity};
use syn::__private::Span;

/// Rule builder to facilitate the creation of static analysis rules
///
/// This builder provides a fluid API for defining rules in a declarative
/// and expressive way, making it easier to create complex rules without
/// having to manually implement the Rule interface.
pub struct RuleBuilder {
    /// Rule ID
    id: String,
    /// Rule title
    title: String,
    /// Rule description
    description: String,
    /// Rule severity
    severity: Severity,
    /// Rule type
    rule_type: RuleType,
    /// Query builder
    query_builder: Option<Box<dyn Fn(&File) -> Vec<Finding> + Send + Sync>>,
    /// References to documentation or additional resources
    references: Vec<String>,
    /// Tags to classify the rule
    tags: Vec<String>,
    /// Indicates if the rule is enabled by default
    enabled: bool,
}

impl RuleBuilder {
    /// Creates a new rule builder with default values
    pub fn new() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            description: String::new(),
            severity: Severity::Medium,
            rule_type: RuleType::Solana,
            query_builder: None,
            references: Vec::new(),
            tags: Vec::new(),
            enabled: true,
        }
    }

    /// Sets the rule ID
    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    /// Sets the rule title
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Sets the rule description
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Sets the rule severity
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Sets the rule type
    pub fn rule_type(mut self, rule_type: RuleType) -> Self {
        self.rule_type = rule_type;
        self
    }

    /// Sets the rule query
    pub fn query<F>(mut self, query_builder: F) -> Self
    where
        F: Fn(&File) -> Vec<Finding> + Send + Sync + 'static,
    {
        self.query_builder = Some(Box::new(query_builder));
        self
    }

    /// Sets the message formatter (now integrated into the query)
    pub fn message<F>(self, _formatter: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        //@todo => implement message formatter
        self
    }

    /// Sets an additional filter for the nodes found (now integrated into the query)
    pub fn filter<F>(self, _filter: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        //@todo => implement filter
        self
    }

    /// Sets a transformer to modify findings before returning them (now integrated into the query)
    pub fn transform<F>(self, _transformer: F) -> Self
    where
        F: Fn(Finding) -> Finding + Send + Sync + 'static,
    {
        //@todo => implement transformer
        self
    }

    /// Adds a reference to documentation or additional resources
    pub fn reference(mut self, reference: &str) -> Self {
        self.references.push(reference.to_string());
        self
    }

    /// Adds multiple references to documentation or additional resources
    pub fn references(mut self, refs: Vec<&str>) -> Self {
        for reference in refs {
            self.references.push(reference.to_string());
        }
        self
    }

    /// Adds a tag to classify the rule
    pub fn tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Adds multiple tags to classify the rule
    pub fn tags(mut self, tags: Vec<&str>) -> Self {
        for tag in tags {
            self.tags.push(tag.to_string());
        }
        self
    }

    /// Sets whether the rule is enabled by default
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Builds the rule
    pub fn build(self) -> Arc<dyn Rule> {
        debug!("Building rule: {}", self.id);

        // Verify that we have all the necessary components
        let query_builder = self.query_builder.expect("Query builder is required");
        let references = self.references;
        let tags = self.tags;
        let enabled = self.enabled;
        let id = self.id.clone();
        let title = self.title.clone();
        let description = self.description.clone();
        let severity = self.severity.clone();
        let rule_type = self.rule_type.clone();

        // Log information about the rule
        if !references.is_empty() {
            info!("References for rule {}: {:?}", id, references);
        }
        if !tags.is_empty() {
            info!("Tags for rule {}: {:?}", id, tags);
        }

        if !enabled {
            info!("Rule {} is disabled by default", self.id);
        }

        // Create the rule
        let id_clone = id.clone();
        Arc::new(RustRule::new(
            &id,
            &title,
            &description,
            severity,
            rule_type,
            move |ast, file_path| {
                debug!("Executing rule {} in {}", id_clone, file_path);

                // Execute the query and get findings directly
                let findings = query_builder(ast);

                // Only return findings if the rule is enabled
                if enabled {
                    Ok(findings)
                } else {
                    debug!("Rule {} is disabled, no findings returned", id_clone);
                    Ok(Vec::new())
                }
            },
        ))
    }
}
