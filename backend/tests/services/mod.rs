use coder::domain::*;
use coder::services::{NormalizerService, PromptCompiler};
use coder::services::xframe5_validator::XFrame5Validator;

#[test]
fn test_normalize_schema_basic() {
    let schema = SchemaInput::new("member")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key())
        .with_column(SchemaColumn::new("name", "VARCHAR(100)").not_null())
        .with_column(SchemaColumn::new("email", "VARCHAR(255)"))
        .with_column(SchemaColumn::new("created_at", "DATETIME"));

    let intent = NormalizerService::normalize_schema(&schema).unwrap();

    assert_eq!(intent.screen_name, "member_list");
    assert_eq!(intent.screen_type, ScreenType::List);
    assert_eq!(intent.datasets.len(), 1);
    assert_eq!(intent.datasets[0].columns.len(), 4);
    assert_eq!(intent.grids.len(), 1);

    // Check that ID is hidden (PK)
    let id_col = intent.datasets[0].columns.iter().find(|c| c.name == "id").unwrap();
    assert_eq!(id_col.ui_type, UiType::Hidden);
    assert!(id_col.is_pk);

    // Check that name is required
    let name_col = intent.datasets[0].columns.iter().find(|c| c.name == "name").unwrap();
    assert!(name_col.required);
    assert_eq!(name_col.ui_type, UiType::Input);

    // Check datetime field
    let created_col = intent.datasets[0].columns.iter().find(|c| c.name == "created_at").unwrap();
    assert_eq!(created_col.ui_type, UiType::DateTimePicker);
}

#[test]
fn test_normalize_schema_with_text_field() {
    let schema = SchemaInput::new("article")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key())
        .with_column(SchemaColumn::new("content", "TEXT"));

    let intent = NormalizerService::normalize_schema(&schema).unwrap();

    let content_col = intent.datasets[0].columns.iter().find(|c| c.name == "content").unwrap();
    assert_eq!(content_col.ui_type, UiType::TextArea);
}

#[test]
fn test_normalize_schema_with_boolean() {
    let schema = SchemaInput::new("setting")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key())
        .with_column(SchemaColumn::new("is_active", "BOOLEAN"));

    let intent = NormalizerService::normalize_schema(&schema).unwrap();

    let active_col = intent.datasets[0].columns.iter().find(|c| c.name == "is_active").unwrap();
    assert_eq!(active_col.ui_type, UiType::Checkbox);
}

#[test]
fn test_normalize_schema_with_decimal() {
    let schema = SchemaInput::new("product")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key())
        .with_column(SchemaColumn::new("price", "DECIMAL(10,2)"));

    let intent = NormalizerService::normalize_schema(&schema).unwrap();

    let price_col = intent.datasets[0].columns.iter().find(|c| c.name == "price").unwrap();
    assert_eq!(price_col.ui_type, UiType::Number);
    assert_eq!(price_col.data_type, DataType::Decimal);
}

#[test]
fn test_normalize_generates_default_actions() {
    let schema = SchemaInput::new("member")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key());

    let intent = NormalizerService::normalize_schema(&schema).unwrap();

    // List screens should have search, add, delete actions
    assert!(!intent.actions.is_empty());
    assert!(intent.actions.iter().any(|a| a.action_type == ActionType::Search));
}

#[test]
fn test_normalize_query_sample() {
    let query = QuerySampleInput::new("SELECT id, name, email FROM members WHERE status = 'active'")
        .with_description("활성 회원 조회");

    let intent = NormalizerService::normalize_query(&query).unwrap();

    assert!(intent.screen_name.contains("member"));
    assert_eq!(intent.datasets.len(), 1);
    assert_eq!(intent.notes, Some("활성 회원 조회".to_string()));
}

#[test]
fn test_normalize_natural_language() {
    let input = NaturalLanguageInput::new("회원 목록 화면을 만들어주세요")
        .with_screen_type("list");

    let intent = NormalizerService::normalize_natural_language(&input).unwrap();

    assert!(intent.screen_name.contains("member"));
    assert_eq!(intent.screen_type, ScreenType::List);
    assert!(intent.notes.is_some());
}

#[test]
fn test_prompt_compiler_with_defaults() {
    let columns = vec![
        ColumnIntent::new("id", "ID").primary_key(),
        ColumnIntent::new("name", "이름").with_ui_type(UiType::Input).required(),
        ColumnIntent::new("email", "이메일").with_ui_type(UiType::Input),
    ];

    let grid_columns = vec![
        GridColumnIntent::new("name", "이름"),
        GridColumnIntent::new("email", "이메일"),
    ];

    let dataset = DatasetIntent::new("ds_member")
        .with_table("member")
        .with_columns(columns);

    let grid = GridIntent::new("grid_member", "ds_member")
        .with_columns(grid_columns);

    let intent = UiIntent::new("member_list", ScreenType::List)
        .with_dataset(dataset)
        .with_grid(grid);

    let prompt = PromptCompiler::compile_with_defaults(&intent, None);

    // System prompt should contain xFrame5 rules
    assert!(prompt.system.contains("xFrame5"));
    assert!(prompt.system.contains("XML"));
    assert!(prompt.system.contains("JavaScript"));

    // User prompt should contain intent details
    assert!(prompt.user.contains("member_list"));
    assert!(prompt.user.contains("ds_member"));
    assert!(prompt.user.contains("이름"));
}

#[test]
fn test_prompt_compiler_with_company_rules() {
    let intent = UiIntent::new("member_list", ScreenType::List);
    let company_rules = "Always use Korean comments. Use company naming convention.";

    let prompt = PromptCompiler::compile_with_defaults(&intent, Some(company_rules));

    assert!(prompt.user.contains("company"));
    assert!(prompt.user.contains("Korean"));
}

#[test]
fn test_prompt_full_combines_system_and_user() {
    let intent = UiIntent::new("test_screen", ScreenType::List);
    let prompt = PromptCompiler::compile_with_defaults(&intent, None);
    let full = prompt.full();

    // Full prompt should contain both system and user content
    assert!(full.contains(&prompt.system));
    assert!(full.contains(&prompt.user));
}

#[test]
fn test_normalize_input_enum() {
    // Test DbSchema
    let schema_input = GenerateInput::DbSchema(SchemaInput::new("test"));
    let result = NormalizerService::normalize(&schema_input);
    assert!(result.is_ok());

    // Test NaturalLanguage
    let nl_input = GenerateInput::NaturalLanguage(NaturalLanguageInput::new("Create a list screen"));
    let result = NormalizerService::normalize(&nl_input);
    assert!(result.is_ok());
}

#[test]
fn test_label_inference_korean() {
    let schema = SchemaInput::new("member")
        .with_column(SchemaColumn::new("email", "VARCHAR(255)"))
        .with_column(SchemaColumn::new("phone", "VARCHAR(20)"))
        .with_column(SchemaColumn::new("created_at", "DATETIME"))
        .with_column(SchemaColumn::new("status", "VARCHAR(10)"));

    let intent = NormalizerService::normalize_schema(&schema).unwrap();
    let columns = &intent.datasets[0].columns;

    assert!(columns.iter().any(|c| c.name == "email" && c.label == "이메일"));
    assert!(columns.iter().any(|c| c.name == "phone" && c.label == "전화번호"));
    assert!(columns.iter().any(|c| c.name == "created_at" && c.label == "등록일"));
    assert!(columns.iter().any(|c| c.name == "status" && c.label == "상태"));
}

#[test]
fn test_label_from_comment() {
    let schema = SchemaInput::new("member")
        .with_column(SchemaColumn::new("custom_field", "VARCHAR(100)").with_comment("사용자 정의 필드"));

    let intent = NormalizerService::normalize_schema(&schema).unwrap();
    let col = &intent.datasets[0].columns[0];

    assert_eq!(col.label, "사용자 정의 필드");
}

// xFrame5 Validator Tests

#[test]
fn test_xframe5_split_output_with_markers() {
    let raw = r#"
--- XML ---
<Dataset id="ds_test">
  <Column name="id" />
</Dataset>

--- JS ---
this.fn_search = function() {
    console.log('search');
};
"#;

    let intent = UiIntent::new("test", ScreenType::List);
    let result = XFrame5Validator::parse_and_validate(raw, &intent);
    assert!(result.is_ok());

    let artifacts = result.unwrap();
    assert!(artifacts.xml.contains("<Dataset"));
    assert!(artifacts.javascript.contains("fn_search"));
}

#[test]
fn test_xframe5_split_output_without_markers() {
    let raw = r#"
<Dataset id="ds_test">
  <Column name="id" />
</Dataset>

this.fn_search = function() {
    console.log('search');
};
"#;

    let intent = UiIntent::new("test", ScreenType::List);
    let result = XFrame5Validator::parse_and_validate(raw, &intent);
    assert!(result.is_ok());

    let artifacts = result.unwrap();
    assert!(artifacts.xml.contains("<Dataset"));
    assert!(artifacts.javascript.contains("fn_search"));
}

#[test]
fn test_xframe5_validates_missing_functions() {
    let raw = r#"
--- XML ---
<Dataset id="ds_test" />

--- JS ---
this.fn_other = function() {};
"#;

    let intent = UiIntent::new("test", ScreenType::List)
        .with_action(ActionIntent::new("search", "조회", ActionType::Search))
        .with_action(ActionIntent::new("save", "저장", ActionType::Save));

    let result = XFrame5Validator::parse_and_validate(raw, &intent);
    assert!(result.is_ok());

    let artifacts = result.unwrap();
    // Should have warnings about missing functions
    assert!(artifacts.warnings.iter().any(|w| w.contains("fn_search")));
    assert!(artifacts.warnings.iter().any(|w| w.contains("fn_save")));
}

#[test]
fn test_xframe5_detects_todos() {
    let raw = r#"
--- XML ---
<Dataset id="ds_test">
  <!-- TODO: Add columns -->
</Dataset>

--- JS ---
// TODO: Implement search
this.fn_search = function() {};
"#;

    let intent = UiIntent::new("test", ScreenType::List);
    let result = XFrame5Validator::parse_and_validate(raw, &intent);
    assert!(result.is_ok());

    let artifacts = result.unwrap();
    // Should have notes about TODO placeholders
    assert!(artifacts.warnings.iter().any(|w| w.contains("TODO")));
}

#[test]
fn test_xframe5_post_process_adds_stubs() {
    let intent = UiIntent::new("test", ScreenType::List)
        .with_action(ActionIntent::new("search", "조회", ActionType::Search));

    let mut artifacts = coder::services::xframe5_validator::ValidatedArtifacts {
        xml: "<Dataset />".to_string(),
        javascript: "// existing".to_string(),
        warnings: vec![],
    };

    XFrame5Validator::post_process(&mut artifacts, &intent);

    // Should have added the missing function stub
    assert!(artifacts.javascript.contains("fn_search"));
    assert!(artifacts.warnings.iter().any(|w| w.contains("stub")));
}
