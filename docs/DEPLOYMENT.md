# xFrame5 Code Generator - Deployment Guide

This guide covers the deployment of the xFrame5 Code Generator for on-premise environments.

## Prerequisites

- Docker 24.0+ and Docker Compose 2.0+
- 8GB RAM minimum (16GB recommended for LLM)
- 20GB disk space (for models and data)
- Network access between components (internal only)

## Quick Start

### 1. Clone and Configure

```bash
# Clone the repository
git clone <repository-url>
cd coder

# Create environment configuration
cp .env.example .env

# Edit .env with your settings
vim .env
```

### 2. Start Services

```bash
# Start all services
./scripts/start.sh

# Or with rebuild
./scripts/start.sh --build

# Check health
./scripts/health-check.sh
```

### 3. Verify Installation

```bash
# Check health endpoint
curl http://localhost:3000/agent/health

# Expected response:
# {"status":"healthy","llm_available":true}
```

## Environment Configuration

### Required Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `POSTGRES_PASSWORD` | Database password | `coder_password` |
| `JWT_SECRET` | JWT signing secret | **MUST CHANGE** |
| `LLM_MODEL` | LLM model to use | `codellama:13b` |

### Optional Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `APP_PORT` | Server port | `3000` |
| `LLM_PROVIDER` | LLM backend | `ollama` |
| `LLM_TIMEOUT` | Request timeout (sec) | `120` |
| `RUST_LOG` | Log level | `info` |

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    Docker Network                         │
│                                                          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐  │
│  │   Agent     │    │  PostgreSQL │    │   Ollama    │  │
│  │   Server    │───▶│   Database  │    │   (LLM)     │  │
│  │  (Rust)     │    │             │    │             │  │
│  └──────┬──────┘    └─────────────┘    └──────▲──────┘  │
│         │                                      │         │
│         └──────────────────────────────────────┘         │
│                                                          │
└────────────────────────┬─────────────────────────────────┘
                         │
                    Port 3000
                         │
                  ┌──────▼──────┐
                  │   Eclipse   │
                  │   Plugin    │
                  └─────────────┘
```

## LLM Configuration

### Using Ollama (Default)

Ollama is the default LLM provider, running locally.

```bash
# .env
LLM_PROVIDER=ollama
LLM_MODEL=codellama:13b
```

Available models:
- `codellama:13b` - Recommended for code generation
- `codellama:7b` - Smaller, faster
- `mistral` - Alternative model

### Using vLLM (GPU Server)

For dedicated GPU servers:

```bash
# .env
LLM_PROVIDER=vllm
LLM_ENDPOINT=http://your-vllm-server:8000
LLM_MODEL=codellama/CodeLlama-13b-hf
```

### Using llama.cpp

For CPU-only environments:

```bash
# .env
LLM_PROVIDER=llama-cpp
LLM_ENDPOINT=http://your-llamacpp-server:8080
LLM_MODEL=codellama
```

## Database Management

### Run Migrations

```bash
# Using docker
docker compose exec agent-server coder db migrate

# Or directly
cd backend
cargo loco db migrate
```

### Backup Database

```bash
# Create backup
docker compose exec postgres pg_dump -U coder coder_production > backup.sql

# Restore backup
docker compose exec -T postgres psql -U coder coder_production < backup.sql
```

## Monitoring

### View Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f agent-server

# Last 100 lines
docker compose logs --tail=100 agent-server
```

### Health Checks

```bash
# Quick check
./scripts/health-check.sh

# Manual check
curl http://localhost:3000/agent/health
```

### Metrics

The agent server exposes the following endpoints:

| Endpoint | Description |
|----------|-------------|
| `/agent/health` | Health status |
| `/agent/products` | Available generators |

## Security Considerations

### On-Premise Requirements

This system is designed for **on-premise deployment only**:

1. **No External Calls**: All processing occurs within your network
2. **No Telemetry**: No analytics or usage data is sent externally
3. **Data Privacy**: Input data is NOT stored (only metadata)

### Security Checklist

- [ ] Change `JWT_SECRET` from default
- [ ] Change `POSTGRES_PASSWORD` from default
- [ ] Restrict network access to port 3000
- [ ] Use HTTPS reverse proxy in production
- [ ] Review audit logs regularly

### Recommended: HTTPS Proxy

For production, use a reverse proxy with HTTPS:

```nginx
# nginx.conf
server {
    listen 443 ssl;
    server_name codegen.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Troubleshooting

### Agent Server Won't Start

1. Check database connection:
   ```bash
   docker compose logs postgres
   ```

2. Check if port is in use:
   ```bash
   lsof -i :3000
   ```

3. Verify environment variables:
   ```bash
   docker compose config
   ```

### LLM Not Available

1. Check Ollama status:
   ```bash
   docker compose logs ollama
   ```

2. Verify model is pulled:
   ```bash
   docker compose exec ollama ollama list
   ```

3. Manually pull model:
   ```bash
   docker compose exec ollama ollama pull codellama:13b
   ```

### Slow Response Times

1. Check LLM resource usage:
   ```bash
   docker stats
   ```

2. Consider using a smaller model:
   ```bash
   # .env
   LLM_MODEL=codellama:7b
   ```

3. Increase timeout if needed:
   ```bash
   # .env
   LLM_TIMEOUT=300
   ```

## Upgrading

### Standard Upgrade

```bash
# Pull latest changes
git pull

# Rebuild and restart
./scripts/stop.sh
./scripts/start.sh --build
```

### Database Migration

Migrations run automatically on startup. For manual control:

```bash
# Check pending migrations
docker compose exec agent-server coder db status

# Run migrations
docker compose exec agent-server coder db migrate
```

## Support

For issues and questions:
- GitHub Issues: `<repository-url>/issues`
- Documentation: `docs/` directory

---

**Version**: 1.0.0
**Last Updated**: 2025-12-28
