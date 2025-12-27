# Schema Input Processing

**Purpose:** DB 스키마 및 쿼리 결과를 UI 생성 의도(UiIntent)로 변환

---

## Input Normalization

### DB Schema → UiIntent

```rust
pub fn schema_to_intent(schema: &SchemaInput) -> UiIntent {
    let columns: Vec<ColumnIntent> = schema.columns.iter().map(|c| {
        ColumnIntent {
            name: c.name.clone(),
            ui_type: infer_ui_type(&c.column_type, c.pk),
            label: to_korean_label(&c.name),
            required: !c.nullable,
            readonly: c.pk,
        }
    }).collect();

    UiIntent {
        screen_name: format!("{}_list", schema.table.to_lowercase()),
        screen_type: ScreenType::List,
        datasets: vec![DatasetIntent {
            id: format!("ds_{}", schema.table.to_lowercase()),
            columns,
            bindings: vec![],
        }],
        grids: vec![GridIntent { /* auto-generated */ }],
        actions: default_list_actions(),
    }
}
```

---

## Column Type → UI Type Mapping

| DB Type | UI Type | Component |
|---------|---------|-----------|
| VARCHAR, CHAR | string | Input |
| INT, BIGINT | number | Input (numeric) |
| DATE | date | DatePicker |
| DATETIME, TIMESTAMP | datetime | DateTimePicker |
| BOOLEAN | boolean | Checkbox |
| TEXT, CLOB | text | TextArea |
| DECIMAL, NUMERIC | decimal | Input (numeric) |

### Special Cases

| Condition | UI Behavior |
|-----------|-------------|
| Primary Key | Hidden or Readonly |
| Foreign Key | Combo/Lookup |
| `_YN` suffix | Checkbox |
| `_CD` suffix | Combo (code) |
| `_DATE` suffix | DatePicker |

---

## Label Inference

```rust
fn to_korean_label(column_name: &str) -> String {
    // Common patterns
    match column_name {
        "CUST_ID" => "고객ID",
        "CUST_NAME" => "고객명",
        "REG_DATE" => "등록일자",
        "UPD_DATE" => "수정일자",
        "USE_YN" => "사용여부",
        _ => column_name.replace("_", " "),
    }
}
```

**Note**: Label mapping table stored in `company_rules` for customer customization.

---

## Query Sample Processing

```rust
pub fn query_to_intent(query: &QueryInput) -> UiIntent {
    // 1. Parse SELECT columns
    let columns = parse_select_columns(&query.query);

    // 2. Infer types from sample data
    let typed_columns = infer_types_from_sample(&columns, &query.sample_data);

    // 3. Generate intent
    schema_to_intent(&SchemaInput {
        table: "QUERY_RESULT",
        columns: typed_columns,
    })
}
```

---

## Validation

1. **Schema Validation**
   - At least one column required
   - Column names must be valid identifiers
   - Type must be recognized

2. **Query Validation**
   - Must be SELECT statement
   - Sample data must match column count

---

**Last Updated**: 2025-12-28
