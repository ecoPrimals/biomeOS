//! # toadStool Runtime Bridge
//!
//! This module provides integration between biomeOS and the toadStool runtime,
//! handling process management, deployment, and monitoring.

use crate::errors::{BiomeError, BiomeResult};
use crate::manifest::ToadStoolManifest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc;

/// Bridge to toadStool runtime
pub struct ToadStoolBridge {
    toadstool_binary: String,
    working_directory: PathBuf,
    temp_dir: PathBuf,
}

impl Default for ToadStoolBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl ToadStoolBridge {
    pub fn new() -> Self {
        Self {
            toadstool_binary: "toadstool".to_string(),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            temp_dir: std::env::temp_dir(),
        }
    }

    pub fn with_binary_path(binary_path: String) -> Self {
        Self {
            toadstool_binary: binary_path,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            temp_dir: std::env::temp_dir(),
        }
    }

    /// Check if toadStool binary is available
    pub async fn check_availability(&self) -> BiomeResult<bool> {
        let output = TokioCommand::new(&self.toadstool_binary)
            .arg("--version")
            .output()
            .await;

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// Deploy a biome using toadStool
    pub async fn deploy_biome(
        &self,
        manifest: &ToadStoolManifest,
    ) -> BiomeResult<DeploymentHandle> {
        // Save manifest to temporary file
        let manifest_path = self.save_manifest(manifest).await?;

        // Execute toadStool run command
        let mut cmd = TokioCommand::new(&self.toadstool_binary);
        cmd.arg("up")
            .arg(&manifest_path)
            .arg("--detached")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&self.working_directory);

        let child = cmd
            .spawn()
            .map_err(|e| BiomeError::RuntimeError(format!("Failed to start toadStool: {}", e)))?;

        Ok(DeploymentHandle {
            process: Some(child),
            manifest_path,
            biome_name: manifest.metadata.name.clone(),
            status: DeploymentStatus::Starting,
        })
    }

    /// List all running biomes
    pub async fn list_biomes(&self) -> BiomeResult<Vec<BiomeStatus>> {
        let output = TokioCommand::new(&self.toadstool_binary)
            .arg("ps")
            .arg("--json")
            .current_dir(&self.working_directory)
            .output()
            .await
            .map_err(|e| BiomeError::RuntimeError(format!("Failed to list biomes: {}", e)))?;

        if output.status.success() {
            let biomes: Vec<BiomeStatus> = serde_json::from_slice(&output.stdout).map_err(|e| {
                BiomeError::ConfigError(format!("Failed to parse biome list: {}", e))
            })?;
            Ok(biomes)
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(BiomeError::RuntimeError(format!(
                "toadStool ps failed: {}",
                error_msg
            )))
        }
    }

    /// Get logs for a specific biome
    pub async fn get_logs(&self, biome_name: &str) -> BiomeResult<String> {
        let output = TokioCommand::new(&self.toadstool_binary)
            .arg("logs")
            .arg(biome_name)
            .current_dir(&self.working_directory)
            .output()
            .await
            .map_err(|e| BiomeError::RuntimeError(format!("Failed to get logs: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(BiomeError::RuntimeError(format!(
                "toadStool logs failed: {}",
                error_msg
            )))
        }
    }

    /// Stop a running biome
    pub async fn stop_biome(&self, biome_name: &str) -> BiomeResult<()> {
        let output = TokioCommand::new(&self.toadstool_binary)
            .arg("stop")
            .arg(biome_name)
            .current_dir(&self.working_directory)
            .output()
            .await
            .map_err(|e| BiomeError::RuntimeError(format!("Failed to stop biome: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BiomeError::RuntimeError(format!(
                "toadStool stop failed: {}",
                error_msg
            )));
        }

        Ok(())
    }

    /// Get federation status
    pub async fn get_federation_status(&self) -> BiomeResult<FederationStatus> {
        let output = TokioCommand::new(&self.toadstool_binary)
            .arg("federation")
            .arg("status")
            .arg("--json")
            .current_dir(&self.working_directory)
            .output()
            .await
            .map_err(|e| {
                BiomeError::RuntimeError(format!("Failed to get federation status: {}", e))
            })?;

        if output.status.success() {
            let status: FederationStatus = serde_json::from_slice(&output.stdout).map_err(|e| {
                BiomeError::ConfigError(format!("Failed to parse federation status: {}", e))
            })?;
            Ok(status)
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(BiomeError::RuntimeError(format!(
                "toadStool federation status failed: {}",
                error_msg
            )))
        }
    }

    /// Save manifest to temporary file
    async fn save_manifest(&self, manifest: &ToadStoolManifest) -> BiomeResult<PathBuf> {
        let manifest_content = manifest.to_yaml_string()?;
        let manifest_path = self
            .temp_dir
            .join(format!("{}.yaml", manifest.metadata.name));

        tokio::fs::write(&manifest_path, manifest_content)
            .await
            .map_err(BiomeError::IoError)?;

        Ok(manifest_path)
    }
}

/// Handle for a deployed biome
#[derive(Debug)]
pub struct DeploymentHandle {
    pub process: Option<tokio::process::Child>,
    pub manifest_path: PathBuf,
    pub biome_name: String,
    pub status: DeploymentStatus,
}

impl DeploymentHandle {
    pub async fn wait_for_completion(&mut self) -> BiomeResult<()> {
        if let Some(mut process) = self.process.take() {
            let status = process
                .wait()
                .await
                .map_err(|e| BiomeError::RuntimeError(format!("Process wait failed: {}", e)))?;

            if status.success() {
                self.status = DeploymentStatus::Running;
            } else {
                self.status = DeploymentStatus::Failed;
            }
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, DeploymentStatus::Running)
    }

    pub async fn stop(&mut self) -> BiomeResult<()> {
        if let Some(mut process) = self.process.take() {
            process
                .kill()
                .await
                .map_err(|e| BiomeError::RuntimeError(format!("Failed to kill process: {}", e)))?;
            self.status = DeploymentStatus::Stopped;
        }

        Ok(())
    }
}

/// Status of a biome deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Starting,
    Running,
    Stopped,
    Failed,
    Unknown,
}

/// Status information for a running biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeStatus {
    pub name: String,
    pub status: DeploymentStatus,
    pub uptime: Option<String>,
    pub resources: ResourceUsage,
    pub services: Vec<ServiceStatus>,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub storage_usage: u64,
    pub network_rx: u64,
    pub network_tx: u64,
}

/// Status of an individual service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: DeploymentStatus,
    pub runtime: String,
    pub port: Option<u16>,
    pub health: HealthStatus,
}

/// Health status of a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

/// Federation status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStatus {
    pub enabled: bool,
    pub peers: Vec<PeerStatus>,
    pub shared_services: Vec<String>,
    pub trust_policy: String,
}

/// Status of a federation peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStatus {
    pub name: String,
    pub address: String,
    pub status: String,
    pub last_seen: Option<String>,
}

/// Real-time monitoring for biomes
pub struct BiomeMonitor {
    bridge: ToadStoolBridge,
    status_tx: mpsc::Sender<BiomeEvent>,
    monitoring_interval: Duration,
}

impl BiomeMonitor {
    pub fn new(bridge: ToadStoolBridge) -> (Self, mpsc::Receiver<BiomeEvent>) {
        let (tx, rx) = mpsc::channel(100);

        (
            Self {
                bridge,
                status_tx: tx,
                monitoring_interval: Duration::from_secs(5),
            },
            rx,
        )
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.monitoring_interval = interval;
        self
    }

    /// Start monitoring specified biomes
    pub async fn start_monitoring(&self, biome_names: Vec<String>) {
        let mut interval = tokio::time::interval(self.monitoring_interval);

        loop {
            interval.tick().await;

            match self.bridge.list_biomes().await {
                Ok(biomes) => {
                    for biome in biomes {
                        if biome_names.is_empty() || biome_names.contains(&biome.name) {
                            let event = BiomeEvent::StatusUpdate {
                                name: biome.name.clone(),
                                status: biome.status,
                                resources: biome.resources,
                                services: biome.services,
                            };

                            if let Err(e) = self.status_tx.send(event).await {
                                eprintln!("Failed to send status update: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    let event = BiomeEvent::Error {
                        message: format!("Monitoring error: {}", e),
                    };

                    if let Err(e) = self.status_tx.send(event).await {
                        eprintln!("Failed to send error event: {}", e);
                        break;
                    }
                }
            }
        }
    }

    /// Monitor logs for a specific biome
    pub async fn monitor_logs(&self, biome_name: String) {
        let mut last_log_length = 0;
        let mut interval = tokio::time::interval(Duration::from_secs(2));

        loop {
            interval.tick().await;

            match self.bridge.get_logs(&biome_name).await {
                Ok(logs) => {
                    if logs.len() > last_log_length {
                        let new_logs = &logs[last_log_length..];
                        last_log_length = logs.len();

                        let event = BiomeEvent::LogMessage {
                            biome_name: biome_name.clone(),
                            message: new_logs.to_string(),
                            timestamp: chrono::Utc::now(),
                        };

                        if let Err(e) = self.status_tx.send(event).await {
                            eprintln!("Failed to send log event: {}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    let event = BiomeEvent::Error {
                        message: format!("Log monitoring error for {}: {}", biome_name, e),
                    };

                    if let Err(e) = self.status_tx.send(event).await {
                        eprintln!("Failed to send log error event: {}", e);
                        break;
                    }
                }
            }
        }
    }
}

/// Events emitted by the biome monitor
#[derive(Debug, Clone)]
pub enum BiomeEvent {
    StatusUpdate {
        name: String,
        status: DeploymentStatus,
        resources: ResourceUsage,
        services: Vec<ServiceStatus>,
    },
    LogMessage {
        biome_name: String,
        message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    Error {
        message: String,
    },
    FederationUpdate {
        status: FederationStatus,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{BiomeMetadata, ToadStoolManifest};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_bridge_creation() {
        let bridge = ToadStoolBridge::new();
        assert_eq!(bridge.toadstool_binary, "toadstool");
    }

    #[tokio::test]
    async fn test_manifest_save() {
        let bridge = ToadStoolBridge::new();

        let manifest = ToadStoolManifest {
            api_version: "biomeOS/v1".to_string(),
            kind: "Biome".to_string(),
            metadata: BiomeMetadata {
                name: "test-biome".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                maintainer: None,
                labels: None,
            },
            primals: HashMap::new(),
            services: vec![],
            federation: None,
            resources: None,
            health_checks: None,
        };

        let manifest_path = bridge.save_manifest(&manifest).await.unwrap();
        assert!(manifest_path.exists());

        // Cleanup
        let _ = tokio::fs::remove_file(manifest_path).await;
    }

    #[test]
    fn test_deployment_handle() {
        let mut handle = DeploymentHandle {
            process: None,
            manifest_path: PathBuf::from("/tmp/test.yaml"),
            biome_name: "test-biome".to_string(),
            status: DeploymentStatus::Starting,
        };

        assert!(!handle.is_running());

        handle.status = DeploymentStatus::Running;
        assert!(handle.is_running());
    }

    #[test]
    fn test_biome_event_serialization() {
        let event = BiomeEvent::StatusUpdate {
            name: "test-biome".to_string(),
            status: DeploymentStatus::Running,
            resources: ResourceUsage {
                cpu_usage: 50.0,
                memory_usage: 1024,
                storage_usage: 2048,
                network_rx: 100,
                network_tx: 200,
            },
            services: vec![],
        };

        // Test that event can be created and matched
        match event {
            BiomeEvent::StatusUpdate { name, .. } => {
                assert_eq!(name, "test-biome");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
