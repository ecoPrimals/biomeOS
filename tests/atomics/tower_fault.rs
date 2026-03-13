// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// Tower Atomic Fault Injection Tests
// Tests for Tower error handling and recovery

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serial_test::serial;
    use std::time::Duration;
    use tokio::time::sleep;
    
    // Import common test infrastructure
    mod common;
    use common::helpers::*;
    use common::fault_injector::{FaultInjector, Fault, CorruptionType};
    
    #[tokio::test]
    #[serial]
    async fn test_tower_malformed_jsonrpc() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Tower Atomic...");
        let tower = start_tower_atomic().await?;
        
        // Create fault injector
        let injector = FaultInjector::new();
        
        // Test various malformed messages
        let test_cases = vec![
            (CorruptionType::MissingId, "Missing ID field"),
            (CorruptionType::MissingJsonrpc, "Missing jsonrpc field"),
            (CorruptionType::WrongVersion, "Wrong JSON-RPC version"),
            (CorruptionType::InvalidJson, "Invalid JSON syntax"),
            (CorruptionType::EmptyMessage, "Empty message"),
            (CorruptionType::TruncatedMessage, "Truncated message"),
        ];
        
        for (corruption_type, description) in test_cases {
            println!("💉 FAULT: Testing {} ...", description);
            
            let malformed = injector.generate_malformed_message(corruption_type);
            let response = send_jsonrpc(
                &tower.beardog.socket_path,
                &malformed
            ).await;
            
            // Should fail gracefully
            assert!(response.is_err(), "{} should be rejected", description);
            
            // Verify primal still operational
            assert!(
                tower.beardog.health_check().await.is_ok(),
                "BearDog crashed after {}", description
            );
            
            println!("✅ {} handled gracefully", description);
        }
        
        // Verify Tower still fully healthy
        assert!(tower.is_healthy().await, "Tower degraded after fault tests");
        println!("✅ Tower survived all malformed messages");
        
        // Cleanup
        tower.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Malformed JSON-RPC fault test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_tower_socket_permission_denied() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Tower Atomic...");
        let tower = start_tower_atomic().await?;
        let songbird_socket = tower.songbird.socket_path.clone();
        
        // Inject fault: deny permissions
        println!("💉 FAULT: Denying socket permissions...");
        let mut injector = FaultInjector::new();
        let fault_handle = injector.inject(Fault::PermissionDenied {
            socket_path: songbird_socket.clone(),
        }).await?;
        
        // Try to connect (should fail)
        sleep(Duration::from_secs(1)).await;
        let result = tower.songbird.health_check().await;
        assert!(result.is_err(), "Should fail with permission denied");
        
        println!("✅ Permission denied fault detected");
        
        // Clear fault
        println!("🔷 Clearing fault...");
        injector.clear(fault_handle).await?;
        sleep(Duration::from_secs(1)).await;
        
        // Verify recovery
        assert!(tower.songbird.health_check().await.is_ok(), "Recovery failed");
        println!("✅ Recovered from permission fault");
        
        // Cleanup
        tower.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Permission denied fault test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_tower_environment_corruption() -> Result<()> {
        cleanup_test_sockets().await?;
        
        // Test starting Tower with corrupted environment
        println!("💉 FAULT: Corrupting FAMILY_ID environment...");
        let mut injector = FaultInjector::new();
        let fault_handle = injector.inject(Fault::EnvironmentCorruption {
            var: "FAMILY_ID".to_string(),
            corrupted_value: "invalid@family!id".to_string(),
        }).await?;
        
        // Try to start BearDog (should fail gracefully)
        println!("🔷 Attempting to start BearDog with corrupted env...");
        let result = start_beardog().await;
        
        // May fail or start with default - both are acceptable
        // The key is it doesn't panic or crash the system
        println!("✅ Handled corrupted environment gracefully");
        
        // Clear fault
        injector.clear(fault_handle).await?;
        
        // Now start normally
        println!("🔷 Starting with clean environment...");
        let tower = start_tower_atomic().await?;
        assert!(tower.is_healthy().await, "Failed to start after clearing fault");
        
        println!("✅ Recovered to normal operation");
        
        // Cleanup
        tower.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Environment corruption fault test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_tower_partial_message_delivery() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Tower Atomic...");
        let tower = start_tower_atomic().await?;
        
        // Test partial message delivery
        println!("💉 FAULT: Sending partial JSON-RPC message...");
        let partial_message = r#"{"jsonrpc":"2.0","method":"h"#; // Truncated
        
        let result = send_jsonrpc(
            &tower.beardog.socket_path,
            partial_message
        ).await;
        
        // Should fail or timeout gracefully
        assert!(result.is_err(), "Partial message should be rejected");
        
        // Verify primal still operational
        assert!(
            tower.beardog.health_check().await.is_ok(),
            "BearDog crashed after partial message"
        );
        
        println!("✅ Partial message handled gracefully");
        
        // Send complete message to verify recovery
        println!("🔷 Verifying full recovery...");
        let health = tower.beardog.health_check().await?;
        assert_eq!(health["result"]["status"], "healthy");
        
        println!("✅ Full recovery confirmed");
        
        // Cleanup
        tower.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Partial message fault test PASSED!");
        Ok(())
    }
}
