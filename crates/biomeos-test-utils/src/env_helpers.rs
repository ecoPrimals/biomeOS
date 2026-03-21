// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Environment variable helpers for testing.
//!
//! # Rust 2024 and `unsafe`
//!
//! The 2024 edition marks [`std::env::set_var`] and [`std::env::remove_var`] as `unsafe` because
//! mutating the process environment is not synchronized with other threads that may read it via
//! [`std::env::var`] or OS APIs. There is **no safe alternative** in `std` for in-process
//! mutation — this is a fundamental platform constraint, not a design flaw we can evolve away.
//!
//! This module is **test-only** (`biomeos-test-utils` never ships in production paths). All
//! mutations are serialized through [`ENV_MUTEX`] so that concurrent test threads cannot race
//! on environment writes. The [`TestEnvGuard`] RAII type captures the previous value before
//! changing the variable and restores it on drop, limiting test pollution.
//!
//! The two `unsafe` blocks below are the **minimum possible surface**: each is a single
//! `std::env` call guarded by the process-wide mutex. The rest of the workspace keeps
//! `#![forbid(unsafe_code)]`.

use std::ffi::OsStr;
use std::sync::Mutex;

/// Process-wide mutex serializing all env mutations, making the `unsafe` blocks sound
/// even when test threads run concurrently. Tests that read the same key should still
/// use `#[serial]` to avoid TOCTOU between read and write.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Sets `key` to `value` in the process environment (test-only).
///
/// Prefer [`TestEnvGuard::set`] when you need automatic restoration after a scope.
#[allow(unsafe_code)]
pub fn set_test_env<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: V) {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // SAFETY: Mutation is serialized by ENV_MUTEX. This crate is test-only; callers
    // must serialize reads of the same key (e.g. `#[serial]`) to avoid TOCTOU.
    unsafe {
        std::env::set_var(key, value);
    }
}

/// Removes `key` from the process environment (test-only).
///
/// Prefer [`TestEnvGuard::remove`] when you need automatic restoration after a scope.
#[allow(unsafe_code)]
pub fn remove_test_env<K: AsRef<OsStr>>(key: K) {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // SAFETY: Same as `set_test_env` — serialized by ENV_MUTEX, test-only.
    unsafe {
        std::env::remove_var(key);
    }
}

/// RAII guard that restores an environment variable to its captured value on [`Drop`].
///
/// Construct with [`TestEnvGuard::set`], [`TestEnvGuard::remove`], or [`TestEnvGuard::new`].
/// Restoration uses [`set_test_env`] / [`remove_test_env`], so the same thread-safety
/// guarantees (mutex-serialized writes) apply when the guard is dropped.
pub struct TestEnvGuard {
    key: String,
    original: Option<String>,
}

impl TestEnvGuard {
    /// Captures the current value of `key`, then sets it to `new_value` or removes it if `None`.
    #[must_use]
    pub fn new(key: &str, new_value: Option<&str>) -> Self {
        let original = std::env::var(key).ok();
        match new_value {
            Some(v) => set_test_env(key, v),
            None => remove_test_env(key),
        }
        Self {
            key: key.to_string(),
            original,
        }
    }

    /// Sets `key` to `value` and restores the previous value (if any) on drop.
    #[must_use]
    pub fn set(key: &str, value: &str) -> Self {
        Self::new(key, Some(value))
    }

    /// Removes `key` and restores the previous value (if any) on drop.
    #[must_use]
    pub fn remove(key: &str) -> Self {
        Self::new(key, None)
    }
}

impl Drop for TestEnvGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(v) => set_test_env(&self.key, v),
            None => remove_test_env(&self.key),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_remove() {
        let key = "BIOMEOS_TEST_ENV_HELPER_UNIQUE_KEY";
        set_test_env(key, "hello");
        assert_eq!(std::env::var(key).unwrap(), "hello");
        remove_test_env(key);
        assert!(std::env::var(key).is_err());
    }

    #[test]
    fn test_guard_restores() {
        let key = "BIOMEOS_TEST_GUARD_UNIQUE_KEY";
        remove_test_env(key);

        {
            let _guard = TestEnvGuard::set(key, "temporary");
            assert_eq!(std::env::var(key).unwrap(), "temporary");
        }
        assert!(std::env::var(key).is_err());
    }

    #[test]
    fn test_guard_remove_and_restore() {
        let key = "BIOMEOS_TEST_GUARD_REMOVE_KEY";
        set_test_env(key, "original_value");

        {
            let _guard = TestEnvGuard::remove(key);
            assert!(std::env::var(key).is_err());
        }
        assert_eq!(std::env::var(key).unwrap(), "original_value");
        remove_test_env(key);
    }

    #[test]
    fn test_mutex_prevents_poisoning() {
        let _lock = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        drop(_lock);
        let _lock2 = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    }
}
