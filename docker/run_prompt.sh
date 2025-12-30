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
    echo "$1" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | sed 's/\t/\\t/g' | sed ':a;N;$!ba;s/\n/\\n/g'
}

ESCAPED_PROMPT=$(escape_json "$PROMPT")

case $MODE in
    gen|generate)
        echo "=== Code Generation ==="
        echo "Product: $PRODUCT"
        echo "Prompt: $PROMPT"
        echo ""

        curl -s -X POST "${SERVER_URL}/api/agent/generate" \
            -H "Content-Type: application/json" \
            -d "{
                \"product\": \"${PRODUCT}\",
                \"inputType\": \"natural-language\",
                \"input\": {
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
            }" | jq . 2>/dev/null || cat
        ;;

    qa|question)
        echo "=== Q&A ==="
        echo "Product: $PRODUCT"
        echo "Question: $PROMPT"
        echo ""

        curl -s -X POST "${SERVER_URL}/api/agent/qa" \
            -H "Content-Type: application/json" \
            -d "{
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
            }" | jq . 2>/dev/null || cat
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

        curl -s -X POST "${SERVER_URL}/api/agent/review" \
            -H "Content-Type: application/json" \
            -d "{
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
            }" | jq . 2>/dev/null || cat
        ;;

    *)
        echo "Error: Invalid mode '$MODE'. Use gen, qa, or review."
        exit 1
        ;;
esac

echo ""
