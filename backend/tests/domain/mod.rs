use coder::domain::*;

#[test]
fn test_ui_intent_builder() {
    let intent = UiIntent::new("member_list", ScreenType::List)
        .with_dataset(DatasetIntent::new("ds_member"))
        .with_notes("Test notes");

    assert_eq!(intent.screen_name, "member_list");
    assert_eq!(intent.screen_type, ScreenType::List);
    assert_eq!(intent.datasets.len(), 1);
    assert_eq!(intent.notes, Some("Test notes".to_string()));
}

#[test]
fn test_dataset_intent_builder() {
    let column = ColumnIntent::new("name", "이름")
        .with_ui_type(UiType::Input)
        .with_data_type(DataType::String)
        .required();

    let dataset = DatasetIntent::new("ds_member")
        .with_table("member")
        .with_column(column);

    assert_eq!(dataset.id, "ds_member");
    assert_eq!(dataset.table_name, Some("member".to_string()));
    assert_eq!(dataset.columns.len(), 1);
    assert!(dataset.columns[0].required);
}

#[test]
fn test_column_intent_builder() {
    let col = ColumnIntent::new("id", "ID")
        .primary_key()
        .with_max_length(10);

    assert!(col.is_pk);
    assert!(col.readonly);
    assert_eq!(col.ui_type, UiType::Hidden);
    assert_eq!(col.max_length, Some(10));
}

#[test]
fn test_grid_intent_builder() {
    let grid = GridIntent::new("grid_member", "ds_member")
        .with_column(GridColumnIntent::new("name", "이름"))
        .editable()
        .not_paginated();

    assert_eq!(grid.id, "grid_member");
    assert_eq!(grid.dataset_id, "ds_member");
    assert!(grid.editable);
    assert!(!grid.paginated);
    assert_eq!(grid.page_size, None);
}

#[test]
fn test_action_intent_builder() {
    let action = ActionIntent::new("search", "조회", ActionType::Search)
        .with_function("fn_custom_search")
        .at_bottom();

    assert_eq!(action.id, "search");
    assert_eq!(action.function_name, "fn_custom_search");
    assert_eq!(action.position, ActionPosition::Bottom);
}

#[test]
fn test_screen_type_as_str() {
    assert_eq!(ScreenType::List.as_str(), "list");
    assert_eq!(ScreenType::Detail.as_str(), "detail");
    assert_eq!(ScreenType::Popup.as_str(), "popup");
    assert_eq!(ScreenType::ListWithPopup.as_str(), "list_with_popup");
}

#[test]
fn test_ui_type_as_str() {
    assert_eq!(UiType::Input.as_str(), "input");
    assert_eq!(UiType::TextArea.as_str(), "textarea");
    assert_eq!(UiType::DatePicker.as_str(), "datepicker");
    assert_eq!(UiType::Checkbox.as_str(), "checkbox");
    assert_eq!(UiType::Hidden.as_str(), "hidden");
}

#[test]
fn test_default_actions_for_list() {
    let actions = default_actions_for_screen_type(ScreenType::List);

    assert_eq!(actions.len(), 3);
    assert!(actions.iter().any(|a| a.action_type == ActionType::Search));
    assert!(actions.iter().any(|a| a.action_type == ActionType::Add));
    assert!(actions.iter().any(|a| a.action_type == ActionType::Delete));
}

#[test]
fn test_default_actions_for_popup() {
    let actions = default_actions_for_screen_type(ScreenType::Popup);

    assert_eq!(actions.len(), 2);
    assert!(actions.iter().any(|a| a.action_type == ActionType::Save));
    assert!(actions.iter().any(|a| a.action_type == ActionType::ClosePopup));
}

#[test]
fn test_schema_input_builder() {
    let schema = SchemaInput::new("member")
        .with_schema("public")
        .with_column(SchemaColumn::new("id", "INTEGER").primary_key())
        .with_column(SchemaColumn::new("name", "VARCHAR(100)").not_null())
        .with_primary_key("id");

    assert_eq!(schema.table, "member");
    assert_eq!(schema.schema, Some("public".to_string()));
    assert_eq!(schema.columns.len(), 2);
    assert!(schema.primary_keys.contains(&"id".to_string()));
}

#[test]
fn test_schema_column_builder() {
    let col = SchemaColumn::new("email", "VARCHAR(255)")
        .not_null()
        .with_default("''")
        .with_comment("이메일 주소");

    assert_eq!(col.name, "email");
    assert!(!col.nullable);
    assert_eq!(col.default, Some("''".to_string()));
    assert_eq!(col.comment, Some("이메일 주소".to_string()));
}

#[test]
fn test_generate_input_serialization() {
    let input = GenerateInput::DbSchema(SchemaInput::new("member"));
    let json = serde_json::to_string(&input).unwrap();

    assert!(json.contains("db_schema"));
    assert!(json.contains("member"));
}

#[test]
fn test_query_sample_input() {
    let input = QuerySampleInput::new("SELECT id, name FROM member")
        .with_description("회원 조회");

    assert!(input.query.contains("SELECT"));
    assert_eq!(input.description, Some("회원 조회".to_string()));
}

#[test]
fn test_natural_language_input() {
    let input = NaturalLanguageInput::new("회원 목록 화면 생성")
        .with_screen_type("list");

    assert!(input.description.contains("회원"));
    assert_eq!(input.screen_type, Some("list".to_string()));
}
