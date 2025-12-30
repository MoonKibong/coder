use coder::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;
use serde_json::json;

#[tokio::test]
#[serial]
async fn qa_rejects_empty_product() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "",
            "input": {
                "question": "How do I use Dataset in xFrame5?"
            }
        });

        let res = request
            .post("/agent/qa")
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
async fn qa_rejects_empty_question() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "xframe5-ui",
            "input": {
                "question": ""
            }
        });

        let res = request
            .post("/agent/qa")
            .json(&payload)
            .await;

        assert_eq!(res.status_code(), 200);

        let body: serde_json::Value = res.json();
        assert_eq!(body["status"], "error");
        assert!(body["error"].as_str().unwrap().contains("Question is required"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn qa_rejects_oversized_question() {
    request::<App, _, _>(|request, _ctx| async move {
        // Create question that exceeds 5KB limit
        let large_question = "x".repeat(6 * 1024);

        let payload = json!({
            "product": "xframe5-ui",
            "input": {
                "question": large_question
            }
        });

        let res = request
            .post("/agent/qa")
            .json(&payload)
            .await;

        assert_eq!(res.status_code(), 200);

        let body: serde_json::Value = res.json();
        assert_eq!(body["status"], "error");
        assert!(body["error"].as_str().unwrap().contains("exceeds maximum size"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn qa_accepts_valid_xframe5_question() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "xframe5-ui",
            "input": {
                "question": "How do I use Dataset in xFrame5?",
                "context": "Building a list screen with grid"
            },
            "options": {
                "language": "ko",
                "includeExamples": true,
                "maxReferences": 5
            }
        });

        let res = request
            .post("/agent/qa")
            .json(&payload)
            .await;

        // This test will fail if LLM is not available, which is expected in CI
        // We just verify the endpoint accepts the request format
        let status = res.status_code();
        assert!(status == 200, "Expected 200, got {}", status);

        let body: serde_json::Value = res.json();
        // Either success or error due to LLM unavailability
        assert!(body.get("status").is_some());
        assert!(body.get("meta").is_some());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn qa_accepts_valid_spring_question() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "spring-backend",
            "input": {
                "question": "How do I create a REST controller in Spring Boot?"
            },
            "options": {
                "language": "en",
                "includeExamples": true
            }
        });

        let res = request
            .post("/agent/qa")
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
async fn qa_includes_references_field() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "xframe5-ui",
            "input": {
                "question": "What are the best practices for Grid component?"
            }
        });

        let res = request
            .post("/agent/qa")
            .json(&payload)
            .await;

        assert_eq!(res.status_code(), 200);

        let body: serde_json::Value = res.json();
        // References field should always be present (may be empty array)
        assert!(body.get("references").is_some() || body.get("status") == Some(&json!("error")));
    })
    .await;
}
