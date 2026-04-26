// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Type-safe numeric cast helpers.
//!
//! Absorbed from airSpring's `cast` module (used by wetSpring, groundSpring).
//! Replaces scattered `#[allow(clippy::cast_*)]` with explicit, auditable helpers.
//!
//! Each function documents the precision/range trade-off so callers understand
//! what they're opting into.

/// Error returned when a numeric cast would lose data.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("cast overflow: {from_type} -> {to_type} (value: {value})")]
pub struct CastError {
    /// Source type name.
    pub from_type: &'static str,
    /// Target type name.
    pub to_type: &'static str,
    /// String representation of the value that could not be converted.
    pub value: String,
}

/// `usize` to `f64`. Lossless for values up to 2^53.
///
/// Returns `Err` if the value exceeds `f64`'s exact integer range.
/// Uses `u64` for the bound so this compiles on 32-bit targets (armv7).
pub fn usize_f64(v: usize) -> Result<f64, CastError> {
    const MAX_EXACT: u64 = 1_u64 << 53;
    if (v as u64) <= MAX_EXACT {
        #[expect(clippy::cast_precision_loss, reason = "guarded by MAX_EXACT check")]
        let r = v as f64;
        Ok(r)
    } else {
        Err(CastError {
            from_type: "usize",
            to_type: "f64",
            value: v.to_string(),
        })
    }
}

/// `f64` to `usize`. Only succeeds for non-negative, finite, integer values in range.
pub fn f64_usize(v: f64) -> Result<usize, CastError> {
    if v.is_finite() && v >= 0.0 && v <= (usize::MAX as f64) && v.fract() == 0.0 {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "guarded by range and fract() checks"
        )]
        let r = v as usize;
        Ok(r)
    } else {
        Err(CastError {
            from_type: "f64",
            to_type: "usize",
            value: v.to_string(),
        })
    }
}

/// `u64` to `usize`. Infallible on 64-bit platforms.
///
/// Returns `Err` on 32-bit platforms when the value exceeds `usize::MAX`.
pub fn u64_usize(v: u64) -> Result<usize, CastError> {
    usize::try_from(v).map_err(|_| CastError {
        from_type: "u64",
        to_type: "usize",
        value: v.to_string(),
    })
}

/// `usize` to `u64`. Infallible (usize <= u64 on all supported targets).
#[must_use]
pub const fn usize_u64(v: usize) -> u64 {
    v as u64
}

/// `usize` to `u32`. Returns `Err` if the value exceeds `u32::MAX`.
pub fn usize_u32(v: usize) -> Result<u32, CastError> {
    u32::try_from(v).map_err(|_| CastError {
        from_type: "usize",
        to_type: "u32",
        value: v.to_string(),
    })
}

/// `u32` to `usize`. Always succeeds.
#[must_use]
pub const fn u32_usize(v: u32) -> usize {
    v as usize
}

/// `i64` to `usize`. Fails for negative values or overflow.
pub fn i64_usize(v: i64) -> Result<usize, CastError> {
    usize::try_from(v).map_err(|_| CastError {
        from_type: "i64",
        to_type: "usize",
        value: v.to_string(),
    })
}

/// `usize` to `i64`. Fails if the value exceeds `i64::MAX`.
pub fn usize_i64(v: usize) -> Result<i64, CastError> {
    i64::try_from(v).map_err(|_| CastError {
        from_type: "usize",
        to_type: "i64",
        value: v.to_string(),
    })
}

/// `f64` to `f32`, checking for overflow.
pub fn f64_f32(v: f64) -> Result<f32, CastError> {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "precision loss is the point"
    )]
    let r = v as f32;
    if v.is_finite() && !r.is_finite() {
        Err(CastError {
            from_type: "f64",
            to_type: "f32",
            value: v.to_string(),
        })
    } else {
        Ok(r)
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usize_f64_small() {
        assert!((usize_f64(42).unwrap() - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn usize_f64_max_exact() {
        // On 32-bit, usize::MAX < 2^53 so all values are exact;
        // on 64-bit, test the actual boundary.
        if cfg!(target_pointer_width = "64") {
            let max = 1_usize << 53;
            assert!(usize_f64(max).is_ok());
        } else {
            assert!(usize_f64(usize::MAX).is_ok());
        }
    }

    #[test]
    fn usize_f64_overflow() {
        if cfg!(target_pointer_width = "64") {
            let big = (1_usize << 53) + 1;
            assert!(usize_f64(big).is_err());
        }
        // On 32-bit, usize can never exceed 2^53, so overflow is impossible.
    }

    #[test]
    fn f64_usize_valid() {
        assert_eq!(f64_usize(42.0).unwrap(), 42);
    }

    #[test]
    fn f64_usize_negative() {
        assert!(f64_usize(-1.0).is_err());
    }

    #[test]
    fn f64_usize_fractional() {
        assert!(f64_usize(1.5).is_err());
    }

    #[test]
    fn f64_usize_nan() {
        assert!(f64_usize(f64::NAN).is_err());
    }

    #[test]
    fn f64_usize_inf() {
        assert!(f64_usize(f64::INFINITY).is_err());
    }

    #[test]
    fn u64_usize_valid() {
        assert_eq!(u64_usize(100).unwrap(), 100);
    }

    #[test]
    fn usize_u64_roundtrip() {
        assert_eq!(usize_u64(999), 999_u64);
    }

    #[test]
    fn usize_u32_valid() {
        assert_eq!(usize_u32(1000).unwrap(), 1000_u32);
    }

    #[test]
    fn u32_usize_valid() {
        assert_eq!(u32_usize(42), 42_usize);
    }

    #[test]
    fn i64_usize_valid() {
        assert_eq!(i64_usize(10).unwrap(), 10);
    }

    #[test]
    fn i64_usize_negative() {
        assert!(i64_usize(-1).is_err());
    }

    #[test]
    fn usize_i64_valid() {
        assert_eq!(usize_i64(42).unwrap(), 42_i64);
    }

    #[test]
    fn f64_f32_valid() {
        let r = f64_f32(1.5).unwrap();
        assert!((r - 1.5_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn f64_f32_overflow() {
        assert!(f64_f32(f64::MAX).is_err());
    }

    #[test]
    fn f64_f32_nan_passthrough() {
        assert!(f64_f32(f64::NAN).unwrap().is_nan());
    }

    #[test]
    fn cast_error_display() {
        let e = CastError {
            from_type: "u64",
            to_type: "u32",
            value: "5000000000".to_owned(),
        };
        assert!(e.to_string().contains("cast overflow"));
        assert!(e.to_string().contains("5000000000"));
    }
}
