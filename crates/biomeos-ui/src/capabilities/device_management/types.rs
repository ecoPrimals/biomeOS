//! Type definitions for device.management capability
//!
//! Generic types for device management - NO primal-specific code!

use serde::{Deserialize, Serialize};

/// Device representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
    pub resource_usage: f64,
    pub assigned_to: Option<String>,
    pub metadata: serde_json::Value,
}

/// Device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Gpu,
    Cpu,
    Storage,
    Network,
    Memory,
    Other,
}

/// Device status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Available,
    InUse,
    Offline,
    Error,
}

/// Primal information for device management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedPrimal {
    pub id: String,
    pub name: String,
    pub status: PrimalStatus,
    pub health: f64, // 0.0-1.0
    pub load: f64,   // 0.0-1.0
    pub capabilities: Vec<String>,
    pub assigned_devices: Vec<String>,
    pub metadata: serde_json::Value,
}

/// Primal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimalStatus {
    Healthy,
    Degraded,
    Offline,
    Unknown,
}

/// Niche template for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required_primals: Vec<PrimalRole>,
    pub optional_primals: Vec<PrimalRole>,
    pub estimated_resources: ResourceRequirements,
    pub metadata: serde_json::Value,
}

/// Primal role in a niche
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRole {
    pub role: String,
    pub capabilities: Vec<String>,
    pub min_health: f64,
    pub metadata: serde_json::Value,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub storage_gb: u64,
    pub gpu_required: bool,
    pub network_bandwidth_mbps: u32,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

