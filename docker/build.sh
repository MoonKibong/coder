#!/bin/bash
set -e

# Build the all-in-one Docker image
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Building Code Generator All-in-One Docker image..."
echo "Project root: $PROJECT_ROOT"

cd "$PROJECT_ROOT"

# Build the image
docker build -f docker/Dockerfile.allinone -t coder-allinone .

echo ""
echo "Build complete!"
echo ""
echo "To run the container:"
echo "  docker run -p 3000:3000 -p 11434:11434 coder-allinone"
echo ""
echo "With persistent data:"
echo "  docker run -p 3000:3000 -p 11434:11434 \\"
echo "    -v coder-postgres-data:/var/lib/postgresql/14/main \\"
echo "    -v coder-ollama-data:/root/.ollama \\"
echo "    coder-allinone"
