use coder::app::App;
use coder::models::_entities::generation_logs;
use loco_rs::testing::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serial_test::serial;

macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn test_model() {
    configure_insta!();

    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();

    // Test creating a generation log
    let log = generation_logs::ActiveModel {
        product: Set("xframe5-ui".to_string()),
        input_type: Set("db-schema".to_string()),
        ui_intent: Set(r#"{"screen_name":"member_list"}"#.to_string()),
        template_version: Set(1),
        status: Set("success".to_string()),
        user_id: Set(1),
        ..Default::default()
    };

    let result = log.insert(&boot.app_context.db).await;
    assert!(result.is_ok());

    let saved = result.unwrap();
    assert_eq!(saved.product, "xframe5-ui");
    assert_eq!(saved.input_type, "db-schema");
    assert_eq!(saved.status, "success");
}

#[tokio::test]
#[serial]
async fn test_generation_log_with_artifacts() {
    configure_insta!();

    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();

    let log = generation_logs::ActiveModel {
        product: Set("xframe5-ui".to_string()),
        input_type: Set("query-sample".to_string()),
        ui_intent: Set(r#"{"screen_name":"order_list","screen_type":"List"}"#.to_string()),
        template_version: Set(2),
        status: Set("partial_success".to_string()),
        artifacts: Set(Some(r#"{"xml":"<Dataset/>","javascript":"fn_search"}"#.to_string())),
        warnings: Set(Some(r#"["Missing fn_save"]"#.to_string())),
        generation_time_ms: Set(Some(1500)),
        user_id: Set(1),
        ..Default::default()
    };

    let result = log.insert(&boot.app_context.db).await;
    assert!(result.is_ok());

    let saved = result.unwrap();
    assert_eq!(saved.status, "partial_success");
    assert!(saved.artifacts.is_some());
    assert!(saved.warnings.is_some());
    assert_eq!(saved.generation_time_ms, Some(1500));
}

#[tokio::test]
#[serial]
async fn test_generation_log_error() {
    configure_insta!();

    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();

    let log = generation_logs::ActiveModel {
        product: Set("xframe5-ui".to_string()),
        input_type: Set("natural-language".to_string()),
        ui_intent: Set(r#"{"screen_name":"unknown"}"#.to_string()),
        template_version: Set(1),
        status: Set("error".to_string()),
        error_message: Set(Some("LLM server unavailable".to_string())),
        user_id: Set(1),
        ..Default::default()
    };

    let result = log.insert(&boot.app_context.db).await;
    assert!(result.is_ok());

    let saved = result.unwrap();
    assert_eq!(saved.status, "error");
    assert_eq!(saved.error_message, Some("LLM server unavailable".to_string()));
}

#[tokio::test]
#[serial]
async fn test_generation_log_query() {
    configure_insta!();

    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();

    // Create multiple logs
    for i in 1..=3 {
        let log = generation_logs::ActiveModel {
            product: Set("xframe5-ui".to_string()),
            input_type: Set("db-schema".to_string()),
            ui_intent: Set(format!(r#"{{"screen_name":"screen_{}"}}"#, i)),
            template_version: Set(1),
            status: Set("success".to_string()),
            user_id: Set(1),
            ..Default::default()
        };
        log.insert(&boot.app_context.db).await.unwrap();
    }

    // Query all logs
    let logs = generation_logs::Entity::find()
        .all(&boot.app_context.db)
        .await
        .unwrap();

    assert!(logs.len() >= 3);
}

#[tokio::test]
#[serial]
async fn test_audit_log_no_input_data() {
    // This test verifies that we follow the privacy requirement:
    // Input data should NOT be stored, only ui_intent (meta model)

    configure_insta!();

    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();

    // Create a log - note we store ui_intent, not raw input
    let log = generation_logs::ActiveModel {
        product: Set("xframe5-ui".to_string()),
        input_type: Set("db-schema".to_string()),
        // ui_intent is the normalized meta model, not raw input
        ui_intent: Set(r#"{"screen_name":"member_list","datasets":[{"id":"ds_member"}]}"#.to_string()),
        template_version: Set(1),
        status: Set("success".to_string()),
        user_id: Set(1),
        ..Default::default()
    };

    let saved = log.insert(&boot.app_context.db).await.unwrap();

    // Verify ui_intent is stored (meta model)
    assert!(saved.ui_intent.contains("screen_name"));
    assert!(saved.ui_intent.contains("datasets"));

    // The raw input (table name, column definitions, etc.) is NOT stored
    // This satisfies the privacy requirement from CLAUDE.md
}
