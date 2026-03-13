// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Protocol Escalation Manager: JSON-RPC → tarpc Runtime Evolution
//!
//! This module manages the runtime escalation of primal connections from
//! JSON-RPC (bootstrap/debug) to tarpc (production/performance).
//!
//! # Design Principles
//!
//! - **Metrics-Based**: Escalate based on measured latency and request volume
//! - **Graceful Degradation**: Automatic fallback on tarpc failure
//! - **Non-Intrusive**: Primals continue working during escalation
//! - **Configurable**: Thresholds tunable per-deployment
//!
//! # Escalation Flow
//!
//! ```text
//! 1. Monitor connection metrics (latency, request count)
//! 2. When threshold met: query target primal for tarpc endpoint
//! 3. Notify source primal to connect via tarpc
//! 4. Verify with test call
//! 5. Update Living Graph
//! ```

mod config;
mod engine;
mod metrics;

pub use config::{EscalationConfig, EscalationResult, TarpcEndpoint};
pub use engine::ProtocolEscalationManager;
