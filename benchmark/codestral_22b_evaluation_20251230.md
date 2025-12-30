# Codestral:22B Evaluation Report

**Date:** 2025-12-30
**Model:** codestral:22b (via Ollama)
**Test Type:** xFrame5 List Screen Generation
**Prompt:** "generate a simple task list" (natural language)

---

## Executive Summary

Codestral:22B produces a minimal skeleton that lacks most required components. The output demonstrates basic understanding of xFrame5 structure but falls significantly short of production requirements. Multiple critical syntax errors and missing components make this output unsuitable without major manual intervention.

**Overall Score: 28%** (vs. qwen3-coder:30b's 56%, llama3.1:70b's 62%)

---

## ⏺ Quality Comparison: codestral:22b Output vs Benchmark

### ✅ What's Good

| Aspect | Generated | Benchmark | Status |
|--------|-----------|-----------|--------|
| Dataset definitions | `ds_search`, `ds_list` | Required | ✅ Good |
| Panel structure | `pnl_grid`, `pnl_buttons` | Required | ✅ Partial |
| Position attributes | x, y, width, height on all | Required | ✅ Good |
| Korean labels | "조회", "신규", "삭제" | Good for ko lang | ✅ Good |

### ❌ Issues Found

| Issue | Generated | Benchmark | Severity |
|-------|-----------|-----------|----------|
| Button event attribute | `onclick="fn_search"` | `on_click="eventfunc:fn_search()"` | 🔴 Critical |
| Missing eventfunc prefix | `onclick="fn_search"` | `on_click="eventfunc:..."` | 🔴 Critical |
| Missing function parens | `fn_search` | `fn_search()` | 🔴 Critical |
| Script tag in XML | `<script>` embedded | Separate .js file | 🔴 Critical |
| Screen ID format | `task_list` | `SCREEN_TASK_LIST` | 🟡 Medium |
| Grid columns | TODO comment only | Full column definitions | 🔴 Critical |
| Missing search panel | None | Required for list screen | 🔴 Critical |
| Missing edit button | None | Required | 🟡 Medium |
| Grid version | Missing | `version="1.1"` | 🟢 Low |
| JS functions | 3 TODO stubs | Full implementation | 🟡 Medium |

### 📊 Score Card

```
Structure & Layout:    ████░░░░░░  40%  (missing search, header)
XML Syntax:            ██░░░░░░░░  20%  (onclick, no eventfunc, script tag)
JavaScript:            ██░░░░░░░░  20%  (minimal stubs)
xFrame5 Conventions:   ███░░░░░░░  30%  (wrong ID, missing version)
Production Ready:      █░░░░░░░░░  15%  (would completely fail)
Grid Columns:          ░░░░░░░░░░   0%  (empty - TODO only)
───────────────────────────────────────
Overall:               ███░░░░░░░  28%
```

### 🔧 Critical Fixes Needed

**Multiple syntax errors** prevent this code from working:

```xml
<!-- WRONG (generated) - THREE errors in one line -->
<pushbutton onclick="fn_search"/>

<!-- CORRECT (benchmark) -->
<pushbutton on_click="eventfunc:fn_search()"/>
```

Errors:
1. `onclick` → `on_click`
2. Missing `eventfunc:` prefix
3. Missing `()` on function call

### 💡 Recommendations

1. **Do not use codestral:22b** for xFrame5 generation without extensive post-processing
2. **Use qwen3-coder:30b or llama3.1:70b** instead (2x higher scores)
3. If codestral must be used, implement aggressive post-processing to fix all syntax issues
4. Consider fine-tuning or providing more few-shot examples in prompt

---

## Generated Output

### XML Output
```xml
<screen id="task_list" width="800" height="600" script_language="JavaScript">
    <!-- Datasets -->
    <xdataset id="ds_search" desc="검색조건"></xdataset>
    <xdataset id="ds_list" desc="작업목록"></xdataset>

    <!-- Grid Panel -->
    <panel control_id="10" name="pnl_grid" x="10" y="80" width="780" height="450">
        <grid control_id="11" name="grid_list" x="0" y="0" width="780" height="450"
              link_data="ds_list" linenumber_show="1" use_checkrow="1">
            <!-- TODO: Define grid columns here -->
        </grid>
    </panel>

    <!-- Button Panel -->
    <panel control_id="20" name="pnl_buttons" x="10" y="540" width="780" height="40">
        <pushbutton control_id="21" name="btn_search" x="10" y="8" width="80" height="28"
                    text="조회" font="맑은 고딕,9,0,0,0,0" onclick="fn_search"/>
        <pushbutton control_id="22" name="btn_add" x="100" y="8" width="80" height="28"
                    text="신규" font="맑은 고딕,9,0,0,0,0" onclick="fn_add"/>
        <pushbutton control_id="23" name="btn_delete" x="190" y="8" width="80" height="28"
                    text="삭제" font="맑은 고딕,9,0,0,0,0" onclick="fn_delete"/>
    </panel>
</screen>

<!-- JavaScript -->
<script language="JavaScript">
```

### JavaScript Output
```javascript
function fn_search() {
    // TODO: Implement search logic here
}

function fn_add() {
    // TODO: Implement add logic here
}

function fn_delete() {
    // TODO: Implement delete logic here
}
</script>
```

---

## Critical Issues Found

### 1. Malformed Output Structure
The model incorrectly embedded `<script>` tags in the XML output. xFrame5 requires separate XML and JS files.

```xml
<!-- WRONG: Script tag embedded in XML -->
<script language="JavaScript">

<!-- CORRECT: Separate .js file -->
```

### 2. Missing Event Syntax Prefix
Button events use bare function names instead of `eventfunc:` prefix.

```xml
<!-- WRONG (generated) -->
onclick="fn_search"

<!-- CORRECT (benchmark) -->
on_click="eventfunc:fn_search()"
```

### 3. Wrong Event Attribute Name
Uses `onclick` (HTML) instead of `on_click` (xFrame5).

### 4. Missing Search Panel
No search panel with input fields and filters - a core requirement for list screens.

### 5. Empty Grid Columns
Grid has no column definitions - just a TODO comment.

### 6. Missing Screen ID Convention
Uses `task_list` instead of `SCREEN_TASK_LIST` convention.

---

## Detailed Comparison

### Component Checklist

| Component | Benchmark | Generated | Status |
|-----------|-----------|-----------|--------|
| Screen root with proper ID | `SCREEN_TASK_LIST` | `task_list` | ❌ Wrong format |
| `ds_list` dataset | Required | Present | ⚠️ No columns |
| `ds_search` dataset | Required | Present | ⚠️ No columns |
| Code datasets (status, etc.) | Required | Missing | ❌ Missing |
| `pnl_header` panel | Required | Missing | ❌ Missing |
| `pnl_search` panel | Required | Missing | ❌ Missing |
| Search input fields | Required | Missing | ❌ Missing |
| Filter comboboxes | Required | Missing | ❌ Missing |
| `pnl_grid` panel | Required | Present | ✅ Good |
| Grid with columns | Required | Empty (TODO) | ❌ Missing |
| Grid `version="1.1"` | Required | Missing | ❌ Missing |
| `pnl_buttons` panel | Required | Present | ✅ Good |
| Query button | Required | Present | ⚠️ Wrong syntax |
| Add button | Required | Present | ⚠️ Wrong syntax |
| Edit button | Required | Missing | ❌ Missing |
| Delete button | Required | Present | ⚠️ Wrong syntax |
| Position attributes | Required | Present | ✅ Good |
| Korean labels | Expected | Present | ✅ Good |

### JavaScript Checklist

| Function | Benchmark | Generated | Status |
|----------|-----------|-----------|--------|
| `on_load` | Required | Missing | ❌ Missing |
| `fn_init` | Required | Missing | ❌ Missing |
| `fn_search` | Required | TODO stub | ⚠️ Incomplete |
| `fn_create/fn_add` | Required | TODO stub | ⚠️ Incomplete |
| `fn_edit` | Required | Missing | ❌ Missing |
| `fn_delete` | Required | TODO stub | ⚠️ Incomplete |
| `fn_onEditorClose` | Required | Missing | ❌ Missing |
| `grid_list_on_itemdblclick` | Required | Missing | ❌ Missing |
| Popup integration | Required | Missing | ❌ Missing |

---

## Score Breakdown

```
Category                Score   Notes
─────────────────────────────────────────────────────────
Structure & Layout      40%     Missing search panel, header
XML Syntax             20%     onclick, missing eventfunc:, script tag
JavaScript Quality     20%     Minimal stubs, missing functions
xFrame5 Conventions    30%     Wrong ID format, missing version
Production Readiness   15%     Would completely fail
Korean Localization    70%     Button labels correct
Grid Columns            0%     Empty - TODO only
─────────────────────────────────────────────────────────
OVERALL                28%     Significantly below qwen3-coder
```

---

## Model Comparison

| Metric | llama3.1 | codestral:22b | qwen3-coder:30b | Benchmark |
|--------|----------|---------------|-----------------|-----------|
| Components | 4 | 8 | 20+ | 40+ |
| Grid columns | 3 | 0 | 7 | 11 |
| Search panel | No | No | Yes | Yes |
| Button syntax | Wrong | Wrong | Wrong | Correct |
| JavaScript functions | 0 | 3 stubs | 4 stubs | 8 full |
| Korean labels | No | Yes | Yes | Yes |
| Popup integration | No | No | No | Yes |
| Overall score | ~20% | 28% | 56% | 100% |

### Ranking
1. **qwen3-coder:30b** - 56% (Best)
2. **codestral:22b** - 28%
3. **llama3.1** - ~20%

---

## Root Cause Analysis

### 1. Minimal Training on xFrame5
Codestral appears to have limited exposure to xFrame5 syntax, defaulting to generic HTML/JavaScript patterns.

### 2. TODO Placeholder Overuse
Instead of generating content, the model outputs TODO comments for complex sections (grid columns).

### 3. Script Tag Confusion
Mixing XML and JavaScript in a single output suggests confusion about xFrame5's file separation requirement.

### 4. Missing Few-Shot Examples
The model likely needs more concrete examples in the prompt to understand xFrame5 conventions.

---

## Recommendations

### For Codestral:22b Specifically

1. **Not recommended** for xFrame5 generation without significant prompt engineering
2. If used, require extensive post-processing
3. Consider using only for simpler tasks

### Prompt Improvements Needed

```
CRITICAL OUTPUT FORMAT:
1. XML file ONLY contains <screen> element - NO <script> tags
2. JavaScript is a SEPARATE output - NOT embedded in XML
3. Button events MUST use: on_click="eventfunc:fn_name()"
4. Grid columns are REQUIRED - never use TODO placeholders
```

### Post-Processing Required

If using codestral:22b output:
1. Remove any `<script>` tags from XML
2. Replace `onclick` with `on_click`
3. Add `eventfunc:` prefix to event handlers
4. Add `()` to function calls
5. Generate grid columns from schema
6. Add missing search panel
7. Fix screen ID format

---

## Test Artifacts

- **Input prompt:** "generate a simple task list"
- **Model:** codestral:22b
- **Temperature:** 0.7
- **Max tokens:** 8192
- **Response time:** ~45 seconds

---

## Conclusion

Codestral:22b is **not suitable** for xFrame5 code generation in its current state. The output:
- Lacks 70% of required components
- Contains multiple syntax errors
- Produces unusable grid (no columns)
- Confuses file separation requirements

**Recommendation:** Use qwen3-coder:30b instead, which scores 2x higher and produces usable output with minor corrections.

---

## Appendix: Side-by-Side Comparison

### Button Syntax

| Model | Output | Correct? |
|-------|--------|----------|
| Benchmark | `on_click="eventfunc:fn_search()"` | ✅ |
| qwen3-coder | `onclick="eventfunc:fn_search(objInst)"` | ❌ (onclick) |
| codestral | `onclick="fn_search"` | ❌ (onclick + no eventfunc + no parens) |

### Grid Definition

| Model | Columns Defined | Quality |
|-------|-----------------|---------|
| Benchmark | 11 columns with full attributes | Complete |
| qwen3-coder | 7 columns with formatters | Good |
| codestral | 0 columns (TODO comment) | Unusable |
