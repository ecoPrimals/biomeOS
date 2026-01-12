//! Integration tests for biomeos-atomic-deploy
//!
//! End-to-end deployment testing

use biomeos_atomic_deploy::*;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test basic orchestrator creation
#[test]
fn test_orchestrator_creation() {
    let temp_dir = TempDir::new().unwrap();
    let binary_dir = temp_dir.path().join("bin");
    std::fs::create_dir(&binary_dir).unwrap();

    let seed_path = temp_dir.path().join("test.seed");
    let config = DeploymentConfig::test_config(seed_path);

    // Should fail because binary_dir is wrong
    let result = DeploymentOrchestrator::new(config);
    assert!(result.is_err());
}

/// Test deployment config creation and serialization
#[test]
fn test_deployment_config_round_trip() {
    let temp_dir = TempDir::new().unwrap();
    let seed_path = temp_dir.path().join("test.seed");
    
    let config = DeploymentConfig::test_config(seed_path.clone());
    
    // JSON round-trip
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: DeploymentConfig = serde_json::from_str(&json).unwrap();
    
    assert_eq!(config.family_id, deserialized.family_id);
    assert_eq!(config.usb_seed_path, deserialized.usb_seed_path);
    assert_eq!(config.neural_api_enabled, deserialized.neural_api_enabled);
}

/// Test atomic type conversions
#[test]
fn test_atomic_types() {
    let atomics = vec![AtomicType::Tower, AtomicType::Node, AtomicType::Nest];
    
    for atomic in atomics {
        let node_id = atomic.node_id();
        let primals = atomic.required_primals();
        
        // All atomics need at least BearDog and Songbird
        assert!(primals.contains(&"beardog-server"));
        assert!(primals.contains(&"songbird-orchestrator"));
        
        // Node has ToadStool
        if matches!(atomic, AtomicType::Node) {
            assert!(primals.contains(&"toadstool"));
        }
        
        // Nest has NestGate
        if matches!(atomic, AtomicType::Nest) {
            assert!(primals.contains(&"nestgate"));
        }
        
        // Node IDs should be lowercase
        assert_eq!(node_id, node_id.to_lowercase());
    }
}

/// Test deployment result aggregation
#[test]
fn test_deployment_result_aggregation() {
    let mut result = DeploymentResult {
        tower: None,
        node: None,
        nest: None,
        success_count: 0,
        errors: Vec::new(),
    };
    
    // Empty result
    assert_eq!(result.all_instances().len(), 0);
    assert!(!result.is_success());
    
    // Add tower
    result.tower = Some(vec![PrimalInstance {
        primal_name: "beardog-server".to_string(),
        pid: 1234,
        socket_path: PathBuf::from("/tmp/test.sock"),
        started_at: chrono::Utc::now(),
    }]);
    result.success_count = 1;
    
    assert_eq!(result.all_instances().len(), 1);
    assert!(!result.is_success()); // Need all 3
    
    // Add node and nest
    result.node = Some(vec![]);
    result.nest = Some(vec![]);
    result.success_count = 3;
    
    assert!(result.is_success());
}

/// Test primal instance lifecycle
#[test]
fn test_primal_instance_lifecycle() {
    let instance = PrimalInstance {
        primal_name: "beardog-server".to_string(),
        pid: std::process::id(), // Use our own PID for testing
        socket_path: PathBuf::from("/tmp/test.sock"),
        started_at: chrono::Utc::now() - chrono::Duration::seconds(5),
    };
    
    // Should be running (our own process)
    assert!(instance.is_running());
    
    // Uptime should be ~5 seconds
    let uptime = instance.uptime();
    assert!(uptime.num_seconds() >= 5);
    assert!(uptime.num_seconds() < 10);
}

/// Test health checker with mock sockets
#[tokio::test]
async fn test_health_checker_integration() {
    use std::os::unix::net::UnixListener;
    
    let temp_dir = TempDir::new().unwrap();
    let checker = HealthChecker::new(temp_dir.path().to_path_buf());
    
    // Create mock sockets for a full atomic
    let beardog_sock = temp_dir.path().join("beardog-tower.sock");
    let songbird_sock = temp_dir.path().join("songbird-tower.sock");
    
    let _beardog = UnixListener::bind(&beardog_sock).unwrap();
    let _songbird = UnixListener::bind(&songbird_sock).unwrap();
    
    // Check individual sockets
    let beardog_health = checker.check_primal(&beardog_sock).await.unwrap();
    let songbird_health = checker.check_primal(&songbird_sock).await.unwrap();
    
    assert!(beardog_health.is_healthy);
    assert!(songbird_health.is_healthy);
    
    // Check all sockets
    let all = checker.check_all("tower").await.unwrap();
    assert_eq!(all.len(), 2);
}

