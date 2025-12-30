#!/bin/bash
set -e

echo "========================================="
echo "Starting Code Generator All-in-One"
echo "========================================="

# Start PostgreSQL
echo "[1/4] Starting PostgreSQL..."
service postgresql start
sleep 3

# Wait for PostgreSQL to be ready
until pg_isready -h localhost -p 5432 -U coder; do
    echo "Waiting for PostgreSQL..."
    sleep 2
done
echo "PostgreSQL is ready!"

# Run database migrations
echo "[2/4] Running database migrations..."
cd /app/backend
./target/release/coder-cli db migrate
./target/release/coder-cli db seed

# Start Ollama in background
echo "[3/4] Starting Ollama..."
ollama serve &
OLLAMA_PID=$!
sleep 5

# Wait for Ollama to be ready
until curl -s http://localhost:11434/api/tags > /dev/null 2>&1; do
    echo "Waiting for Ollama..."
    sleep 2
done
echo "Ollama is ready!"

# Pull the model if specified and not already present
if [ -n "$OLLAMA_MODEL" ]; then
    echo "Checking for model: $OLLAMA_MODEL"
    if ! ollama list | grep -q "$OLLAMA_MODEL"; then
        echo "Pulling model: $OLLAMA_MODEL (this may take a while)..."
        ollama pull "$OLLAMA_MODEL"
    else
        echo "Model $OLLAMA_MODEL already exists"
    fi
fi

# Update LLM config in database to use the correct model
echo "Updating LLM configuration..."
PGPASSWORD=coder_password psql -h localhost -U coder -d coder_development -c "
UPDATE llm_configs
SET endpoint_url = 'http://localhost:11434',
    model_name = '${OLLAMA_MODEL:-qwen2.5-coder:7b}',
    is_active = true
WHERE id = 1;
UPDATE llm_configs SET is_active = false WHERE id != 1;
" || true

# Start the backend
echo "[4/4] Starting Backend Server..."
echo "========================================="
echo "Services running:"
echo "  - Backend API: http://localhost:3000"
echo "  - Admin Panel: http://localhost:3000/admin"
echo "  - Ollama API:  http://localhost:11434"
echo "========================================="
echo ""
echo "Default admin login:"
echo "  Email: admin@example.com"
echo "  Password: 12341234"
echo "========================================="

cd /app/backend
exec ./target/release/coder-cli start
