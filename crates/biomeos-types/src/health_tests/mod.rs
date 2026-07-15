// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test")]

//! Health Module Tests
//!
//! Comprehensive tests for the health monitoring system.
//!
//! Domain-focused modules:
//! - [`status`] — `Health` variants, scores, lifecycle phases, and serde
//! - [`issues`] — `HealthIssue`, categories, severity, and remediation actions
//! - [`checks`] — `HealthCheckConfig`, targets, and metric thresholds
//! - [`events`] — `HealthEventTrigger` and `HealthEvent`
//! - [`reports`] — `HealthReport`, `HealthSubject`, and `ComponentHealth`
//! - [`metrics`] — response time, resource, error, and availability metrics

mod checks;
mod events;
mod issues;
mod metrics;
mod reports;
mod status;
