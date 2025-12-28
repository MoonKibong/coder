#!/bin/bash
# xFrame5 Code Generator - Stop Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "[INFO] Stopping xFrame5 Code Generator..."

docker compose down

echo "[INFO] All services stopped."
