//! Pass 0: Output Parser
//!
//! Splits raw LLM output into XML and JavaScript sections.

use crate::services::pipeline::{GenerationContext, Pass, PassResult};

/// Output Parser - splits raw LLM output into XML and JS sections
pub struct OutputParser;

impl OutputParser {
    pub fn new() -> Self {
        Self
    }

    /// Find a section marker in the text
    fn find_section_marker(text: &str, markers: &[&str]) -> Option<(usize, usize)> {
        for marker in markers {
            if let Some(pos) = text.find(marker) {
                return Some((pos, marker.len()));
            }
        }
        None
    }

    /// Clean a section by removing markdown code blocks and trimming
    fn clean_section(text: &str) -> String {
        let mut result = text.trim().to_string();

        // Remove leading markdown code block markers
        result = result
            .trim_start_matches("```xml")
            .trim_start_matches("```javascript")
            .trim_start_matches("```js")
            .trim_start_matches("```")
            .trim()
            .to_string();

        // Remove trailing markdown code block
        if let Some(backtick_pos) = result.rfind("```") {
            let before_backtick = result[..backtick_pos].trim();
            if !before_backtick.is_empty() {
                result = before_backtick.to_string();
            }
        }

        // For XML: ensure we end at </screen> if present
        if result.contains("</screen>") {
            if let Some(screen_end_pos) = result.rfind("</screen>") {
                result = result[..screen_end_pos + "</screen>".len()].to_string();
            }
        }

        // Remove trailing explanation text
        let explanation_markers = [
            "\n\nNote that",
            "\n\nPlease note",
            "\n\nThis code",
            "\n\nAlso,",
            "\n\nI've ",
            "\n\n**",
            "\n\nThe above",
        ];
        for marker in explanation_markers {
            if let Some(pos) = result.find(marker) {
                result = result[..pos].trim().to_string();
            }
        }

        // Remove malformed artifacts from LLM output
        result = result
            .replace("]]>", "")
            .replace("</script>", "")
            .replace("<script>", "");

        result.trim().to_string()
    }

    /// Try to split content by detecting XML and JS patterns
    fn split_by_content(raw: &str) -> Option<(String, String)> {
        // Look for XML start
        let xml_start = raw.find('<')?;

        // Find where XML ends (look for closing tag followed by JS patterns)
        let js_patterns = ["this.", "function ", "var ", "let ", "const ", "//"];

        let mut xml_end = raw.len();
        let mut js_start = raw.len();

        for pattern in js_patterns {
            if let Some(pos) = raw[xml_start..].find(pattern) {
                let abs_pos = xml_start + pos;
                if let Some(last_bracket) = raw[..abs_pos].rfind('>') {
                    if last_bracket + 1 < abs_pos && abs_pos < js_start {
                        xml_end = last_bracket + 1;
                        js_start = abs_pos;
                    }
                }
            }
        }

        if js_start >= raw.len() {
            return None;
        }

        let xml = raw[xml_start..xml_end].trim().to_string();
        let js = raw[js_start..].trim().to_string();

        if xml.is_empty() || js.is_empty() {
            return None;
        }

        Some((xml, js))
    }
}

impl Default for OutputParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for OutputParser {
    fn name(&self) -> &'static str {
        "OutputParser"
    }

    fn run(&self, ctx: &mut GenerationContext) -> PassResult {
        let raw = &ctx.raw_output;

        // Try marker-based splitting first
        let xml_markers = [
            "--- XML ---",
            "---XML---",
            "<!-- XML -->",
            "```xml",
            "**XML:**",
            "**XML**",
            "## XML",
            "# XML",
        ];
        let js_markers = [
            "--- JS ---",
            "---JS---",
            "// JS",
            "```javascript",
            "```js",
            "**JavaScript:**",
            "**JavaScript**",
            "**JS:**",
            "**JS**",
            "## JavaScript",
            "# JavaScript",
            "## JS",
            "# JS",
        ];

        let xml_marker = Self::find_section_marker(raw, &xml_markers);
        let js_marker = Self::find_section_marker(raw, &js_markers);

        let (xml, js) = match (xml_marker, js_marker) {
            (Some((xml_start, xml_marker_len)), Some((js_start, js_marker_len))) => {
                let xml_content_start = xml_start + xml_marker_len;
                let xml_content_end = js_start;
                let js_content_start = js_start + js_marker_len;

                let xml = Self::clean_section(&raw[xml_content_start..xml_content_end]);
                let js = Self::clean_section(&raw[js_content_start..]);

                if xml.is_empty() {
                    return PassResult::Error("XML section is empty".to_string());
                }

                // If JS is empty after marker-based split, try content-based fallback
                if js.is_empty() {
                    match Self::split_by_content(raw) {
                        Some((_, js_fallback)) => (xml, js_fallback),
                        None => {
                            return PassResult::Error(
                                "JavaScript section is empty".to_string(),
                            );
                        }
                    }
                } else {
                    (xml, js)
                }
            }
            _ => {
                // No markers found, try content-based splitting
                match Self::split_by_content(raw) {
                    Some((xml, js)) => (Self::clean_section(&xml), Self::clean_section(&js)),
                    None => {
                        return PassResult::Error(
                            "Could not separate XML and JavaScript sections".to_string(),
                        );
                    }
                }
            }
        };

        // Store in context
        ctx.xml = Some(xml);
        ctx.javascript = Some(js);

        PassResult::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ScreenType, UiIntent};
    use crate::services::pipeline::ExecutionMode;

    fn create_context(raw: &str) -> GenerationContext {
        let intent = UiIntent::new("test", ScreenType::List);
        GenerationContext::new(raw.to_string(), intent, ExecutionMode::Relaxed)
    }

    #[test]
    fn test_split_with_markers() {
        let raw = r#"
--- XML ---
<screen id="test">
  <dataset id="ds_list"/>
</screen>

--- JS ---
this.fn_search = function() {};
"#;

        let mut ctx = create_context(raw);
        let result = OutputParser::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Ok));
        assert!(ctx.xml.unwrap().contains("<screen"));
        assert!(ctx.javascript.unwrap().contains("fn_search"));
    }

    #[test]
    fn test_split_without_markers() {
        let raw = r#"
<screen id="test">
  <dataset id="ds_list"/>
</screen>

this.fn_search = function() {};
"#;

        let mut ctx = create_context(raw);
        let result = OutputParser::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Ok));
        assert!(ctx.xml.is_some());
        assert!(ctx.javascript.is_some());
    }

    #[test]
    fn test_clean_malformed_artifacts() {
        let raw = r#"
--- XML ---
<screen id="test"/>

--- JS ---
this.fn_test = function() {};
]]>
</script>
"#;

        let mut ctx = create_context(raw);
        let result = OutputParser::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Ok));
        let js = ctx.javascript.unwrap();
        assert!(!js.contains("]]>"));
        assert!(!js.contains("</script>"));
    }

    #[test]
    fn test_no_xml_error() {
        let raw = "just some random text";
        let mut ctx = create_context(raw);
        let result = OutputParser::new().run(&mut ctx);

        assert!(matches!(result, PassResult::Error(_)));
    }
}
