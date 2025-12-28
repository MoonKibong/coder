use coder::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;
use serde_json::json;

#[tokio::test]
#[serial]
async fn can_get_health() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/agent/health").await;
        assert_eq!(res.status_code(), 200);

        let body: serde_json::Value = res.json();
        assert!(body.get("status").is_some());
        assert!(body.get("llm_available").is_some());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_get_products() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/agent/products").await;
        assert_eq!(res.status_code(), 200);

        let body: serde_json::Value = res.json();
        assert!(body.get("products").is_some());

        let products = body["products"].as_array().unwrap();
        assert!(!products.is_empty());
        assert_eq!(products[0]["id"], "xframe5-ui");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn generate_rejects_empty_product() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "",
            "input": {
                "type": "db_schema",
                "table": "member",
                "columns": []
            }
        });

        let res = request
            .post("/agent/generate")
            .json(&payload)
            .await;

        assert_eq!(res.status_code(), 200);

        let body: serde_json::Value = res.json();
        assert_eq!(body["status"], "error");
        assert!(body["error"].as_str().unwrap().contains("Product is required"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn generate_accepts_valid_schema_input() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "xframe5-ui",
            "input": {
                "type": "db_schema",
                "table": "member",
                "columns": [
                    {"name": "id", "column_type": "INTEGER", "nullable": false, "pk": true},
                    {"name": "name", "column_type": "VARCHAR(100)", "nullable": false, "pk": false},
                    {"name": "email", "column_type": "VARCHAR(255)", "nullable": true, "pk": false}
                ],
                "primary_keys": ["id"],
                "foreign_keys": []
            },
            "options": {
                "language": "ko"
            }
        });

        let res = request
            .post("/agent/generate")
            .json(&payload)
            .await;

        // This may fail if LLM is not available, but the request should be valid
        assert_eq!(res.status_code(), 200);

        let body: serde_json::Value = res.json();
        // Status could be success, partial_success, or error (if LLM unavailable)
        assert!(body.get("status").is_some());
        assert!(body.get("meta").is_some());
        assert_eq!(body["meta"]["generator"], "xframe5-ui-v1");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn generate_accepts_natural_language_input() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "xframe5-ui",
            "input": {
                "type": "natural_language",
                "description": "회원 목록 화면을 만들어주세요"
            }
        });

        let res = request
            .post("/agent/generate")
            .json(&payload)
            .await;

        assert_eq!(res.status_code(), 200);

        let body: serde_json::Value = res.json();
        assert!(body.get("status").is_some());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn generate_accepts_query_sample_input() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "xframe5-ui",
            "input": {
                "type": "query_sample",
                "query": "SELECT id, name, email FROM members WHERE status = 'active'",
                "description": "활성 회원 조회"
            }
        });

        let res = request
            .post("/agent/generate")
            .json(&payload)
            .await;

        assert_eq!(res.status_code(), 200);

        let body: serde_json::Value = res.json();
        assert!(body.get("status").is_some());
    })
    .await;
}
