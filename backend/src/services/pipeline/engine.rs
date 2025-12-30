//! Pipeline Engine - Central coordinator for post-processing passes

use super::{ExecutionMode, GenerationContext, GenerationResult, Pass, PassResult};
use crate::domain::UiIntent;
use anyhow::{anyhow, Result};

/// Post-processing pipeline that executes passes in fixed order
pub struct PostProcessingPipeline {
    passes: Vec<Box<dyn Pass>>,
}

impl PostProcessingPipeline {
    /// Create a new pipeline with all passes in correct order
    pub fn new() -> Self {
        use super::passes::*;

        Self {
            passes: vec![
                Box::new(OutputParser::new()),
                Box::new(Canonicalizer::new()),
                Box::new(SymbolLinker::new()),
                Box::new(ApiAllowlistFilter::new()),
                Box::new(GraphValidator::new()),
                Box::new(MinimalismPass::new()),
            ],
        }
    }

    /// Run the complete pipeline on raw LLM output
    ///
    /// # Arguments
    /// * `raw_output` - Raw string output from LLM
    /// * `intent` - Original UI intent for validation
    /// * `mode` - Execution mode (Strict/Relaxed/Dev)
    ///
    /// # Returns
    /// * `Ok(GenerationResult)` - Processed artifacts with warnings
    /// * `Err` - Fatal error (only in Strict mode)
    pub fn run(
        raw_output: String,
        intent: &UiIntent,
        mode: ExecutionMode,
    ) -> Result<GenerationResult> {
        let pipeline = Self::new();
        pipeline.execute(raw_output, intent, mode)
    }

    /// Execute the pipeline
    fn execute(
        &self,
        raw_output: String,
        intent: &UiIntent,
        mode: ExecutionMode,
    ) -> Result<GenerationResult> {
        let mut ctx = GenerationContext::new(raw_output, intent.clone(), mode);

        tracing::info!(
            "Starting post-processing pipeline with {} passes in {:?} mode",
            self.passes.len(),
            mode
        );

        for (i, pass) in self.passes.iter().enumerate() {
            let pass_name = pass.name();
            tracing::debug!("Running pass {}: {}", i, pass_name);

            let result = pass.run(&mut ctx);

            match result {
                PassResult::Ok => {
                    tracing::debug!("Pass {} completed successfully", pass_name);
                }
                PassResult::Warning(ref msg) => {
                    tracing::warn!("Pass {} warning: {}", pass_name, msg);
                    ctx.add_warning(format!("[{}] {}", pass_name, msg));
                }
                PassResult::Error(ref msg) => {
                    tracing::error!("Pass {} error: {}", pass_name, msg);

                    if ctx.is_strict() {
                        return Err(anyhow!(
                            "Pipeline failed at pass '{}': {}",
                            pass_name,
                            msg
                        ));
                    }

                    // Non-strict mode: convert error to warning and continue
                    ctx.add_warning(format!("[{}] Error (non-strict): {}", pass_name, msg));
                }
            }
        }

        tracing::info!(
            "Pipeline completed with {} warnings",
            ctx.warnings.len()
        );

        // Ensure we have both XML and JS
        if ctx.xml.is_none() {
            return Err(anyhow!("Pipeline did not produce XML output"));
        }
        if ctx.javascript.is_none() {
            return Err(anyhow!("Pipeline did not produce JavaScript output"));
        }

        Ok(GenerationResult::from_context(ctx)
            .expect("XML and JS were verified above"))
    }
}

impl Default for PostProcessingPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ScreenType;

    fn create_test_intent() -> UiIntent {
        UiIntent::new("test_screen", ScreenType::List)
    }

    #[test]
    fn test_pipeline_with_valid_output() {
        let raw = r#"
--- XML ---
<screen id="SCREEN_TEST">
  <xlinkdataset id="ds_list"/>
  <grid name="grid_list" link_data="ds_list"/>
</screen>

--- JS ---
this.fn_search = function() {
    console.log('search');
};
"#;

        let intent = create_test_intent();
        let result = PostProcessingPipeline::run(
            raw.to_string(),
            &intent,
            ExecutionMode::Relaxed,
        );

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.xml.is_empty());
        assert!(!result.javascript.is_empty());
    }

    #[test]
    fn test_pipeline_strict_mode_error() {
        // Invalid output that should fail in strict mode
        let raw = "no xml or js here";

        let intent = create_test_intent();
        let result = PostProcessingPipeline::run(
            raw.to_string(),
            &intent,
            ExecutionMode::Strict,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_relaxed_mode_recovers() {
        // Slightly malformed but recoverable
        let raw = r#"
<screen id="SCREEN_TEST">
  <xlinkdataset id="ds_list"/>
</screen>

this.fn_test = function() {};
"#;

        let intent = create_test_intent();
        let result = PostProcessingPipeline::run(
            raw.to_string(),
            &intent,
            ExecutionMode::Relaxed,
        );

        // Should succeed with warnings in relaxed mode
        assert!(result.is_ok());
    }
}
