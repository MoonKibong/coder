# Devstral-2:123B Evaluation Report

**Date:** 2025-12-30
**Model:** devstral-2:123b (via Ollama)
**Test Type:** xFrame5 List Screen Generation
**Prompt:** "generate a simple task list" (natural language)

---

## Executive Summary

Devstral-2:123B is the **first model to generate correct button event syntax** (`on_click="eventfunc:..."`) out of the box. It produces a well-structured screen with proper component hierarchy, correct xFrame5 patterns, and Korean localization. Some minor issues exist with font name typo, duplicate button, and JavaScript quality.

**Overall Score: 68%** (New best performer!)

---

## ⏺ Quality Comparison: devstral-2:123b Output vs Benchmark

### ✅ What's Good

| Aspect | Generated | Benchmark | Status |
|--------|-----------|-----------|--------|
| Screen ID format | `SCREEN_TASK_LIST` | Required | ✅ Correct |
| Dataset definitions | `ds_search`, `ds_list` | Required | ✅ Good |
| Search panel | Present with label + field + button | Required | ✅ Good |
| Panel structure | `pnl_search`, `pnl_grid`, `pnl_buttons` | Required | ✅ Good |
| Grid columns | 4 columns with proper binding | Required | ✅ Good |
| **Button event syntax** | `on_click="eventfunc:fn_search(objInst)"` | Required | ⭐ **CORRECT!** |
| Grid double-click | `on_itemdblclick="eventfunc:fn_edit(...)"` | Required | ✅ Correct |
| Position attributes | All components have x,y,width,height | Required | ✅ Good |
| Korean labels | "작업명:", "조회", "신규", "삭제" | Expected | ✅ Good |
| Naming conventions | `btn_`, `pnl_`, `grid_`, `field_`, `txt_` | Required | ✅ Good |

### ❌ Issues Found

| Issue | Generated | Benchmark | Severity |
|-------|-----------|-----------|----------|
| Font name typo | `맑은 고딭` | `맑은 고딕` | 🟡 Medium |
| Duplicate search button | In both pnl_search AND pnl_buttons | One location | 🟡 Medium |
| Missing edit button | None | Required | 🟡 Medium |
| Grid version | Missing | `version="1.1"` | 🟢 Low |
| JavaScript quality | Stubs with console.log | Full implementation | 🟡 Medium |
| JS output malformed | Contains `]]>` and `</script>` | Clean JS only | 🟡 Medium |
| Missing fn_edit | Not defined | Required (referenced in grid) | 🟡 Medium |
| `on_load` handler | Missing | Required | 🟡 Medium |

### 📊 Score Card

```
Structure & Layout:    █████████░  85%  (complete panel hierarchy)
XML Syntax:            ████████░░  80%  (CORRECT on_click! minor typos)
JavaScript:            ███░░░░░░░  25%  (stubs only, malformed output)
xFrame5 Conventions:   ████████░░  75%  (correct ID, correct events)
Production Ready:      ██████░░░░  55%  (would partially work)
Korean Localization:   █████████░  85%  (good labels, font typo)
Grid Implementation:   ████████░░  80%  (4 columns with bindings)
───────────────────────────────────────
Overall:               ███████░░░  68%
```

### 🔧 Key Achievement: Correct Button Event Syntax

**This is the ONLY model to generate correct xFrame5 button syntax:**

```xml
<!-- CORRECT (devstral-2:123b) -->
<pushbutton on_click="eventfunc:fn_search(objInst)"/>

<!-- WRONG (all other models) -->
<pushbutton onclick="eventfunc:fn_search(objInst)"/>  <!-- qwen3 -->
<pushbutton onclick="fn_search"/>                      <!-- codestral -->
<pushbutton />                                         <!-- llama3.1 - missing entirely -->
```

### 💡 Recommendations

1. **Fix font name typo** via post-processing:
   ```rust
   xml.replace("맑은 고딭", "맑은 고딕")
   ```

2. **Remove duplicate search button** - keep only in pnl_search OR pnl_buttons

3. **Add missing edit button** to button panel

4. **Clean JavaScript output** - remove malformed `]]>` and `</script>` artifacts

5. **Add grid version="1.1"** attribute

---

## Generated Output

### XML Output
```xml
<screen id="SCREEN_TASK_LIST" width="800" height="600" script_language="Java">

  <!-- Datasets -->
  <xdataset id="ds_search" desc="검색조건">
    <!-- TODO: Add search criteria columns -->
  </xdataset>

  <xdataset id="ds_list" desc="작업목록">
    <!-- TODO: Add task list columns -->
  </xdataset>

  <!-- Search Panel -->
  <panel control_id="1" name="pnl_search"
         x="10" y="10" width="780" height="60"
         back_color="00F0F0F0" border="1">

    <text control_id="2" name="txt_task_name_label"
          x="10" y="20" width="80" height="24"
          text="작업명:" font="맑은 고딭,9,0,0,0,0"/>  <!-- TYPO: 고딭 → 고딕 -->

    <normal_field control_id="3" name="field_search_task_name"
                  x="100" y="20" width="200" height="24"
                  max_length="50" font="맑은 고딭,9,0,0,0,0"/>

    <pushbutton control_id="4" name="btn_search"
                x="320" y="18" width="80" height="28"
                text="조회" font="맑은 고딭,9,0,0,0,0"
                on_click="eventfunc:fn_search(objInst)"/>  <!-- ✅ CORRECT! -->
  </panel>

  <!-- Grid Panel -->
  <panel control_id="10" name="pnl_grid"
         x="10" y="80" width="780" height="450">

    <grid control_id="11" name="grid_list"
          x="0" y="0" width="780" height="450"
          link_data="ds_list"
          linenumber_show="1"
          use_checkrow="1"
          on_itemdblclick="eventfunc:fn_edit(objInst, nRow, nColumn, buttonClick, imageIndex)">

      <column>
        <header title="작업ID"/>
        <data name="TASK_ID" link_data="ds_list:TASK_ID"
              width="100" text_horzalign="1" data_type="2"/>
      </column>

      <column>
        <header title="작업명"/>
        <data name="TASK_NAME" link_data="ds_list:TASK_NAME"
              width="150" text_horzalign="1" data_type="2"/>
      </column>

      <column>
        <header title="상태"/>
        <data name="STATUS" link_data="ds_list:STATUS"
              width="100" text_horzalign="1" data_type="2"/>
      </column>

      <column>
        <header title="등록일"/>
        <data name="REG_DATE" link_data="ds_list:REG_DATE"
              width="120" text_horzalign="1" data_type="2"/>
      </column>

    </grid>
  </panel>

  <!-- Button Panel -->
  <panel control_id="20" name="pnl_buttons"
         x="10" y="540" width="780" height="40">

    <pushbutton control_id="21" name="btn_search"  <!-- DUPLICATE -->
                x="10" y="8" width="80" height="28"
                text="조회" font="맑은 고딭,9,0,0,0,0"
                on_click="eventfunc:fn_search(objInst)"/>

    <pushbutton control_id="22" name="btn_add"
                x="100" y="8" width="80" height="28"
                text="신규" font="맑은 고딭,9,0,0,0,0"
                on_click="eventfunc:fn_add(objInst)"/>

    <pushbutton control_id="23" name="btn_delete"
                x="190" y="8" width="80" height="28"
                text="삭제" font="맑은 고딭,9,0,0,0,0"
                on_click="eventfunc:fn_delete(objInst)"/>
  </panel>

</screen>
```

### JavaScript Output
```javascript
// NOTE: Output contains malformed artifacts (]]>, </script>) - cleaned below

this.fn_search = function() {
    // TODO: Implement 조회 functionality
    console.log('fn_search');
};

this.fn_add = function() {
    // TODO: Implement 신규 functionality
    console.log('fn_add');
};

this.fn_delete = function() {
    // TODO: Implement 삭제 functionality
    console.log('fn_delete');
};

// MISSING: fn_edit (referenced in grid on_itemdblclick)
```

---

## Detailed Comparison

### What's Working Well

| Aspect | Generated | Benchmark | Assessment |
|--------|-----------|-----------|------------|
| Screen ID format | `SCREEN_TASK_LIST` | Required | ✅ Correct |
| Button event syntax | `on_click="eventfunc:..."` | Required | ⭐ **First model to get this right!** |
| Grid event syntax | `on_itemdblclick="eventfunc:..."` | Required | ✅ Correct |
| Search panel | Complete with label, field, button | Required | ✅ Good |
| Panel hierarchy | Three logical panels | Required | ✅ Good |
| Grid columns | 4 columns with bindings | Required | ✅ Good |
| Position attributes | All present | Required | ✅ Good |
| Korean labels | Proper Korean text | Expected | ✅ Good |

### Issues Found

| Issue | Generated | Benchmark | Severity |
|-------|-----------|-----------|----------|
| Font name | `맑은 고딭` (typo) | `맑은 고딕` | Medium |
| Duplicate button | btn_search in 2 panels | One location | Medium |
| Edit button | Missing | Required | Medium |
| Grid version | Missing | `version="1.1"` | Low |
| JavaScript | Stubs + malformed | Full implementation | Medium |
| fn_edit | Not defined | Required | Medium |
| on_load | Missing | Required | Medium |

---

## Score Breakdown

```
Category                Score   Notes
─────────────────────────────────────────────────────────
Structure & Layout      85%     Complete panel hierarchy with search
XML Syntax             80%     CORRECT on_click! Font typo, no version
JavaScript Quality     25%     Stubs only, malformed output artifacts
xFrame5 Conventions    75%     Correct ID format, correct event syntax
Production Readiness   55%     Would work after minor fixes
Korean Localization    85%     Good labels, font name typo
Grid Implementation    80%     4 columns with proper bindings
─────────────────────────────────────────────────────────
OVERALL                68%     New best performer!
```

---

## Model Comparison (Updated)

| Metric | devstral-2:123b | llama3.1:70b | qwen3:30b | codestral:22b | Benchmark |
|--------|:---------------:|:------------:|:---------:|:-------------:|:---------:|
| **Overall Score** | **68%** | 62% | 56% | 28% | 100% |
| Button event syntax | ✅ `on_click` | ❌ missing | ⚠️ `onclick` | ❌ `onclick` | ✅ |
| Grid event syntax | ✅ correct | ✅ correct | ⚠️ `onclick` | ❌ wrong | ✅ |
| Screen ID format | ✅ | ✅ | ✅ | ❌ | ✅ |
| Grid columns | 4 | 3 | 7 | 0 | 11 |
| Search panel | ✅ | ✅ | ✅ | ❌ | ✅ |
| JavaScript funcs | 3 stubs | 3 stubs | 4 stubs | 3 stubs | 8 full |
| Korean labels | ✅ | ✅ | ✅ | ✅ | ✅ |

### 🏆 Updated Rankings

```
Rank  Model              Score   Progress Bar
─────────────────────────────────────────────────────
 1    devstral-2:123b    68%    ███████░░░  CORRECT button syntax!
 2    llama3.1:70b       62%    ██████░░░░  Best structure, missing buttons
 3    qwen3-coder:30b    56%    ██████░░░░  Most complete, onclick issue
 4    codestral:22b      28%    ███░░░░░░░  Minimal output, multiple errors
```

---

## Why Devstral-2 is the New Leader

### 1. Correct Event Attribute Name
Only model to use `on_click` instead of `onclick`:
```xml
on_click="eventfunc:fn_search(objInst)"  ✅
```

### 2. Correct Event Handler Format
Uses the proper `eventfunc:` prefix with function call:
```xml
eventfunc:fn_search(objInst)  ✅
```

### 3. Complete Grid Event Binding
Correct `on_itemdblclick` with all parameters:
```xml
on_itemdblclick="eventfunc:fn_edit(objInst, nRow, nColumn, buttonClick, imageIndex)"  ✅
```

---

## Post-Processing Required

Despite being the best performer, some fixes are still needed:

```rust
fn post_process_devstral(xml: &str, js: &str) -> (String, String) {
    let fixed_xml = xml
        // Fix font name typo
        .replace("맑은 고딭", "맑은 고딕")
        // Add grid version if missing
        // Remove duplicate search button
        ;

    let fixed_js = js
        // Remove ]]> artifacts
        .replace("]]>", "")
        // Remove </script> artifacts
        .replace("</script>", "")
        // Add missing fn_edit function
        ;

    (fixed_xml, fixed_js)
}
```

---

## Test Artifacts

- **Input prompt:** "generate a simple task list"
- **Model:** devstral-2:123b
- **Temperature:** 0.7
- **Max tokens:** 8192
- **Response time:** TBD

---

## Conclusion

Devstral-2:123B is the **recommended model** for xFrame5 code generation:

1. ✅ **Only model with correct button event syntax** - no post-processing needed for `on_click`
2. ✅ Good structure with complete panel hierarchy
3. ✅ Proper Korean localization
4. ⚠️ Minor issues (font typo, duplicate button, JS quality)

**Recommendation:** Use devstral-2:123b as the primary model with minimal post-processing for:
- Font name correction (`고딭` → `고딕`)
- JavaScript cleanup (remove artifacts)
- Add missing edit button

This model reduces the need for aggressive post-processing that other models require for event syntax correction.
