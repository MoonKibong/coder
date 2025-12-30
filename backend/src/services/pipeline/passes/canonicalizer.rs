//! Pass 1: Canonicalizer
//!
//! Normalizes framework-specific naming differences.
//! This is HIGH PRIORITY based on benchmark findings.

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

        (result, fixes)
    }

    /// Normalize JavaScript content
    fn canonicalize_js(&self, js: &str) -> (String, Vec<String>) {
        let result = js.to_string();
        let fixes = Vec::new();

        // Remove duplicate function definitions (keep first)
        // This is a simple check - could be enhanced with AST parsing

        // Fix common JS patterns
        // (placeholder for future enhancements)

        (result, fixes)
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
}
