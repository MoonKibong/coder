# Screen Generation Feature

**Purpose:** UI 화면(XML + JS) 자동 생성 (xFrame5, 기타 프레임워크 지원)

---

## Supported Screen Types (PoC Scope)

| Type | Description | Output |
|------|-------------|--------|
| List | 목록 화면 (Grid + 조회) | XML + JS |
| Detail Popup | 상세 팝업 (Form) | XML + JS |
| List + Popup | 목록 + 상세 팝업 조합 | XML + JS (2 sets) |

---

## Generation Flow

```
Input (Schema/Query)
    ↓
Normalize to UiIntent
    ↓
Load Template from DB
    ↓
Compile Prompt
    ↓
LLM Generate
    ↓
Parse & Validate
    ↓
Return Artifacts
```

---

## Input Types

### 1. DB Schema
```json
{
  "inputType": "db-schema",
  "input": {
    "table": "CUSTOMER",
    "columns": [
      {"name": "CUST_ID", "type": "string", "pk": true},
      {"name": "CUST_NAME", "type": "string"},
      {"name": "REG_DATE", "type": "date"}
    ]
  }
}
```

### 2. Query Sample
```json
{
  "inputType": "query-sample",
  "input": {
    "query": "SELECT CUST_ID, CUST_NAME FROM CUSTOMER WHERE STATUS = 'A'",
    "sampleData": [
      {"CUST_ID": "C001", "CUST_NAME": "홍길동"}
    ]
  }
}
```

### 3. Natural Language
```json
{
  "inputType": "natural-language",
  "input": {
    "description": "고객 목록 조회 화면, 검색 조건으로 고객명과 등록일자"
  }
}
```

---

## Output Artifacts

### XML (xFrame5 View)
```xml
<?xml version="1.0" encoding="UTF-8"?>
<Screen>
  <Dataset id="ds_customer">
    <Column id="CUST_ID" type="string"/>
    <Column id="CUST_NAME" type="string"/>
  </Dataset>
  <Grid id="grd_customer" dataset="ds_customer">
    <GridColumn id="CUST_ID" header="고객ID"/>
    <GridColumn id="CUST_NAME" header="고객명"/>
  </Grid>
</Screen>
```

### JavaScript (Event Handlers)
```javascript
this.fn_search = function() {
  // TODO: Set transaction ID
  var ds = this.getDataset("ds_customer");
  ds.clear();
  // Transaction call
};

this.fn_save = function() {
  // TODO: Implement save logic
};
```

---

## Validation Rules

1. **XML**
   - Must parse without errors
   - Dataset ID must be unique
   - Grid must reference valid Dataset

2. **JavaScript**
   - Required functions: `fn_search`, `fn_init`
   - Dataset references must match XML

3. **Missing Info**
   - Add `TODO:` comments
   - Include in response warnings

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Generation time | < 5 minutes |
| Manual modification | < 50% of previous |
| Code review pass | Yes |

---

**Last Updated**: 2025-12-28
