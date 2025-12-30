//! Pass 5: Minimalism Pass
//!
//! Removes AI-generated over-engineering:
//! - Unused JavaScript functions
//! - Unreferenced helpers
//! - Unrequested features

use crate::services::pipeline::{GenerationContext, Pass, PassResult};
use regex::Regex;
use std::collections::HashSet;

/// Minimalism Pass - removes unused code
pub struct MinimalismPass;

impl MinimalismPass {
    pub fn new() -> Self {
        Self
    }

    /// Extract function names referenced in XML event handlers
    fn extract_xml_references(&self, xml: &str) -> HashSet<String> {
        let mut refs = HashSet::new();

        // Match eventfunc:fn_xxx patterns
        let re = Regex::new(r#"eventfunc:(\w+)"#).unwrap();
        for cap in re.captures_iter(xml) {
            refs.insert(cap[1].to_string());
        }

        refs
    }

    /// Extract all function definitions from JS
    fn extract_js_functions(&self, js: &str) -> Vec<(String, usize, usize)> {
        let mut functions = Vec::new();

        // Match this.fn_xxx = function patterns with their positions
        let re = Regex::new(r#"(?m)^[\s]*(?://[^\n]*\n)*[\s]*this\.(\w+)\s*=\s*function[^{]*\{[^}]*\};"#).unwrap();

        for cap in re.captures_iter(js) {
            let func_name = cap[1].to_string();
            let start = cap.get(0).unwrap().start();
            let end = cap.get(0).unwrap().end();
            functions.push((func_name, start, end));
        }

        // Also match standalone function declarations
        let func_re = Regex::new(r#"(?m)^[\s]*(?://[^\n]*\n)*[\s]*function\s+(\w+)\s*\([^)]*\)\s*\{[^}]*\}"#).unwrap();

        for cap in func_re.captures_iter(js) {
            let func_name = cap[1].to_string();
            let start = cap.get(0).unwrap().start();
            let end = cap.get(0).unwrap().end();
            functions.push((func_name, start, end));
        }

        functions
    }

    /// Check if a function is referenced elsewhere in the code
    fn is_function_referenced(&self, func_name: &str, js: &str, xml_refs: &HashSet<String>) -> bool {
        // Check if referenced in XML
        if xml_refs.contains(func_name) {
            return true;
        }

        // Check if it's a standard lifecycle function
        let lifecycle_functions = [
            "on_load", "fn_init", "on_unload", "on_resize",
            "fn_search", "fn_save", "fn_delete", "fn_create", "fn_edit",
            "fn_add", "fn_remove", "fn_refresh", "fn_close",
            "fn_onEditorClose", "fn_onPopupClose",
        ];
        if lifecycle_functions.contains(&func_name) {
            return true;
        }

        // Check if called from another function in JS
        // Look for this.func_name() or func_name() calls
        let call_pattern = format!(r#"(?:this\.)?{}\s*\("#, regex::escape(func_name));
        let call_re = Regex::new(&call_pattern).unwrap();

        // Count occurrences - if more than 1 (the definition), it's used
        let count = call_re.find_iter(js).count();
        count > 1
    }

    /// Remove unused functions from JavaScript
    fn remove_unused_functions(
        &self,
        js: &str,
        xml_refs: &HashSet<String>,
    ) -> (String, Vec<String>) {
        let functions = self.extract_js_functions(js);
        let mut removed = Vec::new();
        let mut result = js.to_string();

        // Process in reverse order to maintain correct positions
        let mut functions_sorted = functions.clone();
        functions_sorted.sort_by(|a, b| b.1.cmp(&a.1));

        for (func_name, start, end) in functions_sorted {
            if !self.is_function_referenced(&func_name, js, xml_refs) {
                // Remove the function
                let before = &result[..start];
                let after = &result[end..];
                result = format!("{}{}", before.trim_end(), after);
                removed.push(func_name);
            }
        }

        // Clean up extra whitespace
        let whitespace_re = Regex::new(r#"\n{3,}"#).unwrap();
        result = whitespace_re.replace_all(&result, "\n\n").to_string();

        (result.trim().to_string(), removed)
    }
}

impl Default for MinimalismPass {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for MinimalismPass {
    fn name(&self) -> &'static str {
        "MinimalismPass"
    }

    fn run(&self, ctx: &mut GenerationContext) -> PassResult {
        // In Dev mode, skip minimalism to preserve all generated code
        if ctx.is_dev() {
            return PassResult::Ok;
        }

        let xml = match &ctx.xml {
            Some(xml) => xml.clone(),
            None => return PassResult::Error("XML not available".to_string()),
        };

        let js = match &ctx.javascript {
            Some(js) => js.clone(),
            None => return PassResult::Error("JavaScript not available".to_string()),
        };

        // Extract XML references
        let xml_refs = self.extract_xml_references(&xml);

        // Remove unused functions
        let (cleaned_js, removed) = self.remove_unused_functions(&js, &xml_refs);

        if removed.is_empty() {
            return PassResult::Ok;
        }

        // Update context
        ctx.javascript = Some(cleaned_js);

        for func in &removed {
            ctx.add_warning(format!("Removed unused function: {}", func));
        }

        if removed.len() > 5 {
            PassResult::Warning(format!(
                "Removed {} unused functions - significant over-engineering detected",
                removed.len()
            ))
        } else {
            PassResult::Ok
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ScreenType, UiIntent};
    use crate::services::pipeline::ExecutionMode;

    fn create_context(xml: &str, js: &str, mode: ExecutionMode) -> GenerationContext {
        let intent = UiIntent::new("test", ScreenType::List);
        let mut ctx = GenerationContext::new("".to_string(), intent, mode);
        ctx.xml = Some(xml.to_string());
        ctx.javascript = Some(js.to_string());
        ctx
    }

    #[test]
    fn test_extract_xml_references() {
        let xml = r#"
            <pushbutton on_click="eventfunc:fn_search()"/>
            <pushbutton on_click="eventfunc:fn_save()"/>
        "#;

        let pass = MinimalismPass::new();
        let refs = pass.extract_xml_references(xml);

        assert!(refs.contains("fn_search"));
        assert!(refs.contains("fn_save"));
    }

    #[test]
    fn test_preserves_used_functions() {
        let xml = r#"<pushbutton on_click="eventfunc:fn_search()"/>"#;
        let js = r#"
this.fn_search = function() {
    console.log('search');
};
"#;

        let mut ctx = create_context(xml, js, ExecutionMode::Relaxed);
        MinimalismPass::new().run(&mut ctx);

        let result = ctx.javascript.unwrap();
        assert!(result.contains("fn_search"));
    }

    #[test]
    fn test_preserves_lifecycle_functions() {
        let xml = r#"<screen id="test"/>"#;
        let js = r#"
this.on_load = function() {
    console.log('loaded');
};

this.fn_init = function() {
    console.log('init');
};
"#;

        let mut ctx = create_context(xml, js, ExecutionMode::Relaxed);
        MinimalismPass::new().run(&mut ctx);

        let result = ctx.javascript.unwrap();
        assert!(result.contains("on_load"));
        assert!(result.contains("fn_init"));
    }

    #[test]
    fn test_dev_mode_preserves_all() {
        let xml = r#"<screen id="test"/>"#;
        let js = r#"
this.fn_unused = function() {
    console.log('unused');
};
"#;

        let mut ctx = create_context(xml, js, ExecutionMode::Dev);
        MinimalismPass::new().run(&mut ctx);

        let result = ctx.javascript.unwrap();
        assert!(result.contains("fn_unused"));
    }
}
