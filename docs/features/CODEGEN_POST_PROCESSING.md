# Feature & Implementation Guide
## Deterministic Post-Processing Pipeline for xFrame5 Code Generation

> **Status**: ✅ **IMPLEMENTED** (2025-12-30)
> **Location**: `backend/src/services/pipeline/`
> **Tests**: 37 unit tests (all passing)

---

## 1. Purpose

This document describes **additional features** implemented on top of the existing Rust-based Agent Server.

The current system already:
- Accepts structured requests from IDE plugins
- Generates code artifacts using LLMs
- Returns XML and JavaScript outputs

However, **LLM output is not deterministic nor policy-safe** for enterprise (financial SI) environments.

This feature introduces a **deterministic post-processing pipeline** that:
- Does NOT change existing generation logic
- Is executed *after* LLM generation
- Enforces xFrame5 framework rules, naming conventions, and safety constraints
- Guarantees stable and auditable outputs regardless of model quality

This pipeline is an **additive feature**, not a replacement.

---

## 2. High-Level Feature Overview

### Feature Name
**Deterministic Generation Pipeline**

### Scope
- Applies only to generated artifacts (XML / JavaScript)
- Independent of LLM backend (ollama, llama.cpp, future models)
- Configurable by execution mode (Strict / Relaxed / Dev)

### Non-Goals
- No LLM calls
- No prompt engineering
- No IDE plugin changes
- No framework-specific code generation logic

---

## 3. Pipeline Concept

### Core Principle

> LLM output must be treated as *untrusted input*.

The system guarantees correctness by passing artifacts through **multiple deterministic passes**.

---

### Pipeline Order (Fixed)

```

Raw LLM Output
↓
[0] Output Parser
↓
[1] Canonicalizer
↓
[2] Symbol Linker
↓
[3] API Allowlist Filter
↓
[4] Graph Validator
↓
[5] Minimalism Pass
↓
Final Artifacts

```

⚠️ **Order must not be changed**  
Several passes depend on previous normalization steps.

---

## 4. Integration Point with Existing Codebase

### Current Flow (Simplified)

```

Request
→ Prompt Compilation
→ LLM Generation
→ Artifact Response

```

### Updated Flow (With This Feature)

```

Request
→ Prompt Compilation
→ LLM Generation
→ Post-Processing Pipeline   ← (NEW)
→ Artifact Response

````

### Integration Rule

- Existing LLM generation code MUST remain untouched
- The pipeline is invoked immediately after LLM output is received
- All failures are handled inside the pipeline layer

---

## 5. Execution Modes

The pipeline behavior depends on execution mode.

### ExecutionMode Enum

```rust
enum ExecutionMode {
    Strict,   // Production / Financial environment
    Relaxed, // PoC / Development
    Dev,     // Internal experimentation
}
````

### Mode Semantics

| Behavior          | Strict  | Relaxed      | Dev     |
| ----------------- | ------- | ------------ | ------- |
| Guessing APIs     | ❌ Error | TODO         | Allowed |
| Missing handlers  | ❌ Error | Stub         | Allowed |
| XML parse failure | ❌ Error | Retry/Repair | Ignore  |
| Over-engineering  | Removed | Warning      | Kept    |
| Model re-query    | ❌       | Optional     | Allowed |

**Production must always use Strict mode.**

---

## 6. Core Abstractions

### Pass Trait

All pipeline steps implement the same trait.

```rust
trait Pass {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &mut GenerationContext) -> PassResult;
}
```

### PassResult

```rust
enum PassResult {
    Ok,
    Warning(String),
    Error(String),
}
```

Errors do **not** crash the server directly.
They are interpreted according to `ExecutionMode`.

---

## 7. Generation Context

### Purpose

Shared mutable state passed through all pipeline passes.

### Required Fields

```rust
struct GenerationContext {
    raw_output: String,
    xml: Option<String>,
    javascript: Option<String>,
    warnings: Vec<String>,
    execution_mode: ExecutionMode,
}
```

* `raw_output` is immutable after initialization
* `xml` and `javascript` are progressively refined
* warnings accumulate and are returned to the client

---

## 8. Pipeline Passes (Implementation Guide)

### Pass 0: Output Parser

**Responsibility**

* Split raw LLM output into XML and JS sections

**Rules**

* Marker-based split using:

  * `--- XML ---`
  * `--- JS ---`
* Strict mode:

  * Missing section → Error
* Relaxed/Dev:

  * Heuristic split allowed

---

### Pass 1: Canonicalizer

**Responsibility**

* Normalize framework-specific naming differences

**Examples**

* `onclick` → `on_click`
* `onLoad` → `onload`

**Scope**

* XML attributes
* JavaScript function names

⚠️ Regex-only replacement is discouraged.
Structure-aware logic is preferred.

---

### Pass 2: Symbol Linker

**Responsibility**

* Ensure XML event handlers match JavaScript functions

**Rules**

* XML is the source of truth
* Missing JS handler:

  * Strict → Error
  * Relaxed → Generate stub
  * Dev → Ignore

Generated stub example:

```js
this.gr_customer_onrowdblclick = function(obj, e) {
    // TODO: implement
};
```

---

### Pass 3: API Allowlist Filter

**Responsibility**

* Block hallucinated or non-existent APIs

**Mechanism**

* Hardcoded allowlist of valid xFrame5 JavaScript APIs
* Detect unknown function calls

**Handling**

* Strict → Error
* Relaxed/Dev → Replace with TODO comment

---

### Pass 4: Graph Validator

**Responsibility**

* Validate Dataset ↔ UI component relationships

**Checks**

* Dataset existence
* binddataset correctness
* Column id matching

**Behavior**

* Auto-fix allowed in non-Strict modes
* Strict mode emits warnings only (no guessing)

---

### Pass 5: Minimalism Pass

**Responsibility**

* Remove AI-generated over-engineering

**Targets**

* Unused JavaScript functions
* Unreferenced helpers
* Unrequested features

**Goal**

* Produce review-friendly, minimal code

---

## 9. Pipeline Engine

### Role

Central coordinator that executes passes in order.

### Responsibilities

* Enforce pass order
* Interpret PassResult based on ExecutionMode
* Accumulate warnings
* Stop execution on fatal errors (Strict mode)

---

## 10. Output Contract

The pipeline must return a deterministic result structure:

```rust
struct GenerationResult {
    xml: String,
    javascript: String,
    warnings: Vec<String>,
}
```

### Important

* No references to LLMs or model internals
* Output must be framework-pure (xFrame5 only)

---

## 11. Design Guarantees

This feature guarantees:

* Model independence
* Deterministic behavior
* Enterprise auditability
* Safe degradation when using quantized models
* Identical behavior across PoC and Production

---

## 12. Summary

This pipeline:

* Treats LLMs as probabilistic components
* Enforces correctness through deterministic systems
* Allows large and small models to coexist safely
* Is critical for financial SI environments

**LLMs may fail.
The system must not.**

---

## 13. Implementation Details (ACTUAL)

### File Structure

```
backend/src/services/pipeline/
├── mod.rs              # Core types: ExecutionMode, Pass trait, PassResult, GenerationContext
├── engine.rs           # Pipeline orchestrator (PostProcessingPipeline)
└── passes/
    ├── mod.rs          # Re-exports all passes
    ├── output_parser.rs    # Pass 0: Split raw output into XML/JS
    ├── canonicalizer.rs    # Pass 1: onclick→on_click, font fixes
    ├── symbol_linker.rs    # Pass 2: Match XML events to JS functions
    ├── api_allowlist.rs    # Pass 3: Block hallucinated APIs
    ├── graph_validator.rs  # Pass 4: Validate Dataset ↔ UI bindings
    └── minimalism.rs       # Pass 5: Remove unused functions
```

### Integration Point

**File**: `backend/src/services/generation.rs` (lines 65-73)

```rust
// Execution mode is derived from strictMode option
let execution_mode = ExecutionMode::from_strict_mode(options.strict_mode);

let pipeline_result = PostProcessingPipeline::run(
    raw_output.clone(),
    &intent,
    execution_mode,
);
```

### API Usage

```rust
use crate::services::pipeline::{PostProcessingPipeline, ExecutionMode};

// Run pipeline on LLM output
let result = PostProcessingPipeline::run(
    raw_llm_output,
    &ui_intent,
    ExecutionMode::Relaxed,  // or Strict for production
)?;

// Result contains processed artifacts
let xml = result.xml;
let javascript = result.javascript;
let warnings = result.warnings;
```

### Pass Implementation Examples

#### Canonicalizer (Pass 1)
Fixes common LLM output issues:
- `onclick` → `on_click`
- `ondblclick` → `on_dblclick`
- Adds `eventfunc:` prefix to event handlers
- Fixes Korean font typos (`맑은 고딭` → `맑은 고딕`)

#### API Allowlist (Pass 3)
Hardcoded allowlist includes:
- Dataset APIs: `getRowCount`, `getColumn`, `setColumn`, `addRow`, etc.
- Grid APIs: `getSelectedRow`, `setCellValue`, `refresh`, etc.
- Popup APIs: `loadpopup`, `closepopup`, `alert`, `confirm`
- Transaction APIs: `transaction`, `submit`, `save`, `search`

#### Symbol Linker (Pass 2)
Generates stubs for missing handlers:
```javascript
this.gr_list_onrowdblclick = function(obj, e) {
    // TODO: implement gr_list_onrowdblclick
};
```

### Test Coverage

| Pass | Test Count | Coverage |
|------|------------|----------|
| mod.rs | 4 | ExecutionMode, PassResult, GenerationContext |
| engine.rs | 3 | Full pipeline, strict mode, relaxed mode |
| output_parser.rs | 4 | Markers, no markers, errors, cleanup |
| canonicalizer.rs | 6 | onclick, eventfunc, fonts, combined |
| symbol_linker.rs | 6 | Extraction, stubs, strict mode |
| api_allowlist.rs | 4 | Allowed APIs, user functions, hallucinated |
| graph_validator.rs | 5 | Dataset extraction, link_data validation |
| minimalism.rs | 4 | XML refs, lifecycle, used functions |
| **Total** | **37** | All passing ✅ |

### Configuration

ExecutionMode is set per-request via `options.strict_mode`:
```json
{
  "options": {
    "strictMode": true   // → ExecutionMode::Strict
  }
}
```

### Future Enhancements

- [ ] Database-backed API allowlist (currently hardcoded)
- [ ] Admin UI for API allowlist management
- [ ] Custom pass injection per product
- [ ] Metrics/telemetry for pass execution times

---