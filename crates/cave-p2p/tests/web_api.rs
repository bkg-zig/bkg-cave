use cave_p2p::wallet::{Wallet, WalletConfig};
use cave_p2p::web::WebState;

#[tokio::test]
async fn test_status_endpoint_returns_json() {
    let wallet = Wallet::new(WalletConfig::default(), 1_000_000);
    let state = WebState::new("test-integration-node".into(), wallet);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = cave_p2p::web::create_router(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/api/status"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["node_id"], "test-integration-node");
    assert!(body["uptime_secs"].as_u64().is_some());
    assert_eq!(body["peers"], 0);
    assert_eq!(body["providers"], 0);
    assert_eq!(body["active_jobs"], 0);
    assert_eq!(body["completed_jobs"], 0);
}

#[tokio::test]
async fn test_root_returns_html() {
    let wallet = Wallet::new(WalletConfig::default(), 0);
    let state = WebState::new("html-test".into(), wallet);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = cave_p2p::web::create_router(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let resp = client.get(format!("http://{addr}/")).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let html = resp.text().await.unwrap();
    assert!(html.contains("bkg-cave"), "HTML should contain 'bkg-cave'");
    assert!(
        html.contains("Dashboard"),
        "HTML should contain 'Dashboard'"
    );
    assert!(html.contains("</html>"), "HTML should be well-formed");
}

#[tokio::test]
async fn test_models_endpoint() {
    let wallet = Wallet::new(WalletConfig::default(), 0);
    let state = WebState::new("models-test".into(), wallet);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = cave_p2p::web::create_router(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/v1/models"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["object"], "list");
    let data = body["data"].as_array().unwrap();
    assert!(!data.is_empty(), "model list should not be empty");

    let model_ids: Vec<&str> = data.iter().filter_map(|m| m["id"].as_str()).collect();
    assert!(
        model_ids.contains(&"bkg-llama-3.1-8b"),
        "should include bkg-llama model"
    );
}

#[tokio::test]
async fn test_wallet_balance_endpoint() {
    let wallet = Wallet::new(WalletConfig::default(), 5_000_000);
    let state = WebState::new("wallet-test".into(), wallet);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = cave_p2p::web::create_router(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/api/wallet/balance"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["available"], 5_000_000);
    assert_eq!(body["in_escrow"], 0);
    assert_eq!(body["staked"], 0);
    assert_eq!(body["total"], 5_000_000);
}

#[tokio::test]
async fn test_jobs_endpoints() {
    let wallet = Wallet::new(WalletConfig::default(), 1_000_000);
    let state = WebState::new("jobs-test".into(), wallet);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = cave_p2p::web::create_router(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{addr}/api/jobs"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["active"], 0);

    let resp = client
        .post(format!("http://{addr}/api/jobs/create"))
        .json(&serde_json::json!({
            "requester": "test-user",
            "task_type": "inference",
            "max_price": 500,
            "description": "test job"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let created: serde_json::Value = resp.json().await.unwrap();
    assert!(created["id"].as_str().unwrap().starts_with("req-"));
}

#[tokio::test]
async fn test_resource_endpoints() {
    let wallet = Wallet::new(WalletConfig::default(), 1_000_000);
    let state = WebState::new("resource-test".into(), wallet)
        .with_resource_monitor(cave_p2p::executor::resource::ResourceMonitor::with_defaults());
    // No provider tracker set, so peers list empty
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = cave_p2p::web::create_router(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();

    // Test /api/resources/local
    let resp = client
        .get(format!("http://{addr}/api/resources/local"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    // Expect the resource monitor state to be present (not null)
    assert!(body.is_object(), "expected JSON object");
    // Check for known fields from ResourceState
    assert!(body.get("cpu_usage").is_some());
    assert!(body.get("ram_total_mb").is_some());
    assert!(body.get("ram_available_mb").is_some());
    assert!(body.get("gpu_usage").is_some());
    assert!(body.get("vram_total_mb").is_some());
    assert!(body.get("vram_available_mb").is_some());
    assert!(body.get("loaded_models").is_some());
    assert!(body.get("active_inference_tasks").is_some());
    assert!(body.get("active_wasm_tasks").is_some());
    assert!(body.get("active_web_tasks").is_some());

    // Test /api/resources/peers (should be empty array)
    let resp = client
        .get(format!("http://{addr}/api/resources/peers"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let peers_body: serde_json::Value = resp.json().await.unwrap();
    // Expect an empty array
    assert!(peers_body.is_array());
    assert_eq!(peers_body.as_array().unwrap().len(), 0);

    // Test /api/resources (aggregate) - should contain local and peers
    let resp = client
        .get(format!("http://{addr}/api/resources"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let agg_body: serde_json::Value = resp.json().await.unwrap();
    // Should have local and peers fields
    assert!(agg_body.get("local").is_some());
    assert!(agg_body.get("peers").is_some());
    // local should be an object (resource state)
    assert!(agg_body["local"].is_object());
    // peers should be empty array
    assert_eq!(agg_body["peers"].as_array().unwrap().len(), 0);
}