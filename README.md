# xFrame5 Code Generator

On-premise code assistant for xFrame5 frontend development automation. Generates XML view files and JavaScript event handlers from DB schema, query samples, or natural language descriptions.

## Features

- **Code Generation**: Generate xFrame5 XML views and JavaScript handlers
- **Multiple Input Types**: DB schema, SQL queries, or natural language
- **LLM Abstraction**: Supports Ollama, vLLM, llama.cpp, and cloud providers
- **Eclipse Plugin**: Integrated development experience
- **On-Premise**: All processing within your network (금융권 보안 요구사항)
- **Audit Trail**: Complete logging of generation requests

## Quick Start

```bash
# 1. Configure environment
cp .env.example .env
vim .env  # Update settings

# 2. Start services
./scripts/start.sh

# 3. Check health
curl http://localhost:3000/agent/health
```

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Eclipse Plugin │────▶│  Agent Server   │────▶│  LLM Runtime    │
│     (Java)      │     │  (Rust/Loco.rs) │     │  (Ollama/etc)   │
└─────────────────┘     └────────┬────────┘     └─────────────────┘
                                 │
                        ┌────────▼────────┐
                        │   PostgreSQL    │
                        │   (templates,   │
                        │    rules, logs) │
                        └─────────────────┘
```

## Project Structure

```
coder/
├── backend/              # Rust Agent Server (Loco.rs)
│   ├── src/
│   │   ├── controllers/  # API endpoints
│   │   ├── services/     # Business logic
│   │   ├── domain/       # Domain types (UiIntent DSL)
│   │   └── llm/          # LLM backend implementations
│   └── config/           # Environment configurations
├── eclipse-plugin/       # Eclipse Plugin (Java)
│   ├── src/              # Plugin source code
│   └── META-INF/         # Plugin manifest
├── docs/                 # Documentation
│   ├── patterns/         # Implementation patterns
│   └── features/         # Feature specifications
└── scripts/              # Deployment scripts
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/agent/generate` | POST | Generate xFrame5 code |
| `/agent/health` | GET | Health check |
| `/agent/products` | GET | List available generators |

### Generate Request

```json
{
  "product": "xframe5-ui",
  "input": {
    "type": "db_schema",
    "table": "member",
    "columns": [
      {"name": "id", "column_type": "INTEGER", "pk": true},
      {"name": "name", "column_type": "VARCHAR(100)", "nullable": false}
    ]
  },
  "options": {
    "language": "ko"
  }
}
```

### Generate Response

```json
{
  "status": "success",
  "artifacts": {
    "xml": "<Dataset id=\"ds_member\">...",
    "javascript": "this.fn_search = function() {...}"
  },
  "warnings": [],
  "meta": {
    "generator": "xframe5-ui-v1",
    "generation_time_ms": 1234
  }
}
```

## Development

### Backend

```bash
cd backend

# Run server
cargo loco start

# Run tests
cargo test

# Run with specific environment
LOCO_ENV=development cargo loco start
```

### Eclipse Plugin

```bash
cd eclipse-plugin

# Build
mvn clean package

# Run tests
mvn test
```

## Configuration

See [.env.example](.env.example) for all configuration options.

Key settings:
- `LLM_PROVIDER`: ollama | llama-cpp | vllm | groq | openai | anthropic
- `LLM_MODEL`: Model name (e.g., codellama:13b)
- `JWT_SECRET`: Authentication secret (change in production!)

## Documentation

- [Deployment Guide](docs/DEPLOYMENT.md)
- [Implementation Plan](docs/implementation/IMPLEMENTATION_PLAN.md)
- [Pattern Documentation](docs/patterns/)
- [Feature Specifications](docs/features/)

## Security

This system is designed for **on-premise deployment only**:

- No external API calls
- No telemetry or analytics
- Input data is NOT stored (privacy)
- Complete audit trail of generations

## License

Proprietary - Softbase Co., Ltd.

---

**Version**: 1.0.0 | **Status**: Production Ready
