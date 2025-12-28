# xFrame5 Code Assistant - Implementation Plan

**Date:** 2025-12-28
**Status:** 📋 Planning
**Scope:** PoC (Proof of Concept)
**Last Updated:** 2025-12-28

---

## Overview

On-premise code assistant for enterprise development automation. Generates both frontend (xFrame5) and backend (Spring Framework) code from DB schema or query samples.

### Supported Products

| Product | Type | Output | Status |
|---------|------|--------|--------|
| **xFrame5 UI** | Frontend | XML views + JavaScript handlers | PoC Complete |
| **Spring Framework** | Backend | Controller, Service, DTO, Mapper | Planned |

### Key Objectives
1. **xFrame5 Frontend**: Generate XML views and JavaScript handlers from DB schema
2. **Spring Backend**: Generate Controller, Service, DTO, and MyBatis Mapper from DB schema
3. Reduce development time by 50%+ for standard CRUD operations
4. Zero external data transmission (금융권 보안 요구사항)
5. Follow company coding standards for both frontend and backend

### PoC Scope (Phase 1 - xFrame5)
- **Target**: 회원 목록 + 조회 + 상세 팝업 화면
- **Output**: xFrame5 XML + JavaScript

### Future Scope (Phase 2 - Spring Framework)
- **Target**: 회원 CRUD API
- **Output**: Controller, Service, ServiceImpl, DTO, Mapper XML

### Development Approach
- **AI-Augmented Engineering**: Leverage AI agents for rapid development
- **Pattern-First**: Reuse Loco.rs patterns from HWS project
- **LLM Abstraction**: Hide all LLM details from plugin/API
- **On-Premise Only**: No external network calls

---

## Architecture Overview

```
[Eclipse Plugin]  ←→  [Agent Server]  ←→  [LLM Runtime]
     (Java)            (Rust/Loco.rs)      (Ollama/llama.cpp)
                            ↓
                      [PostgreSQL]
                   (templates, rules, logs)
```

---

## Implementation Phases

### Phase 0: Foundation Setup ✅ COMPLETED
**Duration**: 1 day
**Status**: ✅ Done

- [x] Project structure setup
- [x] CLAUDE.md context file created
- [x] Pattern documentation established
- [x] Feature specifications documented
- [x] Requirements documented

**Deliverables**:
- ✅ CLAUDE.md - Main context file
- ✅ docs/patterns/ - 5 pattern documents
- ✅ docs/features/ - 2 feature specifications
- ✅ docs/requirements.md - PoC requirements

---

### Phase 1: Database & Backend Foundation ✅ COMPLETED
**Duration**: 2 days
**Status**: ✅ Done

- [x] Loco.rs project initialized with PostgreSQL
- [x] prompt_templates table scaffolded
- [x] company_rules table scaffolded
- [x] generation_logs table scaffolded (with user FK)
- [x] llm_configs table scaffolded (for admin panel)
- [x] Database indexes created
- [x] All 31 tests passing

**Deliverables**:
- ✅ backend/ - Loco.rs project
- ✅ 5 database tables with indexes
- ✅ CRUD API endpoints for all tables
- ✅ Test suite passing

---

### Phase 2: LLM Backend Abstraction ✅ COMPLETED
**Duration**: 2 days
**Status**: ✅ Done

- [x] LlmBackend trait created
- [x] OllamaBackend implementation
- [x] LlamaCppBackend implementation
- [x] VllmBackend implementation
- [x] GroqBackend implementation (remote testing)
- [x] OpenAIBackend implementation (remote testing)
- [x] AnthropicBackend implementation (remote testing)
- [x] Factory pattern with env var configuration
- [x] 16 LLM-specific tests passing

**Deliverables**:
- ✅ src/llm/ module with 6 provider backends
- ✅ create_backend_from_env() factory function
- ✅ Environment variable configuration
- ✅ Health check for all providers

---

### Phase 3: Prompt Compiler ✅ COMPLETED
**Duration**: 1 day
**Status**: ✅ Done

- [x] UiIntent DSL created (ScreenType, DatasetIntent, ColumnIntent, GridIntent, ActionIntent)
- [x] Input types defined (GenerateInput, SchemaInput, QuerySampleInput, NaturalLanguageInput)
- [x] NormalizerService implemented (Schema → UiIntent, Query → UiIntent, NL → UiIntent)
- [x] PromptCompiler implemented (UiIntent → CompiledPrompt)
- [x] TemplateService for DB template loading
- [x] Korean label inference for common column names
- [x] 27 new tests passing (74 total)

**Deliverables**:
- ✅ src/domain/ module with DSL types
- ✅ src/services/ module with normalizer, compiler, template services
- ✅ Type inference (VARCHAR→Input, TEXT→TextArea, DATE→DatePicker, etc.)
- ✅ Default Korean labels for common fields
- ✅ Company rules injection into prompts

---

### Phase 1 (Original): Database & Backend Foundation
**Duration**: 2 days
**Status**: ✅ Completed (see above)

#### 1.1 Loco.rs Project Setup

**AI Prompt** 🤖:
```
Act as a DevOps & Backend Architect for a Rust/Loco.rs project.

🔴 CRITICAL RULES:
1. Run ALL commands from backend/ directory
2. Use Loco SaaS template (includes users table)
3. Follow Loco.rs conventions strictly

TASK:
Initialize Loco.rs backend project for xFrame5 Code Assistant.

EXECUTION STEPS:
1. Create backend directory and initialize Loco project
2. Configure PostgreSQL connection
3. Verify setup with cargo loco doctor

COMMANDS:
```bash
mkdir -p backend && cd backend
cargo install loco
loco new . --template saas
cargo loco doctor
```

DELIVERABLES:
- Loco.rs project initialized
- Database connection verified
- cargo loco start runs successfully
```

---

#### 1.2 Database Schema Setup

**AI Prompt** 🤖:
```
Act as a Rust Database Engineer for Loco.rs/SeaORM.

🔴 CRITICAL RULES:
1. Run from backend/ directory
2. Use Loco scaffold commands (CASCADE is automatic for references)
3. Follow naming conventions: idx- (index), ux- (unique), fk- (foreign key)

CONTEXT:
- Read docs/patterns/LOCO_MIGRATION_PATTERNS.md

TASK:
Create database schema for code assistant.

SCAFFOLDING COMMANDS (in order):
```bash
cd backend

# 1. Prompt Templates (core table)
cargo loco generate scaffold prompt_template \
  name:string! \
  product:string! \
  screen_type:string \
  system_prompt:text! \
  user_prompt_template:text! \
  version:int! \
  is_active:bool \
  --api

# 2. Company Rules (customer-specific)
cargo loco generate scaffold company_rule \
  company_id:string! \
  naming_convention:text \
  additional_rules:text \
  --api

# 3. Generation Logs (audit trail)
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

# 4. Run migrations
cargo loco db migrate
cargo loco db entities
```

DELIVERABLES:
- All tables created
- Entities generated
- Migrations reversible (down() implemented)
```

---

#### 1.3 Add Indexes

**AI Prompt** 🤖:
```
Act as a Database Engineer.

CONTEXT:
- Read docs/patterns/LOCO_MIGRATION_PATTERNS.md → Pattern 4, 5

TASK:
Create migration to add performance and unique indexes.

```bash
cargo loco generate migration add_indexes_to_tables
```

Edit migration:
```rust
use sea_orm_migration::{prelude::*, schema::*};

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Unique: template lookup
        m.create_index(
            Index::create()
                .name("ux-prompt_templates-product-name-screen_type")
                .table(Alias::new("prompt_templates"))
                .col(Alias::new("product"))
                .col(Alias::new("name"))
                .col(Alias::new("screen_type"))
                .unique()
                .to_owned()
        ).await?;

        // Index: active templates
        m.create_index(
            Index::create()
                .name("idx-prompt_templates-product-is_active")
                .table(Alias::new("prompt_templates"))
                .col(Alias::new("product"))
                .col(Alias::new("is_active"))
                .to_owned()
        ).await?;

        // Index: generation logs
        m.create_index(
            Index::create()
                .name("idx-generation_logs-user_id-created_at")
                .table(Alias::new("generation_logs"))
                .col(Alias::new("user_id"))
                .col(Alias::new("created_at"))
                .to_owned()
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Drop all indexes (reverse order)
    }
}
```

DELIVERABLES:
- Indexes created
- Migration reversible
```

---

### Phase 2: LLM Backend Abstraction
**Duration**: 2 days
**Status**: 📋 Planned

#### 2.1 LlmBackend Trait

**AI Prompt** 🤖:
```
Act as a Senior Rust Engineer specializing in trait-based abstractions.

🔴 CRITICAL RULES:
1. LLM details NEVER exposed to API/plugin
2. Trait must be Send + Sync for async contexts
3. Configuration via YAML, not code

CONTEXT:
- Read docs/patterns/LLM_BACKEND_ABSTRACTION.md

TASK:
Implement LlmBackend trait with Ollama implementation.

FILES TO CREATE:

**backend/src/llm/mod.rs**:
```rust
mod ollama;

pub use ollama::OllamaBackend;

use async_trait::async_trait;

#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn generate(&self, prompt: &str) -> anyhow::Result<String>;
    async fn health_check(&self) -> anyhow::Result<bool>;
}

pub fn create_backend(config: &LlmConfig) -> Box<dyn LlmBackend> {
    match config.backend.as_str() {
        "ollama" => Box::new(OllamaBackend::new(config)),
        _ => panic!("Unknown LLM backend"),
    }
}
```

**backend/src/llm/ollama.rs**:
```rust
pub struct OllamaBackend {
    endpoint: String,
    model: String,
    timeout: Duration,
}

#[async_trait]
impl LlmBackend for OllamaBackend {
    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        // POST to /api/generate
    }
}
```

**config/development.yaml** (add section):
```yaml
llm:
  backend: "ollama"
  endpoint: "http://localhost:11434"
  model: "codellama:13b"
  timeout_seconds: 120
```

DELIVERABLES:
- LlmBackend trait
- OllamaBackend implementation
- Configuration in YAML
- Health check endpoint
```

---

### Phase 3: Prompt Compiler
**Duration**: 3 days
**Status**: 📋 Planned

#### 3.1 Internal DSL (UiIntent)

**AI Prompt** 🤖:
```
Act as a Domain-Driven Design expert.

CONTEXT:
- Read docs/patterns/PROMPT_COMPILER.md

TASK:
Implement UiIntent DSL for representing screen generation intent.

**backend/src/domain/ui_intent.rs**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiIntent {
    pub screen_name: String,
    pub screen_type: ScreenType,
    pub datasets: Vec<DatasetIntent>,
    pub grids: Vec<GridIntent>,
    pub actions: Vec<ActionIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreenType {
    List,
    Detail,
    Popup,
    ListWithPopup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetIntent {
    pub id: String,
    pub columns: Vec<ColumnIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnIntent {
    pub name: String,
    pub ui_type: UiType,
    pub label: String,
    pub required: bool,
    pub readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiType {
    Input,
    TextArea,
    DatePicker,
    Checkbox,
    Combo,
    Hidden,
}
```

DELIVERABLES:
- UiIntent struct with all nested types
- Serialization support (for audit logging)
```

---

#### 3.2 Input Normalization

**AI Prompt** 🤖:
```
Act as a Rust Backend Engineer.

CONTEXT:
- Read docs/features/SCHEMA_INPUT.md

TASK:
Implement input normalization from DB schema to UiIntent.

**backend/src/services/normalizer.rs**:
```rust
pub fn normalize_schema(input: &SchemaInput) -> UiIntent {
    let columns = input.columns.iter().map(|c| {
        ColumnIntent {
            name: c.name.clone(),
            ui_type: infer_ui_type(&c.column_type, c.pk),
            label: infer_label(&c.name),
            required: !c.nullable,
            readonly: c.pk,
        }
    }).collect();

    UiIntent {
        screen_name: format!("{}_list", input.table.to_lowercase()),
        screen_type: ScreenType::List,
        datasets: vec![DatasetIntent {
            id: format!("ds_{}", input.table.to_lowercase()),
            columns,
        }],
        grids: vec![/* auto-generate */],
        actions: default_actions(),
    }
}

fn infer_ui_type(db_type: &str, is_pk: bool) -> UiType {
    if is_pk { return UiType::Hidden; }
    match db_type.to_uppercase().as_str() {
        "VARCHAR" | "CHAR" => UiType::Input,
        "TEXT" | "CLOB" => UiType::TextArea,
        "DATE" => UiType::DatePicker,
        "BOOLEAN" => UiType::Checkbox,
        _ => UiType::Input,
    }
}
```

DELIVERABLES:
- SchemaInput → UiIntent conversion
- Type inference logic
- Label inference (Korean naming)
```

---

#### 3.3 Prompt Compilation

**AI Prompt** 🤖:
```
Act as a Prompt Engineering expert.

CONTEXT:
- Read docs/patterns/PROMPT_COMPILER.md

TASK:
Implement prompt compilation from UiIntent to LLM prompt.

**backend/src/services/prompt_compiler.rs**:
```rust
pub struct PromptCompiler;

impl PromptCompiler {
    pub async fn compile(
        db: &DatabaseConnection,
        intent: &UiIntent,
        product: &str,
        company_id: Option<&str>,
    ) -> Result<CompiledPrompt> {
        // 1. Load template from DB
        let template = TemplateService::get_active(db, product, intent.screen_type.as_str()).await?;

        // 2. Load company rules (if any)
        let rules = if let Some(cid) = company_id {
            CompanyRuleService::get(db, cid).await.ok()
        } else {
            None
        };

        // 3. Generate description from intent
        let description = Self::describe_intent(intent);

        // 4. Compile system prompt (with company rules)
        let mut system = template.system_prompt.clone();
        if let Some(r) = &rules {
            system.push_str("\n\nCOMPANY RULES:\n");
            system.push_str(&r.additional_rules);
        }

        // 5. Compile user prompt
        let user = template.user_prompt_template
            .replace("{{dsl_description}}", &description);

        Ok(CompiledPrompt { system, user })
    }

    fn describe_intent(intent: &UiIntent) -> String {
        format!(
            "Create a {} screen named '{}'.\n\
             Datasets: {}\n\
             Actions: {}",
            intent.screen_type.as_str(),
            intent.screen_name,
            Self::describe_datasets(&intent.datasets),
            Self::describe_actions(&intent.actions),
        )
    }
}
```

DELIVERABLES:
- PromptCompiler with DB template loading
- Company rules injection
- Intent → description conversion
```

---

### Phase 4: API & Validation
**Duration**: 2 days
**Status**: 📋 Planned

#### 4.1 Generate Endpoint

**AI Prompt** 🤖:
```
Act as a Rust Backend Engineer for Loco.rs.

🔴 CRITICAL RULES:
1. Thin controller, fat service
2. NEVER expose LLM details in response
3. Log ALL requests (audit trail)

CONTEXT:
- Read CLAUDE.md → "API Specification"
- Read docs/patterns/AUDIT_LOGGING.md

TASK:
Implement /agent/generate endpoint.

**backend/src/controllers/generate.rs**:
```rust
pub async fn generate(
    State(ctx): State<AppContext>,
    Json(req): Json<GenerateRequest>,
) -> Result<Response> {
    let start = Instant::now();

    // 1. Normalize input to UiIntent
    let intent = NormalizerService::normalize(&req)?;

    // 2. Compile prompt
    let template = TemplateService::get_active(&ctx.db, &req.product, None).await?;
    let prompt = PromptCompiler::compile(&ctx.db, &intent, &req.product, None).await?;

    // 3. Generate via LLM
    let llm = create_backend(&ctx.config.llm);
    let raw = llm.generate(&prompt.full()).await?;

    // 4. Parse and validate
    let artifacts = XFrame5Validator::parse_and_validate(&raw, &intent)?;

    // 5. Log (audit trail - NO input data)
    GenerationLogService::log(&ctx.db, GenerationLog {
        product: req.product.clone(),
        input_type: req.input_type.clone(),
        ui_intent: serde_json::to_string(&intent)?,
        template_version: template.version,
        status: "success".to_string(),
        artifacts: Some(serde_json::to_string(&artifacts)?),
        warnings: artifacts.warnings.clone(),
        error_message: None,
        generation_time_ms: start.elapsed().as_millis() as i32,
        user_id: None,  // Anonymous for now
    }).await?;

    // 6. Response (NO LLM details)
    format::json(GenerateResponse {
        status: "success".to_string(),
        artifacts: artifacts.into(),
        warnings: artifacts.warnings,
        meta: ResponseMeta {
            generator: format!("{}-v1", req.product),
            timestamp: Utc::now(),
        },
    })
}
```

DELIVERABLES:
- Generate endpoint with full flow
- Audit logging
- Error handling with retry
```

---

#### 4.2 xFrame5 Validation

**AI Prompt** 🤖:
```
Act as an XML/JavaScript validation expert.

CONTEXT:
- Read docs/patterns/XFRAME5_VALIDATION.md

TASK:
Implement xFrame5 output validation.

**backend/src/services/xframe5_validator.rs**:
```rust
pub struct XFrame5Validator;

impl XFrame5Validator {
    pub fn parse_and_validate(raw: &str, intent: &UiIntent) -> Result<ValidatedArtifacts> {
        // 1. Split XML and JS
        let (xml, js) = Self::split_output(raw)?;

        // 2. Validate XML structure
        Self::validate_xml(&xml)?;

        // 3. Validate JS functions
        let warnings = Self::validate_js(&js, intent)?;

        // 4. Validate bindings
        Self::validate_bindings(&xml)?;

        Ok(ValidatedArtifacts { xml, javascript: js, warnings })
    }

    fn split_output(raw: &str) -> Result<(String, String)> {
        // Split by "--- XML ---" and "--- JS ---" markers
    }

    fn validate_xml(xml: &str) -> Result<()> {
        // Parse XML, check Dataset/Grid elements
    }

    fn validate_js(js: &str, intent: &UiIntent) -> Result<Vec<String>> {
        // Check required functions exist (fn_search, fn_save, etc.)
    }
}
```

DELIVERABLES:
- Output parsing (XML/JS split)
- XML structure validation
- JS function validation
- Binding consistency check
```

---

### Phase 5: Eclipse Plugin
**Duration**: 1 week
**Status**: 📋 Planned

#### 5.1 Plugin Project Setup

**AI Prompt** 🤖:
```
Act as an Eclipse Plugin Developer.

🔴 CRITICAL RULES:
1. Plugin is intentionally "dumb" - no LLM knowledge
2. Only knows: input types, server endpoint, project context
3. Uses HTTP POST to agent server

TASK:
Setup Eclipse plugin project structure.

STRUCTURE:
```
eclipse-plugin/
├── META-INF/
│   └── MANIFEST.MF
├── plugin.xml
├── src/
│   └── com/
│       └── softbase/
│           └── xframe5/
│               └── codegen/
│                   ├── Activator.java
│                   ├── actions/
│                   │   └── GenerateAction.java
│                   ├── dialogs/
│                   │   └── InputDialog.java
│                   ├── client/
│                   │   └── AgentClient.java
│                   └── handlers/
│                       └── GenerateHandler.java
└── build.properties
```

DELIVERABLES:
- Eclipse plugin project
- Menu action registered
- Basic UI dialog
```

---

#### 5.2 Agent Client

**AI Prompt** 🤖:
```
Act as a Java HTTP Client developer.

🔴 CRITICAL RULES:
1. NEVER include model name, temperature, or prompt in request
2. Only send: product, inputType, input, context

TASK:
Implement AgentClient for server communication.

**AgentClient.java**:
```java
public class AgentClient {
    private final String endpoint;
    private final HttpClient httpClient;

    public GenerateResponse generate(GenerateRequest request) throws Exception {
        // POST to /agent/generate
        // Request: { product, inputType, input, context }
        // Response: { status, artifacts, warnings, meta }
    }
}

public class GenerateRequest {
    public String product = "xframe5-ui";
    public String inputType;  // "db-schema" | "query-sample" | "natural-language"
    public Object input;
    public RequestContext context;
    // NO: model, temperature, prompt, systemPrompt
}
```

DELIVERABLES:
- HTTP client for agent server
- Request/Response DTOs
- Error handling
```

---

#### 5.3 File Generation

**AI Prompt** 🤖:
```
Act as an Eclipse Plugin Developer.

TASK:
Implement file generation from server response.

**GenerateHandler.java**:
```java
public class GenerateHandler extends AbstractHandler {
    @Override
    public Object execute(ExecutionEvent event) {
        // 1. Get current project context
        IProject project = getSelectedProject();

        // 2. Show input dialog
        InputDialog dialog = new InputDialog(shell);
        if (dialog.open() != Window.OK) return null;

        // 3. Call agent server
        GenerateResponse response = client.generate(dialog.getRequest());

        // 4. Create XML file
        IFile xmlFile = project.getFile("views/" + screenName + ".xml");
        xmlFile.create(new ByteArrayInputStream(response.artifacts.xml.getBytes()), true, null);

        // 5. Create JS file
        IFile jsFile = project.getFile("scripts/" + screenName + ".js");
        jsFile.create(new ByteArrayInputStream(response.artifacts.javascript.getBytes()), true, null);

        // 6. Refresh and open
        project.refreshLocal(IResource.DEPTH_INFINITE, null);
        IDE.openEditor(page, xmlFile);

        return null;
    }
}
```

DELIVERABLES:
- File creation in project
- Editor opening
- Error display
```

---

### Phase 6: Testing & Integration
**Duration**: 3 days
**Status**: 📋 Planned

#### 6.1 Backend Tests

```bash
cd backend

# Run all tests
cargo test

# Run specific test
cargo test test_generate_endpoint
```

**Test Cases**:
- [ ] Schema normalization
- [ ] Prompt compilation
- [ ] xFrame5 validation
- [ ] Generate endpoint (with mocked LLM)
- [ ] Audit logging

---

#### 6.2 Integration Tests

**Test Scenarios**:
1. DB Schema → XML + JS generation
2. Query Sample → XML + JS generation
3. Error handling (invalid input)
4. Retry on LLM failure
5. Template loading from DB

---

### Phase 7: Deployment
**Duration**: 2 days
**Status**: 📋 Planned

#### 7.1 Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY backend/ .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/coder-backend /usr/local/bin/
COPY config/ /app/config/
CMD ["coder-backend", "start"]
```

```yaml
# docker-compose.yml
services:
  agent-server:
    build: .
    ports:
      - "3000:3000"
    environment:
      - LOCO_ENV=production
    depends_on:
      - postgres
      - ollama

  postgres:
    image: postgres:16
    volumes:
      - pgdata:/var/lib/postgresql/data

  ollama:
    image: ollama/ollama
    volumes:
      - ollama:/root/.ollama
```

---

## Progress Tracking

### Phase Completion Summary

| Phase | Status | Duration |
|-------|--------|----------|
| Phase 0: Foundation Setup | ✅ Complete | 1 day |
| Phase 1: Database & Backend Foundation | ✅ Complete | 1 day |
| Phase 2: LLM Backend Abstraction | ✅ Complete | 1 day |
| Phase 3: Prompt Compiler | ✅ Complete | 1 day |
| Phase 4: API & Validation | ✅ Complete | 1 day |
| Phase 5: Eclipse Plugin | ✅ Complete | 1 day |
| Phase 6: Testing & Integration | ✅ Complete | 1 day |
| Phase 7: Deployment | ✅ Complete | 1 day |
| Phase 8: Admin Panel (HTMX) | ✅ Complete | 1 day |
| Phase 9: Spring Framework Support | 📋 Planned | 5 days |

**Total Duration**: 14 days (9 complete + 5 planned)

---

### Phase 8: Admin Panel (HTMX)
**Duration**: 3 days
**Status**: 📋 Planned

#### Technology Decision

**Chosen**: HTMX + Tera templates (served from Loco.rs)

**Rationale**:
- **Simpler Deployment**: No separate frontend build/deploy, served directly from agent server
- **CRUD Focus**: Admin panel is primarily CRUD operations (templates, rules, logs) - ideal for HTMX
- **Loco.rs Integration**: Built-in Tera template support, no additional framework
- **Primary Focus**: Eclipse plugin is the main user interface; admin panel is secondary
- **Reference Implementation**: HTMX patterns from yatclub project

**Alternatives Considered**:
- React + Vite: Better UX for complex interactions, but adds deployment complexity
- Vue + Vite: Same trade-offs as React

#### 8.1 Admin Views Structure

```
backend/assets/views/admin/
├── layout.html                    # Base layout with navigation
├── prompt_template/
│   ├── main.html                  # Container with search form
│   ├── list.html                  # Table with HTMX pagination
│   ├── row.html                   # Single row template
│   ├── create.html                # Create form modal
│   ├── edit.html                  # Edit form modal
│   └── show.html                  # View details
├── company_rule/
│   ├── main.html
│   ├── list.html
│   ├── row.html
│   ├── create.html
│   └── edit.html
├── generation_log/
│   ├── main.html                  # Audit log viewer
│   ├── list.html
│   └── show.html                  # Log detail view
└── llm_config/
    ├── main.html
    ├── list.html
    └── edit.html
```

#### 8.2 HTMX Patterns

**AI Prompt** 🤖:
```
Act as a Loco.rs + HTMX developer.

CONTEXT:
- Read docs/patterns/ADMIN_PANEL.md
- Reference: yatclub repo HTMX examples

TASK:
Implement admin panel using HTMX for prompt template management.

KEY PATTERNS:
1. Search form with hx-trigger="submit, load"
2. Partial table updates with hx-target="#list-container"
3. Modal dialogs with hx-target="#editor-container"
4. Inline row updates after edit
5. Pagination with hx-trigger="input changed delay:0.5s"

DELIVERABLES:
- Tera templates with HTMX attributes
- Loco.rs view controllers
- CSS using Tailwind
```

#### 8.3 Admin Controllers

```rust
// backend/src/controllers/admin/mod.rs
mod prompt_templates;
mod company_rules;
mod generation_logs;
mod llm_configs;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("admin")
        .add("/prompt-templates", prompt_templates::routes())
        .add("/company-rules", company_rules::routes())
        .add("/generation-logs", generation_logs::routes())
        .add("/llm-configs", llm_configs::routes())
}
```

#### 8.4 Admin Features

| Feature | Description | Priority |
|---------|-------------|----------|
| Prompt Template CRUD | Create, edit, activate/deactivate templates | P0 |
| Company Rules CRUD | Manage customer-specific coding rules | P0 |
| Generation Logs Viewer | Search and view audit trail | P0 |
| LLM Config Management | Update model settings (admin only) | P1 |
| Template Version History | View and rollback template versions | P2 |
| User Management | Manage system users (optional) | P2 |

---

### Phase 9: Spring Framework Support
**Duration**: 5 days
**Status**: 📋 Planned

#### Overview

Extend code generation to support Spring Framework backend development. Generate Controller, Service, DTO, and MyBatis Mapper files from DB schema.

#### 9.1 Spring DSL Extension

**New Domain Types**:
```rust
// backend/src/domain/spring_intent.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringIntent {
    pub entity_name: String,           // e.g., "Member"
    pub table_name: String,            // e.g., "TB_MEMBER"
    pub package_base: String,          // e.g., "com.company.project"
    pub columns: Vec<ColumnIntent>,
    pub crud_operations: Vec<CrudOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrudOperation {
    Create,
    Read,
    ReadList,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringArtifacts {
    pub controller: String,            // MemberController.java
    pub service_interface: String,     // MemberService.java
    pub service_impl: String,          // MemberServiceImpl.java
    pub dto: String,                   // MemberDTO.java
    pub mapper_interface: String,      // MemberMapper.java
    pub mapper_xml: String,            // MemberMapper.xml
}
```

#### 9.2 Spring Prompt Templates

**New Templates (stored in DB)**:

| Template Name | Product | Screen Type | Description |
|--------------|---------|-------------|-------------|
| spring-controller | spring-backend | crud | REST Controller with annotations |
| spring-service | spring-backend | crud | Service interface + implementation |
| spring-dto | spring-backend | crud | DTO with validation annotations |
| spring-mybatis-mapper | spring-backend | crud | MyBatis Mapper interface + XML |

**Example System Prompt**:
```
You are a Spring Framework code generator.

RULES:
1. Use @RestController with @RequestMapping
2. Use @Service and @Autowired annotations
3. DTO fields should use @NotNull, @Size validations
4. MyBatis Mapper should use #{} parameter binding
5. Follow company naming conventions

COMPANY RULES:
{{company_rules}}

OUTPUT FORMAT:
--- CONTROLLER ---
[Controller code here]
--- SERVICE ---
[Service interface here]
--- SERVICE_IMPL ---
[Service implementation here]
--- DTO ---
[DTO class here]
--- MAPPER ---
[Mapper interface here]
--- MAPPER_XML ---
[Mapper XML here]
```

#### 9.3 Spring Validation

**backend/src/services/spring_validator.rs**:
```rust
pub struct SpringValidator;

impl SpringValidator {
    pub fn validate(artifacts: &SpringArtifacts, intent: &SpringIntent) -> Result<Vec<String>> {
        let mut warnings = vec![];

        // 1. Validate Controller
        Self::validate_controller(&artifacts.controller, intent)?;

        // 2. Validate Service matches interface
        Self::validate_service_impl(&artifacts.service_impl, &artifacts.service_interface)?;

        // 3. Validate DTO has all columns
        Self::validate_dto(&artifacts.dto, &intent.columns)?;

        // 4. Validate Mapper XML syntax
        Self::validate_mapper_xml(&artifacts.mapper_xml)?;

        Ok(warnings)
    }
}
```

#### 9.4 API Extension

**New Endpoint**:
```
POST /agent/generate

{
  "product": "spring-backend",    // NEW: Spring Framework support
  "inputType": "db-schema",
  "input": {
    "table": "TB_MEMBER",
    "columns": [...]
  },
  "context": {
    "project": "member-service",
    "output": ["controller", "service", "dto", "mapper"]
  }
}
```

**Response**:
```json
{
  "status": "success",
  "artifacts": {
    "controller": "package com.company...",
    "service_interface": "package com.company...",
    "service_impl": "package com.company...",
    "dto": "package com.company...",
    "mapper_interface": "package com.company...",
    "mapper_xml": "<?xml version=\"1.0\"..."
  },
  "warnings": [],
  "meta": {
    "generator": "spring-backend-v1",
    "timestamp": "2025-xx-xx"
  }
}
```

#### 9.5 Eclipse Plugin Extension

**New Menu Options**:
- xFrame5 > Generate Frontend Code (existing)
- Spring > Generate Backend Code (new)

**File Output**:
```
src/
├── main/
│   ├── java/
│   │   └── com/company/project/
│   │       ├── controller/
│   │       │   └── MemberController.java
│   │       ├── service/
│   │       │   ├── MemberService.java
│   │       │   └── impl/
│   │       │       └── MemberServiceImpl.java
│   │       ├── dto/
│   │       │   └── MemberDTO.java
│   │       └── mapper/
│   │           └── MemberMapper.java
│   └── resources/
│       └── mapper/
│           └── MemberMapper.xml
```

#### 9.6 Deliverables

| Deliverable | Description |
|-------------|-------------|
| SpringIntent DSL | Domain types for Spring code generation |
| Spring Prompt Templates | 4 new templates in database |
| SpringValidator | Validation for generated Java/XML |
| API Extension | Support "spring-backend" product |
| Eclipse Plugin Update | New menu for Spring generation |
| Tests | Unit tests for Spring generation flow |

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Screen skeleton generation | < 5 minutes |
| Manual modification | < 50% of previous |
| Generated code quality | Pass existing code review |
| External data transmission | None |

### Architecture Validation
> "모델을 바꿨는데 Eclipse 플러그인은 단 한 줄도 안 바뀐다"

---

## Related Documentation

- **Main Context**: [CLAUDE.md](../../CLAUDE.md)
- **Requirements**: [docs/requirements.md](../requirements.md)
- **Patterns**: [docs/patterns/](../patterns/)
- **Features**: [docs/features/](../features/)

---

**Last Updated:** 2025-12-28
