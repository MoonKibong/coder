# Backend: OptionalField Pattern for Partial Updates

## Overview
All Rust admin services MUST use the `OptionalField<T>` pattern for optional fields in `UpdateParams` to support proper partial HTTP PATCH/PUT updates.

## The Problem

Using `Option<T>` alone cannot distinguish between:
- **Field not in JSON request**: Should skip update (preserve existing database value)
- **Field in JSON as `null`**: Should clear/set to `None` in database

### Example of the Bug

**Before (using Option<T>):**
```rust
#[derive(Deserialize)]
pub struct UpdateParams {
    pub description: Option<String>,
}

impl UpdateParams {
    fn apply(&self, item: &mut ActiveModel) {
        // BUG: If description is missing from request, this sets it to None!
        item.description = Set(self.description.clone());
    }
}
```

**Problem:**
```json
PATCH /admin/prompt-templates/1
{ "name": "Updated Name" }
```

Result: `description` gets set to `None` in database even though it wasn't in the request!

Why? Because serde deserializes missing fields as `None` for `Option<T>`.

## The Solution: OptionalField<T>

```rust
use crate::utils::OptionalField;

#[derive(Deserialize)]
pub struct UpdateParams {
    // Required fields (can be updated but never null)
    pub name: Option<String>,

    // Optional fields - use OptionalField for proper PATCH semantics
    #[serde(default)]  // CRITICAL!
    pub description: OptionalField<String>,
}

impl UpdateParams {
    fn apply(&self, item: &mut ActiveModel) {
        // Required field
        if let Some(name) = self.name.clone() {
            item.name = Set(name);
        }

        // Optional field - only update if Present (not Missing)
        if let OptionalField::Present(opt_value) = self.description.clone() {
            item.description = Set(opt_value);
        }
        // If Missing, nothing happens - preserves existing value
    }
}
```

**Now it works correctly:**
```json
PATCH /admin/prompt-templates/1
{ "name": "Updated Name" }
```
Result: Only `name` updated, `description` unchanged ✅

```json
PATCH /admin/prompt-templates/1
{ "description": null }
```
Result: `description` set to `None` in database ✅

```json
PATCH /admin/prompt-templates/1
{ "description": "New description" }
```
Result: `description` set to `Some("New description")` ✅

## Pattern Checklist

### Admin Services Status

**✅ Correct (uses OptionalField):**
- `prompt_template.rs` - screen_type, is_active
- `company_rule.rs` - naming_convention, additional_rules
- `llm_config.rs` - api_key, temperature, max_tokens, is_active
- `knowledge_base.rs` - component, section, relevance_tags, priority, token_estimate, is_active
- `user.rs` - No optional fields (all required)

## Implementation Steps

### 1. Add imports
```rust
use crate::utils::OptionalField;
```

### 2. Update UpdateParams struct

**For each optional field:**

Before:
```rust
pub struct UpdateParams {
    pub description: Option<String>,
    pub priority: Option<String>,
    pub is_active: Option<bool>,
}
```

After:
```rust
pub struct UpdateParams {
    // Required fields (never nullable in DB)
    pub name: Option<String>,

    // Optional fields - use OptionalField
    #[serde(default)]  // DON'T FORGET THIS!
    pub description: OptionalField<String>,
    #[serde(default)]
    pub priority: OptionalField<String>,
    #[serde(default)]
    pub is_active: OptionalField<bool>,
}
```

### 3. Update the update() method

Before:
```rust
fn update(&self, item: &mut ActiveModel) {
    if let Some(description) = params.description {
        item.description = Set(Some(description));
    }
    if params.priority.is_some() {  // BUG!
        item.priority = Set(params.priority);
    }
}
```

After:
```rust
fn update(&self, item: &mut ActiveModel) {
    // Required fields
    if let Some(name) = params.name {
        item.name = Set(name);
    }

    // Optional fields - only update if Present
    if let OptionalField::Present(opt_value) = params.description {
        item.description = Set(opt_value);
    }
    if let OptionalField::Present(opt_value) = params.priority {
        item.priority = Set(opt_value);
    }
}
```

## Type-Specific Notes

### For Clone types (String, etc.)
```rust
if let OptionalField::Present(opt_value) = self.description.clone() {
    item.description = Set(opt_value);
}
```

### For Copy types (i32, bool, f32, etc.)
```rust
if let OptionalField::Present(opt_value) = self.is_active {
    item.is_active = Set(opt_value);  // No .clone()
}
```

### For complex transformations
```rust
if let OptionalField::Present(opt_tags_str) = params.relevance_tags {
    let relevance_tags = opt_tags_str.map(|tags_str| {
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if tags.is_empty() {
            None
        } else {
            Some(JsonValue::Array(
                tags.into_iter().map(JsonValue::String).collect(),
            ))
        }
    }).flatten();

    active_model.relevance_tags = Set(relevance_tags);
}
```

## Frontend Requirements

**CRITICAL**: HTML forms must use `hx-ext="json-enc"` to send JSON:

```html
<!-- Create form -->
<form hx-post="/admin/prompt-templates" hx-ext="json-enc"
      hx-target="#search-result" hx-swap="outerHTML">
    <!-- form fields -->
</form>

<!-- Edit form -->
<form hx-put="/admin/prompt-templates/{{ item.id }}" hx-ext="json-enc"
      hx-target="#search-result" hx-swap="outerHTML">
    <!-- form fields -->
</form>
```

Without `hx-ext="json-enc"`, forms send Form data and controllers will fail with 415 Unsupported Media Type.

## Testing Partial Updates

### Test 1: Update only one field
```bash
PATCH /admin/prompt-templates/1
{
  "name": "Updated Name"
}
```

Expected: Only `name` changes, all other fields preserved.

### Test 2: Clear a field (set to null)
```bash
PATCH /admin/prompt-templates/1
{
  "screen_type": null
}
```

Expected: `screen_type` set to `None` in database.

### Test 3: Set a null field to a value
```bash
PATCH /admin/prompt-templates/1
{
  "screen_type": "list"
}
```

Expected: `screen_type` set to `Some("list")`.

## Why This Matters

### Without OptionalField (BAD):
- Frontend must always send ALL fields in PATCH/PUT requests
- Can't do partial updates
- Risk of accidentally clearing fields
- Brittle API contract

### With OptionalField (GOOD):
- Frontend can send only changed fields
- True partial update support
- Safe - won't accidentally clear fields
- Flexible API contract

## Implementation

The `OptionalField<T>` utility is located at:
- `src/utils/optional_field.rs` - Implementation
- `src/utils/mod.rs` - Module export
- `src/lib.rs` - Public module

---

**Pattern Status**: Implemented ✅
**Reference**: All admin services in `src/services/admin/`
**Documentation**: `docs/patterns/OPTIONALFIELD_PATTERN.md`
**Updated**: 2025-12-28
