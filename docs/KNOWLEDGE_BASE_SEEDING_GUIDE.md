# Knowledge Base Seeding Guide

**Purpose**: Step-by-step guide for seeding and adding knowledge base entries using Loco.rs fixtures
**Date**: 2025-12-28

---

## Overview

The knowledge base stores xFrame5 framework documentation that the AI agent uses during code generation. Instead of including all knowledge in every prompt, we selectively query relevant entries based on screen type and components needed.

**Storage Method**: YAML fixtures (Loco.rs best practice)
**Location**: `backend/src/fixtures/knowledge_bases.yaml`

---

## Table Schema

**Table Name**: `knowledge_bases` (plural, following Loco.rs convention)

**Key Fields**:
- `name` - Unique identifier (e.g., "dataset_basic_syntax")
- `category` - Type: "component", "architecture", "pattern", "example", "standard"
- `component` - Component name (e.g., "dataset", "grid", "button")
- `section` - Sub-section (e.g., "basic_syntax", "events", "api_methods")
- `content` - Markdown documentation content
- `relevance_tags` - JSON array of when to include (e.g., ["list_screen", "detail_screen"])
- `priority` - Importance: "high", "medium", "low"
- `token_estimate` - Approximate token count for budgeting

---

## Initial Setup (Already Done)

The knowledge base was created using Loco.rs scaffold:

```bash
cargo loco generate scaffold knowledge_base \
  name:string! \
  category:string! \
  component:string \
  section:string \
  content:text! \
  relevance_tags:json \
  priority:string \
  token_estimate:int \
  version:int \
  is_active:bool \
  --api
```

This generated:
- ✅ Migration: `migration/src/m20251228_125645_knowledge_bases.rs`
- ✅ Model: `src/models/_entities/knowledge_bases.rs`
- ✅ Controller: `src/controllers/knowledge_base.rs`
- ✅ Tests: `tests/models/knowledge_bases.rs` and `tests/requests/knowledge_base.rs`

---

## Seeding Process

### Step 1: Ensure App Configuration

The `App::seed()` function in `src/app.rs` must include knowledge_bases:

```rust
async fn seed(ctx: &AppContext, base: &Path) -> Result<()> {
    db::seed::<users::ActiveModel>(&ctx.db, &base.join("users.yaml").display().to_string())
        .await?;

    // Add this line:
    db::seed::<knowledge_bases::ActiveModel>(&ctx.db, &base.join("knowledge_bases.yaml").display().to_string())
        .await?;

    Ok(())
}
```

Also add to imports:
```rust
use crate::{
    controllers, initializers,
    models::_entities::{users, knowledge_bases},  // Add knowledge_bases here
    services, tasks,
    workers::downloader::DownloadWorker,
};
```

### Step 2: Run Database Seed

```bash
cd backend

# Seed without truncating (incremental)
cargo loco db seed

# Seed with reset (truncate first)
cargo loco db seed --reset
```

### Step 3: Verify Seeding

**Option 1: Via API**
```bash
curl http://localhost:3000/api/knowledge_bases | grep -o '"id":' | wc -l
# Should return 7 (or your total count)
```

**Option 2: Via Test**
```bash
cargo test models::knowledge_bases::test_model
```

---

## YAML Fixture Format

### File Location
`backend/src/fixtures/knowledge_bases.yaml`

### Basic Structure

```yaml
---
- id: 1
  name: entry_name_here
  category: component
  component: dataset
  section: basic_syntax
  content: |
    # Component Name

    ## Description
    Content here...

    ```xml
    <example/>
    ```
  relevance_tags: ["list_screen", "detail_screen"]
  priority: high
  token_estimate: 500
  version: 1
  is_active: true
  created_at: "2025-12-28T12:00:00.000Z"
  updated_at: "2025-12-28T12:00:00.000Z"
```

### Field Guidelines

**name** (required):
- Use snake_case
- Pattern: `{component}_{section}` or `{category}_{topic}`
- Examples: `dataset_basic_syntax`, `grid_events`, `naming_conventions`

**category** (required):
- `architecture` - Framework concepts and patterns
- `component` - Component-specific documentation
- `pattern` - Implementation patterns (transactions, popups, etc.)
- `example` - Complete working examples
- `standard` - Coding standards and conventions

**component** (optional):
- Component name: `dataset`, `grid`, `button`, `field`, `combobox`, etc.
- Set to `null` for non-component entries

**section** (optional):
- Sub-section within component: `basic_syntax`, `events`, `api_methods`, `advanced_features`
- Set to `null` if not applicable

**content** (required):
- Use `|` for multiline content
- Write in Markdown format
- Include code examples in triple backticks
- Keep focused and concise

**relevance_tags** (required):
- JSON array format: `["tag1", "tag2", "tag3"]`
- Common tags:
  - `list_screen` - Include for list/table screens
  - `detail_screen` - Include for detail/form screens
  - `master_detail` - Include for master-detail screens
  - `popup` - Include for popup dialogs
  - `all` - Include for all screen types

**priority** (required):
- `high` - Always include for matching screen type
- `medium` - Include when component is used
- `low` - Include only for advanced scenarios

**token_estimate** (optional):
- Approximate token count (rough: 4 chars ≈ 1 token)
- Used for prompt budget planning
- Can be `null` initially

---

## Adding New Knowledge Entries

### Method 1: Edit YAML Fixture (Recommended)

**Step 1**: Open the fixture file
```bash
vim backend/src/fixtures/knowledge_bases.yaml
# or your preferred editor
```

**Step 2**: Add new entry at the end
```yaml
- id: 8  # Increment from last ID
  name: button_component_basic
  category: component
  component: button
  section: basic_syntax
  content: |
    # Button Component

    ## Basic Usage
    ```xml
    <pushbutton control_id="1"
                name="btn_save"
                text="Save"
                onclick="fn_save"/>
    ```

    ## Key Attributes
    - `control_id` - Unique control identifier
    - `name` - Button name (use btn_ prefix)
    - `text` - Button label
    - `onclick` - JavaScript function name
  relevance_tags: ["list_screen", "detail_screen", "all"]
  priority: medium
  token_estimate: 400
  version: 1
  is_active: true
  created_at: "2025-12-28T12:00:00.000Z"
  updated_at: "2025-12-28T12:00:00.000Z"
```

**Step 3**: Re-seed database
```bash
cd backend
cargo loco db seed --reset
```

**Step 4**: Verify
```bash
curl http://localhost:3000/api/knowledge_bases | grep "button_component_basic"
```

### Method 2: Via REST API (Runtime)

**Create new entry**:
```bash
curl -X POST http://localhost:3000/api/knowledge_bases \
  -H "Content-Type: application/json" \
  -d '{
    "name": "button_component_basic",
    "category": "component",
    "component": "button",
    "section": "basic_syntax",
    "content": "# Button Component\n\n```xml\n<pushbutton/>\n```",
    "relevance_tags": ["list_screen", "detail_screen"],
    "priority": "medium",
    "token_estimate": 400,
    "version": 1,
    "is_active": true
  }'
```

**Note**: API changes are not persisted to the fixture file. For permanent additions, use Method 1.

### Method 3: Via Service Layer (Programmatic)

```rust
use coder::services::knowledge_base_service::KnowledgeBaseService;

let entry = KnowledgeBaseService::create(
    &db,
    "button_component_basic".to_string(),
    "component".to_string(),
    Some("button".to_string()),
    Some("basic_syntax".to_string()),
    "# Button Component\n\n```xml\n<pushbutton/>\n```".to_string(),
    Some(vec!["list_screen".to_string(), "detail_screen".to_string()]),
    "medium".to_string(),
    Some(400),
).await?;
```

---

## Common YAML Pitfalls

### ❌ Wrong: JSON string instead of array
```yaml
relevance_tags: '["list_screen", "detail_screen"]'  # This is a string!
```

### ✅ Correct: YAML array
```yaml
relevance_tags: ["list_screen", "detail_screen"]
```

### ❌ Wrong: No multiline indicator
```yaml
content: # New Component
  ```xml
  <component/>
  ```
```

### ✅ Correct: Use pipe for multiline
```yaml
content: |
  # New Component

  ```xml
  <component/>
  ```
```

### ❌ Wrong: Incorrect indentation
```yaml
- id: 1
name: test  # Not indented!
```

### ✅ Correct: Proper indentation
```yaml
- id: 1
  name: test  # 2 spaces
  category: component  # 2 spaces
```

---

## Current Knowledge Base Entries

The fixture currently contains 7 core entries:

1. **core_architecture** - xFrame5 architecture overview
2. **dataset_component_basic** - Dataset component syntax
3. **grid_component_basic** - Grid component structure
4. **popup_patterns_basic** - Popup usage patterns
5. **io_mapping_transactions** - Transaction patterns
6. **naming_conventions** - Standard naming rules
7. **xml_complete_example** - Complete XML example

**Tags Coverage**:
- `list_screen` - 5 entries
- `detail_screen` - 4 entries
- `master_detail` - 4 entries
- `popup` - 1 entry
- `all` - 1 entry (naming_conventions)

---

## Querying Knowledge Base

### By Screen Type
```rust
use coder::services::knowledge_base_service::KnowledgeBaseService;

// Get all entries for list screens
let entries = KnowledgeBaseService::for_screen_type(&db, "list_screen").await?;
```

### By Component
```rust
// Get all grid-related entries
let grid_knowledge = KnowledgeBaseService::for_component(&db, "grid").await?;
```

### Custom Query
```rust
use coder::services::knowledge_base_service::KnowledgeQuery;

let query = KnowledgeQuery {
    category: Some("component".to_string()),
    component: Some("dataset".to_string()),
    relevance_tags: Some(vec!["list_screen".to_string()]),
    priority: None,
};

let entries = KnowledgeBaseService::query(&db, &query).await?;
```

### Assemble Content
```rust
// Combine entries into single string for prompt
let content = KnowledgeBaseService::assemble_content(&entries);

// Estimate total tokens
let tokens = KnowledgeBaseService::estimate_tokens(&entries);
println!("Total tokens: {}", tokens);
```

---

## Updating Existing Entries

### Option 1: Edit Fixture and Re-seed

1. Modify `backend/src/fixtures/knowledge_bases.yaml`
2. Run `cargo loco db seed --reset`

### Option 2: Via Service
```rust
KnowledgeBaseService::update(
    &db,
    entry_id,
    Some(new_content),  // Update content
    Some(vec!["new_tag".to_string()]),  // Update tags
    Some("high".to_string()),  // Update priority
    Some(true),  // Keep active
).await?;
```

### Option 3: Via API
```bash
curl -X PUT http://localhost:3000/api/knowledge_bases/1 \
  -H "Content-Type: application/json" \
  -d '{
    "name": "core_architecture",
    "category": "architecture",
    "content": "Updated content...",
    "relevance_tags": ["list_screen", "detail_screen", "master_detail"],
    "priority": "high"
  }'
```

---

## Deactivating Entries

### Soft Delete (Recommended)
```rust
// Sets is_active = false
KnowledgeBaseService::delete(&db, entry_id).await?;
```

### Via API
```bash
curl -X DELETE http://localhost:3000/api/knowledge_bases/1
```

---

## Best Practices

### Content Guidelines

✅ **DO**:
- Keep entries focused and concise
- Use markdown formatting
- Include code examples
- Estimate token counts
- Tag accurately for relevant screen types
- Use high priority for essential knowledge

❌ **DON'T**:
- Include duplicate information across entries
- Write overly verbose explanations
- Forget to update token estimates
- Over-tag entries (reduces selectivity)
- Include deprecated patterns

### Organization

- **Group by category** - Keep similar entries together
- **Use consistent naming** - Follow `{component}_{section}` pattern
- **Version control** - Increment version field when updating
- **Document changes** - Update updated_at timestamp

### Token Budget

Target token counts per screen type:
- **List screens**: ~2,500 tokens
- **Detail screens**: ~3,000 tokens
- **Master-detail**: ~4,500 tokens

Monitor with:
```rust
let tokens = KnowledgeBaseService::estimate_tokens(&entries);
tracing::info!("Knowledge tokens: {}", tokens);
```

---

## Troubleshooting

### Problem: Seeding fails with "duplicate key"
**Solution**: Use `--reset` flag to truncate before seeding
```bash
cargo loco db seed --reset
```

### Problem: YAML parsing error
**Solution**: Validate YAML syntax
- Check indentation (2 spaces)
- Ensure arrays use `["item"]` format, not `'["item"]'`
- Use `|` for multiline content

### Problem: No entries returned from query
**Solution**: Check relevance_tags match query
```rust
// Debug query results
let entries = KnowledgeBaseService::for_screen_type(&db, "list_screen").await?;
println!("Found {} entries", entries.len());
for entry in &entries {
    println!("- {} (tags: {:?})", entry.name, entry.relevance_tags);
}
```

### Problem: Empty array from API
**Solution**: Ensure server restarted after seeding
```bash
# Stop server (Ctrl+C)
cargo loco start
```

---

## References

- [KNOWLEDGE_BASE_ARCHITECTURE.md](./KNOWLEDGE_BASE_ARCHITECTURE.md) - System architecture
- [Loco.rs Database Seeding](https://loco.rs/docs/the-app/seeding/) - Official docs
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/) - ORM reference

---

## Quick Reference

```bash
# Seed database
cd backend && cargo loco db seed --reset

# Verify count
curl http://localhost:3000/api/knowledge_bases | grep -o '"id":' | wc -l

# View all entries
curl http://localhost:3000/api/knowledge_bases

# Add entry via fixture
vim backend/src/fixtures/knowledge_bases.yaml
cargo loco db seed --reset

# Run tests
cargo test knowledge_bases
```

---

**Last Updated**: 2025-12-28
