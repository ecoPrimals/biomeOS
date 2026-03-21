// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Integration utilities for live service monitoring and system status

pub mod live_service;

pub use live_service::{
    HealthCheckResult, InterfaceStatus, LiveService, MountPoint, NetworkInterface, NetworkStatus,
    PrimalStatus, StorageMetrics, SystemStatus,
};
