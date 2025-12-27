# Audit Logging Pattern

**Purpose:** 금융권 감사 요구사항 충족을 위한 생성 요청 로깅

---

## Core Principle

| Item | Store | Reason |
|------|-------|--------|
| Input data | ❌ | 개인정보 보호 |
| Meta Model (UiIntent) | ⭕ | 재현 가능성 |
| Generated artifacts | ⭕ | 감사 추적 |
| User/timestamp | ⭕ | 책임 추적 |

> "데이터는 남기지 않고, 결과와 구조만 남깁니다."

---

## Step 1: Scaffold generation_logs Table

```bash
cd backend

cargo loco generate scaffold generation_log \
  product:string! \
  input_type:string! \
  ui_intent:text! \
  template_version:int! \
  status:string! \
  artifacts:text \
  warnings:text \
  error_message:text \
  generation_time_ms:int \
  user:references \
  --api
```

**Note**: `user:references` automatically creates:
- `user_id` column (NOT NULL)
- Foreign key with `ON DELETE CASCADE`

---

## Step 2: Add Nullable Template Reference

```bash
cargo loco generate migration add_template_ref_to_generation_logs
```

Edit the migration file (nullable FK requires manual setup):

```rust
use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

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
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_reference(m, "generation_logs", "prompt_templates", "").await?;
        remove_column(m, "generation_logs", "template_id").await
    }
}
```

---

## Step 3: Add Performance Indexes

```bash
cargo loco generate migration add_indexes_to_generation_logs
```

```rust
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_index(
            Index::create()
                .name("idx-generation_logs-user_id-created_at")
                .table(Alias::new("generation_logs"))
                .col(Alias::new("user_id"))
                .col(Alias::new("created_at"))
                .to_owned()
        ).await?;

        m.create_index(
            Index::create()
                .name("idx-generation_logs-status")
                .table(Alias::new("generation_logs"))
                .col(Alias::new("status"))
                .to_owned()
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_index(
            Index::drop()
                .name("idx-generation_logs-user_id-created_at")
                .table(Alias::new("generation_logs"))
                .to_owned()
        ).await?;

        m.drop_index(
            Index::drop()
                .name("idx-generation_logs-status")
                .table(Alias::new("generation_logs"))
                .to_owned()
        ).await
    }
}
```

---

## Step 4: Run Migrations

```bash
cargo loco db migrate
cargo loco db entities
```

---

## Service Usage

```rust
impl GenerationLogService {
    pub async fn log(db: &DatabaseConnection, log: CreateLog) -> Result<Model> {
        let item = ActiveModel {
            user_id: Set(log.user_id),
            product: Set(log.product),
            input_type: Set(log.input_type),
            ui_intent: Set(serde_json::to_string(&log.ui_intent)?),
            template_id: Set(log.template_id),
            template_version: Set(log.template_version),
            status: Set(log.status),
            artifacts: Set(log.artifacts),
            warnings: Set(log.warnings),
            error_message: Set(log.error_message),
            generation_time_ms: Set(log.generation_time_ms),
            ..Default::default()
        };
        item.insert(db).await
    }
}
```

---

## Security Considerations

1. **Access control**: Admin-only log access
2. **Retention**: Configure per regulation (e.g., 5 years)
3. **Immutability**: INSERT only in service layer
4. **Encryption**: Encrypt artifacts field if required

---

**Last Updated**: 2025-12-28
