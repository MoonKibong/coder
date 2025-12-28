use coder::app::App;
use loco_rs::testing::prelude::*;
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

    // Verify knowledge_bases were seeded
    use coder::models::_entities::prelude::*;
    use sea_orm::EntityTrait;

    let entries = KnowledgeBases::find()
        .all(&boot.app_context.db)
        .await
        .unwrap();

    // Should have 7 seeded entries
    assert_eq!(entries.len(), 7, "Expected 7 knowledge base entries");

    // Verify first entry
    let first = &entries[0];
    assert_eq!(first.name, "core_architecture");
    assert_eq!(first.category, "architecture");
}
