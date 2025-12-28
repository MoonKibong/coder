#!/bin/bash
# xFrame5 Code Generator - Startup Script
#
# Usage:
#   ./scripts/start.sh           # Start all services
#   ./scripts/start.sh --dev     # Start in development mode
#   ./scripts/start.sh --build   # Rebuild and start

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if .env file exists
if [ ! -f ".env" ]; then
    log_warn ".env file not found. Creating from .env.example..."
    if [ -f ".env.example" ]; then
        cp .env.example .env
        log_info "Created .env file. Please review and update the values."
    else
        log_error ".env.example not found. Please create .env file manually."
        exit 1
    fi
fi

# Parse arguments
BUILD_FLAG=""
DEV_MODE=false

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --build) BUILD_FLAG="--build" ;;
        --dev) DEV_MODE=true ;;
        *) log_error "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

if [ "$DEV_MODE" = true ]; then
    log_info "Starting in development mode..."

    # Start only database and ollama
    docker compose up -d postgres ollama

    log_info "Waiting for database to be ready..."
    sleep 5

    # Run backend locally
    cd backend
    log_info "Starting backend server..."
    LOCO_ENV=development cargo loco start
else
    log_info "Starting xFrame5 Code Generator..."

    # Start all services
    docker compose up -d $BUILD_FLAG

    log_info "Waiting for services to be ready..."
    sleep 10

    # Check health
    log_info "Checking service health..."

    if curl -s http://localhost:3000/agent/health | grep -q "healthy\|degraded"; then
        log_info "Agent server is running!"
    else
        log_warn "Agent server health check failed. Check logs with: docker compose logs agent-server"
    fi

    log_info ""
    log_info "============================================"
    log_info "xFrame5 Code Generator is starting up!"
    log_info "============================================"
    log_info ""
    log_info "Services:"
    log_info "  - Agent Server: http://localhost:3000"
    log_info "  - Health Check: http://localhost:3000/agent/health"
    log_info "  - API Docs:     http://localhost:3000/agent/products"
    log_info ""
    log_info "Commands:"
    log_info "  - View logs:    docker compose logs -f"
    log_info "  - Stop:         docker compose down"
    log_info "  - Restart:      docker compose restart"
    log_info ""
fi
