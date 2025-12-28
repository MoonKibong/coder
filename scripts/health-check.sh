#!/bin/bash
# Enterprise Code Generator - Health Check Script

set -e

ENDPOINT="${1:-http://localhost:3000}"

echo "Checking Enterprise Code Generator health..."
echo ""

# Check agent server
echo -n "Agent Server: "
if response=$(curl -s -w "\n%{http_code}" "$ENDPOINT/agent/health" 2>/dev/null); then
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$http_code" = "200" ]; then
        echo "OK"
        echo "  Response: $body"
    else
        echo "FAILED (HTTP $http_code)"
    fi
else
    echo "FAILED (Connection refused)"
fi

echo ""

# Check products endpoint
echo -n "Products API: "
if response=$(curl -s -w "\n%{http_code}" "$ENDPOINT/agent/products" 2>/dev/null); then
    http_code=$(echo "$response" | tail -n1)

    if [ "$http_code" = "200" ]; then
        echo "OK"
    else
        echo "FAILED (HTTP $http_code)"
    fi
else
    echo "FAILED (Connection refused)"
fi

echo ""

# If using docker compose, show container status
if command -v docker &> /dev/null; then
    echo "Docker Container Status:"
    docker compose ps 2>/dev/null || echo "  (docker compose not available)"
fi
