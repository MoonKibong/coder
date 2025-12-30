# Post-Processing Pipeline Pattern

> **Location**: `backend/src/services/pipeline/`
> **Feature Doc**: `docs/features/CODEGEN_POST_PROCESSING.md`

---

## Overview

The post-processing pipeline is a 6-pass deterministic system that treats LLM output as untrusted input and enforces xFrame5 framework rules.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    PostProcessingPipeline                    │
├─────────────────────────────────────────────────────────────┤
│  GenerationContext (shared mutable state)                   │
│  ├── raw_output: String                                     │
│  ├── xml: Option<String>                                    │
│  ├── javascript: Option<String>                             │
│  ├── warnings: Vec<String>                                  │
│  ├── execution_mode: ExecutionMode                          │
│  └── intent: UiIntent                                       │
├─────────────────────────────────────────────────────────────┤
│  Passes (executed in order):                                │
│  [0] OutputParser      → Split raw output into XML/JS       │
│  [1] Canonicalizer     → Normalize syntax                   │
│  [2] SymbolLinker      → Match XML events to JS functions   │
│  [3] ApiAllowlistFilter→ Block hallucinated APIs            │
│  [4] GraphValidator    → Validate Dataset ↔ UI bindings     │
│  [5] MinimalismPass    → Remove unused functions            │
└─────────────────────────────────────────────────────────────┘
```

## Core Abstractions

### ExecutionMode

```rust
pub enum ExecutionMode {
    Strict,   // Production - errors halt pipeline
    Relaxed,  // PoC/Development - warnings, auto-fix
    Dev,      // Experimentation - permissive
}
```

### Pass Trait

```rust
pub trait Pass: Send + Sync {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &mut GenerationContext) -> PassResult;
}
```

### PassResult

```rust
pub enum PassResult {
    Ok,
    Warning(String),
    Error(String),
}
```

## Pass Implementations

### Pass 0: OutputParser (`output_parser.rs`)

Splits raw LLM output into XML and JavaScript sections.

**Markers supported**:
- `--- XML ---` / `--- JS ---`
- `<!-- XML -->` / `// JS`
- Heuristic split (find `<Screen` tag)

**Strict mode**: Missing section → Error
**Relaxed mode**: Heuristic fallback allowed

### Pass 1: Canonicalizer (`canonicalizer.rs`)

Normalizes framework-specific naming differences.

**Attribute fixes**:
| From | To |
|------|-----|
| `onclick` | `on_click` |
| `ondblclick` | `on_dblclick` |
| `onchange` | `on_change` |
| `onLoad` | `on_load` |

**Other fixes**:
- Adds `eventfunc:` prefix to event handlers
- Adds missing `()` to function calls
- Fixes Korean font typos (`맑은 고딭` → `맑은 고딕`)

### Pass 2: SymbolLinker (`symbol_linker.rs`)

Ensures XML event handlers have corresponding JavaScript functions.

**Process**:
1. Extract handlers from XML (`on_click="eventfunc:fn_xxx()"`)
2. Extract function definitions from JS (`this.fn_xxx = function`)
3. Generate stubs for missing functions

**Stub format**:
```javascript
this.fn_search = function() {
    // TODO: implement fn_search
};

// For grid row handlers with parameters
this.gr_list_onrowdblclick = function(obj, e) {
    // TODO: implement gr_list_onrowdblclick
};
```

### Pass 3: ApiAllowlistFilter (`api_allowlist.rs`)

Blocks hallucinated or non-existent xFrame5 APIs.

**Allowed APIs** (hardcoded):
- Dataset: `getRowCount`, `getColumn`, `setColumn`, `addRow`, `deleteRow`, etc.
- Grid: `getSelectedRow`, `setCellValue`, `refresh`, `getCheckedRows`, etc.
- Popup: `loadpopup`, `closepopup`, `alert`, `confirm`
- Transaction: `transaction`, `submit`, `save`, `search`
- Standard JS: `console.log`, `JSON.parse`, etc.

**Detection**:
- Extracts method calls using regex: `(\w+)\.(\w+)\s*\(`
- Checks if method name is in allowlist
- Allows user-defined functions (detected by definition pattern)

**Strict mode**: Unknown API → Error
**Relaxed mode**: Add `/* TODO: verify API */` comment

### Pass 4: GraphValidator (`graph_validator.rs`)

Validates Dataset ↔ UI component relationships.

**Checks**:
- Extract dataset IDs from XML (`<xlinkdataset id="ds_xxx">`)
- Extract `link_data` references (`link_data="ds_xxx"`)
- Verify all references point to existing datasets

**Strict mode**: Invalid reference → Error
**Relaxed mode**: Warning only

### Pass 5: MinimalismPass (`minimalism.rs`)

Removes AI-generated over-engineering.

**Process**:
1. Extract function references from XML event handlers
2. Identify lifecycle functions (`fn_onload`, `fn_init`, etc.)
3. Remove JS functions not referenced in XML (except lifecycle)

**Dev mode**: Preserves all functions

## Usage

### In Generation Service

```rust
use crate::services::pipeline::{PostProcessingPipeline, ExecutionMode};

let execution_mode = ExecutionMode::from_strict_mode(options.strict_mode);

let pipeline_result = PostProcessingPipeline::run(
    raw_output,
    &intent,
    execution_mode,
);

match pipeline_result {
    Ok(result) => {
        // result.xml, result.javascript, result.warnings
    }
    Err(e) => {
        // Pipeline failed (strict mode error)
    }
}
```

### Creating Custom Passes

```rust
use crate::services::pipeline::{Pass, PassResult, GenerationContext};

pub struct MyCustomPass;

impl Pass for MyCustomPass {
    fn name(&self) -> &'static str {
        "MyCustomPass"
    }

    fn run(&self, ctx: &mut GenerationContext) -> PassResult {
        // Access ctx.xml, ctx.javascript
        // Modify in place
        // Add warnings via ctx.add_warning()

        if ctx.is_strict() {
            // Strict mode behavior
        }

        PassResult::Ok
    }
}
```

## Testing

Run pipeline tests:
```bash
cargo test services::pipeline
```

Current coverage: 37 tests across all passes.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| API Allowlist Storage | Hardcoded | Simplicity; DB migration planned |
| ExecutionMode | Per-request | Allows PoC flexibility |
| Minimalism Behavior | Remove entirely | Cleaner output for review |
| Pass Order | Fixed | Dependencies between passes |

## Future Work

- [ ] Database-backed API allowlist
- [ ] Admin UI for allowlist management
- [ ] Per-product pass configuration
- [ ] Pass execution metrics
