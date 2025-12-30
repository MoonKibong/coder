//! Deterministic Post-Processing Pipeline for xFrame5 Code Generation
//!
//! This module implements a 6-pass pipeline that treats LLM output as untrusted input
//! and enforces deterministic correctness for enterprise (financial SI) environments.
//!
//! ## Pipeline Order (Fixed)
//! 1. Output Parser - Split raw output into XML/JS sections
//! 2. Canonicalizer - Normalize syntax (onclick → on_click, font fixes)
//! 3. Symbol Linker - Match XML events to JS functions
//! 4. API Allowlist Filter - Block hallucinated APIs
//! 5. Graph Validator - Validate Dataset ↔ UI bindings
//! 6. Minimalism Pass - Remove unused functions

pub mod engine;
pub mod passes;

pub use engine::PostProcessingPipeline;

use crate::domain::UiIntent;

/// Execution mode determines how errors and warnings are handled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutionMode {
    /// Production / Financial environment - errors halt pipeline
    Strict,
    /// PoC / Development - warnings, auto-fix allowed
    #[default]
    Relaxed,
    /// Internal experimentation - permissive
    Dev,
}

impl ExecutionMode {
    /// Create from strictMode boolean (from request options)
    pub fn from_strict_mode(strict: bool) -> Self {
        if strict {
            ExecutionMode::Strict
        } else {
            ExecutionMode::Relaxed
        }
    }
}

/// Result of executing a single pass
#[derive(Debug, Clone)]
pub enum PassResult {
    /// Pass completed successfully
    Ok,
    /// Pass completed with warning (non-fatal)
    Warning(String),
    /// Pass failed (fatal in Strict mode)
    Error(String),
}

impl PassResult {
    /// Check if this is an error result
    pub fn is_error(&self) -> bool {
        matches!(self, PassResult::Error(_))
    }

    /// Check if this is a warning result
    pub fn is_warning(&self) -> bool {
        matches!(self, PassResult::Warning(_))
    }

    /// Get the message if this is a warning or error
    pub fn message(&self) -> Option<&str> {
        match self {
            PassResult::Ok => None,
            PassResult::Warning(msg) | PassResult::Error(msg) => Some(msg),
        }
    }
}

/// Trait for pipeline passes
///
/// All pipeline steps implement this trait for consistent execution.
pub trait Pass: Send + Sync {
    /// Returns the name of this pass for logging/debugging
    fn name(&self) -> &'static str;

    /// Execute the pass on the given context
    ///
    /// # Arguments
    /// * `ctx` - Mutable reference to the generation context
    ///
    /// # Returns
    /// * `PassResult` indicating success, warning, or error
    fn run(&self, ctx: &mut GenerationContext) -> PassResult;
}

/// Shared mutable state passed through all pipeline passes
#[derive(Debug, Clone)]
pub struct GenerationContext {
    /// Original raw LLM output (immutable after initialization)
    pub raw_output: String,

    /// XML content being progressively refined
    pub xml: Option<String>,

    /// JavaScript content being progressively refined
    pub javascript: Option<String>,

    /// Accumulated warnings from all passes
    pub warnings: Vec<String>,

    /// Current execution mode
    pub execution_mode: ExecutionMode,

    /// Original intent for validation reference
    pub intent: UiIntent,
}

impl GenerationContext {
    /// Create a new generation context
    pub fn new(raw_output: String, intent: UiIntent, execution_mode: ExecutionMode) -> Self {
        Self {
            raw_output,
            xml: None,
            javascript: None,
            warnings: Vec::new(),
            execution_mode,
            intent,
        }
    }

    /// Add a warning to the context
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Check if we're in strict mode
    pub fn is_strict(&self) -> bool {
        matches!(self.execution_mode, ExecutionMode::Strict)
    }

    /// Check if we're in dev mode
    pub fn is_dev(&self) -> bool {
        matches!(self.execution_mode, ExecutionMode::Dev)
    }
}

/// Final result after pipeline execution
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Processed XML content
    pub xml: String,

    /// Processed JavaScript content
    pub javascript: String,

    /// All warnings accumulated during processing
    pub warnings: Vec<String>,
}

impl GenerationResult {
    /// Create from a generation context (consumes the context)
    pub fn from_context(ctx: GenerationContext) -> Option<Self> {
        Some(Self {
            xml: ctx.xml?,
            javascript: ctx.javascript?,
            warnings: ctx.warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ScreenType;

    #[test]
    fn test_execution_mode_from_strict() {
        assert_eq!(
            ExecutionMode::from_strict_mode(true),
            ExecutionMode::Strict
        );
        assert_eq!(
            ExecutionMode::from_strict_mode(false),
            ExecutionMode::Relaxed
        );
    }

    #[test]
    fn test_pass_result_is_error() {
        assert!(!PassResult::Ok.is_error());
        assert!(!PassResult::Warning("test".to_string()).is_error());
        assert!(PassResult::Error("test".to_string()).is_error());
    }

    #[test]
    fn test_generation_context_new() {
        let intent = UiIntent::new("test_screen", ScreenType::List);
        let ctx = GenerationContext::new(
            "raw output".to_string(),
            intent,
            ExecutionMode::Relaxed,
        );

        assert_eq!(ctx.raw_output, "raw output");
        assert!(ctx.xml.is_none());
        assert!(ctx.javascript.is_none());
        assert!(ctx.warnings.is_empty());
        assert!(!ctx.is_strict());
    }

    #[test]
    fn test_generation_context_strict_mode() {
        let intent = UiIntent::new("test_screen", ScreenType::List);
        let ctx = GenerationContext::new(
            "raw".to_string(),
            intent,
            ExecutionMode::Strict,
        );

        assert!(ctx.is_strict());
        assert!(!ctx.is_dev());
    }
}
