# Code Review Feature

AI-powered code review for xFrame5 and Spring Framework code.

## Overview

The Code Review feature analyzes submitted code and provides:
- Issue detection (errors, warnings, suggestions)
- Best practice recommendations
- Pattern compliance checking
- Security vulnerability detection
- Overall code quality score

## API Endpoint

```
POST /api/agent/review
```

### Request

```json
{
  "product": "xframe5-ui | spring-backend",
  "input": {
    "code": "...(XML/JS/Java code)...",
    "fileType": "xml | javascript | java",
    "context": "Optional description of what the code does"
  },
  "options": {
    "language": "ko | en",
    "reviewFocus": ["syntax", "patterns", "performance", "security"]
  }
}
```

### Response

```json
{
  "status": "success",
  "review": {
    "summary": "Overall assessment of the code",
    "issues": [
      {
        "severity": "error | warning | info | suggestion",
        "category": "syntax | pattern | naming | performance | security",
        "line": 42,
        "message": "Dataset binding mismatch",
        "suggestion": "Add <Dataset id=\"ds_member\">..."
      }
    ],
    "score": {
      "overall": 75,
      "categories": {
        "syntax": 90,
        "patterns": 70,
        "naming": 80,
        "security": 60
      }
    },
    "improvements": [
      "Consider using fn_grid_click pattern...",
      "Add error handling for async calls..."
    ]
  }
}
```

## Issue Severity Levels

| Severity | Description | Action Required |
|----------|-------------|-----------------|
| `error` | Critical issues that will cause failures | Must fix |
| `warning` | Potential problems or bad practices | Should fix |
| `info` | Informational notes | Optional |
| `suggestion` | Improvement recommendations | Consider |

## Review Categories

| Category | Description |
|----------|-------------|
| `syntax` | Code syntax errors, malformed XML/JS |
| `pattern` | Framework pattern violations |
| `naming` | Naming convention issues |
| `performance` | Performance anti-patterns |
| `security` | Security vulnerabilities |

## Eclipse Plugin Usage

### Review Selection

1. Select code in the editor
2. Right-click or use menu: **xFrame5 > Review Selection** (or **Spring > Review Selection**)
3. View results in the result dialog

### Review Code Dialog

1. Menu: **xFrame5 > Review Code...** (or **Spring > Review Code...**)
2. Paste code into the dialog
3. Click "Review"
4. View results

## CLI Testing

```bash
# Review xFrame5 XML
./docker/run_prompt.sh --mode review --prompt '<Screen id="test">
  <Dataset id="ds_member"/>
  <Grid id="grd_list" dataset="ds_member"/>
</Screen>'

# Review Spring Java
./docker/run_prompt.sh --mode review --product spring-backend --prompt '
@RestController
public class MemberController {
    @GetMapping("/members")
    public List<Member> list() {
        return memberService.findAll();
    }
}'
```

## Knowledge Integration

The review service uses the knowledge base to:
1. Load framework-specific best practices
2. Check against documented patterns
3. Suggest improvements based on examples

## Audit Logging

All review requests are logged with:
- Product type
- File type
- Issue count (NOT the actual code)
- Timestamp

No raw code is stored for privacy compliance.

---

**Version**: 1.0.0
**Last Updated**: 2025-12-30
