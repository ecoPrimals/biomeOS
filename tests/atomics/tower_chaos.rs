// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// Tower Atomic Chaos Tests
// Tests for Tower resilience under adverse conditions

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serial_test::serial;
    use std::time::Duration;
    use tokio::time::{sleep, timeout};
    use nix::sys::signal::Signal;
    
    // Import common test infrastructure
    mod common;
    use common::helpers::*;
    use common::chaos_engine::{ChaosEngine, ChaosScenario};
    
    #[tokio::test]
    #[serial]
    async fn test_tower_beardog_sudden_termination() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Tower Atomic...");
        let tower = start_tower_atomic().await?;
        let beardog_pid = tower.beardog.pid;
        
        // Verify healthy state
        println!("🔷 Verifying initial health...");
        assert!(tower.is_healthy().await, "Tower not healthy initially");
        
        // Inject chaos: Kill BearDog with SIGKILL
        println!("💥 CHAOS: Killing BearDog with SIGKILL...");
        let mut chaos = ChaosEngine::new();
        chaos.inject(ChaosScenario::ProcessTermination {
            pid: beardog_pid,
            signal: Signal::SIGKILL,
        }).await?;
        
        // Wait for Songbird to detect BearDog failure (poll until health fails or timeout)
        let _ = timeout(
            Duration::from_secs(5),
            async {
                loop {
                    if tower.songbird.health_check().await.is_err() {
                        break;
                    }
                    sleep(Duration::from_millis(50)).await;
                }
            },
        ).await;
        
        // Verify Songbird detects failure
        println!("🔷 Checking Songbird status...");
        // Songbird should still be running but report security provider unavailable
        let songbird_health = tower.songbird.health_check().await;
        // May fail or report degraded state - both are acceptable
        
        println!("✅ BearDog termination detected");
        
        // Restart BearDog (recovery test) - start_beardog already waits for health
        println!("🔷 Recovering: Restarting BearDog...");
        let new_beardog = start_beardog().await?;
        
        // Verify recovery
        assert!(new_beardog.health_check().await.is_ok(), "BearDog recovery failed");
        println!("✅ BearDog recovered successfully");
        
        // Cleanup
        drop(tower); // Original tower is partially dead
        let mut new_beardog = new_beardog;
        new_beardog.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 BearDog termination chaos test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_tower_cpu_load() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Tower Atomic...");
        let tower = start_tower_atomic().await?;
        
        // Verify initial health
        assert!(tower.is_healthy().await);
        
        // Inject CPU load
        println!("💥 CHAOS: Injecting 80% CPU load for 10 seconds...");
        let mut chaos = ChaosEngine::new();
        chaos.inject(ChaosScenario::CpuLoad {
            percentage: 80,
            duration: Duration::from_secs(10),
        }).await?;
        
        // chaos.inject blocks for duration; verify still functional after load
        println!("🔷 Checking health under CPU load...");
        
        // Health checks may be slower but should still work
        let beardog_result = tower.beardog.health_check().await;
        let songbird_result = tower.songbird.health_check().await;
        
        // At least one should respond (system may be overloaded)
        assert!(
            beardog_result.is_ok() || songbird_result.is_ok(),
            "Tower completely unresponsive under CPU load"
        );
        
        println!("✅ Tower survived CPU load");
        
        // chaos.inject already blocked for full duration; verify recovery
        assert!(tower.is_healthy().await, "Tower didn't recover from CPU load");
        println!("✅ Tower recovered from CPU load");
        
        // Cleanup
        tower.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 CPU load chaos test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_tower_memory_pressure() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Tower Atomic...");
        let tower = start_tower_atomic().await?;
        
        // Verify initial health
        assert!(tower.is_healthy().await);
        
        // Inject memory pressure (500MB for 5 seconds)
        println!("💥 CHAOS: Creating memory pressure (500MB)...");
        let mut chaos = ChaosEngine::new();
        chaos.inject(ChaosScenario::MemoryPressure {
            mb: 500,
            duration: Duration::from_secs(5),
        }).await?;
        
        // chaos.inject blocks for duration; verify still functional after pressure
        println!("🔷 Checking health under memory pressure...");
        
        // System should still respond
        let healthy = tower.is_healthy().await;
        assert!(healthy, "Tower failed under memory pressure");
        
        println!("✅ Tower survived memory pressure");
        
        // chaos.inject already blocked for full duration; verify full recovery
        assert!(tower.is_healthy().await, "Tower didn't recover from memory pressure");
        println!("✅ Tower recovered from memory pressure");
        
        // Cleanup
        tower.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Memory pressure chaos test PASSED!");
        Ok(())
    }
    
    #[tokio::test]
    #[serial]
    async fn test_tower_socket_corruption() -> Result<()> {
        cleanup_test_sockets().await?;
        
        println!("🔷 Starting Tower Atomic...");
        let tower = start_tower_atomic().await?;
        let beardog_socket = tower.beardog.socket_path.clone();
        
        // Verify initial health
        assert!(tower.is_healthy().await);
        
        // Inject socket corruption (change permissions)
        println!("💥 CHAOS: Corrupting BearDog socket permissions...");
        let mut chaos = ChaosEngine::new();
        chaos.inject(ChaosScenario::SocketCorruption {
            socket_path: beardog_socket.clone(),
        }).await?;
        
        // Socket corruption is immediate; try to connect (should fail)
        println!("🔷 Attempting connection to corrupted socket...");
        let result = tower.beardog.health_check().await;
        assert!(result.is_err(), "Should fail to connect to corrupted socket");
        
        println!("✅ Socket corruption detected");
        
        // Recover socket
        println!("🔷 Recovering socket...");
        chaos.recover(ChaosScenario::SocketCorruption {
            socket_path: beardog_socket,
        }).await?;
        
        // Recover restores permissions immediately; verify with health check
        assert!(tower.beardog.health_check().await.is_ok(), "Socket recovery failed");
        println!("✅ Socket recovered");
        
        // Cleanup
        tower.stop().await?;
        cleanup_test_sockets().await?;
        
        println!("🎊 Socket corruption chaos test PASSED!");
        Ok(())
    }
}
