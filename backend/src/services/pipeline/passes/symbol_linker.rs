//! Pass 2: Symbol Linker
//!
//! Ensures XML event handlers match JavaScript functions.
//! Generates stubs for missing functions in non-strict modes.

use crate::services::pipeline::{GenerationContext, Pass, PassResult};
use regex::Regex;
use std::collections::HashSet;

/// Symbol Linker - matches XML events to JS functions
pub struct SymbolLinker;

impl SymbolLinker {
    pub fn new() -> Self {
        Self
    }

    /// Extract function names referenced in XML event handlers
    fn extract_xml_handlers(&self, xml: &str) -> HashSet<String> {
        let mut handlers = HashSet::new();

        // Match eventfunc:fn_xxx patterns
        let re = Regex::new(r#"eventfunc:(fn_\w+)"#).unwrap();
        for cap in re.captures_iter(xml) {
            handlers.insert(cap[1].to_string());
        }

        // Also match grid_xxx_on_xxx patterns (grid event handlers)
        let grid_re = Regex::new(r#"eventfunc:(\w+_on_\w+)"#).unwrap();
        for cap in grid_re.captures_iter(xml) {
            handlers.insert(cap[1].to_string());
        }

        handlers
    }

    /// Extract function names defined in JavaScript
    fn extract_js_functions(&self, js: &str) -> HashSet<String> {
        let mut functions = HashSet::new();

        // Match this.fn_xxx = function patterns
        let this_re = Regex::new(r#"this\.(\w+)\s*=\s*function"#).unwrap();
        for cap in this_re.captures_iter(js) {
            functions.insert(cap[1].to_string());
        }

        // Match function fn_xxx patterns
        let func_re = Regex::new(r#"function\s+(\w+)\s*\("#).unwrap();
        for cap in func_re.captures_iter(js) {
            functions.insert(cap[1].to_string());
        }

        // Match const/let/var fn_xxx = function patterns
        let var_re = Regex::new(r#"(?:const|let|var)\s+(\w+)\s*=\s*function"#).unwrap();
        for cap in var_re.captures_iter(js) {
            functions.insert(cap[1].to_string());
        }

        functions
    }

    /// Generate a stub function for missing handler
    fn generate_stub(&self, func_name: &str) -> String {
        // Determine if it's a grid handler with parameters
        let (params, comment) = if func_name.contains("_on_itemdblclick")
            || func_name.contains("_on_rowdblclick")
        {
            (
                "objInst, nRow, nColumn, buttonClick, imageIndex",
                "grid item double-click",
            )
        } else if func_name.contains("_on_itemclick") || func_name.contains("_on_rowclick") {
            ("objInst, nRow, nColumn", "grid item click")
        } else if func_name.contains("_on_") {
            ("objInst, e", "event handler")
        } else {
            ("", "action handler")
        };

        format!(
            r#"
// TODO: Implement {} ({})
this.{} = function({}) {{
    // TODO: Implement functionality
    console.log('{}');
}};"#,
            func_name, comment, func_name, params, func_name
        )
    }
}

impl Default for SymbolLinker {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for SymbolLinker {
    fn name(&self) -> &'static str {
        "SymbolLinker"
    }

    fn run(&self, ctx: &mut GenerationContext) -> PassResult {
        let xml = match &ctx.xml {
            Some(xml) => xml.clone(),
            None => return PassResult::Error("XML not available".to_string()),
        };

        let js = match &ctx.javascript {
            Some(js) => js.clone(),
            None => return PassResult::Error("JavaScript not available".to_string()),
        };

        // Extract handlers and functions
        let xml_handlers = self.extract_xml_handlers(&xml);
        let js_functions = self.extract_js_functions(&js);

        // Find missing functions
        let missing: Vec<_> = xml_handlers
            .difference(&js_functions)
            .cloned()
            .collect();

        if missing.is_empty() {
            return PassResult::Ok;
        }

        // Handle based on execution mode
        if ctx.is_strict() {
            return PassResult::Error(format!(
                "Missing JavaScript functions for XML handlers: {}",
                missing.join(", ")
            ));
        }

        // Generate stubs for missing functions
        let mut updated_js = js;
        for func_name in &missing {
            let stub = self.generate_stub(func_name);
            updated_js.push_str(&stub);
            ctx.add_warning(format!("Generated stub for missing function: {}", func_name));
        }

        ctx.javascript = Some(updated_js);

        if missing.len() > 3 {
            PassResult::Warning(format!(
                "Generated {} stub functions - review carefully",
                missing.len()
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
    fn test_extract_handlers() {
        let xml = r#"
            <pushbutton on_click="eventfunc:fn_search()"/>
            <pushbutton on_click="eventfunc:fn_save()"/>
            <grid on_itemdblclick="eventfunc:grid_list_on_itemdblclick()"/>
        "#;

        let linker = SymbolLinker::new();
        let handlers = linker.extract_xml_handlers(xml);

        assert!(handlers.contains("fn_search"));
        assert!(handlers.contains("fn_save"));
        assert!(handlers.contains("grid_list_on_itemdblclick"));
    }

    #[test]
    fn test_extract_js_functions() {
        let js = r#"
            this.fn_search = function() {};
            function fn_save() {}
            const fn_delete = function() {};
        "#;

        let linker = SymbolLinker::new();
        let functions = linker.extract_js_functions(js);

        assert!(functions.contains("fn_search"));
        assert!(functions.contains("fn_save"));
        assert!(functions.contains("fn_delete"));
    }

    #[test]
    fn test_generates_stubs_in_relaxed_mode() {
        let xml = r#"<pushbutton on_click="eventfunc:fn_missing()"/>"#;
        let js = "this.fn_existing = function() {};";

        let mut ctx = create_context(xml, js, ExecutionMode::Relaxed);
        let result = SymbolLinker::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Ok));
        assert!(ctx.javascript.unwrap().contains("fn_missing"));
    }

    #[test]
    fn test_error_in_strict_mode() {
        let xml = r#"<pushbutton on_click="eventfunc:fn_missing()"/>"#;
        let js = "this.fn_existing = function() {};";

        let mut ctx = create_context(xml, js, ExecutionMode::Strict);
        let result = SymbolLinker::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Error(_)));
    }

    #[test]
    fn test_no_action_when_all_present() {
        let xml = r#"<pushbutton on_click="eventfunc:fn_search()"/>"#;
        let js = "this.fn_search = function() {};";

        let mut ctx = create_context(xml, js, ExecutionMode::Relaxed);
        let original_js = ctx.javascript.clone();
        let result = SymbolLinker::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Ok));
        assert_eq!(ctx.javascript, original_js);
    }

    #[test]
    fn test_grid_handler_stub_has_params() {
        let xml = r#"<grid on_itemdblclick="eventfunc:grid_list_on_itemdblclick()"/>"#;
        let js = "";

        let mut ctx = create_context(xml, js, ExecutionMode::Relaxed);
        SymbolLinker::new().run(&mut ctx);

        let result_js = ctx.javascript.unwrap();
        assert!(result_js.contains("objInst, nRow, nColumn"));
    }
}
