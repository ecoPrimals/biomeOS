// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Type definitions for device.management capability
//!
//! Generic types for device management - NO primal-specific code!
//!
//! # Architecture
//!
//! These types define the data model for device management:
//! - [`Device`] - Hardware resources (GPU, CPU, storage, etc.)
//! - [`ManagedPrimal`] - Primals with their health and capabilities
//! - [`NicheTemplate`] - Deployment templates for orchestration
//! - [`ResourceRequirements`] - Resource needs for templates
//!
//! # TRUE PRIMAL Compliance
//!
//! All types are capability-generic with NO hardcoded primal names.
//! Discovery happens at runtime via capability lookup.

use serde::{Deserialize, Serialize};

/// Device representation for hardware resource management
///
/// Represents a hardware device (GPU, CPU, storage, etc.) that can be
/// assigned to primals for compute workloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Unique device identifier (e.g., "gpu-0", "nvme-1")
    pub id: String,
    /// Human-readable device name (e.g., "RTX 4090", "Intel i9-13900K")
    pub name: String,
    /// Type of hardware device
    pub device_type: DeviceType,
    /// Current operational status
    pub status: DeviceStatus,
    /// Current resource utilization (0.0 - 1.0)
    pub resource_usage: f64,
    /// Primal ID this device is assigned to, if any
    pub assigned_to: Option<String>,
    /// Additional device-specific metadata (JSON)
    pub metadata: serde_json::Value,
}

/// Type of hardware device
///
/// Categorizes devices for resource allocation and capability matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    /// Graphics Processing Unit (NVIDIA, AMD, Intel)
    Gpu,
    /// Central Processing Unit (compute cores)
    Cpu,
    /// Storage device (NVMe, SSD, HDD)
    Storage,
    /// Network interface (NIC, virtual network)
    Network,
    /// RAM or other memory device
    Memory,
    /// Unclassified device type
    Other,
}

/// Device operational status
///
/// Indicates whether a device is available for use, currently in use,
/// or experiencing issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    /// Device is idle and ready for assignment
    Available,
    /// Device is currently assigned to a primal
    InUse,
    /// Device is not responding or disconnected
    Offline,
    /// Device encountered an error condition
    Error,
}

/// Primal information for device management
///
/// Represents a managed primal with its current health, capabilities,
/// and device assignments. Used for orchestration decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedPrimal {
    /// Unique primal identifier
    pub id: String,
    /// Primal name (e.g., "beardog", "songbird")
    pub name: String,
    /// Current operational status
    pub status: PrimalStatus,
    /// Health score from 0.0 (dead) to 1.0 (fully healthy)
    pub health: f64,
    /// Load factor from 0.0 (idle) to 1.0 (fully loaded)
    pub load: f64,
    /// List of capabilities this primal provides (e.g., "security", "discovery")
    pub capabilities: Vec<String>,
    /// Device IDs assigned to this primal
    pub assigned_devices: Vec<String>,
    /// Additional primal-specific metadata (JSON)
    pub metadata: serde_json::Value,
}

/// Primal operational status
///
/// Indicates the health state of a primal for orchestration decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimalStatus {
    /// Primal is fully operational
    Healthy,
    /// Primal is running but with reduced capability
    Degraded,
    /// Primal is not responding
    Offline,
    /// Primal status cannot be determined
    Unknown,
}

/// Niche template for orchestration
///
/// Defines a deployment template with required primals, roles, and
/// resource requirements. Used by biomeOS to bootstrap coordinated
/// primal deployments.
///
/// # Example Templates
///
/// - `tower`: Full security stack with discovery
/// - `node`: Compute-focused node with GPU support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTemplate {
    /// Unique template identifier (e.g., "tower", "node")
    pub id: String,
    /// Human-readable template name
    pub name: String,
    /// Template description explaining its purpose
    pub description: String,
    /// Required primal roles (must be satisfied for deployment)
    pub required_primals: Vec<PrimalRole>,
    /// Optional primal roles (enhance functionality if available)
    pub optional_primals: Vec<PrimalRole>,
    /// Estimated resource requirements for this template
    pub estimated_resources: ResourceRequirements,
    /// Additional template metadata (JSON)
    pub metadata: serde_json::Value,
}

/// Primal role within a niche template
///
/// Defines what capabilities are required for a specific role
/// and the minimum health threshold for that role to be considered
/// fulfilled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRole {
    /// Role identifier (e.g., "security", "discovery", "compute")
    pub role: String,
    /// Required capabilities for this role
    pub capabilities: Vec<String>,
    /// Minimum health threshold (0.0 - 1.0) for role fulfillment
    pub min_health: f64,
    /// Additional role-specific metadata (JSON)
    pub metadata: serde_json::Value,
}

/// Resource requirements for a niche template
///
/// Specifies the hardware resources needed to run a niche template.
/// Used for capacity planning and deployment validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum CPU cores required
    pub cpu_cores: u32,
    /// Minimum memory in megabytes
    pub memory_mb: u64,
    /// Minimum storage in gigabytes
    pub storage_gb: u64,
    /// Whether GPU acceleration is required
    pub gpu_required: bool,
    /// Minimum network bandwidth in Mbps
    pub network_bandwidth_mbps: u32,
}

/// Validation result for deployment checks
///
/// Contains the outcome of validating a deployment request,
/// including any errors that block deployment and warnings
/// that indicate potential issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed (true = can proceed)
    pub valid: bool,
    /// Blocking errors that prevent deployment
    pub errors: Vec<String>,
    /// Non-blocking warnings about potential issues
    pub warnings: Vec<String>,
}
