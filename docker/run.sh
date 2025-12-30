#!/bin/bash
set -e

# Run the all-in-one Docker container

# Default model
MODEL="${OLLAMA_MODEL:-qwen2.5-coder:7b}"

# Check if persistent mode is requested
if [ "$1" == "--persistent" ] || [ "$1" == "-p" ]; then
    echo "Running with persistent volumes..."
    docker run -p 3000:3000 -p 11434:11434 \
        -e OLLAMA_MODEL="$MODEL" \
        -v coder-postgres-data:/var/lib/postgresql/14/main \
        -v coder-ollama-data:/root/.ollama \
        coder-allinone
else
    echo "Running without persistent volumes..."
    echo "(Use --persistent or -p flag to persist data)"
    docker run -p 3000:3000 -p 11434:11434 \
        -e OLLAMA_MODEL="$MODEL" \
        coder-allinone
fi
