use log::{debug, info};
use std::sync::Arc;
use syn::File;

use crate::analyzer::{Finding, Severity};
use crate::analyzer::engine::{Rule, RuleType, RustRule};

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
    /// Query builder with `SpanExtractor` support
    query_builder: Option<Box<dyn Fn(&File, &str, &crate::analyzer::span_utils::SpanExtractor) -> Vec<Finding> + Send + Sync>>,
    /// References to documentation or additional resources
    references: Vec<String>,
    /// Recommendations for fixing the issue
    recommendations: Vec<String>,
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
            recommendations: Vec::new(),
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

    /// Sets a visitor-based rule implementation
    pub fn visitor_rule<F>(mut self, rule_fn: F) -> Self
    where
        F: Fn(&syn::File, &str, &crate::analyzer::span_utils::SpanExtractor) -> Vec<crate::analyzer::Finding> + Send + Sync + 'static,
    {
        self.query_builder = Some(Box::new(rule_fn));
        self
    }

    /// Sets a DSL-based rule implementation with `SpanExtractor` for precise locations
    pub fn dsl_rule<F>(mut self, rule_fn: F) -> Self
    where
        F: Fn(&syn::File, &str, &crate::analyzer::span_utils::SpanExtractor) -> Vec<crate::analyzer::Finding> + Send + Sync + 'static,
    {
        // Store the rule function that expects SpanExtractor
        // The SpanExtractor will be provided when the rule is executed
        self.query_builder = Some(Box::new(move |file, file_path, span_extractor| {
            rule_fn(file, file_path, span_extractor)
        }));
        self
    }

    /// Sets the query builder (function that analyzes the AST and returns findings)
    pub fn query<F>(mut self, query_builder: F) -> Self
    where
        F: Fn(&File, &str, &crate::analyzer::span_utils::SpanExtractor) -> Vec<Finding> + Send + Sync + 'static,
    {
        self.query_builder = Some(Box::new(query_builder));
        self
    }

    /// Sets a DSL-based query builder (function that returns `AstQuery` for more expressive queries)
    /// This is the new, preferred way to define rules using the DSL
    pub fn dsl_query<F>(mut self, dsl_builder: F) -> Self
    where
        F: for<'a> Fn(&'a File, &'a str, &'a crate::analyzer::span_utils::SpanExtractor) -> crate::analyzer::dsl::query::AstQuery<'a> + Send + Sync + 'static,
    {
        // Capture rule metadata for use in the wrapped builder
        let rule_severity = self.severity.clone();
        let rule_title = self.title.clone();
        let rule_description = self.description.clone();
        let rule_recommendations = self.recommendations.clone();
        
        // Wrap the DSL builder to convert AstQuery to Vec<Finding>
        let wrapped_builder = move |ast: &File, file_path: &str, span_extractor: &crate::analyzer::span_utils::SpanExtractor| -> Vec<Finding> {
            let query_result = dsl_builder(ast, file_path, span_extractor);
            
            // Convert AstQuery to findings using the rule's actual metadata
            query_result.to_findings_with_span_extractor(
                rule_severity.clone(),
                &rule_title,
                &rule_description,
                &rule_recommendations,
                file_path,
                span_extractor
            )
        };
        
        self.query_builder = Some(Box::new(wrapped_builder));
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

    /// Adds a recommendation for fixing the issue
    pub fn recommendation(mut self, recommendation: &str) -> Self {
        self.recommendations.push(recommendation.to_string());
        self
    }

    /// Adds multiple recommendations for fixing the issue
    pub fn recommendations(mut self, recs: Vec<&str>) -> Self {
        for recommendation in recs {
            self.recommendations.push(recommendation.to_string());
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
        let recommendations = self.recommendations;
        let tags = self.tags;
        let enabled = self.enabled;
        let id = self.id.clone();
        let title = self.title.clone();
        let description = self.description.clone();
        let severity = self.severity.clone();
        let rule_type = self.rule_type.clone();

        // Log information about the rule
        if !references.is_empty() {
            info!("References for rule {id}: {references:?}");
        }
        if !tags.is_empty() {
            info!("Tags for rule {id}: {tags:?}");
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
            recommendations,
            move |ast, file_path, span_extractor| {
                debug!("Executing rule {id_clone} in {file_path}");

                // Execute the query with SpanExtractor and get findings directly
                let findings = query_builder(ast, file_path, span_extractor);

                // Only return findings if the rule is enabled
                if enabled {
                    Ok(findings)
                } else {
                    debug!("Rule {id_clone} is disabled, no findings returned");
                    Ok(Vec::new())
                }
            },
        ))
    }
}
