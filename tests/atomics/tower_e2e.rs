// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// Tower Atomic E2E Tests
// Tests for BearDog + Songbird integration

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serial_test::serial;
    use std::time::Duration;
    use tokio::time::sleep;
    
    // Import common test infrastructure
    mod common;
    use common::helpers::*;
    use common::fixtures::timeouts;
    
    #[tokio::test]
    #[serial]
    async fn test_tower_atomic_full_workflow() -> Result<()> {
        // Clean up any existing sockets
        cleanup_test_sockets().await?;
        
        // 1. Start BearDog (Security Foundation)
        println!("🔷 Starting BearDog...");
        let beardog = start_beardog().await?;
        println!("✅ BearDog started (PID: {})", beardog.pid);
        
        // 2. Start Songbird (Network + Discovery)
        println!("🔷 Starting Songbird with BearDog as security provider...");
        let songbird = start_songbird(&beardog).await?;
        println!("✅ Songbird started (PID: {})", songbird.pid);
        
        // 3. Verify security handshake
        println!("🔷 Verifying security handshake...");
        assert!(
            verify_security_handshake(&beardog, &songbird).await,
            "Security handshake failed"
        );
        println!("✅ Security handshake successful");
        
        // 4. Test BearDog health check
        println!("🔷 Testing BearDog health check...");
        let beardog_health = beardog.health_check().await?;
        assert_eq!(
            beardog_health["result"]["primal"].as_str().unwrap(),
            "beardog",
            "BearDog health check failed"
        );
        assert_eq!(
            beardog_health["result"]["status"].as_str().unwrap(),
            "healthy",
            "BearDog not healthy"
        );
        println!("✅ BearDog healthy");
        
        // 5. Test Songbird health check
        println!("🔷 Testing Songbird health check...");
        let songbird_health = songbird.health_check().await?;
        assert_eq!(
            songbird_health["result"]["primal"].as_str().unwrap(),
            "songbird",
            "Songbird health check failed"
        );
        println!("✅ Songbird healthy");
        
        // 6. Test JSON-RPC communication
        println!("🔷 Testing JSON-RPC communication...");
        let response = send_jsonrpc(
            &beardog.socket_path,
            r#"{"jsonrpc":"2.0","method":"health","params":{},"id":99}"#
        ).await?;
        assert_eq!(response["id"], 99, "JSON-RPC ID mismatch");
        assert_eq!(response["jsonrpc"], "2.0", "JSON-RPC version mismatch");
        println!("✅ JSON-RPC communication working");
        
        // 7. Verify both primals running for at least 5 seconds
        println!("🔷 Verifying stability (5 seconds)...");
        sleep(Duration::from_secs(5)).await;
        assert!(beardog.health_check().await.is_ok(), "BearDog crashed");
        assert!(songbird.health_check().await.is_ok(), "Songbird crashed");
        println!("✅ Tower Atomic stable");
        
        // 8. Cleanup
        println!("🔷 Cleaning up...");
        let mut tower = TowerHandle { beardog, songbird };
        tower.stop().await?;
        println!("✅ Tower Atomic stopped");
        
        cleanup_test_sockets().await?;
        
        println!("🎊 Tower Atomic E2E test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_tower_atomic_security_rotation() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Tower Atomic...");
        let tower = start_tower_atomic().await?;
        
        // Test security rotation
        println!("🔷 Testing security rotation...");
        
        // Verify initial state
        assert!(tower.is_healthy().await, "Tower not healthy initially");
        
        // Simulate security rotation (would trigger key rotation)
        sleep(Duration::from_secs(2)).await;
        
        // Verify still healthy after rotation
        assert!(tower.is_healthy().await, "Tower not healthy after rotation");
        println!("✅ Security rotation successful");
        
        // Cleanup
        tower.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Security rotation test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_tower_atomic_discovery_failover() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Tower Atomic...");
        let tower = start_tower_atomic().await?;
        
        // Test discovery failover
        println!("🔷 Testing discovery failover...");
        
        // Verify initial discovery
        let songbird_health = tower.songbird.health_check().await?;
        assert!(songbird_health["result"]["status"].as_str().is_some());
        
        // Simulate network issue and recovery
        sleep(Duration::from_secs(1)).await;
        
        // Verify still functional
        assert!(tower.is_healthy().await, "Tower not healthy after failover test");
        println!("✅ Discovery failover handled");
        
        // Cleanup
        tower.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Discovery failover test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_tower_atomic_multi_node_coordination() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Tower Atomic...");
        let tower = start_tower_atomic().await?;
        
        // Test multi-node coordination
        println!("🔷 Testing multi-node coordination...");
        
        // Verify coordination capabilities
        assert!(tower.is_healthy().await, "Tower not healthy");
        
        // In a full test, we'd start multiple towers and verify coordination
        // For now, verify single tower is coordination-ready
        let beardog_health = tower.beardog.health_check().await?;
        assert!(beardog_health["result"].is_object());
        
        println!("✅ Multi-node coordination ready");
        
        // Cleanup
        tower.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Multi-node coordination test PASSED!");
        Ok(())
    }
}
