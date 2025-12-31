//! Pass 2: Canonicalizer
//!
//! Normalizes framework-specific naming differences.
//! This is HIGH PRIORITY based on benchmark findings.
//!
//! Fixes applied:
//! - Event attribute: onclick → on_click
//! - Font names: typo corrections
//! - Dataset type: <xdataset> → <xlinkdataset> (with columns attr)
//! - Grid version: adds version="1.1" if missing
//! - Event handler: ensures eventfunc: prefix

use crate::services::pipeline::{GenerationContext, Pass, PassResult};
use regex::Regex;

/// Canonicalizer - normalizes xFrame5 syntax issues from LLM output
pub struct Canonicalizer {
    /// XML attribute replacements (wrong → correct)
    attr_replacements: Vec<(&'static str, &'static str)>,
    /// Font name fixes
    font_fixes: Vec<(&'static str, &'static str)>,
}

impl Canonicalizer {
    pub fn new() -> Self {
        Self {
            attr_replacements: vec![
                // Event attribute fixes (HTML → xFrame5)
                ("onclick=", "on_click="),
                ("ondblclick=", "on_dblclick="),
                ("onchange=", "on_change="),
                ("onfocus=", "on_focus="),
                ("onblur=", "on_blur="),
                ("onload=", "on_load="),
                ("onLoad=", "on_load="),
                ("onitemclick=", "on_itemclick="),
                ("onitemdblclick=", "on_itemdblclick="),
                ("onrowclick=", "on_rowclick="),
                ("onrowdblclick=", "on_rowdblclick="),
            ],
            font_fixes: vec![
                // Known font name typos from LLM output
                ("맑은 고딭", "맑은 고딕"),
                ("맑은고딭", "맑은 고딕"),
            ],
        }
    }

    /// Ensure eventfunc: prefix on event handler values
    fn ensure_eventfunc_prefix(&self, xml: &str) -> String {
        // Match on_click="fn_xxx" or on_click="fn_xxx()" without eventfunc: prefix
        let re = Regex::new(r#"(on_\w+)="(fn_\w+)(\([^)]*\))?""#).unwrap();

        re.replace_all(xml, |caps: &regex::Captures| {
            let attr = &caps[1];
            let func = &caps[2];
            let args = caps.get(3).map(|m| m.as_str()).unwrap_or("()");

            format!("{}=\"eventfunc:{}{}\"", attr, func, args)
        })
        .to_string()
    }

    /// Fix missing function call parentheses in event handlers
    fn fix_event_handler_parens(&self, xml: &str) -> String {
        // Match eventfunc:fn_xxx" without parentheses
        let re = Regex::new(r#"eventfunc:(fn_\w+)""#).unwrap();

        re.replace_all(xml, |caps: &regex::Captures| {
            let func = &caps[1];
            format!("eventfunc:{}()\"", func)
        })
        .to_string()
    }

    /// Convert <xdataset> to <xlinkdataset> for proper xFrame5 syntax
    /// Note: Only converts if not already xlinkdataset
    fn fix_dataset_type(&self, xml: &str) -> (String, Vec<String>) {
        let mut result = xml.to_string();
        let mut fixes = Vec::new();

        // Count occurrences of <xdataset (not xlinkdataset)
        // Match <xdataset but not <xlinkdataset
        let re = Regex::new(r"<xdataset(\s)").unwrap();
        let count = re.find_iter(&result).count();

        if count > 0 {
            // Replace opening tags
            result = re.replace_all(&result, "<xlinkdataset$1").to_string();
            // Replace closing tags
            result = result.replace("</xdataset>", "</xlinkdataset>");

            fixes.push(format!(
                "Converted {} <xdataset> element(s) to <xlinkdataset>",
                count
            ));
        }

        (result, fixes)
    }

    /// Add version="1.1" to grid elements if missing
    fn fix_grid_version(&self, xml: &str) -> (String, Vec<String>) {
        let mut result = xml.to_string();
        let mut fixes = Vec::new();

        // Match <grid ... > without version attribute
        // Use regex to find grid tags without version
        let grid_re = Regex::new(r#"<grid\s+([^>]*?)(/?>)"#).unwrap();

        let fixed = grid_re.replace_all(&result, |caps: &regex::Captures| {
            let attrs = &caps[1];
            let closing = &caps[2];

            // Check if version already exists
            if attrs.contains("version=") {
                format!("<grid {}{}", attrs, closing)
            } else {
                // Add version before closing
                let trimmed = attrs.trim_end();
                format!("<grid {} version=\"1.1\"{}", trimmed, closing)
            }
        });

        if fixed != result {
            let count = grid_re.find_iter(&result)
                .filter(|m| !m.as_str().contains("version="))
                .count();
            if count > 0 {
                fixes.push(format!("Added version=\"1.1\" to {} grid element(s)", count));
            }
            result = fixed.to_string();
        }

        (result, fixes)
    }

    /// Normalize XML content
    fn canonicalize_xml(&self, xml: &str) -> (String, Vec<String>) {
        let mut result = xml.to_string();
        let mut fixes = Vec::new();

        // Apply attribute replacements
        for (wrong, correct) in &self.attr_replacements {
            if result.contains(*wrong) {
                let count = result.matches(*wrong).count();
                result = result.replace(*wrong, *correct);
                fixes.push(format!(
                    "Fixed {} occurrence(s) of '{}' → '{}'",
                    count, wrong, correct
                ));
            }
        }

        // Apply font fixes
        for (wrong, correct) in &self.font_fixes {
            if result.contains(*wrong) {
                let count = result.matches(*wrong).count();
                result = result.replace(*wrong, *correct);
                fixes.push(format!(
                    "Fixed {} font name(s): '{}' → '{}'",
                    count, wrong, correct
                ));
            }
        }

        // Ensure eventfunc: prefix
        let before_eventfunc = result.clone();
        result = self.ensure_eventfunc_prefix(&result);
        if result != before_eventfunc {
            fixes.push("Added missing 'eventfunc:' prefix to event handlers".to_string());
        }

        // Fix missing parentheses
        let before_parens = result.clone();
        result = self.fix_event_handler_parens(&result);
        if result != before_parens {
            fixes.push("Added missing '()' to function calls in event handlers".to_string());
        }

        // Fix dataset type (<xdataset> → <xlinkdataset>)
        let (fixed_ds, ds_fixes) = self.fix_dataset_type(&result);
        result = fixed_ds;
        fixes.extend(ds_fixes);

        // Fix grid version (add version="1.1" if missing)
        let (fixed_grid, grid_fixes) = self.fix_grid_version(&result);
        result = fixed_grid;
        fixes.extend(grid_fixes);

        (result, fixes)
    }

    /// Normalize JavaScript content
    fn canonicalize_js(&self, js: &str) -> (String, Vec<String>) {
        let mut result = js.to_string();
        let mut fixes = Vec::new();

        // Convert function declarations to xFrame5 method style
        // function fn_xxx(...) { → this.fn_xxx = function(...) {
        let (normalized, count) = self.normalize_function_style(&result);
        if count > 0 {
            result = normalized;
            fixes.push(format!(
                "Converted {} function(s) to xFrame5 method style (this.fn_xxx = function())",
                count
            ));
        }

        // Add on_load handler if missing but fn_init exists
        let (with_onload, added_onload) = self.ensure_on_load_handler(&result);
        if added_onload {
            result = with_onload;
            fixes.push("Added on_load handler calling fn_init and fn_search".to_string());
        }

        (result, fixes)
    }

    /// Convert `function fn_xxx(...)` to `this.fn_xxx = function(...)`
    fn normalize_function_style(&self, js: &str) -> (String, usize) {
        // Match: function fn_xxx(...) { or function fn_xxx(...){
        let re = Regex::new(r"(?m)^function\s+(fn_\w+|on_\w+|grid_\w+)\s*\(([^)]*)\)\s*\{").unwrap();

        let mut count = 0;
        let result = re.replace_all(js, |caps: &regex::Captures| {
            count += 1;
            let func_name = &caps[1];
            let params = &caps[2];
            format!("this.{} = function({}) {{", func_name, params)
        });

        (result.to_string(), count)
    }

    /// Ensure on_load handler exists
    fn ensure_on_load_handler(&self, js: &str) -> (String, bool) {
        // Check if on_load already exists
        if js.contains("this.on_load") || js.contains("function on_load") {
            return (js.to_string(), false);
        }

        // Check if there are any functions that should be called on load
        let has_fn_init = js.contains("fn_init");
        let has_fn_search = js.contains("fn_search");

        if !has_fn_init && !has_fn_search {
            return (js.to_string(), false);
        }

        // Build on_load handler
        let mut on_load_body = Vec::new();
        if has_fn_init {
            on_load_body.push("    fn_init();");
        }
        if has_fn_search {
            on_load_body.push("    fn_search();");
        }

        let on_load_handler = format!(
            r#"
// Screen initialization
this.on_load = function() {{
{}
}};
"#,
            on_load_body.join("\n")
        );

        // Prepend to the JS
        let result = format!("{}{}", on_load_handler, js);
        (result, true)
    }
}

impl Default for Canonicalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for Canonicalizer {
    fn name(&self) -> &'static str {
        "Canonicalizer"
    }

    fn run(&self, ctx: &mut GenerationContext) -> PassResult {
        // Process XML
        if let Some(ref xml) = ctx.xml {
            let (normalized_xml, xml_fixes) = self.canonicalize_xml(xml);
            ctx.xml = Some(normalized_xml);

            for fix in xml_fixes {
                ctx.add_warning(fix);
            }
        }

        // Process JavaScript
        if let Some(ref js) = ctx.javascript {
            let (normalized_js, js_fixes) = self.canonicalize_js(js);
            ctx.javascript = Some(normalized_js);

            for fix in js_fixes {
                ctx.add_warning(fix);
            }
        }

        PassResult::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ScreenType, UiIntent};
    use crate::services::pipeline::ExecutionMode;

    fn create_context_with_xml(xml: &str, js: &str) -> GenerationContext {
        let intent = UiIntent::new("test", ScreenType::List);
        let mut ctx = GenerationContext::new("".to_string(), intent, ExecutionMode::Relaxed);
        ctx.xml = Some(xml.to_string());
        ctx.javascript = Some(js.to_string());
        ctx
    }

    #[test]
    fn test_onclick_to_on_click() {
        let xml = r#"<pushbutton onclick="eventfunc:fn_search()"/>"#;
        let mut ctx = create_context_with_xml(xml, "");

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.xml.unwrap();
        assert!(result.contains("on_click="));
        assert!(!result.contains("onclick="));
    }

    #[test]
    fn test_add_eventfunc_prefix() {
        let xml = r#"<pushbutton on_click="fn_search()"/>"#;
        let mut ctx = create_context_with_xml(xml, "");

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.xml.unwrap();
        assert!(result.contains("eventfunc:fn_search"));
    }

    #[test]
    fn test_fix_font_name() {
        let xml = r#"<text font="맑은 고딭,9,0,0,0,0"/>"#;
        let mut ctx = create_context_with_xml(xml, "");

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.xml.unwrap();
        assert!(result.contains("맑은 고딕"));
        assert!(!result.contains("고딭"));
    }

    #[test]
    fn test_fix_missing_parens() {
        let xml = r#"<pushbutton on_click="eventfunc:fn_search"/>"#;
        let mut ctx = create_context_with_xml(xml, "");

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.xml.unwrap();
        assert!(result.contains("fn_search()"));
    }

    #[test]
    fn test_combined_fixes() {
        // Test the full chain of fixes as seen in benchmark models
        let xml = r#"<pushbutton onclick="fn_search" font="맑은 고딭,9"/>"#;
        let mut ctx = create_context_with_xml(xml, "");

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.xml.unwrap();
        assert!(result.contains("on_click="));
        assert!(result.contains("eventfunc:fn_search()"));
        assert!(result.contains("맑은 고딕"));
    }

    #[test]
    fn test_preserves_correct_syntax() {
        // Already correct syntax should be preserved
        let xml = r#"<pushbutton on_click="eventfunc:fn_search()"/>"#;
        let mut ctx = create_context_with_xml(xml, "");

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.xml.unwrap();
        assert_eq!(result, xml);
    }

    #[test]
    fn test_xdataset_to_xlinkdataset() {
        let xml = r#"<xdataset id="ds_list" desc="Task List"/>"#;
        let mut ctx = create_context_with_xml(xml, "");

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.xml.unwrap();
        assert!(result.contains("<xlinkdataset"));
        assert!(!result.contains("<xdataset"));
    }

    #[test]
    fn test_xdataset_preserves_xlinkdataset() {
        let xml = r#"<xlinkdataset id="ds_list" columns="..."/>"#;
        let mut ctx = create_context_with_xml(xml, "");

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.xml.unwrap();
        assert!(result.contains("<xlinkdataset"));
        assert_eq!(result.matches("<xlinkdataset").count(), 1);
    }

    #[test]
    fn test_grid_version_injection() {
        let xml = r#"<grid control_id="1" name="grid_list" link_data="ds_list">"#;
        let mut ctx = create_context_with_xml(xml, "");

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.xml.unwrap();
        assert!(result.contains(r#"version="1.1""#));
    }

    #[test]
    fn test_grid_version_preserves_existing() {
        let xml = r#"<grid control_id="1" name="grid_list" version="1.0">"#;
        let mut ctx = create_context_with_xml(xml, "");

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.xml.unwrap();
        assert!(result.contains(r#"version="1.0""#));
        assert!(!result.contains(r#"version="1.1""#));
    }

    #[test]
    fn test_function_style_normalization() {
        let js = r#"function fn_search(objInst) {
    console.log('search');
}"#;
        let mut ctx = create_context_with_xml("", js);

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.javascript.unwrap();
        assert!(result.contains("this.fn_search = function(objInst)"));
        assert!(!result.contains("function fn_search"));
    }

    #[test]
    fn test_multiple_function_normalization() {
        let js = r#"function fn_search() {}
function fn_add() {}
function fn_delete() {}"#;
        let mut ctx = create_context_with_xml("", js);

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.javascript.unwrap();
        assert!(result.contains("this.fn_search = function()"));
        assert!(result.contains("this.fn_add = function()"));
        assert!(result.contains("this.fn_delete = function()"));
    }

    #[test]
    fn test_on_load_handler_injection() {
        let js = r#"this.fn_search = function() {
    console.log('search');
};"#;
        let mut ctx = create_context_with_xml("", js);

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.javascript.unwrap();
        assert!(result.contains("this.on_load = function()"));
        assert!(result.contains("fn_search();"));
    }

    #[test]
    fn test_on_load_not_added_if_exists() {
        let js = r#"this.on_load = function() { fn_init(); };
this.fn_search = function() {};"#;
        let mut ctx = create_context_with_xml("", js);

        Canonicalizer::new().run(&mut ctx);

        let result = ctx.javascript.unwrap();
        // Should only have one on_load
        assert_eq!(result.matches("this.on_load").count(), 1);
    }
}
