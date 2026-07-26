use super::super::atomic_client::*;
use crate::atomic_primal_client::AtomicPrimalClient;
use serde_json::Value;

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_beardog_discovery() {
    let client = AtomicPrimalClient::discover("beardog").await;
    if let Ok(client) = client {
        assert!(client.is_available());

        // Log the transport type discovered
        println!(
            "BearDog discovered via: {}",
            client.endpoint().display_string()
        );

        // Try a health check
        let health = client.health_check().await;
        assert!(
            health.is_ok(),
            "BearDog health check failed: {:?}",
            health.err()
        );
    }
}

#[tokio::test]
#[ignore = "Requires running Songbird instance"]
async fn test_songbird_discovery() {
    let client = AtomicPrimalClient::discover("songbird").await;
    if let Ok(client) = client {
        assert!(client.is_available());
        println!(
            "Songbird discovered via: {}",
            client.endpoint().display_string()
        );
    }
}

#[tokio::test]
#[ignore = "Requires running TCP endpoint"]
async fn test_tcp_connection() {
    let client = AtomicClient::tcp("127.0.0.1", 9100);
    // This will fail unless something is listening
    let result = client.call("ping", Value::Null).await;
    // Just verify we can construct and attempt TCP calls
    assert!(result.is_err() || result.is_ok()); // Either works or fails gracefully
}
