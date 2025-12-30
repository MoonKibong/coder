//! Pass 3: API Allowlist Filter
//!
//! Blocks hallucinated or non-existent xFrame5 APIs.
//! Uses database-stored allowlist (falls back to hardcoded for now).

use crate::services::pipeline::{GenerationContext, Pass, PassResult};
use regex::Regex;
use std::collections::HashSet;

/// API Allowlist Filter - blocks hallucinated APIs
pub struct ApiAllowlistFilter {
    /// Allowed xFrame5 API patterns
    allowed_apis: HashSet<&'static str>,
}

impl ApiAllowlistFilter {
    pub fn new() -> Self {
        // Hardcoded allowlist - will be replaced with DB lookup
        let allowed_apis: HashSet<&'static str> = [
            // Dataset APIs
            "getRowCount",
            "getColumn",
            "setColumn",
            "getRowType",
            "setRowType",
            "addRow",
            "deleteRow",
            "clearData",
            "getSelectedIndex",
            "setSelectedIndex",
            "getItemText",
            "setItemText",
            "filter",
            "sort",
            "copyRow",
            "moveRow",
            "getData",
            "setData",
            "getMaxRow",
            "insertRow",
            // Grid APIs
            "getSelectedRow",
            "setSelectedRow",
            "getCellValue",
            "setCellValue",
            "refresh",
            "getCheckedRows",
            "setCheckedRow",
            "checkAll",
            "uncheckAll",
            // Popup/Dialog APIs
            "loadpopup",
            "closepopup",
            "alert",
            "confirm",
            "getPopupData",
            "setPopupData",
            // Transaction APIs
            "transaction",
            "submit",
            "save",
            "search",
            // Component APIs
            "setValue",
            "getValue",
            "setEnabled",
            "setVisible",
            "setReadOnly",
            "focus",
            "blur",
            // Utility APIs
            "console.log",
            "console.error",
            "console.warn",
            "JSON.parse",
            "JSON.stringify",
            // Common patterns that are allowed
            "function",
            "this.",
            "var ",
            "let ",
            "const ",
            "if ",
            "else ",
            "for ",
            "while ",
            "return ",
            "new ",
        ]
        .into_iter()
        .collect();

        Self { allowed_apis }
    }

    /// Check if an API call is in the allowlist
    fn is_allowed(&self, api: &str) -> bool {
        // Check exact match
        if self.allowed_apis.contains(api) {
            return true;
        }

        // For method calls like "ds_list.getRowCount", check if the method name is allowed
        if let Some(method_name) = api.split('.').last() {
            if self.allowed_apis.contains(method_name) {
                return true;
            }
        }

        // Check if it's a common pattern
        for pattern in &self.allowed_apis {
            if api.contains(pattern) {
                return true;
            }
        }

        // Allow standard JS functions
        let standard_patterns = [
            "function ", "this.", "var ", "let ", "const ",
            "if ", "else ", "for ", "while ", "return ",
            "console.", "JSON.", "Math.", "Date.",
            ".length", ".push", ".pop", ".map", ".filter",
            ".forEach", ".indexOf", ".substring", ".split",
            ".trim", ".toLowerCase", ".toUpperCase",
        ];

        for pattern in standard_patterns {
            if api.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Extract API calls from JavaScript code
    fn extract_api_calls(&self, js: &str) -> Vec<String> {
        let mut calls = Vec::new();

        // Match method calls like obj.method()
        let method_re = Regex::new(r#"(\w+)\.(\w+)\s*\("#).unwrap();
        for cap in method_re.captures_iter(js) {
            calls.push(format!("{}.{}", &cap[1], &cap[2]));
        }

        // Match function calls like functionName() - NOT preceded by dot or word char
        // Use [^\w.] to match non-word, non-dot chars (like space, =, etc.)
        let func_re = Regex::new(r#"(?:^|[^\w.])(\w+)\s*\("#).unwrap();
        for cap in func_re.captures_iter(js) {
            let func = &cap[1];
            // Skip common keywords
            if !["if", "for", "while", "function", "return", "switch", "catch"].contains(&func) {
                calls.push(func.to_string());
            }
        }

        calls
    }

    /// Check JavaScript for disallowed API calls
    fn check_js(&self, js: &str) -> Vec<String> {
        let calls = self.extract_api_calls(js);
        let mut violations = Vec::new();

        for call in calls {
            if !self.is_allowed(&call) {
                // Additional check: is it a user-defined function?
                // Handle "this.fn_xxx" calls - strip "this." prefix for function definition check
                let func_name = if call.starts_with("this.") {
                    &call[5..] // Strip "this." prefix
                } else {
                    &call
                };

                // Check if this function is defined in the JS code
                let is_user_defined = js.contains(&format!("this.{} = function", func_name))
                    || js.contains(&format!("function {}", func_name))
                    || js.contains(&format!("{} = function", call)); // Also check full call pattern

                if !is_user_defined {
                    violations.push(call);
                }
            }
        }

        // Deduplicate
        violations.sort();
        violations.dedup();
        violations
    }

    /// Replace disallowed API with TODO comment
    fn replace_with_todo(&self, js: &str, api: &str) -> String {
        let pattern = format!(r#"{}(\s*\([^)]*\))"#, regex::escape(api));
        let re = Regex::new(&pattern).unwrap();

        re.replace_all(js, |caps: &regex::Captures| {
            format!("/* TODO: verify API '{}' */ {}{}", api, api, &caps[1])
        })
        .to_string()
    }
}

impl Default for ApiAllowlistFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for ApiAllowlistFilter {
    fn name(&self) -> &'static str {
        "ApiAllowlistFilter"
    }

    fn run(&self, ctx: &mut GenerationContext) -> PassResult {
        let js = match &ctx.javascript {
            Some(js) => js.clone(),
            None => return PassResult::Error("JavaScript not available".to_string()),
        };

        let violations = self.check_js(&js);

        if violations.is_empty() {
            return PassResult::Ok;
        }

        // Handle based on execution mode
        if ctx.is_strict() {
            return PassResult::Error(format!(
                "Disallowed API calls detected: {}",
                violations.join(", ")
            ));
        }

        // In relaxed/dev mode, add TODO comments
        let mut updated_js = js;
        for api in &violations {
            updated_js = self.replace_with_todo(&updated_js, api);
            ctx.add_warning(format!("Flagged potentially hallucinated API: {}", api));
        }

        ctx.javascript = Some(updated_js);

        PassResult::Warning(format!(
            "Found {} potentially invalid API call(s)",
            violations.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ScreenType, UiIntent};
    use crate::services::pipeline::ExecutionMode;

    fn create_context(js: &str, mode: ExecutionMode) -> GenerationContext {
        let intent = UiIntent::new("test", ScreenType::List);
        let mut ctx = GenerationContext::new("".to_string(), intent, mode);
        ctx.xml = Some("<screen/>".to_string());
        ctx.javascript = Some(js.to_string());
        ctx
    }

    #[test]
    fn test_allowed_apis_pass() {
        let js = r#"
            this.fn_search = function() {
                var count = ds_list.getRowCount();
                console.log(count);
            };
        "#;

        let mut ctx = create_context(js, ExecutionMode::Strict);
        let result = ApiAllowlistFilter::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Ok));
    }

    #[test]
    fn test_user_defined_functions_allowed() {
        let js = r#"
            this.fn_custom = function() {};
            this.fn_other = function() {
                this.fn_custom();
            };
        "#;

        let mut ctx = create_context(js, ExecutionMode::Strict);
        let result = ApiAllowlistFilter::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Ok));
    }

    #[test]
    fn test_hallucinated_api_flagged() {
        let js = r#"
            this.fn_test = function() {
                someHallucinatedApi.doThing();
            };
        "#;

        let mut ctx = create_context(js, ExecutionMode::Relaxed);
        let result = ApiAllowlistFilter::new().run(&mut ctx);

        // Should add TODO comment in relaxed mode
        assert!(matches!(result, PassResult::Warning(_)));
        assert!(ctx.javascript.unwrap().contains("TODO"));
    }

    #[test]
    fn test_strict_mode_error() {
        let js = r#"
            this.fn_test = function() {
                fakeApi.fake();
            };
        "#;

        let mut ctx = create_context(js, ExecutionMode::Strict);
        let result = ApiAllowlistFilter::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Error(_)));
    }
}
