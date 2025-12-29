use crate::domain::{GeneratedArtifacts, UiIntent};
use anyhow::{anyhow, Result};
use regex::Regex;

/// Validated artifacts after parsing and checking
#[derive(Debug, Clone)]
pub struct ValidatedArtifacts {
    /// Validated XML content
    pub xml: String,

    /// Validated JavaScript content
    pub javascript: String,

    /// Warnings found during validation
    pub warnings: Vec<String>,

    /// Screen name for filename generation
    pub screen_name: Option<String>,
}

impl ValidatedArtifacts {
    /// Convert to GeneratedArtifacts with filenames based on screen_name
    pub fn into_artifacts(self) -> GeneratedArtifacts {
        let (xml_filename, js_filename) = if let Some(ref name) = self.screen_name {
            // Convert screen_name to snake_case filename
            let base_name = name
                .to_lowercase()
                .replace(' ', "_")
                .replace('-', "_");
            (
                Some(format!("{}.xml", base_name)),
                Some(format!("{}.js", base_name)),
            )
        } else {
            (None, None)
        };

        GeneratedArtifacts {
            xml: Some(self.xml),
            javascript: Some(self.javascript),
            xml_filename,
            js_filename,
        }
    }
}

impl From<ValidatedArtifacts> for GeneratedArtifacts {
    fn from(v: ValidatedArtifacts) -> Self {
        v.into_artifacts()
    }
}

/// Service for validating xFrame5 output (XML and JavaScript)
pub struct XFrame5Validator;

impl XFrame5Validator {
    /// Parse and validate LLM output
    pub fn parse_and_validate(raw: &str, intent: &UiIntent) -> Result<ValidatedArtifacts> {
        // 1. Split XML and JS sections
        let (xml, js) = Self::split_output(raw)?;

        // 2. Validate XML structure
        let xml_warnings = Self::validate_xml(&xml)?;

        // 3. Validate JS functions
        let js_warnings = Self::validate_js(&js, intent)?;

        // 4. Combine warnings
        let mut warnings = xml_warnings;
        warnings.extend(js_warnings);

        Ok(ValidatedArtifacts {
            xml,
            javascript: js,
            warnings,
            screen_name: Some(intent.screen_name.clone()),
        })
    }

    /// Split LLM output into XML and JS sections
    fn split_output(raw: &str) -> Result<(String, String)> {
        // Look for markers like "--- XML ---" and "--- JS ---"
        // Also handle markdown format that some LLMs produce
        let xml_marker = Self::find_section_marker(raw, &[
            "--- XML ---", "---XML---", "<!-- XML -->", "```xml",
            "**XML:**", "**XML**", "## XML", "# XML"
        ]);
        let js_marker = Self::find_section_marker(raw, &[
            "--- JS ---", "---JS---", "// JS", "```javascript", "```js",
            "**JavaScript:**", "**JavaScript**", "**JS:**", "**JS**",
            "## JavaScript", "# JavaScript", "## JS", "# JS"
        ]);

        tracing::debug!(
            "Section markers found - XML: {:?}, JS: {:?}",
            xml_marker.map(|(pos, _)| pos),
            js_marker.map(|(pos, _)| pos)
        );

        match (xml_marker, js_marker) {
            (Some((xml_start, xml_marker_len)), Some((js_start, js_marker_len))) => {
                let xml_content_start = xml_start + xml_marker_len;
                let xml_content_end = js_start;
                let js_content_start = js_start + js_marker_len;

                let xml = Self::clean_section(&raw[xml_content_start..xml_content_end]);
                let js = Self::clean_section(&raw[js_content_start..]);

                tracing::debug!(
                    "Marker-based split - XML length: {}, JS length: {}",
                    xml.len(),
                    js.len()
                );

                if xml.is_empty() {
                    return Err(anyhow!("XML section is empty"));
                }
                if js.is_empty() {
                    // JS section empty with markers - try content-based splitting as fallback
                    tracing::warn!("JS section empty after marker-based split, trying content-based fallback");
                    return Self::split_by_content(raw);
                }

                Ok((xml, js))
            }
            _ => {
                // Try to detect XML and JS without explicit markers
                tracing::debug!("No markers found, trying content-based splitting");
                Self::split_by_content(raw)
            }
        }
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
        result = result.trim_start_matches("```xml")
            .trim_start_matches("```javascript")
            .trim_start_matches("```js")
            .trim_start_matches("```")
            .trim()
            .to_string();

        // Remove trailing markdown code block and any text after it
        // Look for ``` that ends the code block
        if let Some(backtick_pos) = result.rfind("```") {
            // Check if this is at the end of actual code
            let before_backtick = result[..backtick_pos].trim();
            // If there's actual content before the backticks, truncate there
            if !before_backtick.is_empty() {
                result = before_backtick.to_string();
            }
        }

        // For XML: ensure we end at </screen> if present (remove any trailing explanation)
        if result.contains("</screen>") {
            if let Some(screen_end_pos) = result.rfind("</screen>") {
                result = result[..screen_end_pos + "</screen>".len()].to_string();
            }
        }

        // For JS: remove any trailing explanation text that starts with common patterns
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

        result.trim().to_string()
    }

    /// Try to split content by detecting XML and JS patterns
    fn split_by_content(raw: &str) -> Result<(String, String)> {
        // Look for XML start
        let xml_start = raw.find('<')
            .ok_or_else(|| anyhow!("No XML content found (no '<' character)"))?;

        // Find where XML ends (look for closing tag followed by JS patterns)
        let js_patterns = ["this.", "function ", "var ", "let ", "const ", "//"];

        let mut xml_end = raw.len();
        let mut js_start = raw.len();

        for pattern in js_patterns {
            if let Some(pos) = raw[xml_start..].find(pattern) {
                let abs_pos = xml_start + pos;
                // Find the last '>' before this JS pattern
                if let Some(last_bracket) = raw[..abs_pos].rfind('>') {
                    if last_bracket + 1 < abs_pos && abs_pos < js_start {
                        xml_end = last_bracket + 1;
                        js_start = abs_pos;
                        tracing::debug!(
                            "Found JS pattern '{}' at position {}, XML ends at {}",
                            pattern, abs_pos, xml_end
                        );
                    }
                }
            }
        }

        if js_start >= raw.len() {
            // Log what we found to help diagnose
            let raw_preview = if raw.len() > 500 {
                format!("{}...[truncated]", &raw[..500])
            } else {
                raw.to_string()
            };
            tracing::warn!(
                "Could not find JavaScript section. Raw content preview:\n{}",
                raw_preview
            );
            return Err(anyhow!(
                "Could not separate XML and JavaScript sections. The LLM may not have generated JavaScript code."
            ));
        }

        let xml = raw[xml_start..xml_end].trim().to_string();
        let js = raw[js_start..].trim().to_string();

        tracing::debug!(
            "Content-based split - XML length: {}, JS length: {}",
            xml.len(),
            js.len()
        );

        if xml.is_empty() {
            return Err(anyhow!("XML section is empty"));
        }
        if js.is_empty() {
            return Err(anyhow!("JavaScript section is empty"));
        }

        Ok((xml, js))
    }

    /// Validate XML structure
    fn validate_xml(xml: &str) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for basic XML structure
        if !xml.contains('<') || !xml.contains('>') {
            return Err(anyhow!("Invalid XML: no tags found"));
        }

        // Check for Dataset element (including xlinkdataset which is the xFrame5 format)
        if !xml.contains("<Dataset") && !xml.contains("<dataset") &&
           !xml.contains("<xlinkdataset") && !xml.contains("<Xlinkdataset") {
            warnings.push("Warning: No Dataset element found in XML".to_string());
        }

        // Check for Grid element (for list screens)
        if !xml.contains("<Grid") && !xml.contains("<grid") {
            // This might be okay for detail screens
            warnings.push("Note: No Grid element found in XML".to_string());
        }

        // Check for unclosed tags (basic check)
        let open_count = xml.matches('<').count() - xml.matches("</").count() - xml.matches("/>").count();
        let close_count = xml.matches("</").count();

        if open_count != close_count * 2 && open_count > close_count {
            warnings.push("Warning: Possible unclosed XML tags".to_string());
        }

        // Check for TODO placeholders
        let todo_count = xml.to_uppercase().matches("TODO").count();
        if todo_count > 0 {
            warnings.push(format!("Note: {} TODO placeholder(s) found in XML", todo_count));
        }

        Ok(warnings)
    }

    /// Validate JavaScript functions
    fn validate_js(js: &str, intent: &UiIntent) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for basic JS structure
        if js.is_empty() {
            return Err(anyhow!("JavaScript is empty"));
        }

        // Check for expected functions based on intent actions
        for action in &intent.actions {
            let fn_name = &action.function_name;
            if !js.contains(fn_name) {
                warnings.push(format!("Warning: Expected function '{}' not found", fn_name));
            }
        }

        // Check for common xFrame5 patterns (either this. or function declarations)
        let has_this_pattern = js.contains("this.");
        let has_function_pattern = js.contains("function ");

        if !has_this_pattern && !has_function_pattern {
            warnings.push("Warning: No function declarations found in JavaScript".to_string());
        }

        // Check for TODO placeholders
        let todo_count = js.to_uppercase().matches("TODO").count();
        if todo_count > 0 {
            warnings.push(format!("Note: {} TODO placeholder(s) found in JavaScript", todo_count));
        }

        // Check for hardcoded API endpoints (should be TODO)
        let endpoint_regex = Regex::new(r#"["']/api/[^"']*["']"#).unwrap();
        let endpoint_matches: Vec<_> = endpoint_regex.find_iter(js).collect();
        if !endpoint_matches.is_empty() {
            warnings.push(format!(
                "Note: {} API endpoint(s) found - verify they are correct",
                endpoint_matches.len()
            ));
        }

        Ok(warnings)
    }

    /// Post-process the output to fix common issues
    pub fn post_process(artifacts: &mut ValidatedArtifacts, intent: &UiIntent) {
        // Add missing function stubs
        for action in &intent.actions {
            if !artifacts.javascript.contains(&action.function_name) {
                let stub = format!(
                    "\n\n// TODO: Implement {}\nthis.{} = function() {{\n    // TODO: Implement {} functionality\n    console.log('{}');\n}};",
                    action.label,
                    action.function_name,
                    action.label,
                    action.function_name
                );
                artifacts.javascript.push_str(&stub);
                artifacts.warnings.push(format!(
                    "Added stub for missing function: {}",
                    action.function_name
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ActionIntent, ActionType, ScreenType};

    fn create_test_intent() -> UiIntent {
        UiIntent::new("test_screen", ScreenType::List)
            .with_action(ActionIntent::new("search", "조회", ActionType::Search))
            .with_action(ActionIntent::new("save", "저장", ActionType::Save))
    }

    #[test]
    fn test_split_output_with_markers() {
        let raw = r#"
--- XML ---
<Dataset id="ds_test">
  <Column name="id" />
</Dataset>

--- JS ---
this.fn_search = function() {
    console.log('search');
};
"#;

        let (xml, js) = XFrame5Validator::split_output(raw).unwrap();
        assert!(xml.contains("<Dataset"));
        assert!(js.contains("fn_search"));
    }

    #[test]
    fn test_split_output_without_markers() {
        let raw = r#"
<Dataset id="ds_test">
  <Column name="id" />
</Dataset>

this.fn_search = function() {
    console.log('search');
};
"#;

        let (xml, js) = XFrame5Validator::split_output(raw).unwrap();
        assert!(xml.contains("<Dataset"));
        assert!(js.contains("fn_search"));
    }

    #[test]
    fn test_validate_xml_basic() {
        let xml = r#"
<Dataset id="ds_member">
  <Column name="id" type="string" />
  <Column name="name" type="string" />
</Dataset>
<Grid id="grid_member" dataset="ds_member">
</Grid>
"#;

        let warnings = XFrame5Validator::validate_xml(xml).unwrap();
        assert!(warnings.is_empty() || warnings.iter().all(|w| w.starts_with("Note:")));
    }

    #[test]
    fn test_validate_xml_missing_dataset() {
        let xml = "<div>Not valid xFrame5</div>";
        let warnings = XFrame5Validator::validate_xml(xml).unwrap();
        assert!(warnings.iter().any(|w| w.contains("Dataset")));
    }

    #[test]
    fn test_validate_js_missing_function() {
        let intent = create_test_intent();
        let js = "this.fn_other = function() {};";

        let warnings = XFrame5Validator::validate_js(js, &intent).unwrap();
        assert!(warnings.iter().any(|w| w.contains("fn_search")));
        assert!(warnings.iter().any(|w| w.contains("fn_save")));
    }

    #[test]
    fn test_validate_js_with_todo() {
        let intent = UiIntent::new("test", ScreenType::List);
        let js = "// TODO: implement search\nthis.fn_search = function() {};";

        let warnings = XFrame5Validator::validate_js(js, &intent).unwrap();
        assert!(warnings.iter().any(|w| w.contains("TODO")));
    }

    #[test]
    fn test_parse_and_validate_complete() {
        let intent = create_test_intent();
        let raw = r#"
--- XML ---
<Dataset id="ds_test">
  <Column name="id" />
</Dataset>

--- JS ---
this.fn_search = function() {
    console.log('search');
};
this.fn_save = function() {
    console.log('save');
};
"#;

        let result = XFrame5Validator::parse_and_validate(raw, &intent).unwrap();
        assert!(!result.xml.is_empty());
        assert!(!result.javascript.is_empty());
    }

    #[test]
    fn test_post_process_adds_missing_stubs() {
        let intent = create_test_intent();
        let mut artifacts = ValidatedArtifacts {
            xml: "<Dataset />".to_string(),
            javascript: "// existing code".to_string(),
            warnings: vec![],
            screen_name: Some("test_screen".to_string()),
        };

        XFrame5Validator::post_process(&mut artifacts, &intent);

        assert!(artifacts.javascript.contains("fn_search"));
        assert!(artifacts.javascript.contains("fn_save"));
        assert!(!artifacts.warnings.is_empty());
    }
}
