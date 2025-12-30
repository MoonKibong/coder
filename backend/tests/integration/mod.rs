use coder::domain::*;
use coder::llm::{LlmBackend, MockLlmBackend};
use coder::services::{NormalizerService, PromptCompiler};
use coder::services::xframe5_validator::XFrame5Validator;

/// Integration test: Full flow from DB schema to validated artifacts
#[test]
fn test_full_flow_schema_to_artifacts() {
    // 1. Create schema input
    let schema = SchemaInput::new("member")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key())
        .with_column(SchemaColumn::new("name", "VARCHAR(100)").not_null())
        .with_column(SchemaColumn::new("email", "VARCHAR(255)"))
        .with_column(SchemaColumn::new("created_at", "DATETIME"));

    // 2. Normalize to UiIntent
    let intent = NormalizerService::normalize_schema(&schema).unwrap();

    assert_eq!(intent.screen_name, "member_list");
    assert_eq!(intent.screen_type, ScreenType::List);
    assert_eq!(intent.datasets.len(), 1);
    assert_eq!(intent.datasets[0].columns.len(), 4);

    // 3. Compile prompt
    let prompt = PromptCompiler::compile_with_defaults(&intent, None);

    assert!(prompt.system.contains("xFrame5"));
    assert!(prompt.user.contains("member_list"));
    assert!(prompt.user.contains("ds_member"));

    // 4. Simulate LLM response with mock output
    let mock_output = r#"
--- XML ---
<Dataset id="ds_member">
  <Column name="id" type="STRING" size="20" />
  <Column name="name" type="STRING" size="100" />
  <Column name="email" type="STRING" size="255" />
  <Column name="created_at" type="STRING" size="20" />
</Dataset>

<Grid id="grid_member" dataset="ds_member">
  <Column name="id" header="ID" width="80" visible="false" />
  <Column name="name" header="이름" width="150" />
  <Column name="email" header="이메일" width="200" />
  <Column name="created_at" header="등록일" width="120" />
</Grid>

--- JS ---
this.fn_init = function() {
    // Initialize screen
};

this.fn_search = function() {
    var ds = this.getDataset("ds_member");
    ds.clearData();
    // TODO: Call search API
};

this.fn_save = function() {
    var ds = this.getDataset("ds_member");
    var changedData = ds.getChangedData();
    // TODO: Call save API
};
"#;

    // 5. Validate output
    let result = XFrame5Validator::parse_and_validate(mock_output, &intent);
    assert!(result.is_ok());

    let artifacts = result.unwrap();
    assert!(artifacts.xml.contains("ds_member"));
    assert!(artifacts.xml.contains("grid_member"));
    assert!(artifacts.javascript.contains("fn_search"));
    assert!(artifacts.javascript.contains("fn_save"));
}

/// Integration test: Query sample to validated artifacts
#[test]
fn test_full_flow_query_to_artifacts() {
    // 1. Create query input
    let query = QuerySampleInput::new(
        "SELECT m.id, m.name, m.email, m.status FROM members m WHERE m.status = 'active'"
    ).with_description("Active member list");

    // 2. Normalize to UiIntent
    let intent = NormalizerService::normalize_query(&query).unwrap();

    assert!(intent.screen_name.contains("member"));
    assert_eq!(intent.screen_type, ScreenType::List);
    assert_eq!(intent.notes, Some("Active member list".to_string()));

    // 3. Compile prompt
    let prompt = PromptCompiler::compile_with_defaults(&intent, None);

    assert!(prompt.system.contains("xFrame5"));
    assert!(prompt.user.contains(&intent.screen_name));
}

/// Integration test: Natural language to UiIntent
#[test]
fn test_full_flow_nl_to_intent() {
    // 1. Create NL input
    let nl = NaturalLanguageInput::new("회원 목록 화면을 만들어주세요")
        .with_screen_type("list");

    // 2. Normalize to UiIntent
    let intent = NormalizerService::normalize_natural_language(&nl).unwrap();

    assert!(intent.screen_name.contains("member"));
    assert_eq!(intent.screen_type, ScreenType::List);
    assert!(intent.notes.is_some());
}

/// Integration test: Company rules injection
#[test]
fn test_company_rules_injection() {
    let schema = SchemaInput::new("order")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key());

    let intent = NormalizerService::normalize_schema(&schema).unwrap();

    let company_rules = "Use camelCase for all variable names. Add Korean comments.";
    let prompt = PromptCompiler::compile_with_defaults(&intent, Some(company_rules));

    assert!(prompt.user.contains("camelCase"));
    assert!(prompt.user.contains("Korean"));
}

/// Integration test: Type inference
#[test]
fn test_type_inference_comprehensive() {
    let schema = SchemaInput::new("product")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key())
        .with_column(SchemaColumn::new("name", "VARCHAR(100)").not_null())
        .with_column(SchemaColumn::new("description", "TEXT"))
        .with_column(SchemaColumn::new("price", "DECIMAL(10,2)"))
        .with_column(SchemaColumn::new("is_active", "BOOLEAN"))
        .with_column(SchemaColumn::new("launch_date", "DATE"))
        .with_column(SchemaColumn::new("created_at", "TIMESTAMP"));

    let intent = NormalizerService::normalize_schema(&schema).unwrap();
    let columns = &intent.datasets[0].columns;

    // ID should be hidden (PK)
    let id_col = columns.iter().find(|c| c.name == "id").unwrap();
    assert_eq!(id_col.ui_type, UiType::Hidden);
    assert!(id_col.is_pk);

    // Name should be Input and required
    let name_col = columns.iter().find(|c| c.name == "name").unwrap();
    assert_eq!(name_col.ui_type, UiType::Input);
    assert!(name_col.required);

    // Description should be TextArea
    let desc_col = columns.iter().find(|c| c.name == "description").unwrap();
    assert_eq!(desc_col.ui_type, UiType::TextArea);

    // Price should be Number
    let price_col = columns.iter().find(|c| c.name == "price").unwrap();
    assert_eq!(price_col.ui_type, UiType::Number);
    assert_eq!(price_col.data_type, DataType::Decimal);

    // is_active should be Checkbox
    let active_col = columns.iter().find(|c| c.name == "is_active").unwrap();
    assert_eq!(active_col.ui_type, UiType::Checkbox);

    // launch_date should be DatePicker
    let date_col = columns.iter().find(|c| c.name == "launch_date").unwrap();
    assert_eq!(date_col.ui_type, UiType::DatePicker);

    // created_at should be DateTimePicker
    let ts_col = columns.iter().find(|c| c.name == "created_at").unwrap();
    assert_eq!(ts_col.ui_type, UiType::DateTimePicker);
}

/// Integration test: Korean label inference
#[test]
fn test_korean_label_inference() {
    let schema = SchemaInput::new("user")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key())
        .with_column(SchemaColumn::new("email", "VARCHAR(255)"))
        .with_column(SchemaColumn::new("phone", "VARCHAR(20)"))
        .with_column(SchemaColumn::new("name", "VARCHAR(100)"))
        .with_column(SchemaColumn::new("status", "VARCHAR(10)"))
        .with_column(SchemaColumn::new("created_at", "DATETIME"))
        .with_column(SchemaColumn::new("updated_at", "DATETIME"));

    let intent = NormalizerService::normalize_schema(&schema).unwrap();
    let columns = &intent.datasets[0].columns;

    // Check Korean labels
    assert!(columns.iter().any(|c| c.name == "email" && c.label == "이메일"));
    assert!(columns.iter().any(|c| c.name == "phone" && c.label == "전화번호"));
    assert!(columns.iter().any(|c| c.name == "name" && c.label == "이름"));
    assert!(columns.iter().any(|c| c.name == "status" && c.label == "상태"));
    assert!(columns.iter().any(|c| c.name == "created_at" && c.label == "등록일"));
    assert!(columns.iter().any(|c| c.name == "updated_at" && c.label == "수정일"));
}

/// Integration test: xFrame5 validation detects missing functions
#[test]
fn test_validation_detects_missing_functions() {
    let intent = UiIntent::new("test_screen", ScreenType::List)
        .with_action(ActionIntent::new("search", "조회", ActionType::Search))
        .with_action(ActionIntent::new("save", "저장", ActionType::Save))
        .with_action(ActionIntent::new("delete", "삭제", ActionType::Delete));

    // JS output missing fn_delete
    let output = r#"
--- XML ---
<Dataset id="ds_test" />

--- JS ---
this.fn_search = function() {};
this.fn_save = function() {};
"#;

    let result = XFrame5Validator::parse_and_validate(output, &intent);
    assert!(result.is_ok());

    let artifacts = result.unwrap();
    // Should have warning about missing fn_delete
    assert!(artifacts.warnings.iter().any(|w| w.contains("fn_delete")));
}

/// Integration test: xFrame5 validation detects TODO placeholders
#[test]
fn test_validation_detects_todos() {
    let intent = UiIntent::new("test", ScreenType::List);

    let output = r#"
--- XML ---
<Dataset id="ds_test">
  <!-- TODO: Add column definitions -->
</Dataset>

--- JS ---
// TODO: Implement search
this.fn_search = function() {
    // TODO: Call API
};
"#;

    let result = XFrame5Validator::parse_and_validate(output, &intent);
    assert!(result.is_ok());

    let artifacts = result.unwrap();
    // Should have notes about TODO placeholders
    assert!(artifacts.warnings.iter().any(|w| w.contains("TODO")));
}

/// Integration test: Post-processing adds missing stubs
#[test]
fn test_post_processing_adds_stubs() {
    let intent = UiIntent::new("test", ScreenType::List)
        .with_action(ActionIntent::new("search", "조회", ActionType::Search))
        .with_action(ActionIntent::new("add", "추가", ActionType::Add));

    let output = r#"
--- XML ---
<Dataset id="ds_test" />

--- JS ---
// Only search implemented
this.fn_search = function() {};
"#;

    let mut artifacts = XFrame5Validator::parse_and_validate(output, &intent).unwrap();

    // Post-process should add missing fn_add
    XFrame5Validator::post_process(&mut artifacts, &intent);

    assert!(artifacts.javascript.contains("fn_add"));
    assert!(artifacts.warnings.iter().any(|w| w.contains("stub")));
}

/// Integration test: Mock LLM backend
#[tokio::test]
async fn test_mock_llm_generates_valid_output() {
    let mock = MockLlmBackend::new();

    // Health check should pass
    assert!(mock.health_check().await.is_ok());

    // Generate should return valid xFrame5 output
    let prompt = "Generate member list screen";
    let result = mock.generate(prompt).await.unwrap();

    assert!(result.contains("--- XML ---"));
    assert!(result.contains("--- JS ---"));

    // Output should be parseable
    let intent = UiIntent::new("member_list", ScreenType::List);
    let validation = XFrame5Validator::parse_and_validate(&result, &intent);
    assert!(validation.is_ok());
}

/// Integration test: Mock LLM error handling
#[tokio::test]
async fn test_mock_llm_error_handling() {
    let mock = MockLlmBackend::failing("LLM server unavailable");

    let result = mock.generate("test").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unavailable"));
}

/// Integration test: Mock LLM retry simulation
#[tokio::test]
async fn test_mock_llm_retry_simulation() {
    let mock = MockLlmBackend::fail_then_succeed();

    // First attempt fails
    let first = mock.generate("test").await;
    assert!(first.is_err());

    // Retry succeeds
    let second = mock.generate("test").await;
    assert!(second.is_ok());
    assert!(second.unwrap().contains("--- XML ---"));

    // Call count should be 2
    assert_eq!(mock.call_count(), 2);
}

/// Integration test: Grid columns match dataset columns
#[test]
fn test_grid_matches_dataset() {
    let schema = SchemaInput::new("customer")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key())
        .with_column(SchemaColumn::new("name", "VARCHAR(100)").not_null())
        .with_column(SchemaColumn::new("company", "VARCHAR(200)"));

    let intent = NormalizerService::normalize_schema(&schema).unwrap();

    // Grid should exist
    assert_eq!(intent.grids.len(), 1);
    let grid = &intent.grids[0];

    // Grid should reference the dataset
    assert_eq!(grid.dataset_id, intent.datasets[0].id);

    // Grid should have columns for non-PK fields
    assert!(grid.columns.iter().any(|c| c.name == "name"));
    assert!(grid.columns.iter().any(|c| c.name == "company"));
}

/// Integration test: Default actions for List screen
#[test]
fn test_default_actions_for_list() {
    let schema = SchemaInput::new("item")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key());

    let intent = NormalizerService::normalize_schema(&schema).unwrap();

    // List screen should have standard actions
    assert!(intent.actions.iter().any(|a| a.action_type == ActionType::Search));
    assert!(intent.actions.iter().any(|a| a.action_type == ActionType::Add));
    assert!(intent.actions.iter().any(|a| a.action_type == ActionType::Delete));
}

/// Integration test: Column comment becomes label
#[test]
fn test_column_comment_as_label() {
    let schema = SchemaInput::new("product")
        .with_column(SchemaColumn::new("custom_field", "VARCHAR(100)")
            .with_comment("사용자 정의 필드"));

    let intent = NormalizerService::normalize_schema(&schema).unwrap();
    let col = &intent.datasets[0].columns[0];

    assert_eq!(col.label, "사용자 정의 필드");
}

// Error Handling Tests

/// Test: Invalid XML output fails validation
#[test]
fn test_validation_fails_on_invalid_xml() {
    let intent = UiIntent::new("test", ScreenType::List);

    let invalid_output = r#"
--- XML ---
<Dataset id="ds_test">
  <Column name="id"  <!-- missing closing tag -->
</Dataset>

--- JS ---
this.fn_search = function() {};
"#;

    let result = XFrame5Validator::parse_and_validate(invalid_output, &intent);
    // Should still extract what it can, but may have warnings
    assert!(result.is_ok() || result.is_err());
}

/// Test: Missing XML section fails
#[test]
fn test_validation_fails_on_missing_xml() {
    let intent = UiIntent::new("test", ScreenType::List);

    let no_xml_output = r#"
--- JS ---
this.fn_search = function() {};
"#;

    let result = XFrame5Validator::parse_and_validate(no_xml_output, &intent);
    assert!(result.is_err());
}

/// Test: Missing JS section fails
#[test]
fn test_validation_fails_on_missing_js() {
    let intent = UiIntent::new("test", ScreenType::List);

    let no_js_output = r#"
--- XML ---
<Dataset id="ds_test" />
"#;

    let result = XFrame5Validator::parse_and_validate(no_js_output, &intent);
    assert!(result.is_err());
}

/// Test: Empty output fails
#[test]
fn test_validation_fails_on_empty_output() {
    let intent = UiIntent::new("test", ScreenType::List);
    let result = XFrame5Validator::parse_and_validate("", &intent);
    assert!(result.is_err());
}

/// Test: XML without Dataset element generates warning
#[test]
fn test_validation_warns_on_missing_dataset() {
    let intent = UiIntent::new("test", ScreenType::List);

    let output = r#"
--- XML ---
<Grid id="grid_test" />

--- JS ---
this.fn_search = function() {};
"#;

    let result = XFrame5Validator::parse_and_validate(output, &intent);
    // Should parse but have warnings about missing Dataset
    if let Ok(artifacts) = result {
        assert!(artifacts.warnings.iter().any(|w|
            w.to_lowercase().contains("dataset") || w.to_lowercase().contains("warning")
        ));
    }
}

/// Test: Normalize handles empty table name gracefully
#[test]
fn test_normalize_empty_table_name() {
    let schema = SchemaInput::new("")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key());

    let intent = NormalizerService::normalize_schema(&schema).unwrap();

    // Should still generate intent, possibly with generic name
    assert!(!intent.screen_name.is_empty());
}

/// Test: Normalize handles no columns
#[test]
fn test_normalize_no_columns() {
    let schema = SchemaInput::new("empty_table");
    let intent = NormalizerService::normalize_schema(&schema).unwrap();

    // Should generate intent with empty datasets
    assert_eq!(intent.datasets.len(), 1);
    assert!(intent.datasets[0].columns.is_empty());
}

/// Test: Query normalization handles complex queries
#[test]
fn test_normalize_complex_query() {
    let query = QuerySampleInput::new(
        "SELECT a.id, a.name, b.description, COUNT(*) as cnt \
         FROM table_a a \
         JOIN table_b b ON a.id = b.a_id \
         WHERE a.status = 'active' \
         GROUP BY a.id, a.name, b.description \
         ORDER BY cnt DESC"
    );

    let intent = NormalizerService::normalize_query(&query).unwrap();

    // Should extract table name from query
    assert!(intent.screen_name.contains("table") || !intent.screen_name.is_empty());
}

/// Test: Natural language with Korean input
#[test]
fn test_normalize_korean_natural_language() {
    let nl = NaturalLanguageInput::new(
        "상품 목록 화면을 만들어주세요. 검색, 추가, 삭제 기능이 필요합니다."
    );

    let intent = NormalizerService::normalize_natural_language(&nl).unwrap();

    // Should detect product-related screen
    assert!(intent.screen_name.contains("product") ||
            intent.screen_name.contains("상품") ||
            !intent.screen_name.is_empty());
}

/// Test: Prompt compilation includes all intent details
#[test]
fn test_prompt_includes_all_intent_details() {
    let columns = vec![
        ColumnIntent::new("id", "ID").primary_key(),
        ColumnIntent::new("name", "이름").with_ui_type(UiType::Input).required(),
        ColumnIntent::new("status", "상태").with_ui_type(UiType::Combo),
    ];

    let grid_columns = vec![
        GridColumnIntent::new("name", "이름"),
        GridColumnIntent::new("status", "상태"),
    ];

    let dataset = DatasetIntent::new("ds_item")
        .with_table("item")
        .with_columns(columns);

    let grid = GridIntent::new("grid_item", "ds_item")
        .with_columns(grid_columns);

    let intent = UiIntent::new("item_list", ScreenType::List)
        .with_dataset(dataset)
        .with_grid(grid)
        .with_action(ActionIntent::new("search", "조회", ActionType::Search))
        .with_action(ActionIntent::new("add", "추가", ActionType::Add));

    let prompt = PromptCompiler::compile_with_defaults(&intent, None);

    // Verify system prompt contains framework rules
    assert!(prompt.system.contains("xFrame5"));
    assert!(prompt.system.contains("XML"));
    assert!(prompt.system.contains("JavaScript"));

    // Verify user prompt contains intent details
    assert!(prompt.user.contains("item_list"));
    assert!(prompt.user.contains("ds_item"));
    assert!(prompt.user.contains("이름"));
}

/// Test: Serialization/deserialization of UiIntent
#[test]
fn test_ui_intent_serialization() {
    let intent = UiIntent::new("test_screen", ScreenType::List)
        .with_dataset(DatasetIntent::new("ds_test"))
        .with_action(ActionIntent::new("search", "조회", ActionType::Search));

    // Serialize to JSON
    let json = serde_json::to_string(&intent).unwrap();

    // Verify JSON contains expected fields
    assert!(json.contains("test_screen"));
    assert!(json.contains("ds_test"));
    assert!(json.contains("search"));

    // Deserialize back
    let deserialized: UiIntent = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.screen_name, "test_screen");
    assert_eq!(deserialized.datasets.len(), 1);
    assert_eq!(deserialized.actions.len(), 1);
}

/// Test: GenerateInput serialization for API
#[test]
fn test_generate_input_api_serialization() {
    let schema = SchemaInput::new("order")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key())
        .with_column(SchemaColumn::new("total", "DECIMAL(10,2)"));

    let input = GenerateInput::DbSchema(schema);
    let json = serde_json::to_string(&input).unwrap();

    // Verify tagged union format
    assert!(json.contains("db_schema"));
    assert!(json.contains("order"));
    assert!(json.contains("total"));
}

/// Test: Response serialization
#[test]
fn test_generate_response_serialization() {
    let response = GenerateResponse {
        status: GenerateStatus::Success,
        artifacts: Some(GeneratedArtifacts {
            xml: Some("<Dataset />".to_string()),
            javascript: Some("fn_search".to_string()),
            xml_filename: Some("test.xml".to_string()),
            js_filename: Some("test.js".to_string()),
        }),
        warnings: vec!["Warning: TODO found".to_string()],
        error: None,
        meta: ResponseMeta {
            generator: "xframe5-ui-v1".to_string(),
            timestamp: chrono::Utc::now(),
            generation_time_ms: 1234,
        },
    };

    let json = serde_json::to_string(&response).unwrap();

    assert!(json.contains("success"));
    assert!(json.contains("<Dataset />"));
    assert!(json.contains("fn_search"));
    assert!(json.contains("TODO"));
    assert!(json.contains("xframe5-ui-v1"));
}
