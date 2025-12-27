# Loco.rs Migration Patterns

**Purpose:** Database migrations guide for xFrame5 Code Assistant using SeaORM helpers

---

## Critical Rules

**NEVER write raw SQL for schema changes. Always use Loco.rs/SeaORM migration helpers.**

1. **Use schema helpers**: `add_reference()`, `add_column()`, `m.create_index()`, etc.
2. **Follow naming conventions**: `idx-` (index), `ux-` (unique index), `fk-` (foreign key)
3. **Always implement `down()`**: Migrations must be reversible
4. **Run from backend/ directory**: All migration commands execute in backend/

---

## Quick Reference

| Task | Helper Function | Example |
|------|----------------|---------|
| Add NOT NULL foreign key | `add_reference()` | `add_reference(m, "generation_logs", "users", "").await?` |
| Add nullable foreign key | `add_column()` + `TableForeignKey` | See Pattern 2 |
| Add column | `add_column()` | `add_column(m, "users", "role", ColType::string().default("user"))` |
| Add index | `m.create_index()` | `Index::create().name("idx-table-column").table("table").col("column")` |
| Add unique index | `m.create_index()` + `.unique()` | `Index::create().name("ux-table-column").unique()` |

---

## Pattern 1: Foreign Keys (NOT NULL) via Scaffold

**Use Case**: Required relationships (user_id on generation_logs)

**CASCADE is automatic** - Loco scaffold with `references` type automatically sets `ON DELETE CASCADE` for NOT NULL foreign keys. No manual migration needed.

```bash
# This automatically creates FK with CASCADE
cargo loco generate scaffold generation_log \
  user:references \
  --api
```

**Creates**:
- Column: `user_id` (INTEGER NOT NULL)
- Foreign key: `fk-generation_logs-users` → `users(id)`
- `ON DELETE CASCADE, ON UPDATE CASCADE` (automatic)

**Adding FK to existing table** (also automatic CASCADE):
```rust
// CASCADE DELETE is automatically set for NOT NULL FK
add_reference(m, "generation_logs", "users", "").await?;
```

---

## Pattern 2: Foreign Keys (Nullable)

**Use Case**: Optional relationships (template_id on generation_logs)

```rust
use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column(m, "generation_logs", "template_id", ColType::UuidNull).await?;

        let fk = TableForeignKey::new()
            .name("fk-generation_logs-template_id-to-prompt_templates")
            .from_tbl(Alias::new("generation_logs"))
            .from_col(Alias::new("template_id"))
            .to_tbl(Alias::new("prompt_templates"))
            .to_col(Alias::new("id"))
            .on_delete(ForeignKeyAction::SetNull)
            .on_update(ForeignKeyAction::Cascade)
            .to_owned();

        m.alter_table(
            alter(Alias::new("generation_logs"))
                .add_foreign_key(&fk)
                .to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_reference(m, "generation_logs", "prompt_templates", "").await?;
        remove_column(m, "generation_logs", "template_id").await?;
        Ok(())
    }
}
```

---

## Pattern 3: Adding Columns

**Common Column Types**:
```rust
ColType::string()                          // VARCHAR(255)
ColType::string().default("value")         // VARCHAR(255) DEFAULT 'value'
ColType::text()                            // TEXT (for prompts, artifacts)
ColType::integer()                         // INTEGER
ColType::IntegerNull                       // INTEGER NULL
ColType::boolean().default(false)          // BOOLEAN DEFAULT false
ColType::timestamp()                       // TIMESTAMP
ColType::UuidNull                          // UUID NULL
```

**Example**: Add version column to prompt_templates
```rust
add_column(m, "prompt_templates", "version", ColType::integer().default(1)).await?;
```

---

## Pattern 4: Unique Index

**Use Case**: Enforce business rules (unique template name per product)

```rust
async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.create_index(
        Index::create()
            .name("ux-prompt_templates-product-name")
            .table(Alias::new("prompt_templates"))
            .col(Alias::new("product"))
            .col(Alias::new("name"))
            .unique()
            .to_owned()
    ).await
}

async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.drop_index(
        Index::drop()
            .name("ux-prompt_templates-product-name")
            .table(Alias::new("prompt_templates"))
            .to_owned()
    ).await
}
```

---

## Pattern 5: Performance Index

**Use Case**: Query optimization (generation_logs by user and date)

```rust
async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.create_index(
        Index::create()
            .name("idx-generation_logs-user_id-created_at")
            .table(Alias::new("generation_logs"))
            .col(Alias::new("user_id"))
            .col(Alias::new("created_at"))
            .to_owned()
    ).await
}
```

---

## Naming Conventions

| Type | Format | Example |
|------|--------|---------|
| Regular index | `idx-{table}-{columns}` | `idx-generation_logs-created_at` |
| Unique index | `ux-{table}-{columns}` | `ux-prompt_templates-product-name` |
| Foreign key | `fk-{from}-{to}` | `fk-generation_logs-users` |

---

## Project-Specific Tables

### Core Tables
```
users                 - System users (developers)
roles                 - RBAC roles
prompt_templates      - LLM prompt templates (dynamic)
company_rules         - Customer-specific coding rules
generation_logs       - Audit trail
```

### Recommended Indexes
```sql
-- prompt_templates
ux-prompt_templates-product-name-screen_type  -- Unique template lookup
idx-prompt_templates-product-is_active        -- Active template filter

-- generation_logs
idx-generation_logs-user_id-created_at        -- User history query
idx-generation_logs-status                    -- Error analysis
```

---

## Common Commands

```bash
cd backend

# Generate migration
cargo loco generate migration create_prompt_templates

# Run migrations
cargo loco db migrate

# Rollback
cargo loco db down

# Regenerate entities
cargo loco db entities

# Check status
cargo loco db status

# Verify schema
psql -d coder_dev -c "\d prompt_templates"
```

---

## Anti-Patterns

### ❌ DON'T: Raw SQL
```rust
m.get_connection().execute_unprepared("CREATE INDEX...").await?;
```

### ✅ DO: Use helpers
```rust
m.create_index(Index::create().name("idx-...").table(...).col(...).to_owned()).await
```

### ❌ DON'T: Empty down()
```rust
async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
    Ok(())  // No rollback!
}
```

### ✅ DO: Reversible migrations
```rust
async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.drop_index(Index::drop().name("idx-...").table(...).to_owned()).await
}
```

---

**Last Updated**: 2025-12-28
