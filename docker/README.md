# Docker All-in-One Image

A single Docker image containing PostgreSQL, Ollama (LLM), and the Code Generator backend.

## Quick Start

### Option 1: Pull from GitHub Container Registry (Recommended)

```bash
# Pull the latest image
docker pull ghcr.io/moonkibong/coder:latest

# Run the container
docker run -p 3000:3000 -p 11434:11434 ghcr.io/moonkibong/coder:latest
```

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/MoonKibong/coder.git
cd coder

# Build the image
docker build -f docker/Dockerfile.allinone -t coder-allinone .

# Run the container
docker run -p 3000:3000 -p 11434:11434 coder-allinone
```

### Run the container

```bash
# Basic run (uses default model: qwen2.5-coder:7b)
docker run -p 3000:3000 -p 11434:11434 coder-allinone

# With a specific Ollama model
docker run -p 3000:3000 -p 11434:11434 -e OLLAMA_MODEL=codellama:7b coder-allinone

# With persistent data (recommended for testing)
docker run -p 3000:3000 -p 11434:11434 \
  -v coder-postgres-data:/var/lib/postgresql/14/main \
  -v coder-ollama-data:/root/.ollama \
  coder-allinone
```

## Services

| Service | Port | URL |
|---------|------|-----|
| Backend API | 3000 | http://localhost:3000 |
| Admin Panel | 3000 | http://localhost:3000/admin |
| Ollama API | 11434 | http://localhost:11434 |

## Default Credentials

**Admin Panel:**
- Email: admin@example.com
- Password: 12341234

**PostgreSQL:**
- User: coder
- Password: coder_password
- Database: coder_development

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| OLLAMA_MODEL | qwen2.5-coder:7b | Ollama model to use |
| DATABASE_URL | postgres://coder:coder_password@localhost:5432/coder_development | Database connection |
| RUST_LOG | info | Log level |

## Eclipse Plugin Configuration

Configure the Eclipse plugin to connect to this container:

1. Open Eclipse preferences: Window > Preferences
2. Navigate to: Code Generator
3. Set Server Endpoint: `http://<docker-host>:3000`

If running Docker locally, use `http://localhost:3000`.

## Model Selection

The image supports any Ollama-compatible model. Recommended models:

| Model | Size | Best For |
|-------|------|----------|
| qwen2.5-coder:7b | ~4GB | Balanced (default) |
| codellama:7b | ~4GB | Code generation |
| codellama:13b | ~7GB | Higher quality |
| deepseek-coder:6.7b | ~4GB | Code completion |

Note: First run will download the model, which may take several minutes depending on network speed.

## Troubleshooting

### Container fails to start
Check logs:
```bash
docker logs <container-id>
```

### Model download is slow
Pre-pull the model:
```bash
docker run -it --rm -v coder-ollama-data:/root/.ollama ollama/ollama pull qwen2.5-coder:7b
```

### Out of memory
Ensure your Docker has at least 8GB memory allocated for 7B models, 16GB for 13B models.

## Building for Different Architectures

```bash
# For ARM64 (Apple Silicon, etc.)
docker build --platform linux/arm64 -f docker/Dockerfile.allinone -t coder-allinone:arm64 .

# For AMD64 (Intel/AMD)
docker build --platform linux/amd64 -f docker/Dockerfile.allinone -t coder-allinone:amd64 .
```
