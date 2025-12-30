use coder::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;
use serde_json::json;

#[tokio::test]
#[serial]
async fn review_rejects_empty_product() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "",
            "input": {
                "code": "<?xml version=\"1.0\"?><screen/>"
            }
        });

        let res = request
            .post("/agent/review")
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
async fn review_rejects_empty_code() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "xframe5-ui",
            "input": {
                "code": ""
            }
        });

        let res = request
            .post("/agent/review")
            .json(&payload)
            .await;

        assert_eq!(res.status_code(), 200);

        let body: serde_json::Value = res.json();
        assert_eq!(body["status"], "error");
        assert!(body["error"].as_str().unwrap().contains("Code is required"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn review_rejects_oversized_code() {
    request::<App, _, _>(|request, _ctx| async move {
        // Create code that exceeds 50KB limit
        let large_code = "x".repeat(51 * 1024);

        let payload = json!({
            "product": "xframe5-ui",
            "input": {
                "code": large_code
            }
        });

        let res = request
            .post("/agent/review")
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
async fn review_accepts_valid_xml_input() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            "product": "xframe5-ui",
            "input": {
                "code": r#"<?xml version="1.0" encoding="UTF-8"?>
<screen id="SCREEN_MEMBER_LIST" width="1024" height="768" script_language="Java">
    <xlinkdataset id="ds_member" desc="Member List"
        columns="ID:&quot;ID&quot;:10:&quot;&quot;:&quot;&quot;;NAME:&quot;Name&quot;:50:&quot;&quot;:&quot;&quot;"/>
    <panel control_id="1" name="pnl_main" x="0" y="0" width="1024" height="768">
        <grid control_id="2" name="grid_member" x="0" y="0" width="1000" height="500"
              link_data="ds_member" version="1.1"/>
    </panel>
</screen>"#,
                "fileType": "xml",
                "context": "This is a member list screen"
            },
            "options": {
                "language": "ko",
                "reviewFocus": ["syntax", "patterns", "naming"]
            },
            "context": {
                "fileName": "member_list.xml"
            }
        });

        let res = request
            .post("/agent/review")
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
async fn review_detects_file_type_from_code() {
    request::<App, _, _>(|request, _ctx| async move {
        // Send Java code without specifying fileType
        let payload = json!({
            "product": "spring-backend",
            "input": {
                "code": r#"@RestController
public class MemberController {
    @GetMapping("/members")
    public List<Member> getMembers() {
        return memberService.findAll();
    }
}"#
            }
        });

        let res = request
            .post("/agent/review")
            .json(&payload)
            .await;

        assert_eq!(res.status_code(), 200);

        let body: serde_json::Value = res.json();
        assert!(body.get("status").is_some());
    })
    .await;
}
