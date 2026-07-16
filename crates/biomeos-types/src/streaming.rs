// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Streaming pipeline item types shared across graph execution and atomic clients.

use serde::{Deserialize, Serialize};

/// An item flowing through a streaming pipeline.
///
/// The streaming protocol is simple: a source produces `Data` items,
/// each node transforms them into new `Data` items, and the pipeline
/// ends when the source sends `End`. Errors are non-fatal by default —
/// they are logged and the pipeline continues with the next item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum StreamItem {
    /// A data item flowing through the pipeline.
    Data(serde_json::Value),
    /// End of stream — no more items will be produced by the upstream node.
    End,
    /// A non-fatal error from a node. Logged and skipped.
    Error {
        /// The node that produced the error.
        node_id: String,
        /// Error description.
        message: String,
    },
}

impl StreamItem {
    /// Returns true if this is a data item (not End or Error).
    #[must_use]
    pub const fn is_data(&self) -> bool {
        matches!(self, Self::Data(_))
    }

    /// Returns the inner value if this is a `Data` item.
    #[must_use]
    pub fn into_data(self) -> Option<serde_json::Value> {
        match self {
            Self::Data(v) => Some(v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]

    use super::StreamItem;

    #[test]
    fn stream_item_is_data() {
        assert!(StreamItem::Data(serde_json::json!(1)).is_data());
        assert!(!StreamItem::End.is_data());
        assert!(
            !StreamItem::Error {
                node_id: "n".to_string(),
                message: "e".to_string(),
            }
            .is_data()
        );
    }

    #[test]
    fn stream_item_into_data() {
        let item = StreamItem::Data(serde_json::json!(42));
        assert_eq!(item.into_data(), Some(serde_json::json!(42)));
        assert!(StreamItem::End.into_data().is_none());
    }

    #[test]
    fn stream_item_round_trip_json() {
        for item in [
            StreamItem::Data(serde_json::json!({"key": "value"})),
            StreamItem::End,
            StreamItem::Error {
                node_id: "node-1".to_string(),
                message: "boom".to_string(),
            },
        ] {
            let json = serde_json::to_string(&item).unwrap();
            let back: StreamItem = serde_json::from_str(&json).unwrap();
            assert_eq!(back.is_data(), item.is_data());
        }
    }
}
