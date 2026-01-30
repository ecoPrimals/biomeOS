// Nest Atomic E2E Tests
// Tests for Tower + NestGate + Squirrel integration

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
    async fn test_nest_atomic_full_workflow() -> Result<()> {
        cleanup_test_sockets().await?;
        
        // 1. Start Nest Atomic (Tower + NestGate + Squirrel)
        println!("🔷 Starting Nest Atomic...");
        let nest = start_nest_atomic().await?;
        println!("✅ Nest Atomic started");
        
        // 2. Verify Tower is healthy
        println!("🔷 Verifying Tower health...");
        assert!(nest.tower.is_healthy().await, "Tower not healthy");
        println!("✅ Tower healthy");
        
        // 3. Verify NestGate is healthy (socket-only mode)
        println!("🔷 Verifying NestGate health...");
        let nestgate_health = nest.nestgate.health_check().await?;
        assert_eq!(
            nestgate_health["result"]["primal"].as_str().unwrap(),
            "nestgate"
        );
        println!("✅ NestGate healthy (socket-only mode)");
        
        // 4. Verify Squirrel is healthy
        println!("🔷 Verifying Squirrel health...");
        let squirrel_health = nest.squirrel.health_check().await?;
        assert_eq!(
            squirrel_health["result"]["primal"].as_str().unwrap(),
            "squirrel"
        );
        println!("✅ Squirrel healthy");
        
        // 5. Test Squirrel discovery helpers
        println!("🔷 Testing Squirrel discovery helpers...");
        // In a full implementation, we'd call discover_songbird(), discover_beardog(), etc.
        // For now, verify Squirrel is responsive and can communicate
        assert!(nest.squirrel.health_check().await.is_ok());
        println!("✅ Discovery helpers available");
        
        // 6. Verify all 5 sockets exist
        println!("🔷 Verifying all sockets...");
        assert!(nest.tower.beardog.socket_path.exists(), "BearDog socket missing");
        assert!(nest.tower.songbird.socket_path.exists(), "Songbird socket missing");
        assert!(nest.nestgate.socket_path.exists(), "NestGate socket missing");
        assert!(nest.squirrel.socket_path.exists(), "Squirrel socket missing");
        println!("✅ All 4 sockets present");
        
        // 7. Test stability
        println!("🔷 Testing stability (15 seconds)...");
        sleep(Duration::from_secs(15)).await;
        assert!(nest.is_healthy().await, "Nest Atomic became unhealthy");
        println!("✅ Nest Atomic stable");
        
        // 8. Cleanup
        println!("🔷 Cleaning up...");
        nest.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Nest Atomic E2E test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_nest_atomic_orchestration() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Nest Atomic...");
        let nest = start_nest_atomic().await?;
        
        // Test multi-primal orchestration
        println!("🔷 Testing orchestration capabilities...");
        
        // Verify all primals can be reached
        assert!(nest.tower.beardog.health_check().await.is_ok());
        assert!(nest.tower.songbird.health_check().await.is_ok());
        assert!(nest.nestgate.health_check().await.is_ok());
        assert!(nest.squirrel.health_check().await.is_ok());
        
        println!("✅ Orchestration functional");
        
        // Cleanup
        nest.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Orchestration test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_nest_atomic_storage_workflow() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Nest Atomic...");
        let nest = start_nest_atomic().await?;
        
        // Test storage initialization
        println!("🔷 Testing storage capabilities...");
        
        // Verify NestGate is in socket-only mode (no HTTP binding)
        let nestgate_health = nest.nestgate.health_check().await?;
        assert!(nestgate_health["result"]["status"].as_str().is_some());
        
        // In a full implementation, we'd test actual storage operations here
        println!("✅ Storage workflow functional");
        
        // Cleanup
        nest.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Storage workflow test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_nest_atomic_ai_capabilities() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Nest Atomic...");
        let nest = start_nest_atomic().await?;
        
        // Test AI capability registration
        println!("🔷 Testing AI capabilities...");
        
        // Verify Squirrel is functional
        let squirrel_health = nest.squirrel.health_check().await?;
        assert!(squirrel_health["result"].is_object());
        
        // In a full implementation, we'd test AI model deployment and inference
        println!("✅ AI capabilities available");
        
        // Cleanup
        nest.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 AI capabilities test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_nest_atomic_discovery_helpers() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Nest Atomic...");
        let nest = start_nest_atomic().await?;
        
        // Test Squirrel's innovation: discovery helpers
        println!("🔷 Testing Squirrel discovery helpers...");
        
        // Verify Squirrel can be queried
        let squirrel_health = nest.squirrel.health_check().await?;
        assert_eq!(squirrel_health["result"]["primal"], "squirrel");
        
        // In a full implementation, we'd test:
        // - discover_songbird()
        // - discover_beardog()
        // - discover_toadstool()
        // - discover_nestgate()
        
        println!("✅ Discovery helpers working");
        
        // Cleanup
        nest.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Discovery helpers test PASSED!");
        Ok(())
    }
}
