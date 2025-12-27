# Documentation

> **Source of Truth** for xFrame5 Code Assistant

## Folder Structure

```
docs/
├── README.md              # This file
├── requirements.md        # Project requirements and PoC scope
├── implementation/        # Implementation guides
│   └── IMPLEMENTATION_PLAN.md  # Phase-by-phase plan with AI prompts
├── patterns/              # Implementation patterns
│   ├── LLM_BACKEND_ABSTRACTION.md
│   ├── PROMPT_COMPILER.md
│   ├── XFRAME5_VALIDATION.md
│   ├── AUDIT_LOGGING.md
│   └── LOCO_MIGRATION_PATTERNS.md
└── features/              # Feature specifications
    ├── SCREEN_GENERATION.md
    └── SCHEMA_INPUT.md
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

### When to Create New Feature Docs
- New screen types to support
- New input types (beyond schema, query, natural language)
- New output formats

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
