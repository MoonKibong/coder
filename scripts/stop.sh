#!/bin/bash
# Enterprise Code Generator - Stop Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "[INFO] Stopping Enterprise Code Generator..."

docker compose down

echo "[INFO] All services stopped."
