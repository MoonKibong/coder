#!/bin/bash
set -e

# Simple CLI for testing Code Generator API
# Usage: ./run_prompt.sh --mode gen|qa|review --prompt "your prompt here" [options]

# Default values
SERVER_URL="${SERVER_URL:-http://localhost:3000}"
PRODUCT="${PRODUCT:-xframe5-ui}"
LANGUAGE="${LANGUAGE:-ko}"
MODE=""
PROMPT=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -m|--mode)
            MODE="$2"
            shift 2
            ;;
        -p|--prompt)
            PROMPT="$2"
            shift 2
            ;;
        --product)
            PRODUCT="$2"
            shift 2
            ;;
        --server)
            SERVER_URL="$2"
            shift 2
            ;;
        --lang)
            LANGUAGE="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 --mode <gen|qa|review> --prompt \"your prompt\""
            echo ""
            echo "Options:"
            echo "  -m, --mode     Mode: gen (generate), qa (question), review (code review)"
            echo "  -p, --prompt   The prompt text"
            echo "  --product      Product: xframe5-ui or spring-backend (default: xframe5-ui)"
            echo "  --server       Server URL (default: http://localhost:3000)"
            echo "  --lang         Language: ko or en (default: ko)"
            echo "  -h, --help     Show this help"
            echo ""
            echo "Examples:"
            echo "  $0 --mode gen --prompt \"generate a member list page with search\""
            echo "  $0 --mode qa --prompt \"How do I use Dataset in xFrame5?\""
            echo "  $0 --mode review --prompt '<Screen id=\"test\">...</Screen>'"
            echo "  $0 --mode gen --product spring-backend --prompt \"create CRUD for User entity\""
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate required arguments
if [ -z "$MODE" ]; then
    echo "Error: --mode is required (gen, qa, or review)"
    exit 1
fi

if [ -z "$PROMPT" ]; then
    echo "Error: --prompt is required"
    exit 1
fi

# Escape JSON special characters in prompt
escape_json() {
    local input="$1"
    # Escape backslashes, quotes, tabs, and newlines for JSON
    input="${input//\\/\\\\}"    # backslash
    input="${input//\"/\\\"}"    # double quote
    input="${input//$'\t'/\\t}"  # tab
    input="${input//$'\n'/\\n}"  # newline
    echo "$input"
}

ESCAPED_PROMPT=$(escape_json "$PROMPT")

# Create temp file for output
TEMP_OUTPUT=$(mktemp)
trap "rm -f $TEMP_OUTPUT" EXIT

# Function to run curl with timing
run_api_call() {
    local url="$1"
    local data="$2"

    local start_time=$(date +%s)
    echo "  Waiting for LLM response (this may take 1-3 minutes)..."
    echo ""

    # Run curl and save to temp file
    local http_code
    http_code=$(curl -s -w "%{http_code}" --max-time 300 -X POST "$url" \
        -H "Content-Type: application/json" \
        -d "$data" \
        -o "$TEMP_OUTPUT" 2>&1)
    local curl_exit=$?

    local end_time=$(date +%s)
    local elapsed=$((end_time - start_time))

    echo "  [✓] Response received in ${elapsed}s (HTTP: $http_code)"
    echo ""
    echo "--- Response ---"

    # Pretty print if jq is available, otherwise cat
    if command -v jq &> /dev/null; then
        jq . "$TEMP_OUTPUT" 2>/dev/null || cat "$TEMP_OUTPUT"
    else
        cat "$TEMP_OUTPUT"
    fi

    echo ""
    echo "--- End ---"

    return $curl_exit
}

case $MODE in
    gen|generate)
        echo "=== Code Generation ==="
        echo "Product: $PRODUCT"
        echo "Prompt: $PROMPT"
        echo ""

        run_api_call "${SERVER_URL}/agent/generate" "{
            \"product\": \"${PRODUCT}\",
            \"input\": {
                \"type\": \"natural_language\",
                \"description\": \"${ESCAPED_PROMPT}\"
            },
            \"options\": {
                \"language\": \"${LANGUAGE}\",
                \"strictMode\": false
            },
            \"context\": {
                \"project\": \"test\",
                \"target\": \"frontend\",
                \"output\": [\"xml\", \"javascript\"]
            }
        }"
        ;;

    qa|question)
        echo "=== Q&A ==="
        echo "Product: $PRODUCT"
        echo "Question: $PROMPT"
        echo ""

        run_api_call "${SERVER_URL}/agent/qa" "{
            \"product\": \"${PRODUCT}\",
            \"input\": {
                \"question\": \"${ESCAPED_PROMPT}\",
                \"context\": \"\"
            },
            \"options\": {
                \"language\": \"${LANGUAGE}\",
                \"includeExamples\": true,
                \"maxReferences\": 5
            }
        }"
        ;;

    review)
        echo "=== Code Review ==="
        echo "Product: $PRODUCT"
        echo "Code length: ${#PROMPT} chars"
        echo ""

        # Detect file type from content
        FILE_TYPE="xml"
        if [[ "$PROMPT" == *"function "* ]] || [[ "$PROMPT" == *"var "* ]]; then
            FILE_TYPE="javascript"
        elif [[ "$PROMPT" == *"public class"* ]] || [[ "$PROMPT" == *"@Controller"* ]]; then
            FILE_TYPE="java"
        fi

        run_api_call "${SERVER_URL}/agent/review" "{
            \"product\": \"${PRODUCT}\",
            \"input\": {
                \"code\": \"${ESCAPED_PROMPT}\",
                \"fileType\": \"${FILE_TYPE}\",
                \"context\": \"\"
            },
            \"options\": {
                \"language\": \"${LANGUAGE}\",
                \"reviewFocus\": [\"syntax\", \"patterns\", \"performance\", \"security\"]
            }
        }"
        ;;

    *)
        echo "Error: Invalid mode '$MODE'. Use gen, qa, or review."
        exit 1
        ;;
esac

echo "Done."
