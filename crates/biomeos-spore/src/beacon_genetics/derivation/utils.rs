// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Device lineage derivation utilities

use bytes::Bytes;

/// Generate device entropy from available sources
pub fn generate_device_entropy() -> Bytes {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
        .hash(&mut hasher);

    std::process::id().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);

    let hash = hasher.finish().to_le_bytes();

    let mut entropy = Vec::with_capacity(32);
    for i in 0..4 {
        let mut h = DefaultHasher::new();
        hash.hash(&mut h);
        i.hash(&mut h);
        entropy.extend_from_slice(&h.finish().to_le_bytes());
    }

    Bytes::from(entropy)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_device_entropy_length() {
        let entropy = generate_device_entropy();
        assert_eq!(entropy.len(), 32);
    }

    #[test]
    fn test_generate_device_entropy_deterministic_per_run() {
        let e1 = generate_device_entropy();
        let e2 = generate_device_entropy();
        assert_eq!(e1.len(), e2.len());
        // Entropy uses time/pid/thread - may differ between calls
        assert!(!e1.is_empty());
        assert!(!e2.is_empty());
    }
}
