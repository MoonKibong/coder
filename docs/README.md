# Documentation

> **Source of Truth** for Enterprise Code Generator

## Monorepo Documentation Structure

**IMPORTANT**: All documentation lives in this top-level `docs/` folder, NOT in component subdirectories.

```
coder/
├── docs/                    # ALL documentation here (monorepo pattern)
│   ├── README.md
│   ├── patterns/
│   ├── features/
│   └── implementation/
├── backend/                 # Rust server (NO docs/ subfolder)
├── eclipse-plugin/          # Eclipse plugin (NO docs/ subfolder)
└── CLAUDE.md               # AI agent instructions
```

Do NOT create `backend/docs/` or `eclipse-plugin/docs/`. All docs go in the top-level `docs/` folder.

## Folder Structure

```
docs/
├── README.md              # This file
├── requirements.md        # Project requirements and PoC scope
├── DEPLOYMENT.md          # Deployment guide
├── implementation/        # Implementation guides
│   └── IMPLEMENTATION_PLAN.md
├── patterns/              # Implementation patterns
│   ├── LLM_BACKEND_ABSTRACTION.md   # LLM trait design
│   ├── PROMPT_COMPILER.md           # DSL to prompt transformation
│   ├── XFRAME5_VALIDATION.md        # XML/JS validation
│   ├── AUDIT_LOGGING.md             # Generation logging
│   ├── LOCO_MIGRATION_PATTERNS.md   # Database migrations
│   ├── ADMIN_PANEL.md               # HTMX admin UI
│   ├── PAGINATION_PATTERN.md        # Pagination with service layer
│   ├── CONTROLLER_SERVICE_SEPARATION.md  # Thin controller pattern
│   └── COOKIE_AUTH.md               # Cookie-based JWT auth
└── features/              # Feature specifications
    ├── SCREEN_GENERATION.md         # List/Detail screen generation
    ├── SCHEMA_INPUT.md              # DB schema input processing
    ├── CODE_REVIEW.md               # AI-powered code review
    └── QA_CHATBOT.md                # Knowledge-based Q&A
```

## Naming Conventions

### Pattern Files
- `UPPERCASE_WITH_UNDERSCORES.md`
- Example: `LLM_BACKEND_ABSTRACTION.md`

### Feature Files
- `UPPERCASE_WITH_UNDERSCORES.md`
- Named by feature area

## Contributing

### When to Create New Pattern Docs
- Reusable implementation patterns
- Cross-cutting concerns (logging, validation)
- Integration patterns (LLM, external systems)
- Architecture patterns (controller/service separation)

### When to Create New Feature Docs
- New screen types to support
- New input types (beyond schema, query, natural language)
- New output formats

### Documentation Rules
1. **Monorepo pattern**: All docs in top-level `docs/` folder
2. **Reference HWS project**: For established patterns, reference `../HWS/docs/patterns/`
3. **Keep CLAUDE.md lean**: Move details to pattern docs, reference from CLAUDE.md

## Key Patterns (Quick Reference)

| Pattern | File | Purpose |
|---------|------|---------|
| Pagination | `PAGINATION_PATTERN.md` | Search with filters, sorting, pagination |
| Controller/Service | `CONTROLLER_SERVICE_SEPARATION.md` | Thin controllers, fat services |
| Cookie Auth | `COOKIE_AUTH.md` | HTTP-only JWT cookies for admin pages |
| Admin Panel | `ADMIN_PANEL.md` | HTMX-based CRUD interface |

## Tech Stack Rationale

### Why Rust (Loco.rs)?
- LLM 서버 옆에 배치해도 안정적
- 프롬프트 템플릿 컴파일에 강함
- 금융권이 요구하는 "통제 가능성"
- 단일 바이너리 배포 가능

### Why Eclipse Plugin?
- 고객사 기존 개발 환경 (Eclipse-based xFrame5 IDE)
- 기존 워크플로우 변경 없음

### Why On-Premise Only?
- 금융권 보안 요구사항
- 데이터 외부 전송 금지
- 감사 대응 필수

## Development Workflow

### AI Agent Collaboration
1. CLAUDE.md의 Priority Guide 참조
2. 보안 규칙 (🔴) 항상 준수
3. LLM 추상화 유지

### Pattern Documentation Updates
- 새로운 패턴 발견 시 문서화
- CLAUDE.md 용량 초과 시 상세 내용 이동
- HWS 프로젝트의 패턴 참조 및 적용
