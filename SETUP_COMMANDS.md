# Enterprise Code Generator - Setup Commands

**Project**: Enterprise Code Generator (엔터프라이즈 코드 생성기)
**Date**: 2025-12-28
**Tech Stack**: Loco.rs (backend) + Eclipse Plugin (Java/Tycho) + PostgreSQL + Ollama/llama.cpp

---

## Prerequisites

### Required Versions
- **Rust**: 1.91+ (stable)
- **Maven**: 3.9+ (IMPORTANT: Maven 3.3.x has XML parsing bugs)
- **Java**: 11+ (for Eclipse plugin compilation)
- **PostgreSQL**: 15+
- **Docker**: 24+
- **Docker Compose**: 2.20+

### Verify Installations
```bash
# Check Rust version (should be 1.91.0 or higher)
rustc --version
cargo --version

# Check Maven version (MUST be 3.9+ to avoid XML parsing bugs)
mvn --version

# Check Java version (should be 11+)
java --version

# Check Docker version
docker --version
docker compose version

# Check PostgreSQL client (optional, for verification)
psql --version
```

---

## Project Structure (Monorepo)

```
coder/
├── backend/           # Loco.rs application
├── eclipse-plugin/    # Eclipse Tycho plugin (Java)
├── docs/              # Shared documentation
├── scripts/           # Deployment scripts
├── CLAUDE.md          # AI agent context
├── SETUP_COMMANDS.md  # This file
└── docker-compose.yml # Development environment
```

---

## Step 1: Clone Repository (if not already done)

```bash
# If starting fresh
mkdir coder
cd coder
git init
```

---

## Step 2: Backend Setup (Loco.rs)

### 2.1: Install Loco CLI (if not installed)
```bash
# Install loco-cli globally
cargo install loco-cli

# Install sea-orm-cli for database operations
cargo install sea-orm-cli
```

### 2.2: Build Backend
```bash
cd backend

# Build dependencies (this will take a few minutes on first run)
cargo build

# Run Loco doctor to verify setup
cargo loco doctor
```

### 2.3: Backend with Native LLM (Optional)
If you want to use native llama.cpp bindings (instead of Ollama server):

```bash
# Build with local-llm feature
cargo build --features local-llm

# This requires llama.cpp to be available on the system
# Download models to ~/.llm-models/ or set LLM_MODEL_PATH
```

---

## Step 3: Eclipse Plugin Setup (Java/Tycho)

### 3.1: Upgrade Maven (if version < 3.9)
```bash
# On macOS with Homebrew
brew install maven

# Verify new version is in PATH
/usr/local/bin/mvn --version  # Should be 3.9+

# If old Maven is still used, update PATH
export PATH="/usr/local/bin:$PATH"
```

### 3.2: Build Eclipse Plugin
```bash
cd eclipse-plugin

# Build with XML entity size limits disabled (for Java 25+)
MAVEN_OPTS="-Djdk.xml.maxGeneralEntitySizeLimit=0 -Djdk.xml.entityExpansionLimit=0 -Djdk.xml.totalEntitySizeLimit=0" mvn clean package

# Or for Java 11-17 (no special options needed)
mvn clean package
```

### 3.3: Plugin Artifacts
After successful build, the plugin JAR is at:
```
eclipse-plugin/target/com.kosac.ai.codegen-1.0.0-SNAPSHOT.jar
```

---

## Step 4: Development Environment (Docker)

### 4.1: Start Development Environment
```bash
# From coder root directory
docker compose up -d

# Verify services are running
docker compose ps

# Check logs if needed
docker compose logs -f postgres
docker compose logs -f ollama
```

### 4.2: Access Services
- **Backend Server**: `localhost:3000` (when running)
- **Admin Panel**: `localhost:3000/admin/` (when running)

**Note**: PostgreSQL and Ollama ports are NOT exposed externally for security.
To access them, use `docker compose exec`:
```bash
# PostgreSQL CLI
docker compose exec postgres psql -U coder -d coder_development

# Ollama API
docker compose exec ollama ollama list
```

For **local development without Docker**, expose ports by uncommenting in `docker-compose.yml`:
```yaml
# PostgreSQL
ports:
  - "5432:5432"

# Ollama
ports:
  - "11434:11434"
```

---

## Step 5: Environment Configuration

### 5.1: Backend Environment
Copy `.env.example` to `.env` and configure:

```bash
cp .env.example .env
```

Key settings in `.env`:
```bash
# LLM Provider: ollama | llama-cpp | local-llama-cpp | vllm | groq | openai | anthropic
LLM_PROVIDER=ollama

# Database
DATABASE_URL=postgres://coder:coder_password@localhost:5432/coder_development

# JWT (CHANGE IN PRODUCTION!)
JWT_SECRET=your-secret-key-change-this-in-production

# For local-llama-cpp provider
# LLM_MODEL_PATH=/path/to/model.gguf
# LLM_CONTEXT_SIZE=4096
# LLM_THREADS=4
```

---

## Step 6: Database Initialization

### 6.1: Run Migrations
```bash
cd backend

# Create database (if not exists)
cargo loco db create

# Run migrations
cargo loco db migrate

# Seed initial data (if available)
LOCO_ENV=development cargo loco db seed
```

---

## Step 7: Verify Installation

### 7.1: Backend Verification
```bash
cd backend

# Run tests
cargo test

# Start backend server
cargo loco start

# In another terminal, test health endpoint
curl http://localhost:3000/agent/health

# Expected response:
# {"status":"healthy",...}
```

### 7.2: Eclipse Plugin Verification
```bash
cd eclipse-plugin

# Run tests (after Maven build)
MAVEN_OPTS="-Djdk.xml.maxGeneralEntitySizeLimit=0" mvn test

# Plugin JAR should be in target/
ls -la target/*.jar
```

---

## Troubleshooting

### Issue: Maven XML parsing error (ArrayIndexOutOfBoundsException)
```bash
# This happens with Maven < 3.9
# Solution: Upgrade Maven
brew install maven
export PATH="/usr/local/bin:$PATH"
```

### Issue: JDK XML entity size limit error
```bash
# This happens with Java 17+ and large XML files
# Solution: Set MAVEN_OPTS
export MAVEN_OPTS="-Djdk.xml.maxGeneralEntitySizeLimit=0 -Djdk.xml.entityExpansionLimit=0 -Djdk.xml.totalEntitySizeLimit=0"
mvn clean package
```

### Issue: Rust version mismatch
```bash
# Update to Rust 1.91+
rustup update stable

# Verify version
rustc --version
```

### Issue: Loco CLI not found
```bash
# Ensure cargo bin is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Add to ~/.zshrc or ~/.bashrc for persistence
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### Issue: Port already in use
```bash
# Find process using port 5432 (PostgreSQL)
lsof -i :5432

# Kill process if needed
kill -9 <PID>

# Or change port in docker-compose.yml
```

### Issue: Ollama model not found
```bash
# Pull required model
docker compose exec ollama ollama pull codellama:13b

# Or for smaller model
docker compose exec ollama ollama pull codellama:7b
```

---

## Development Workflow

### Daily Development Startup
```bash
# 1. Start Docker services
docker compose up -d

# 2. Start backend (Terminal 1)
cd backend
cargo loco start

# 3. Access admin panel
open http://localhost:3000/admin/

# Optional: Build Eclipse plugin
cd eclipse-plugin
MAVEN_OPTS="-Djdk.xml.maxGeneralEntitySizeLimit=0" mvn clean package
```

### Daily Development Shutdown
```bash
# Stop Docker services (from coder root)
docker compose down

# Or keep data and just stop
docker compose stop
```

---

## Quick Commands Reference

```bash
# Backend
cd backend
cargo loco start                    # Start server
cargo test                          # Run tests
cargo loco db migrate               # Run migrations
LOCO_ENV=test cargo loco start      # Start with test config

# Eclipse Plugin
cd eclipse-plugin
MAVEN_OPTS="-Djdk.xml.maxGeneralEntitySizeLimit=0" mvn clean package
mvn test                            # Run tests

# Docker
docker compose up -d                # Start services
docker compose ps                   # Check status
docker compose logs -f              # Follow logs
docker compose down                 # Stop services

# Health Check
curl http://localhost:3000/agent/health
```

---

## Version Summary

**Backend Stack:**
- Rust: 1.91+
- Loco.rs: 0.12+
- SeaORM: 1.1+
- PostgreSQL: 15+

**Eclipse Plugin Stack:**
- Java: 11+
- Maven: 3.9+ (REQUIRED)
- Eclipse Tycho: 4.0.4
- Target Platform: Eclipse 2023-12

**LLM Stack:**
- Ollama: latest (default)
- llama.cpp: via llama-cpp-2 crate (optional)
- Supported providers: ollama, llama-cpp, local-llama-cpp, vllm, groq, openai, anthropic

**DevOps:**
- Docker: 24+
- Docker Compose: 2.20+

---

## Next Steps

After completing this setup:

1. **Verify all services are running** using Step 7 verification commands
2. **Read CLAUDE.md** for project context and AI agent guidelines
3. **Read docs/README.md** for documentation structure
4. **Access admin panel** at `http://localhost:3000/admin/` to configure LLM settings

---

**Maintainer**: Development Team
**Last Updated**: 2025-12-28
**Status**: Ready for development
