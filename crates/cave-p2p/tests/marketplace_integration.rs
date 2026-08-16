use std::time::Duration;

use cave_p2p::executor::router::RouterConfig;
use cave_p2p::executor::task::{ResourceRequirements, WebFetchTask};
use cave_p2p::executor::task_executor::TaskExecutor;
use cave_p2p::job::{JobBroadcast, JobBroadcastTx, JobManager, PricingStrategy, ResourceType};

fn setup_broadcast() -> JobBroadcastTx {
    let (tx, _rx) = tokio::sync::broadcast::channel(64);
    tx
}

#[test]
fn test_full_job_lifecycle() {
    let mut job_manager = JobManager::new();
    let broadcast_tx = setup_broadcast();
    job_manager.set_broadcast_tx(broadcast_tx.clone());

    let config = RouterConfig::default();
    let executor = TaskExecutor::new(config);

    let mut rx = broadcast_tx.subscribe();

    let req = job_manager.create_request(
        "alice".into(),
        ResourceType::WebFetch,
        ResourceRequirements {
            ram_mb: 512,
            vram_mb: None,
            cpu_cores: 1,
            estimated_duration: Duration::from_secs(15),
        },
        1000,
        PricingStrategy::Auction,
        "fetch example.com".into(),
    );

    let broadcast = rx.try_recv().expect("should receive broadcast");
    match broadcast {
        JobBroadcast::NewRequest(ref r) => assert_eq!(r.id, req.id),
        _ => panic!("expected NewRequest"),
    }

    let bid = job_manager
        .submit_bid(&req.id, "bob".into(), 500, 30, 4.8)
        .expect("bid should be accepted");

    let broadcast = rx.try_recv().expect("should receive bid broadcast");
    match broadcast {
        JobBroadcast::BidSubmitted(ref b) => assert_eq!(b.id, bid.id),
        _ => panic!("expected BidSubmitted"),
    }

    let _job = job_manager
        .accept_bid(&req.id, &bid.id)
        .expect("bid should be accepted");

    let broadcast = rx.try_recv().expect("should receive accept broadcast");
    match broadcast {
        JobBroadcast::BidAccepted { ref job_id, bid: _ } => assert_eq!(job_id, &req.id),
        _ => panic!("expected BidAccepted"),
    }

    assert_eq!(job_manager.active_jobs().len(), 1);

    let task = WebFetchTask::get("https://example.com".into());

    let output = executor
        .execute_local(&cave_p2p::executor::task::ExecutionTask::WebFetch(task))
        .expect("local web fetch should succeed");

    assert!(output.success);
    assert_eq!(output.data["status"], 200);

    job_manager
        .submit_result(&req.id, "bob".into(), output.data, true, None)
        .expect("result should be accepted");

    assert_eq!(job_manager.active_jobs().len(), 0);
    assert_eq!(job_manager.completed_jobs().len(), 1);
}

#[test]
fn test_marketplace_no_bids_fallback() {
    let mut job_manager = JobManager::new();
    let tx = setup_broadcast();
    job_manager.set_broadcast_tx(tx);

    let req = job_manager.create_request(
        "alice".into(),
        ResourceType::Inference,
        ResourceRequirements {
            ram_mb: 16384,
            vram_mb: Some(12288),
            cpu_cores: 4,
            estimated_duration: Duration::from_secs(60),
        },
        100,
        PricingStrategy::Fixed,
        "large model request".into(),
    );

    let bids = job_manager.get_bids(&req.id);
    assert!(bids.is_empty(), "should have no bids initially");

    let result = job_manager.accept_bid(&req.id, "nonexistent");
    assert!(result.is_err(), "should fail with no bids");
}

#[test]
fn test_bid_rejected_when_over_max_price() {
    let mut job_manager = JobManager::new();

    let req = job_manager.create_request(
        "alice".into(),
        ResourceType::Inference,
        ResourceRequirements::minimal(),
        500,
        PricingStrategy::Fixed,
        "budget job".into(),
    );

    let result = job_manager.submit_bid(&req.id, "bob".into(), 1000, 30, 4.0);
    assert!(result.is_err(), "bid over max price should be rejected");
}
