# xFrame5 Knowledge Base Usage Guide

This guide explains how to use the xFrame5 Knowledge Base for selective prompt inclusion.

---

## Overview

The xFrame5 Knowledge Base (`XFRAME5_KNOWLEDGE_BASE.md`) organizes framework knowledge into categorized sections that can be selectively included in LLM prompts based on the specific generation task.

### Problem Solved

**Challenge**: xFrame5 documentation is extensive. Including all knowledge in every prompt would:
- Exceed token limits
- Reduce focus on relevant information
- Decrease generation quality

**Solution**: Categorize knowledge by relevance and include only pertinent sections based on task type.

---

## Knowledge Organization

The knowledge base is organized into **7 main categories**:

1. **Core Architecture** - Always include (brief)
2. **Dataset Component** - Include for all data-handling screens
3. **Grid Component** - Include for list/tabular screens
4. **Popup Patterns** - Include for popup/dialog screens
5. **IO Mapping & Transactions** - Include when generating transaction functions
6. **Global Modules** - Include when generating shared utilities
7. **Naming Conventions** - Always include

Each section includes a **"WHEN TO INCLUDE"** directive indicating relevance.

---

## Task-Based Selection Strategy

### List Screen Generation

**Include Sections**:
- Core Architecture (brief overview only)
- Dataset Component → Column Definition
- Grid Component → Structure, Properties, Basic Events
- IO Mapping → Transaction Type
- Naming Conventions (all)

**Focus Events**: `on_itemclick`, `on_itemdblclick`, `on_load`
**Focus Functions**: `fn_search`, `fn_delete`, `fn_add`

**Example**: User requests "Generate member list screen"
```
Prompt should include:
- Brief architecture (component relationships)
- Dataset column definition patterns
- Grid XML structure with link_data binding
- Grid basic events (item click, double click)
- Transaction mapping for fn_search
- Naming conventions for ds_*, grid_*, fn_*
```

### Detail/Form Screen Generation

**Include Sections**:
- Core Architecture (brief)
- Dataset Component → Column Definition, Data Binding
- Popup Patterns (if popup-based)
- IO Mapping → Transaction Type
- Naming Conventions (all)

**Focus Events**: `on_load`, `on_popupdestroy` (if popup)
**Focus Functions**: `fn_save`, `fn_delete`, `fn_load`

**Example**: User requests "Generate member detail form popup"
```
Prompt should include:
- Popup patterns (modal/modeless, extra_data, return mechanisms)
- Dataset definition for form fields
- Transaction mapping for fn_save
- on_popupdestroy event for returning data to parent
- Naming conventions
```

### Master-Detail Screen Generation

**Include Sections**:
- Core Architecture → Data Flow
- Dataset Component (all)
- Grid Component (comprehensive including events)
- IO Mapping → Transaction Type
- Naming Conventions (all)

**Focus Events**: `on_itemclick`, `on_itemselchange`, `on_itemdblclick`
**Focus Functions**: `fn_search_master`, `fn_search_detail`, `fn_save`, `fn_delete`

**Example**: User requests "Generate order list with order items detail"
```
Prompt should include:
- Data flow patterns for master-detail relationships
- Grid events for row selection triggering detail load
- Multiple dataset patterns (ds_master, ds_detail)
- Transaction patterns for coordinated saves
- Grid component events comprehensively
```

---

## Prompt Template Structure

### Recommended Template Format

```yaml
system_prompt: |
  You are an expert xFrame5 code generator.

  # ARCHITECTURE OVERVIEW
  {{#if always}}
  [Include Core Architecture - Brief]
  {{/if}}

  # COMPONENT KNOWLEDGE
  {{#if screen_type == "list"}}
  [Include Dataset Component - Column Definition]
  [Include Grid Component - Structure, Properties, Basic Events]
  {{/if}}

  {{#if screen_type == "detail" || screen_type == "form"}}
  [Include Dataset Component - All]
  [Include Popup Patterns - If popup-based]
  {{/if}}

  {{#if has_nested_grids}}
  [Include Grid Component - Comprehensive Events]
  [Include Dataset Component - Parent-Child Relationships]
  {{/if}}

  # TRANSACTION PATTERNS
  {{#if needs_transactions}}
  [Include IO Mapping - Transaction Type]
  [Include Java API - Server-Side Patterns]
  {{/if}}

  # NAMING CONVENTIONS
  {{#if always}}
  [Include Naming Conventions - All]
  {{/if}}

  # VALIDATION REQUIREMENTS
  {{#if always}}
  [Include Validation Requirements]
  {{/if}}

  # OUTPUT FORMAT
  --- XML ---
  <your XML content>

  --- JS ---
  <your JavaScript content>

  {{company_rules}}
```

### Example: List Screen Template

```yaml
system_prompt: |
  You are an expert xFrame5 frontend code generator for LIST screens.

  # FRAMEWORK BASICS
  xFrame5 is HTML5-based with component layers: Dataset (data) → Grid (display)

  # DATASET PATTERNS
  - Datasets store tabular data with column definitions
  - ID prefix: ds_* (e.g., ds_list, ds_search)
  - Column structure:
    <XDataSet id="ds_list">
        <columns>
            <column id="col_id" type="string" size="50"/>
        </columns>
    </XDataSet>

  # GRID PATTERNS
  - Grid binds to dataset via link_data property
  - ID prefix: grid_* (e.g., grid_list)
  - Column structure:
    <Grid id="grid_list" link_data="ds_list">
        <columns>
            <column title="ID" data_column="col_id" width="100"/>
        </columns>
    </Grid>

  # ESSENTIAL GRID EVENTS
  - on_itemclick: Single click handler
  - on_itemdblclick: Double click handler (common for opening detail)
  - on_load: Screen initialization

  # TRANSACTION FUNCTIONS
  - fn_search: Retrieve list data
    - Use Transaction mapping type
    - Map ds_list to server response
    - TODO: API endpoint placeholder
  - fn_add: Add new row to grid
  - fn_delete: Delete selected rows

  # NAMING CONVENTIONS
  - Datasets: ds_*
  - Grids: grid_*
  - Functions: fn_*
  - Events: on_*

  # VALIDATION
  - Dataset IDs must start with ds_
  - Grid IDs must start with grid_
  - Grid data_column must match dataset column id
  - NEVER make up API endpoints - use TODO placeholders

  OUTPUT FORMAT:
  --- XML ---
  --- JS ---

  {{company_rules}}
```

---

## Implementation Steps

### Step 1: Analyze User Request

Determine:
- **Screen type**: list, detail, form, popup, master-detail
- **Components needed**: Grid, Dataset, Popup, etc.
- **Transactions needed**: search, save, delete, etc.
- **Special features**: nested grids, file upload, etc.

### Step 2: Select Relevant Knowledge Sections

Use the **"WHEN TO INCLUDE"** directives in `XFRAME5_KNOWLEDGE_BASE.md`:

```
IF screen_type == "list" THEN
    include: Dataset (column definition)
    include: Grid (structure, properties, basic events)
    include: IO Mapping (Transaction type)
ENDIF

IF has_transactions THEN
    include: IO Mapping (relevant type)
    include: Java API (server-side patterns)
ENDIF

ALWAYS include:
    - Core Architecture (brief)
    - Naming Conventions
    - Validation Requirements
```

### Step 3: Compile Prompt

Build system prompt by concatenating selected sections from knowledge base.

### Step 4: Validate Generated Code

Ensure output follows:
- Naming conventions
- Validation requirements
- TODO comment rules for unknowns

---

## Example Scenarios

### Scenario 1: Simple List Screen

**Request**: "Generate member list screen with search"

**Analysis**:
- screen_type: list
- Components: Dataset, Grid
- Transactions: fn_search
- Features: basic list with search

**Selected Knowledge**:
1. Core Architecture (brief)
2. Dataset Component → Column Definition
3. Grid Component → Structure, Basic Properties, Events (on_itemclick, on_itemdblclick)
4. IO Mapping → Transaction Type
5. Naming Conventions
6. Validation Requirements

**Excluded Knowledge**:
- Popup Patterns (not needed)
- Grid Advanced Events (not needed for basic list)
- Global Modules (not needed)
- Java API details (basic TODO placeholder sufficient)

### Scenario 2: Master-Detail with Nested Grids

**Request**: "Generate order list with order items detail in nested grid"

**Analysis**:
- screen_type: master-detail
- Components: Multiple Datasets, Multiple Grids
- Transactions: fn_search_master, fn_search_detail
- Features: nested grids, row selection triggering detail load

**Selected Knowledge**:
1. Core Architecture → Data Flow
2. Dataset Component (comprehensive) → Parent-Child Relationships
3. Grid Component (comprehensive) → ALL Events, especially:
   - on_itemselchange (for master selection)
   - on_itemclick
4. IO Mapping → Transaction Type
5. Naming Conventions
6. Validation Requirements

**Excluded Knowledge**:
- Popup Patterns (not popup-based)
- Global Modules (not needed)

### Scenario 3: Detail Popup with Save

**Request**: "Generate member detail popup with save/cancel"

**Analysis**:
- screen_type: detail, popup
- Components: Dataset, Form controls, Popup
- Transactions: fn_save
- Features: popup with return value, form validation

**Selected Knowledge**:
1. Core Architecture (brief)
2. Dataset Component → Data Binding
3. Popup Patterns (comprehensive) → Modal, Parameter Passing, Return Mechanisms
4. IO Mapping → Transaction Type
5. Naming Conventions
6. Validation Requirements

**Excluded Knowledge**:
- Grid Component (no grid in detail form)
- Global Modules (not needed)
- Most Grid Events (no grid)

---

## Token Budget Guidelines

Approximate token counts for knowledge sections:

| Section | Tokens (approx) | Priority |
|---------|----------------|----------|
| Core Architecture (brief) | 300 | High |
| Dataset Component (full) | 800 | Medium |
| Dataset Component (column def only) | 200 | High |
| Grid Component (structure + basic events) | 600 | High |
| Grid Component (all events) | 1500 | Medium |
| Popup Patterns (all) | 400 | High |
| IO Mapping (all types) | 500 | Medium |
| IO Mapping (Transaction type only) | 200 | High |
| Global Modules | 300 | Low |
| Naming Conventions | 200 | High |
| Validation Requirements | 300 | High |
| Common Patterns | 400 | Medium |

**Strategy**:
- For simple tasks: ~2000 tokens of knowledge (High priority only)
- For complex tasks: ~4000 tokens of knowledge (High + Medium priority)
- Always include: Core Architecture (brief), Naming Conventions, Validation Requirements

---

## Updating Templates

### Current Templates

Located in database table: `prompt_templates`

**Existing Templates**:
1. `xframe5-list` - List screen generation
2. `xframe5-detail` - Detail screen generation

### Improvement Strategy

1. **Identify task type** from template name/metadata
2. **Reference knowledge base sections** instead of embedding full knowledge
3. **Use conditional inclusion** based on user request features
4. **Keep templates focused** on task-specific patterns

### Example Updated Template

**Before** (embedded knowledge):
```yaml
system_prompt: |
  You are an xFrame5 expert. Generate code following these patterns:
  [200 lines of embedded framework knowledge]
  [50 lines of naming conventions]
  [100 lines of event descriptions]
  ...
```

**After** (selective knowledge):
```yaml
system_prompt: |
  You are an xFrame5 expert generating LIST screens.

  # CORE FRAMEWORK (brief)
  [30 lines - focused on Dataset → Grid binding]

  # GRID ESSENTIALS (task-specific)
  [50 lines - structure, basic events only]

  # TRANSACTION PATTERN (focused)
  [20 lines - Transaction type mapping for fn_search]

  # CONVENTIONS & VALIDATION (always)
  [30 lines - naming rules, validation, TODO rules]

  OUTPUT FORMAT:
  --- XML ---
  --- JS ---
```

**Result**: Reduced from ~350 lines to ~130 lines while maintaining quality

---

## Best Practices

### DO:
✅ Include only knowledge sections relevant to the specific task
✅ Always include Core Architecture (brief), Naming Conventions, Validation Requirements
✅ Use "WHEN TO INCLUDE" directives as decision criteria
✅ Favor depth over breadth for task-specific components
✅ Include TODO comment rules for handling unknowns

### DON'T:
❌ Include entire knowledge base in every prompt
❌ Include Grid events for non-grid screens
❌ Include Popup patterns for non-popup screens
❌ Skip Naming Conventions or Validation Requirements
❌ Make up API endpoints or server URLs

---

## Maintenance

### When to Update Knowledge Base

- New xFrame5 version released
- Additional documentation pages become available
- Company-specific patterns identified
- Generation errors reveal knowledge gaps

### Update Process

1. Add new knowledge to appropriate category
2. Add "WHEN TO INCLUDE" directive
3. Update token budget guidelines
4. Update task-based selection mappings
5. Test with prompt templates

---

## Summary

The xFrame5 Knowledge Base enables:

1. **Selective Inclusion**: Include only relevant knowledge per task
2. **Token Efficiency**: Reduce prompt size by 50-70%
3. **Better Focus**: LLM concentrates on pertinent information
4. **Consistent Quality**: Standard patterns applied uniformly
5. **Maintainability**: Central knowledge source, not scattered across templates

**Key Principle**: "Include what you need, exclude what you don't"
