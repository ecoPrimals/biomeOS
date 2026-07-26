// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use crate::neural_router::NeuralRouter;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub(super) fn make_handler(
    family_id: &str,
    router: Arc<NeuralRouter>,
    graphs_dir: impl Into<PathBuf>,
) -> TopologyHandler {
    let executions = Arc::new(RwLock::new(HashMap::new()));
    TopologyHandler::new(family_id, router, executions, graphs_dir)
}
