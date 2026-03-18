// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Zero-panic startup validation via the `OrExit` trait.
//!
//! Absorbed from groundSpring, loamSpine, and ludoSpring. Provides a clean
//! process exit with a human-readable message instead of a panic backtrace
//! during early startup validation where recovery is impossible.
//!
//! # Example
//!
//! ```rust,no_run
//! use biomeos_types::OrExit;
//!
//! let port: u16 = std::env::var("PORT")
//!     .or_exit("PORT env var must be set")
//!     .parse::<u16>()
//!     .or_exit("PORT must be a valid u16");
//! ```

use std::fmt;
use std::process;

/// Extension trait for `Result` that exits the process on error.
///
/// Intended **only** for irrecoverable startup validation (config parsing,
/// env var loading, socket binding). Never use in request-handling hot paths.
pub trait OrExit<T> {
    /// Unwrap the value or exit the process with a message to stderr.
    ///
    /// Prints `"FATAL: {context}: {error}"` and calls `process::exit(1)`.
    fn or_exit(self, context: &str) -> T;
}

impl<T, E: fmt::Display> OrExit<T> for Result<T, E> {
    fn or_exit(self, context: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                eprintln!("FATAL: {context}: {e}");
                process::exit(1);
            }
        }
    }
}

impl<T> OrExit<T> for Option<T> {
    fn or_exit(self, context: &str) -> T {
        if let Some(v) = self {
            v
        } else {
            eprintln!("FATAL: {context}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_ok_returns_value() {
        let r: Result<u32, &str> = Ok(42);
        assert_eq!(r.or_exit("should not exit"), 42);
    }

    #[test]
    fn option_some_returns_value() {
        let o: Option<&str> = Some("hello");
        assert_eq!(o.or_exit("should not exit"), "hello");
    }
}
