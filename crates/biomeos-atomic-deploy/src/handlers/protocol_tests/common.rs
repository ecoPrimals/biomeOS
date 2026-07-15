// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::protocol::ProtocolHandler;
use crate::living_graph::LivingGraph;
use crate::protocol_escalation::{EscalationConfig, ProtocolEscalationManager};
use std::sync::Arc;
use tokio::sync::RwLock;

pub(super) fn make_handler() -> ProtocolHandler {
    let graph = Arc::new(LivingGraph::new("protocol-cov-family"));
    let manager = Arc::new(RwLock::new(ProtocolEscalationManager::new(
        graph.clone(),
        EscalationConfig::default(),
    )));
    ProtocolHandler::new(graph, manager)
}

pub(super) fn create_test_handler() -> ProtocolHandler {
    let graph = Arc::new(LivingGraph::new("test-family"));
    let manager = Arc::new(RwLock::new(ProtocolEscalationManager::new(
        graph.clone(),
        EscalationConfig::default(),
    )));
    ProtocolHandler::new(graph, manager)
}
