# Q&A Chatbot Feature

Knowledge-based Q&A for xFrame5 and Spring Framework questions.

## Overview

The Q&A Chatbot feature provides:
- Natural language question answering
- Knowledge base integration
- Code examples in responses
- Related topic suggestions
- Reference links to documentation

## API Endpoint

```
POST /api/agent/qa
```

### Request

```json
{
  "product": "xframe5-ui | spring-backend",
  "input": {
    "question": "How do I use Dataset in xFrame5?",
    "context": "Building a list screen with grid"
  },
  "options": {
    "language": "ko | en",
    "includeExamples": true,
    "maxReferences": 5
  }
}
```

### Response

```json
{
  "status": "success",
  "answer": {
    "text": "Detailed answer with markdown formatting...",
    "codeExamples": [
      {
        "language": "xml",
        "code": "<Dataset id=\"ds_member\">\n  <Column id=\"member_id\" type=\"STRING\"/>\n</Dataset>"
      },
      {
        "language": "javascript",
        "code": "function fn_search() {\n  gfn_transaction(\"ds_member\", \"select\");\n}"
      }
    ],
    "relatedTopics": ["Grid Component", "Data Binding", "Transaction Functions"]
  },
  "references": [
    {
      "knowledgeId": 15,
      "name": "Dataset Component",
      "relevance": 0.95
    },
    {
      "knowledgeId": 23,
      "name": "Grid Binding",
      "relevance": 0.82
    }
  ]
}
```

## Eclipse Plugin Usage

### Ask Question

1. Menu: **xFrame5 > Ask Question...** (or **Spring > Ask Question...**)
2. Enter your question in the dialog
3. Optionally add context about what you're building
4. Click "Ask"
5. View the answer with code examples and references

### Result Dialog Features

- **Answer**: Formatted text with markdown support
- **Code Examples**: Syntax-highlighted code blocks with copy button
- **References**: Links to related knowledge base entries
- **Related Topics**: Tags for exploring related concepts

## CLI Testing

```bash
# xFrame5 Q&A
./docker/run_prompt.sh --mode qa --prompt "How do I use Dataset in xFrame5?"

# Spring Q&A
./docker/run_prompt.sh --mode qa --product spring-backend --prompt "How do I create a REST controller?"

# With context
./docker/run_prompt.sh --mode qa --prompt "How do I handle pagination?" --lang ko
```

## Example Questions

### xFrame5

- "How do I bind a Dataset to a Grid?"
- "What are the standard transaction functions?"
- "How do I implement popup screens?"
- "What is the difference between fn_search and fn_save?"
- "How do I validate form input?"

### Spring

- "How do I create a REST endpoint?"
- "What annotations are used for validation?"
- "How do I implement pagination in Spring?"
- "What is the service layer pattern?"
- "How do I handle exceptions in controllers?"

## Knowledge Base Integration

The Q&A service:
1. Extracts keywords from the question
2. Searches the knowledge base for relevant entries
3. Ranks entries by relevance
4. Includes top matches in the LLM prompt
5. Returns references with the answer

### Knowledge Categories

| Product | Categories |
|---------|------------|
| xframe5-ui | Components, Patterns, Functions, XML Structure |
| spring-backend | Controllers, Services, Repositories, Annotations |

## Response Quality

The answer quality depends on:
- **Knowledge Base Coverage**: More entries = better answers
- **Question Clarity**: Specific questions get better answers
- **Context Provided**: Additional context improves relevance

## Audit Logging

All Q&A requests are logged with:
- Product type
- Question category (inferred)
- Reference count
- Timestamp

Questions are NOT stored for privacy compliance.

---

**Version**: 1.0.0
**Last Updated**: 2025-12-30
