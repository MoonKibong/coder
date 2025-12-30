# LLM Model Comparison for xFrame5 Code Generation

**Date:** 2025-12-30 (Updated)
**Test:** xFrame5 List Screen Generation
**Prompt:** "generate a simple task list" (natural language)

---

## 🏆 Model Rankings

```
Rank  Model              Score   Progress Bar
─────────────────────────────────────────────────────
 1    devstral-2:123b    68%    ███████░░░  CORRECT button syntax! New leader
 2    llama3.1:70b       62%    ██████░░░░  Best structure, missing button events
 3    qwen3-coder:30b    56%    ██████░░░░  Most complete output, onclick issue
 4    codestral:22b      28%    ███░░░░░░░  Minimal output, multiple errors
```

---

## 📊 Detailed Comparison Matrix

| Metric | devstral-2:123b | llama3.1:70b | qwen3-coder:30b | codestral:22b | Benchmark |
|--------|:---------------:|:------------:|:---------------:|:-------------:|:---------:|
| **Overall Score** | **68%** | 62% | 56% | 28% | 100% |
| Components | 15 | 15 | 20+ | 8 | 40+ |
| Grid columns | 4 | 3 | 7 | 0 (TODO) | 11 |
| Search panel | ✅ | ✅ | ✅ | ❌ | ✅ |
| Screen ID format | ✅ `SCREEN_*` | ✅ `SCREEN_*` | ✅ `SCREEN_*` | ❌ lowercase | ✅ |
| Grid event syntax | ✅ `on_itemdblclick` | ✅ `on_itemdblclick` | ⚠️ `onclick` | ❌ `onclick` | ✅ |
| Button event syntax | ✅ **`on_click`** | ❌ missing | ⚠️ `onclick` | ❌ `onclick` + no prefix | ✅ `on_click` |
| JavaScript funcs | 3 stubs | 3 stubs | 4 stubs | 3 stubs | 8 full |
| Korean labels | ✅ (typo) | ✅ | ✅ | ✅ | ✅ |
| Popup integration | ❌ | ❌ | ❌ | ❌ | ✅ |
| Position attributes | ✅ | ✅ | ✅ | ✅ | ✅ |
| Data types | ✅ | ✅ | ✅ | ❌ | ✅ |

---

## 🔍 Key Findings by Model

### 1️⃣ devstral-2:123b (68%) - New Best Overall

**Strengths:**
- ✅ **Only model with correct `on_click` button syntax** - no post-processing needed
- ✅ Correct `on_itemdblclick` grid event syntax
- ✅ Proper `SCREEN_TASK_LIST` ID format
- ✅ Complete panel hierarchy with search
- ✅ 4 grid columns with proper bindings

**Weaknesses:**
- ⚠️ Font name typo: `맑은 고딭` instead of `맑은 고딕`
- ⚠️ Duplicate search button (in both pnl_search and pnl_buttons)
- ❌ Missing edit button
- ❌ JavaScript output contains malformed artifacts (`]]>`, `</script>`)

**Verdict:** Best model - correct event syntax eliminates the most critical post-processing need.

---

### 2️⃣ llama3.1:70b (62%) - Best Structure

**Strengths:**
- ✅ Only model with correct `on_itemdblclick` syntax on grid
- ✅ Proper `SCREEN_TASK_LIST` ID format
- ✅ Clean panel hierarchy with search
- ✅ Correct `data_type` values (0, 2, 3)

**Weaknesses:**
- ❌ Buttons have NO event handlers at all
- ❌ Missing edit button
- ❌ Only 3 grid columns vs 7-11 expected

**Verdict:** Best structure but buttons are non-functional without post-processing.

---

### 3️⃣ qwen3-coder:30b (56%) - Most Complete

**Strengths:**
- ✅ Most components generated (20+)
- ✅ 7 grid columns with formatters
- ✅ Combobox with status options
- ✅ Status code formatter (nice touch)

**Weaknesses:**
- ❌ Uses `onclick` instead of `on_click`
- ❌ `<xdataset>` instead of `<xlinkdataset>`
- ❌ Missing grid `version="1.1"`

**Verdict:** Most complete but wrong event attribute breaks all interactivity.

---

### 4️⃣ codestral:22b (28%) - Not Recommended

**Strengths:**
- ✅ Korean button labels
- ✅ Basic panel structure

**Weaknesses:**
- ❌ `onclick="fn_search"` - THREE errors in one attribute
- ❌ `<script>` tag embedded in XML (wrong)
- ❌ Grid has 0 columns (just TODO comment)
- ❌ Missing search panel entirely
- ❌ Wrong screen ID format

**Verdict:** Too many fundamental errors. Not suitable for xFrame5 generation.

---

## ⚠️ Common Issues Across All Models

| Issue | All Models | Fix |
|-------|------------|-----|
| No working button events | All fail here | Post-process to add `on_click` |
| TODO-only JavaScript | Stubs only | Enhance JS examples in prompt |
| Missing popup patterns | None generate | Add `loadpopup()` examples |
| No `on_load` handler | All missing | Add to prompt template |

---

## 📈 Score Breakdown by Category

```
Category             devstral:123b  llama3.1:70b  qwen3:30b  codestral:22b
─────────────────────────────────────────────────────────────────────────────
Structure & Layout      85%            85%          80%         40%
XML Syntax              80%            65%          60%         20%
JavaScript Quality      25%            30%          40%         20%
xFrame5 Conventions     75%            70%          60%         30%
Production Readiness    55%            50%          40%         15%
Korean Localization     85%            90%          90%         70%
Grid Implementation     80%            80%          80%          0%
─────────────────────────────────────────────────────────────────────────────
OVERALL                 68%            62%          56%         28%
```

---

## 💡 Recommendations

### Immediate Actions

1. **Use devstral-2:123b as primary model**
   - Only model with correct `on_click` button syntax out of the box
   - Eliminates need for event syntax post-processing
   - Add minor post-processor for font typo and JS cleanup

2. **Implement Minimal Post-Processing Pipeline**
   ```rust
   fn post_process_devstral(xml: &str, js: &str) -> (String, String) {
       let fixed_xml = xml
           // Fix font name typo
           .replace("맑은 고딭", "맑은 고딕");

       let fixed_js = js
           // Remove malformed artifacts
           .replace("]]>", "")
           .replace("</script>", "");

       (fixed_xml, fixed_js)
   }
   ```

3. **Fallback: llama3.1:70b with aggressive post-processing**
   ```rust
   fn post_process_llama(xml: &str) -> String {
       xml
           // Fix event attribute names
           .replace("onclick=", "on_click=")
           // Ensure eventfunc prefix
           .replace("on_click=\"fn_", "on_click=\"eventfunc:fn_")
   }
   ```

4. **Update Prompt Template** (for all models)
   ```
   CRITICAL RULES:
   - NEVER use "onclick" → ALWAYS use "on_click"
   - Every button MUST have: on_click="eventfunc:fn_name()"
   - Grid MUST have: version="1.1"
   - Font name is "맑은 고딕" (NOT "고딭")
   ```

### Medium-Term Improvements

1. **Enhance Knowledge Base**
   - Add more xFrame5 code samples for few-shot learning
   - Include complete screen examples with all patterns

2. **Consider Hybrid Approach**
   - Use llama3.1:70b for structure
   - Apply qwen3-coder patterns for completeness
   - Post-process all outputs for consistency

3. **Add Validation Gates**
   - [ ] No `onclick` attributes present
   - [ ] All grids have `version="1.1"`
   - [ ] All buttons have `on_click` handlers
   - [ ] JavaScript includes required functions

---

## 🎯 Quality Gap Analysis (vs Benchmark)

| Aspect | Best LLM (devstral-2:123b) | Benchmark | Gap |
|--------|----------------------------|-----------|-----|
| Components | 15 | 40+ | -62% |
| Grid columns | 4 | 11 | -64% |
| JS functions | 3 stubs | 8 full impl | -100% logic |
| Button events | ✅ Working syntax | All working | Minor fixes |
| Popup integration | None | Full | Missing |

---

## 📋 Test Configuration

| Parameter | Value |
|-----------|-------|
| Temperature | 0.7 |
| Max tokens | 8192 |
| Response time (avg) | 45-90 seconds |
| Ollama version | Latest |

---

## 📁 Related Files

- `devstral2_123b_evaluation_20251230.md` - Detailed devstral-2 evaluation (NEW LEADER)
- `llama31_70b_evaluation_20251230.md` - Detailed llama3.1 evaluation
- `qwen3_coder_30b_evaluation_20251230.md` - Detailed qwen3 evaluation
- `codestral_22b_evaluation_20251230.md` - Detailed codestral evaluation
- `samples/` - Generated XML/JS samples from each model

---

## Conclusion

**devstral-2:123b is the recommended model** for xFrame5 code generation. It is the **only model that generates correct `on_click` button event syntax** out of the box, eliminating the most critical post-processing need.

### Model Selection Guide

| Use Case | Recommended Model | Reason |
|----------|-------------------|--------|
| Production | devstral-2:123b | Correct event syntax, minimal fixes needed |
| Fallback | llama3.1:70b | Good structure, needs event syntax fixes |
| Most components | qwen3-coder:30b | Generates more, but all events wrong |

### Post-Processing Requirements

| Model | Event Syntax Fix | Font Fix | JS Cleanup | Effort |
|-------|:----------------:|:--------:|:----------:|:------:|
| devstral-2:123b | ❌ Not needed | ✅ Required | ✅ Required | Low |
| llama3.1:70b | ✅ Required | ❌ | ❌ | Medium |
| qwen3-coder:30b | ✅ Required | ❌ | ❌ | Medium |
| codestral:22b | ✅ Heavy fixes | ❌ | ❌ | High |

**Next Steps:**
1. Configure devstral-2:123b as primary model in LLM Config
2. Implement minimal post-processing (font fix + JS cleanup)
3. Update prompt templates with explicit rules for all models
4. Add quality validation gates before returning generated code
