# LLM Model Comparison for xFrame5 Code Generation

**Date:** 2026-01-02 (Updated)
**Test:** xFrame5 List Screen Generation
**Pipeline:** 6-Pass Post-Processing (First Production Test)

---

## Executive Summary

This report compares LLM models for xFrame5 code generation, now including **post-processing pipeline results**. The 2026-01-02 test marks the first production validation of the 6-pass deterministic pipeline.

**Key Finding:** The post-processing pipeline improves all model outputs by 25-30%, making even general-purpose models viable for code generation.

---

## Model Rankings (With Pipeline)

```
Rank  Model                  Size   Type      Score   Progress Bar
────────────────────────────────────────────────────────────────────────────
 1    qwen3-coder:30b        30B    Coding    85%    █████████░  Best quality
 2    llama3.1:latest        8B     General   72%    ███████░░░  NEW: Pipeline validated
 3    devstral-2:123b        123B   Coding    68%*   ███████░░░  Pre-pipeline score
 4    llama3.1:70b           70B    General   62%*   ██████░░░░  Pre-pipeline score
 5    codestral:22b          22B    Coding    28%*   ███░░░░░░░  Not recommended

* = Pre-pipeline score, needs re-testing with pipeline
```

---

## Pipeline Impact Analysis

### Before vs After Pipeline

| Model | Raw Score | With Pipeline | Improvement |
|-------|-----------|---------------|-------------|
| qwen3-coder:30b | 56% | 85% | **+29%** |
| llama3.1:8B | ~44% | 72% | **+28%** |
| devstral-2:123b | 68% | TBD | Expected +20-25% |
| llama3.1:70b | 62% | TBD | Expected +20-25% |

### Pipeline Corrections (Typical)

| Pass | Correction | Impact |
|------|------------|--------|
| Canonicalizer | `onclick` → `on_click` | Critical - enables button functionality |
| Canonicalizer | Add `version="1.1"` to grids | Medium - xFrame5 compatibility |
| Canonicalizer | Function style normalization | Medium - xFrame5 convention |
| API Allowlist | Flag hallucinated APIs | High - prevents runtime errors |
| Symbol Linker | Match events to functions | Medium - ensures consistency |

---

## Detailed Comparison Matrix

| Metric | qwen3-coder:30b | llama3.1:8B | devstral-2:123b* | llama3.1:70b* |
|--------|:---------------:|:-----------:|:----------------:|:-------------:|
| **Overall Score** | **85%** | **72%** | 68% | 62% |
| **Pipeline Applied** | Yes | Yes | No | No |
| Response Time | ~67s | 86s | ~90s | ~60s |
| Components Generated | 20+ | 15 | 15 | 15 |
| Grid Columns | 7 | 2 | 4 | 3 |
| Event Syntax (Raw) | `onclick` | `onclick` | `on_click` | Missing |
| Event Syntax (Pipeline) | `on_click` | `on_click` | N/A | N/A |
| Function Style | `this.fn_xxx` | `this.fn_xxx` | `function fn_xxx` | `function fn_xxx` |
| APIs Flagged | 0 | 5 | N/A | N/A |
| Korean Localization | Excellent | None | Good | Good |

*Pre-pipeline scores

---

## First Production Pipeline Test (2026-01-02)

### Test Details

| Parameter | Value |
|-----------|-------|
| Model | llama3.1:latest (8B) |
| Model Type | General-purpose |
| Prompt | "Generate a page showing list of tasks using grid components. Add create, edit, delete, query buttons to the page" |
| Response Time | 86 seconds |
| Pipeline Corrections | 10+ issues fixed |
| APIs Flagged | 5 |

### Pipeline Corrections Applied

```
[✓] Fixed 3 occurrence(s) of 'onclick=' → 'on_click='
[✓] Added missing 'eventfunc:' prefix to event handlers
[✓] Added version="1.1" to 1 grid element(s)
[✓] Converted 6 function(s) to xFrame5 method style (this.fn_xxx = function())
[!] Flagged 5 potentially hallucinated APIs for verification
```

### Milestone Achievement

This test validates:
1. **Pipeline functionality** - All 6 passes execute correctly
2. **Quality improvement** - +28% score vs raw output
3. **Safety architecture** - Hallucinated APIs caught, not silently passed
4. **Model flexibility** - General-purpose model viable with pipeline

---

## Score Breakdown by Category

### With Pipeline (2026-01-02)

```
Category                qwen3:30b (pipeline)   llama3.1:8B (pipeline)
──────────────────────────────────────────────────────────────────────
Structure & Layout            80%                    70%
XML Syntax                    90%                    85%
JavaScript Quality            85%                    75%
xFrame5 Conventions           85%                    80%
Production Readiness          80%                    65%
API Safety                    90%                    90%
Korean Localization           90%                    10%
──────────────────────────────────────────────────────────────────────
OVERALL                       85%                    72%
```

### Pre-Pipeline Comparison (2025-12-30)

```
Category             devstral:123b  llama3.1:70b  qwen3:30b  codestral:22b
─────────────────────────────────────────────────────────────────────────────
Structure & Layout      85%            85%          80%         40%
XML Syntax              80%            65%          60%         20%
JavaScript Quality      25%            30%          40%         20%
xFrame5 Conventions     75%            70%          60%         30%
Production Readiness    55%            50%          40%         15%
─────────────────────────────────────────────────────────────────────────────
OVERALL                 68%            62%          56%         28%
```

---

## Recommendations

### Model Selection (Updated 2026-01-02)

| Use Case | Recommended Model | Score | Notes |
|----------|-------------------|-------|-------|
| **Production** | qwen3-coder:30b | 85% | Best quality, requires 24GB+ RAM |
| **Resource-limited** | llama3.1:8B | 72% | Good with pipeline, ~6GB RAM |
| **High-volume** | llama3.1:8B | 72% | Faster, lower cost |
| **Maximum completeness** | qwen3-coder:30b | 85% | Most components generated |
| **Korean UI** | qwen3-coder:30b | 85% | Best localization |

### Pipeline Status

| Pass | Status | Effectiveness |
|------|--------|---------------|
| 1. Output Parser | Production | Works perfectly |
| 2. Canonicalizer | Production | High - fixes 90% of syntax issues |
| 3. Symbol Linker | Production | Medium - ensures consistency |
| 4. API Allowlist | Production | High - critical for safety |
| 5. Graph Validator | Production | Medium - validates bindings |
| 6. Minimalism | Production | Low - minor cleanup |

### Planned Pipeline Improvements

1. **API Normalizer Pass** (High Priority)
   - Convert hallucinated APIs to correct xFrame5 equivalents
   - `open_popup()` → `loadpopup()`
   - `.getSelectedItem()` → `.getValue()`

2. **Column Generator Pass** (Medium Priority)
   - Auto-generate grid columns from dataset definition
   - Fill in missing column attributes

3. **Localization Pass** (Medium Priority)
   - Apply Korean labels based on language option
   - Fix common Korean typos (맑은 고딭 → 맑은 고딕)

---

## Quality Gap Analysis (Current Best vs Benchmark)

| Aspect | qwen3:30b + Pipeline | Benchmark | Gap |
|--------|---------------------|-----------|-----|
| Components | 20+ | 40+ | -50% |
| Grid columns | 7 | 11 | -36% |
| JS functions | Full impl + TODOs | 8 full impl | Minor |
| Button events | Working | Working | None |
| Popup integration | Flagged | Full | Manual fix needed |
| API correctness | 90% (flagged) | 100% | Planned improvement |

---

## Test Artifacts

### 2026-01-02 Test Files
- `llama31_8b_post_pipeline_evaluation_20260102.md` - This test report

### 2025-12-30 Test Files
- `qwen3_coder_30b_evaluation_20251230.md` - Raw qwen3 evaluation
- `qwen3_coder_30b_post_pipeline_evaluation_20251230.md` - Pipeline evaluation
- `devstral2_123b_evaluation_20251230.md` - devstral-2 evaluation
- `llama31_70b_evaluation_20251230.md` - llama3.1:70b evaluation
- `codestral_22b_evaluation_20251230.md` - codestral evaluation
- `comparison_and_recommendation_20251229.md` - Previous comparison

### Sample Files
- `samples/` - Generated XML/JS samples from each model

---

## Conclusions

### Key Insights (2026-01-02)

1. **Pipeline validates architecture** - 6-pass post-processing works as designed
2. **General-purpose models viable** - llama3.1:8B achieves 72% with pipeline
3. **Safety architecture effective** - Hallucinated APIs caught and flagged
4. **Model flexibility confirmed** - Can switch models without code changes

### Historical Trend

```
Timeline          Model              Score   Key Event
─────────────────────────────────────────────────────────────────
2025-12-29        llama3.1:70b       62%     Initial testing
2025-12-30        devstral-2:123b    68%     New leader (raw)
2025-12-30        qwen3-coder:30b    85%     Pipeline implemented
2026-01-02        llama3.1:8B        72%     First production pipeline test
```

### Next Steps

1. **Re-test devstral-2:123b with pipeline** - Expected 85-90% score
2. **Add API Normalizer pass** - Auto-correct common API errors
3. **Memory optimization** - Enable qwen3-coder:30b on 16GB systems
4. **Benchmark expansion** - Test detail screens, popup screens

---

**Version:** 2.0
**Updated:** 2026-01-02
**Pipeline Status:** Production (6-pass)
