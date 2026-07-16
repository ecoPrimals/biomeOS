// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project
//
// Test module for service/core.rs - included via #[path]

#![expect(clippy::unwrap_used, reason = "test")]

mod scaling_lifecycle;
mod serde;
mod spec_status;

use super::*;
use std::collections::HashMap;

pub(super) fn test_metadata() -> ServiceMetadata {
    let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let now = Utc::now();
    ServiceMetadata {
        id,
        name: "test-service".to_string(),
        namespace: Some("default".to_string()),
        version: "1.2.3".to_string(),
        description: Some("A test service".to_string()),
        author: Some("test-author".to_string()),
        labels: {
            let mut m = HashMap::new();
            m.insert("app".to_string(), "test".to_string());
            m
        },
        annotations: HashMap::new(),
        tags: vec!["web".to_string(), "api".to_string()],
        created_at: now,
        updated_at: now,
        owner_references: vec![],
    }
}
