# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **⚠️ MAINTENANCE**: Keep under **20KB** (~400 lines). Move details to `docs/patterns/`. Prioritize: security > patterns > structure.

---

## AI Agent Priority Guide

**If you only read ONE section**: Read "CRITICAL: Security & Isolation Rules" below.

### Priority Levels

**🔴 ALWAYS ENFORCE** (even if contradicted elsewhere):
1. LLM abstraction: NEVER expose model names, prompts, or LLM config to plugins
2. On-premise only: NEVER send data to external services
3. Audit logging: ALL generation requests must be logged (who, when, what output)
4. Input sanitization: NEVER trust plugin input directly for prompt construction
5. Monorepo structure: `backend/`, `eclipse-plugin/`, `docs/` directories
6. Documentation: ALL docs in top-level `docs/` folder (NO `backend/docs/` or component docs)

**🟡 PREFER** (use unless specifically overridden):
- Loco.rs patterns over custom implementations
- Trait abstractions for all external dependencies (LLM, storage)
- xFrame5 XML validation before returning artifacts
- Structured error responses over raw exceptions

**🟢 REFERENCE** (for comprehensive details):
- `docs/patterns/*.md` for implementation patterns
- `docs/features/*.md` for business requirements

---

## Project Overview

### Purpose
xFrame5 프론트엔드 개발 자동화 도구 - On-premise code assistant for xFrame5 UI development

### Key Goals
1. Generate xFrame5 XML view files from DB schema/query samples
2. Generate JavaScript event handler files following company standards
3. Reduce frontend development time by 50%+ for standard screen types
4. Zero external data transmission (금융권 보안 요구사항)

### PoC Scope
- **Target**: 회원 목록 + 조회 + 상세 팝업 화면
- **Excludes**: Backend generation, full automation, runtime intervention

---

## Project Structure

| Component | Path | Stack |
|-----------|------|-------|
| Agent Server | `backend/` | Rust + Loco.rs + SeaORM + PostgreSQL |
| Admin Panel | `backend/assets/views/admin/` | HTMX + Tera (Loco.rs built-in) |
| Eclipse Plugin | `eclipse-plugin/` | Java + Eclipse PDE |
| LLM Runtime | (external) | Ollama / llama.cpp / vLLM |
| Docs | `docs/` | Shared documentation |

---

## CRITICAL: Security & Isolation Rules

### LLM Abstraction (핵심 설계 원칙)
**❌ NEVER expose to plugin/API**:
- Model names (codellama, mistral, etc.)
- Temperature, token limits, system prompts
- Ollama/llama.cpp existence
- Prompt templates or structure

**✅ Plugin sees ONLY**:
```json
{
  "product": "xframe5-ui",
  "inputType": "db-schema",
  "input": { ... },
  "context": { "project": "xframe5", "output": ["xml", "javascript"] }
}
```

### On-Premise Requirement
- All processing must occur within customer network
- No external API calls, telemetry, or analytics
- Docker or native deployment only

### Audit Trail (금융권 필수)
- Input data: ❌ (개인정보 보호)
- Meta model/intent: ⭕
- Generated artifacts: ⭕
- Who/when/what: ⭕

---

## Agent Server Architecture

### Request Flow (변경 금지)
```
Plugin Request
    ↓
① Normalize (정규화)
    ↓
② Compile Prompt (DSL → Prompt)
    ↓
③ LLM Generate (trait-based)
    ↓
④ Parse & Validate (xFrame5 문법 검증)
    ↓
⑤ Response (artifacts + warnings)
```

### Core Trait (제품의 심장)
```rust
pub trait LlmBackend: Send + Sync {
    async fn generate(&self, prompt: &str) -> anyhow::Result<String>;
}
```

Implementations: `OllamaBackend`, `LlamaCppBackend`, (future GPU servers)

### Internal DSL
```rust
struct UiIntent {
    screen_name: String,
    datasets: Vec<Dataset>,
    grids: Vec<Grid>,
    actions: Vec<Action>,
}
```

LLM receives structured intent, not raw input.

---

## xFrame5 Code Generation

### XML Structure
- Dataset definitions with column bindings
- Grid components with header/data configurations
- UI controls with Attribute Map properties

### JavaScript Patterns
- Transaction stub functions (`fn_search`, `fn_save`, etc.)
- Dataset event handlers
- Grid cell event handlers

### Validation Rules
- XML must parse without errors
- Dataset/Grid bindings must match
- Missing info → `TODO` comments (never hide unknowns)

---

## Database Schema

### Core Tables
| Table | Purpose |
|-------|---------|
| `users` | System users (developers) |
| `roles` | Role-based access control |
| `prompt_templates` | LLM prompt templates (dynamic, no file storage) |
| `company_rules` | Customer-specific coding rules |
| `generation_logs` | Audit trail (who/when/output, NO input data) |

### Key Design: Templates in DB
- Enables dynamic updates without redeployment
- Supports customer-specific customization
- Version control for rollback
- See `docs/patterns/PROMPT_COMPILER.md`

---

## API Specification

### Generate Endpoint
```
POST /agent/generate
```

**Request**:
```json
{
  "product": "xframe5-ui",
  "inputType": "db-schema | query-sample | natural-language",
  "input": { "payload": "..." },
  "options": { "language": "ko", "strictMode": true },
  "context": {
    "project": "xframe5",
    "target": "frontend",
    "output": ["xml", "javascript"]
  }
}
```

**Response**:
```json
{
  "status": "success | error",
  "artifacts": {
    "xml": "<Dataset id=...>",
    "javascript": "this.fn_search = function() {...}"
  },
  "warnings": ["API endpoint not defined yet"],
  "meta": {
    "generator": "xframe5-ui-v1",
    "timestamp": "2025-xx-xx"
  }
}
```

---

## Backend Patterns

### Directory Structure
```
backend/
├── src/
│   ├── controllers/         # Thin - request/response only
│   │   └── admin/           # HTMX admin panel controllers
│   ├── services/            # Fat - business logic
│   │   └── admin/           # Admin CRUD services (pagination, validation)
│   ├── middleware/          # Custom extractors (cookie_auth)
│   ├── domain/              # Request, Artifact, InputKind types
│   ├── prompt/              # Prompt compiler + templates
│   ├── llm/                 # LlmBackend trait + implementations
│   └── validator/           # xFrame5 syntax validation
```

### Controller/Service Separation
- **Controllers**: Thin - HTTP handling, auth extraction, response formatting
- **Services**: Fat - business logic, validation, database operations
- See `docs/patterns/CONTROLLER_SERVICE_SEPARATION.md`

### Pagination Pattern
- Use service layer `search()` method with `QueryParams` and `PageResponse<T>`
- Multi-select filters with `Vec<T>`
- Default/max page size enforcement
- See `docs/patterns/PAGINATION_PATTERN.md`

### Configuration
- `config/development.yaml` - Local ollama settings
- `config/production.yaml` - Customer LLM server settings
- LLM endpoint/model configured here, NEVER in code
- **Prompt templates stored in database** (not text files) for dynamic updates

### Error Handling
- XML parse failure → Prompt retry (max 2)
- JS function mismatch → Post-processing correction
- Unknown requirements → Force `TODO` annotation

---

## Eclipse Plugin Patterns

### Design Principle
> 플러그인은 의도적으로 "멍청"해야 한다

**Plugin knows**:
- Input type (schema, query, natural language)
- Project context (selected file, package, path)
- Server endpoint URL

**Plugin MUST NOT know**:
- Model name, prompt structure
- LLM configuration
- Ollama/llama.cpp existence

### Plugin Flow
```
Collect Input → Call Server → Create Files
```

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Screen skeleton generation | < 5 minutes |
| Manual modification | < 50% of previous |
| Generated code quality | Pass existing code review |
| External data transmission | None |

### Architecture Validation
> 모델을 바꿨는데 Eclipse 플러그인은 단 한 줄도 안 바뀐다

This proves proper abstraction.

---

## Development Guidelines

### Code Quality
- **Backend**: Thin controllers, fat services
- **File Length**: < 200 lines target, 400 acceptable, 800+ must split
- **Testing**: Required for prompt compiler and validators

### LLM Prompt Development
1. Start with actual xFrame5 XML samples (2-3)
2. Define clear RULES in system prompt
3. Specify exact output format (--- XML --- / --- JS ---)
4. Include fallback for missing information

### Financial Industry Compliance
- No input data storage
- Meta model + output storage only
- Complete audit trail
- Rule-based validation (not pure LLM output)

---

## Quick Reference

### Backend Commands
```bash
cd backend

# Run server
cargo loco start

# Run with specific config
LOCO_ENV=development cargo loco start

# Generate migration
cargo loco generate migration create_generation_logs

# Run migrations
cargo loco db migrate

# Run tests
cargo test

# Run single test
cargo test test_name
```

### Eclipse Plugin Commands
```bash
cd eclipse-plugin

# Build plugin
mvn clean package

# Run tests
mvn test
```

---

## Documentation

### Implementation Plan
- **docs/implementation/IMPLEMENTATION_PLAN.md** - Phase-by-phase implementation guide with AI prompts

### Pattern Documentation (docs/patterns/)
1. **LLM_BACKEND_ABSTRACTION.md** - Trait design and implementations
2. **PROMPT_COMPILER.md** - DSL to prompt transformation
3. **XFRAME5_VALIDATION.md** - XML/JS validation rules
4. **AUDIT_LOGGING.md** - Generation request logging
5. **LOCO_MIGRATION_PATTERNS.md** - Database migration patterns
6. **ADMIN_PANEL.md** - HTMX admin panel architecture
7. **PAGINATION_PATTERN.md** - Pagination with service layer (from HWS)
8. **CONTROLLER_SERVICE_SEPARATION.md** - Thin controller, fat service pattern
9. **COOKIE_AUTH.md** - Cookie-based JWT auth for admin pages

### Feature Documentation (docs/features/)
1. **SCREEN_GENERATION.md** - List/Detail screen generation
2. **SCHEMA_INPUT.md** - DB schema input processing

---

**Version**: 0.1 (PoC) | **Updated**: 2025-12-28
