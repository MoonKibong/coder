# Qwen3-Coder:30B Evaluation Report

**Date:** 2025-12-30
**Model:** qwen3-coder:30b (via Ollama)
**Test Type:** xFrame5 List Screen Generation
**Prompt:** "generate a simple task list" (natural language)

---

## Executive Summary

Qwen3-Coder:30B shows significant improvement over previous llama3.1 results, generating a complete screen structure with proper Korean localization. However, critical xFrame5 syntax issues remain that would prevent the code from working in production.

**Overall Score: 56%** (vs. llama3.1's ~20%)

---

## ⏺ Quality Comparison: qwen3-coder:30b Output vs Benchmark

### ✅ What's Good

| Aspect | Generated | Benchmark | Status |
|--------|-----------|-----------|--------|
| Screen root element | `<screen id="SCREEN_TASK_LIST">` | Required | ✅ Good |
| Datasets defined | `ds_search`, `ds_list` | Required | ✅ Good |
| Panel structure | `pnl_search`, `pnl_grid`, `pnl_buttons` | Required | ✅ Good |
| Grid with columns | 7 columns with headers | Required | ✅ Good |
| Korean labels | "업무명:", "상태:" | Good for ko lang | ✅ Good |
| Combobox with options | Status dropdown | Required | ✅ Good |
| Position attributes | x, y, width, height on all | Required | ✅ Good |
| Formatter for status | Code-to-text mapping | Nice touch | ⭐ Excellent |

### ❌ Issues Found

| Issue | Generated | Benchmark | Severity |
|-------|-----------|-----------|----------|
| Button event | `onclick="eventfunc:..."` | `on_click="eventfunc:..."` | 🔴 Critical |
| Dataset type | `<xdataset>` | `<xlinkdataset>` with columns | 🟡 Medium |
| Grid version | Missing | `version="1.1"` | 🟡 Medium |
| JS functions | Stub with TODO | Full implementation | 🟡 Medium |
| on_load handler | Missing | Required | 🟡 Medium |
| fn_init function | Missing | Required | 🟢 Low |
| Popup patterns | Missing `loadpopup()` | Full pattern | 🟡 Medium |

### 📊 Score Card

```
Structure & Layout:    ████████░░  80%
XML Syntax:            ██████░░░░  60%  (onclick vs on_click)
JavaScript:            ████░░░░░░  40%  (stubs only)
xFrame5 Conventions:   ██████░░░░  60%
Production Ready:      ████░░░░░░  40%
───────────────────────────────────────
Overall:               ██████░░░░  56%
```

### 🔧 Critical Fix Needed

The `onclick` attribute is **wrong** - xFrame5 uses `on_click`:

```xml
<!-- WRONG (generated) -->
<pushbutton onclick="eventfunc:fn_search(objInst)"/>

<!-- CORRECT (benchmark) -->
<pushbutton on_click="eventfunc:fn_search()"/>
```

### 💡 Recommendations

1. **Update prompt template** to explicitly forbid `onclick` and enforce `on_click`
2. **Add post-processing** to auto-correct `onclick` → `on_click`
3. **Enhance JS generation** - current output is mostly TODO stubs
4. Consider adding more examples to the knowledge base for qwen3-coder to learn from

---

## Generated Output

### XML Output
```xml
<?xml version="1.0" encoding="UTF-8"?>
<screen id="SCREEN_TASK_LIST" width="1000" height="700" script_language="Java">

  <!-- Datasets -->
  <xdataset id="ds_search" desc="검색조건"/>
  <xdataset id="ds_list" desc="업무목록"/>

  <!-- Search Panel -->
  <panel control_id="1" name="pnl_search"
         x="10" y="10" width="980" height="60"
         back_color="00F0F0F0" border="1">
    <text control_id="2" name="txt_task_name_label"
          x="10" y="20" width="80" height="24"
          text="업무명:" font="맑은 고딕,9,0,0,0,0"/>
    <normal_field control_id="3" name="field_search_task_name"
                  x="100" y="20" width="200" height="24"
                  max_length="100" font="맑은 고딕,9,0,0,0,0"/>
    <combobox control_id="5" name="cbo_search_status"
              x="390" y="20" width="120" height="24"
              link_data="ds_search:STATUS">
      <option value="" text="전체"/>
      <option value="0" text="대기"/>
      <option value="1" text="진행"/>
      <option value="2" text="완료"/>
    </combobox>
    <pushbutton control_id="6" name="btn_search"
                x="530" y="18" width="80" height="28"
                text="조회" font="맑은 고딕,9,0,0,0,0"
                onclick="eventfunc:fn_search(objInst)"/>  <!-- WRONG: should be on_click -->
  </panel>

  <!-- Grid Panel with 7 columns -->
  <panel control_id="10" name="pnl_grid"
         x="10" y="80" width="980" height="500">
    <grid control_id="11" name="grid_list"
          x="0" y="0" width="980" height="500"
          link_data="ds_list"
          linenumber_show="1"
          use_checkrow="1"
          on_itemdblclick="eventfunc:fn_edit(...)">
      <!-- 7 columns: TASK_ID, TASK_NAME, STATUS, ASSIGNEE, START_DATE, END_DATE, PROGRESS -->
    </grid>
  </panel>

  <!-- Button Panel -->
  <panel control_id="20" name="pnl_buttons"
         x="10" y="590" width="980" height="40">
    <pushbutton control_id="21" name="btn_add" text="신규" onclick="eventfunc:fn_add(objInst)"/>
    <pushbutton control_id="22" name="btn_edit" text="수정" onclick="eventfunc:fn_edit(...)"/>
    <pushbutton control_id="23" name="btn_delete" text="삭제" onclick="eventfunc:fn_delete(objInst)"/>
  </panel>

</screen>
```

### JavaScript Output
```javascript
function fn_search(objInst) {
    // TODO: Implement search functionality
}

function fn_add(objInst) {
    // TODO: Implement add functionality
}

function fn_edit(objInst, nRow, nColumn, buttonClick, imageIndex) {
    // TODO: Implement edit functionality
}

function fn_delete(objInst) {
    // TODO: Implement delete functionality
}
```

---

## Detailed Comparison

### What's Working Well

| Aspect | Generated | Benchmark | Assessment |
|--------|-----------|-----------|------------|
| Screen root element | `<screen id="SCREEN_TASK_LIST">` | Required | ✅ Good |
| Dataset definitions | `ds_search`, `ds_list` | Required | ✅ Good |
| Panel structure | `pnl_search`, `pnl_grid`, `pnl_buttons` | Required | ✅ Good |
| Grid with columns | 7 columns with proper headers | Required | ✅ Good |
| Korean localization | "업무명:", "상태:", "조회" | Expected for ko lang | ✅ Excellent |
| Position attributes | x, y, width, height on all components | Required | ✅ Good |
| Status formatter | Code-to-text mapping in grid | Nice to have | ✅ Excellent |
| Combobox options | Dropdown with status values | Required | ✅ Good |
| Naming conventions | `btn_`, `pnl_`, `grid_`, `field_` prefixes | Required | ✅ Good |

### Critical Issues

| Issue | Generated | Benchmark | Severity |
|-------|-----------|-----------|----------|
| Button event attribute | `onclick="eventfunc:..."` | `on_click="eventfunc:..."` | **CRITICAL** |
| Dataset element | `<xdataset>` | `<xlinkdataset columns="...">` | Medium |
| Grid version attribute | Missing | `version="1.1"` | Medium |
| JavaScript implementation | TODO stubs only | Full implementation | Medium |
| `on_load` handler | Missing | Required | Medium |
| `fn_init` function | Missing | Required | Low |
| Popup integration | Missing `loadpopup()` | Full pattern with callbacks | Medium |

---

## Score Breakdown

```
Category                Score   Notes
─────────────────────────────────────────────────────────
Structure & Layout      80%     Complete panel hierarchy
XML Syntax             60%     onclick vs on_click issue
JavaScript Quality     40%     Stubs only, no real logic
xFrame5 Conventions    60%     Missing version, wrong dataset type
Production Readiness   40%     Would fail at runtime
Korean Localization    90%     Proper labels and terms
─────────────────────────────────────────────────────────
OVERALL                56%     Major improvement over llama3.1
```

---

## Comparison with Previous Model (llama3.1)

| Metric | llama3.1 | qwen3-coder:30b | Improvement |
|--------|----------|-----------------|-------------|
| Components generated | 4 | 20+ | +400% |
| Dataset columns | 3 | 7 | +133% |
| Positioning | None | Complete | Fixed |
| Panel structure | Missing | Complete | Fixed |
| JavaScript | None | Stubs | Partial |
| Korean support | None | Complete | Fixed |
| Event syntax | Wrong | Wrong | No change |
| Overall score | ~20% | 56% | +180% |

---

## Root Cause Analysis

### 1. `onclick` vs `on_click` Issue

The model consistently uses HTML-style `onclick` instead of xFrame5's `on_click`. This is likely because:
- General web training data uses `onclick`
- The prompt template examples may not be emphasized enough
- No explicit "NEVER use onclick" instruction

**Fix:** Add explicit negative instruction in prompt:
```
CRITICAL: NEVER use "onclick" - this is WRONG
ALWAYS use "on_click" - this is CORRECT for xFrame5
```

### 2. TODO-Only JavaScript

The model generates function stubs instead of implementations because:
- The prompt may not include enough JS examples
- Natural language prompts don't specify implementation details

**Fix:** Include complete JS patterns in system prompt.

### 3. Missing `xlinkdataset` Columns

The model uses simple `<xdataset>` without the column definition string format.

**Fix:** Add explicit dataset syntax example with columns attribute.

---

## Recommendations

### Immediate Actions

1. **Add post-processor** to auto-correct `onclick` → `on_click`
   ```rust
   fn fix_event_syntax(xml: &str) -> String {
       xml.replace("onclick=", "on_click=")
   }
   ```

2. **Update prompt template** with explicit prohibition:
   ```
   WRONG: onclick="eventfunc:fn_search()"
   CORRECT: on_click="eventfunc:fn_search()"
   ```

3. **Add grid version** to post-processor if missing

### Medium-Term Improvements

1. Enhance JavaScript examples in knowledge base
2. Add more xFrame5 code samples for few-shot learning
3. Consider fine-tuning or LoRA for xFrame5-specific syntax

### Quality Gates

Before returning generated code, validate:
- [ ] No `onclick` attributes present
- [ ] All grids have `version="1.1"`
- [ ] All datasets use `xlinkdataset` format
- [ ] JavaScript includes `on_load` and `fn_init`

---

## Test Artifacts

- **Input prompt:** "generate a simple task list"
- **Model:** qwen3-coder:30b
- **Temperature:** 0.7
- **Max tokens:** 8192
- **Response time:** ~67 seconds

---

## Conclusion

Qwen3-Coder:30B represents a significant improvement over llama3.1 for xFrame5 code generation:
- Generates complete screen structures
- Proper Korean localization
- Correct naming conventions
- Good understanding of list screen patterns

However, the critical `onclick` vs `on_click` syntax error means generated code **will not work** without post-processing. This should be the highest priority fix.

**Recommendation:** Implement automatic post-processing correction while also updating prompt templates for long-term improvement.
