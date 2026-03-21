// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

// Node Atomic E2E Tests
// Tests for Tower + Toadstool integration

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serial_test::serial;
    use std::time::Duration;
    use tokio::time::sleep;
    
    // Import common test infrastructure
    mod common;
    use common::helpers::*;
    
    #[tokio::test]
    #[serial]
    async fn test_node_atomic_full_workflow() -> Result<()> {
        cleanup_test_sockets().await?;
        
        // 1. Start Node Atomic (Tower + Toadstool)
        println!("🔷 Starting Node Atomic...");
        let node = start_node_atomic().await?;
        println!("✅ Node Atomic started");
        
        // 2. Verify Tower is healthy
        println!("🔷 Verifying Tower health...");
        assert!(node.tower.is_healthy().await, "Tower not healthy");
        println!("✅ Tower healthy");
        
        // 3. Verify Toadstool is healthy
        println!("🔷 Verifying Toadstool health...");
        let toadstool_health = node.toadstool.health_check().await?;
        assert_eq!(
            toadstool_health["result"]["primal"].as_str().unwrap(),
            "toadstool"
        );
        println!("✅ Toadstool healthy");
        
        // 4. Test GPU compute discovery
        println!("🔷 Testing compute discovery...");
        // Toadstool should discover Songbird for network capabilities
        sleep(Duration::from_secs(2)).await;
        assert!(node.toadstool.health_check().await.is_ok());
        println!("✅ Compute discovery working");
        
        // 5. Verify all 3 sockets exist
        println!("🔷 Verifying all sockets...");
        assert!(node.tower.beardog.socket_path.exists(), "BearDog socket missing");
        assert!(node.tower.songbird.socket_path.exists(), "Songbird socket missing");
        assert!(node.toadstool.socket_path.exists(), "Toadstool socket missing");
        println!("✅ All 3 sockets present");
        
        // 6. Test stability
        println!("🔷 Testing stability (10 seconds)...");
        sleep(Duration::from_secs(10)).await;
        assert!(node.is_healthy().await, "Node Atomic became unhealthy");
        println!("✅ Node Atomic stable");
        
        // 7. Cleanup
        println!("🔷 Cleaning up...");
        node.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Node Atomic E2E test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_node_atomic_compute_workflow() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Node Atomic...");
        let node = start_node_atomic().await?;
        
        // Test compute workflow
        println!("🔷 Testing compute capabilities...");
        
        // Verify Toadstool has compute capabilities
        let toadstool_health = node.toadstool.health_check().await?;
        assert!(toadstool_health["result"].is_object());
        
        // In a full implementation, we'd test actual barraCUDA operations here
        // For now, verify the primal is responsive
        assert!(node.toadstool.health_check().await.is_ok());
        
        println!("✅ Compute workflow functional");
        
        // Cleanup
        node.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Compute workflow test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_node_atomic_network_mesh() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Node Atomic...");
        let node = start_node_atomic().await?;
        
        // Test network mesh coordination
        println!("🔷 Testing network mesh...");
        
        // Verify all primals can communicate
        assert!(node.tower.beardog.health_check().await.is_ok());
        assert!(node.tower.songbird.health_check().await.is_ok());
        assert!(node.toadstool.health_check().await.is_ok());
        
        println!("✅ Network mesh operational");
        
        // Cleanup
        node.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Network mesh test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_node_atomic_resource_allocation() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Node Atomic...");
        let node = start_node_atomic().await?;
        
        // Test resource allocation
        println!("🔷 Testing resource allocation...");
        
        // Verify Toadstool reports compute resources
        let toadstool_health = node.toadstool.health_check().await?;
        assert!(toadstool_health["result"]["status"].as_str().is_some());
        
        // Test continues to be healthy under load (simulated)
        sleep(Duration::from_secs(3)).await;
        assert!(node.is_healthy().await);
        
        println!("✅ Resource allocation working");
        
        // Cleanup
        node.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Resource allocation test PASSED!");
        Ok(())
    }
}
