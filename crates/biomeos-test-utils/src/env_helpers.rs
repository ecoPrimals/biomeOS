// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Environment variable helpers for testing.
//!
//! # Rust 2024 and `unsafe`
//!
//! The 2024 edition marks [`std::env::set_var`] and [`std::env::remove_var`] as `unsafe` because
//! mutating the process environment is not synchronized with other threads that may read it via
//! [`std::env::var`] or OS APIs. There is no fully safe alternative for in-process mutation; the
//! intended pattern for production code is to avoid globals and pass configuration explicitly or
//! use [`std::process::Command::env`] for subprocesses.
//!
//! This module is **test-only** (see crate root: `biomeos-test-utils` must not ship in production
//! paths). The [`TestEnvGuard`] RAII type captures the previous value before changing the variable
//! and restores it on drop, limiting test pollution. Callers that run tests in parallel must still
//! avoid concurrent reads/writes of the **same** key unless those tests are serialized (e.g.
//! `serial_test`)—that is a Rust/platform contract, not something this crate can enforce.
//!
//! All `unsafe` is confined to thin wrappers around the two `std::env` calls below; the rest of the
//! workspace keeps `#![deny(unsafe_code)]` via narrowly scoped `#[allow(unsafe_code)]` here.

use std::ffi::OsStr;

/// Sets `key` to `value` in the process environment (test-only).
///
/// Prefer [`TestEnvGuard::set`] when you need automatic restoration after a scope.
///
/// # Caller contract
///
/// No other thread may read `key` concurrently while this runs. In practice, use per-key guards
/// and avoid sharing keys across parallel tests, or serialize tests that touch the same key.
#[allow(unsafe_code)]
pub fn set_test_env<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: V) {
    // SAFETY: Required by Rust 2024 for `set_var`. This crate is test-only; callers must ensure no
    // concurrent `std::env::var`/OS reads of `key` on other threads (see module docs). The call
    // only forwards to libc/OS env update; invariants are process-global ordering, not pointer
    // validity—`AsRef<OsStr>` arguments are valid for the duration of the call.
    unsafe {
        std::env::set_var(key, value);
    }
}

/// Removes `key` from the process environment (test-only).
///
/// Prefer [`TestEnvGuard::remove`] when you need automatic restoration after a scope.
///
/// See [`set_test_env`] for the caller contract.
#[allow(unsafe_code)]
pub fn remove_test_env<K: AsRef<OsStr>>(key: K) {
    // SAFETY: Same as [`set_test_env`], but for `remove_var`. Test-only; no concurrent readers of
    // `key` on other threads.
    unsafe {
        std::env::remove_var(key);
    }
}

/// RAII guard that restores an environment variable to its captured value on [`Drop`].
///
/// Construct with [`TestEnvGuard::set`], [`TestEnvGuard::remove`], or [`TestEnvGuard::new`].
/// Restoration uses [`set_test_env`] / [`remove_test_env`], so the same thread-safety contract
/// applies when the guard is dropped (typically at end of test scope, single-threaded relative to
/// that key).
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
#[allow(clippy::unwrap_used)]
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
}
