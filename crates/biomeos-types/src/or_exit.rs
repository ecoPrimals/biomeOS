// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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
    use std::io;

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

    /// `OrExit` is implemented for `Result<T, E>` where `E: Display`; exercise distinct `E` types on the Ok path.
    #[test]
    fn or_exit_accepts_result_with_string_error_type() {
        let r: Result<u32, String> = Ok(100);
        assert_eq!(r.or_exit("ctx"), 100);
    }

    #[test]
    fn or_exit_accepts_result_with_io_error_type() {
        let r: Result<u32, io::Error> = Ok(200);
        assert_eq!(r.or_exit("ctx"), 200);
    }

    #[test]
    fn or_exit_accepts_result_with_anyhow_error_type() {
        let r: Result<u32, anyhow::Error> = Ok(300);
        assert_eq!(r.or_exit("ctx"), 300);
    }

    #[test]
    fn or_exit_accepts_option_of_owned_string() {
        let o: Option<String> = Some("owned".to_string());
        assert_eq!(o.or_exit("ctx").as_str(), "owned");
    }

    #[test]
    fn result_string_ok() {
        let r: Result<String, String> = Ok("value".to_string());
        assert_eq!(r.or_exit("ctx"), "value");
    }

    #[test]
    fn option_u32_some() {
        let o: Option<u32> = Some(7);
        assert_eq!(o.or_exit("ctx"), 7);
    }

    #[test]
    fn result_vec_u8_ok_str_err() {
        let r: Result<Vec<u8>, &str> = Ok(vec![1, 2, 3]);
        assert_eq!(r.or_exit("ctx"), vec![1, 2, 3]);
    }

    /// Zero-sized `T` in `Ok` — still returns through the success path.
    #[test]
    fn result_ok_zero_sized_value() {
        let r: Result<(), String> = Ok(());
        let () = r.or_exit("ctx");
    }
}
