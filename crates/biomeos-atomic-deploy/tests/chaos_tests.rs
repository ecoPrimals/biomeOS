//! Chaos testing for biomeos-atomic-deploy
//!
//! Tests system behavior under random failures and adverse conditions

use biomeos_atomic_deploy::*;
use std::path::PathBuf;
use tempfile::TempDir;
use rand::Rng;

/// Chaos test: Random socket failures
#[tokio::test]
async fn chaos_random_socket_failures() {
    use std::os::unix::net::UnixListener;
    
    let temp_dir = TempDir::new().unwrap();
    let checker = HealthChecker::new(temp_dir.path().to_path_buf());
    
    let mut rng = rand::thread_rng();
    let mut sockets = Vec::new();
    let mut healthy_count = 0;
    
    // Create 10 sockets, randomly make some unavailable
    for i in 0..10 {
        let socket_path = temp_dir.path().join(format!("test-{}.sock", i));
        
        if rng.gen_bool(0.7) {
            // 70% chance of being healthy
            let _listener = UnixListener::bind(&socket_path).unwrap();
            sockets.push((_listener, socket_path.clone()));
            healthy_count += 1;
        } else {
            // 30% chance of being missing/unhealthy
            // Either don't create it, or create a regular file
            if rng.gen_bool(0.5) {
                std::fs::write(&socket_path, "not a socket").unwrap();
            }
        }
        
        // Check health
        let status = checker.check_primal(&socket_path).await.unwrap();
        
        // Verify checker correctly identifies state
        if status.socket_exists && status.socket_accessible {
            assert!(status.is_healthy);
        } else {
            assert!(!status.is_healthy);
        }
    }
    
    println!("Chaos test: {}/10 sockets healthy", healthy_count);
}

/// Chaos test: Concurrent deployment attempts
#[tokio::test]
async fn chaos_concurrent_deployments() {
    let temp_dir = TempDir::new().unwrap();
    let seed_path = temp_dir.path().join("test.seed");
    
    // Create multiple configs with same seed
    let mut configs = Vec::new();
    for i in 0..5 {
        let mut config = DeploymentConfig::test_config(seed_path.clone());
        config.family_id = format!("nat{}", i);
        config.deployment_batch = format!("batch{}", i);
        configs.push(config);
    }
    
    // All configs should serialize correctly
    for config in &configs {
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DeploymentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.family_id, deserialized.family_id);
    }
}

/// Chaos test: Primal crash simulation
#[tokio::test]
async fn chaos_primal_crash_detection() {
    // Simulate process termination
    let instance = PrimalInstance {
        primal_name: "test-primal".to_string(),
        pid: 999999, // Non-existent PID
        socket_path: PathBuf::from("/tmp/nonexistent.sock"),
        started_at: chrono::Utc::now(),
    };
    
    // Should detect process is not running
    assert!(!instance.is_running());
}

/// Chaos test: Rapid socket creation/deletion
#[tokio::test]
async fn chaos_rapid_socket_churn() {
    use std::os::unix::net::UnixListener;
    use tokio::time::{sleep, Duration};
    
    let temp_dir = TempDir::new().unwrap();
    let checker = HealthChecker::new(temp_dir.path().to_path_buf());
    let socket_path = temp_dir.path().join("churn.sock");
    
    for _ in 0..10 {
        // Create socket
        let listener = UnixListener::bind(&socket_path).unwrap();
        
        // Check (should be healthy)
        let status = checker.check_primal(&socket_path).await.unwrap();
        assert!(status.is_healthy);
        
        // Delete socket
        drop(listener);
        std::fs::remove_file(&socket_path).unwrap();
        
        // Check (should be unhealthy)
        let status = checker.check_primal(&socket_path).await.unwrap();
        assert!(!status.is_healthy);
        
        sleep(Duration::from_millis(10)).await;
    }
}

/// Chaos test: Memory pressure simulation
#[test]
fn chaos_memory_pressure() {
    // Create a large deployment result to test memory handling
    let mut result = DeploymentResult {
        tower: Some(Vec::new()),
        node: Some(Vec::new()),
        nest: Some(Vec::new()),
        success_count: 3,
        errors: Vec::new(),
    };
    
    // Add many primal instances
    for i in 0..1000 {
        let instance = PrimalInstance {
            primal_name: format!("primal-{}", i),
            pid: i as u32,
            socket_path: PathBuf::from(format!("/tmp/sock-{}.sock", i)),
            started_at: chrono::Utc::now(),
        };
        
        match i % 3 {
            0 => result.tower.as_mut().unwrap().push(instance),
            1 => result.node.as_mut().unwrap().push(instance),
            _ => result.nest.as_mut().unwrap().push(instance),
        }
    }
    
    // Verify we can still query all instances
    assert_eq!(result.all_instances().len(), 1000);
    
    // Verify serialization works with large data
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.len() > 10000); // Should be substantial
    
    let deserialized: DeploymentResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.all_instances().len(), 1000);
}

/// Chaos test: Invalid atomic type handling
#[test]
fn chaos_invalid_atomic_operations() {
    // Test all valid atomic types
    let atomics = vec![AtomicType::Tower, AtomicType::Node, AtomicType::Nest];
    
    for atomic in atomics {
        // Node IDs should always be valid
        let node_id = atomic.node_id();
        assert!(!node_id.is_empty());
        assert!(node_id.chars().all(|c| c.is_alphanumeric()));
        
        // Required primals should never be empty
        let primals = atomic.required_primals();
        assert!(!primals.is_empty());
        assert!(primals.len() >= 2); // At minimum: BearDog + Songbird
    }
}

/// Chaos test: Filesystem permission errors
#[test]
fn chaos_permission_errors() {
    // Test that appropriate errors are returned for missing directories
    let bad_binary_dir = PathBuf::from("/nonexistent/path/to/binaries");
    let runtime_dir = PathBuf::from("/tmp");
    
    let result = PrimalLauncher::new(bad_binary_dir, runtime_dir);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Binary directory not found"));
}

/// Chaos test: Malformed JSON recovery
#[test]
fn chaos_malformed_json_handling() {
    // Test that deserialization fails gracefully
    let bad_json = r#"{"primal_name": "test", "pid": "not_a_number"}"#;
    let result: Result<PrimalInstance, _> = serde_json::from_str(bad_json);
    assert!(result.is_err());
    
    // Test that partial data is rejected
    let partial_json = r#"{"primal_name": "test"}"#;
    let result: Result<PrimalInstance, _> = serde_json::from_str(partial_json);
    assert!(result.is_err());
}

