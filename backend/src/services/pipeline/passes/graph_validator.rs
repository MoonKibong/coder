//! Pass 4: Graph Validator
//!
//! Validates Dataset ↔ UI component relationships.
//! Ensures link_data attributes reference valid datasets.

use crate::services::pipeline::{GenerationContext, Pass, PassResult};
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Graph Validator - validates dataset/component bindings
pub struct GraphValidator;

impl GraphValidator {
    pub fn new() -> Self {
        Self
    }

    /// Extract dataset IDs from XML
    fn extract_datasets(&self, xml: &str) -> HashSet<String> {
        let mut datasets = HashSet::new();

        // Match <xdataset id="..."> or <xlinkdataset id="..."> or <dataset id="...">
        let re = Regex::new(r#"<(?:x?(?:link)?dataset|Dataset)\s+[^>]*id="([^"]+)""#).unwrap();
        for cap in re.captures_iter(xml) {
            datasets.insert(cap[1].to_string());
        }

        datasets
    }

    /// Extract link_data references from XML
    fn extract_link_data_refs(&self, xml: &str) -> Vec<(String, String)> {
        let mut refs = Vec::new();

        // Match link_data="ds_xxx" or link_data="ds_xxx:COLUMN"
        let re = Regex::new(r#"(\w+)\s+[^>]*link_data="([^":]+)(?::[^"]+)?""#).unwrap();
        for cap in re.captures_iter(xml) {
            refs.push((cap[1].to_string(), cap[2].to_string()));
        }

        refs
    }

    /// Extract column definitions from datasets
    /// TODO: Use this for full column binding validation
    #[allow(dead_code)]
    fn extract_dataset_columns(&self, xml: &str) -> HashMap<String, HashSet<String>> {
        let mut columns: HashMap<String, HashSet<String>> = HashMap::new();

        // Find datasets and their columns
        // Pattern: <xlinkdataset id="ds_xxx" columns="COL1:type,COL2:type,...">
        let ds_re = Regex::new(
            r#"<(?:x?(?:link)?dataset|Dataset)\s+[^>]*id="([^"]+)"[^>]*columns="([^"]*)""#,
        )
        .unwrap();

        for cap in ds_re.captures_iter(xml) {
            let ds_id = cap[1].to_string();
            let cols_str = &cap[2];

            let mut col_set = HashSet::new();
            for col_def in cols_str.split(',') {
                if let Some(col_name) = col_def.split(':').next() {
                    col_set.insert(col_name.trim().to_string());
                }
            }
            columns.insert(ds_id, col_set);
        }

        // Also check for <column> elements inside datasets
        // This is a simplified check - a full XML parser would be better
        let col_re = Regex::new(r#"<(?:column|Column)[^>]*name="([^"]+)""#).unwrap();
        let data_re = Regex::new(r#"<data[^>]*name="([^"]+)""#).unwrap();

        // For now, add all found column names to a default set
        // (In real implementation, would parse XML structure properly)
        for cap in col_re.captures_iter(xml) {
            columns
                .entry("_all_columns".to_string())
                .or_default()
                .insert(cap[1].to_string());
        }
        for cap in data_re.captures_iter(xml) {
            columns
                .entry("_all_columns".to_string())
                .or_default()
                .insert(cap[1].to_string());
        }

        columns
    }

    /// Validate dataset references
    fn validate_references(
        &self,
        datasets: &HashSet<String>,
        refs: &[(String, String)],
    ) -> Vec<String> {
        let mut errors = Vec::new();

        for (component, dataset_ref) in refs {
            if !datasets.contains(dataset_ref) {
                errors.push(format!(
                    "Component '{}' references non-existent dataset '{}'",
                    component, dataset_ref
                ));
            }
        }

        errors
    }
}

impl Default for GraphValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for GraphValidator {
    fn name(&self) -> &'static str {
        "GraphValidator"
    }

    fn run(&self, ctx: &mut GenerationContext) -> PassResult {
        let xml = match &ctx.xml {
            Some(xml) => xml.clone(),
            None => return PassResult::Error("XML not available".to_string()),
        };

        // Extract datasets and references
        let datasets = self.extract_datasets(&xml);
        let refs = self.extract_link_data_refs(&xml);

        // Validate references
        let errors = self.validate_references(&datasets, &refs);

        if errors.is_empty() {
            // All references are valid
            if datasets.is_empty() {
                ctx.add_warning("No datasets found in XML");
            }
            return PassResult::Ok;
        }

        // Handle errors based on execution mode
        if ctx.is_strict() {
            return PassResult::Error(errors.join("; "));
        }

        // In non-strict mode, add warnings
        for error in &errors {
            ctx.add_warning(error.clone());
        }

        PassResult::Warning(format!(
            "Found {} invalid dataset reference(s)",
            errors.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ScreenType, UiIntent};
    use crate::services::pipeline::ExecutionMode;

    fn create_context(xml: &str, mode: ExecutionMode) -> GenerationContext {
        let intent = UiIntent::new("test", ScreenType::List);
        let mut ctx = GenerationContext::new("".to_string(), intent, mode);
        ctx.xml = Some(xml.to_string());
        ctx.javascript = Some("".to_string());
        ctx
    }

    #[test]
    fn test_extract_datasets() {
        let xml = r#"
            <xlinkdataset id="ds_list"/>
            <xdataset id="ds_search"/>
            <Dataset id="ds_code"/>
        "#;

        let validator = GraphValidator::new();
        let datasets = validator.extract_datasets(xml);

        assert!(datasets.contains("ds_list"));
        assert!(datasets.contains("ds_search"));
        assert!(datasets.contains("ds_code"));
    }

    #[test]
    fn test_extract_link_data_refs() {
        let xml = r#"
            <grid name="grid_list" link_data="ds_list"/>
            <column>
                <data link_data="ds_list:NAME"/>
            </column>
        "#;

        let validator = GraphValidator::new();
        let refs = validator.extract_link_data_refs(xml);

        assert!(refs.iter().any(|(_, ds)| ds == "ds_list"));
    }

    #[test]
    fn test_valid_references_pass() {
        let xml = r#"
            <xlinkdataset id="ds_list"/>
            <grid name="grid_list" link_data="ds_list"/>
        "#;

        let mut ctx = create_context(xml, ExecutionMode::Strict);
        let result = GraphValidator::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Ok));
    }

    #[test]
    fn test_invalid_reference_error_strict() {
        let xml = r#"
            <xlinkdataset id="ds_list"/>
            <grid name="grid_list" link_data="ds_nonexistent"/>
        "#;

        let mut ctx = create_context(xml, ExecutionMode::Strict);
        let result = GraphValidator::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Error(_)));
    }

    #[test]
    fn test_invalid_reference_warning_relaxed() {
        let xml = r#"
            <xlinkdataset id="ds_list"/>
            <grid name="grid_list" link_data="ds_nonexistent"/>
        "#;

        let mut ctx = create_context(xml, ExecutionMode::Relaxed);
        let result = GraphValidator::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Warning(_)));
    }

    #[test]
    fn test_no_datasets_warning() {
        let xml = r#"<screen id="test"/>"#;

        let mut ctx = create_context(xml, ExecutionMode::Relaxed);
        GraphValidator::new().run(&mut ctx);

        assert!(ctx.warnings.iter().any(|w| w.contains("No datasets")));
    }
}
