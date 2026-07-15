// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use super::super::*;

pub(super) fn test_empty_subscriptions() -> Arc<RwLock<HashMap<Arc<str>, Subscription>>> {
    Arc::new(RwLock::new(HashMap::new()))
}
