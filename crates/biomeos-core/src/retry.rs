// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Retry Logic and Circuit Breaker Patterns
//!
//! This module provides fault tolerance patterns for resilient primal communication.
//!
//! ## Retry Logic
//!
//! Implements exponential backoff with jitter for transient failures.
//!
//! ## Circuit Breaker
//!
//! Prevents cascade failures by opening circuit after sustained errors,
//! allowing system to recover before retrying.
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_core::retry::{RetryPolicy, CircuitBreaker};
//!
//! // Configure retry policy
//! let policy = RetryPolicy::exponential(3, Duration::from_millis(100));
//!
//! // Execute with retries
//! let result = policy.execute(|| async {
//!     // Your async operation
//!     Ok::<_, Error>(response)
//! }).await;
//!
//! // Use circuit breaker
//! let breaker = CircuitBreaker::new(5, Duration::from_secs(30));
//! let result = breaker.call(|| async {
//!     // Your async operation
//!     Ok::<_, Error>(response)
//! }).await;
//! ```

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;

/// Error type for retry logic operations
#[derive(Debug, thiserror::Error)]
pub enum RetryError {
    /// All retry attempts were exhausted
    #[error("Operation failed after retries: {0}")]
    RetryExhausted(String),

    /// The circuit breaker is open, blocking requests
    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),
}

// Alias for backward compatibility
type BirdSongError = RetryError;
use tracing::{debug, warn};

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    max_attempts: usize,

    /// Initial delay before first retry
    initial_delay: Duration,

    /// Maximum delay between retries
    max_delay: Duration,

    /// Backoff multiplier (e.g., 2.0 for exponential)
    multiplier: f64,

    /// Add random jitter to prevent thundering herd
    jitter: bool,
}

impl RetryPolicy {
    /// Create a new retry policy with exponential backoff
    pub fn exponential(max_attempts: usize, initial_delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: true,
        }
    }

    /// Create a new retry policy with fixed delays
    pub fn fixed(max_attempts: usize, delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay: delay,
            max_delay: delay,
            multiplier: 1.0,
            jitter: false,
        }
    }

    /// Create a new retry policy with no retries
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            initial_delay: Duration::from_secs(0),
            max_delay: Duration::from_secs(0),
            multiplier: 1.0,
            jitter: false,
        }
    }

    /// Builder: Set maximum delay
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Builder: Set backoff multiplier
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Builder: Enable/disable jitter
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Calculate delay for a given attempt
    fn calculate_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        let base_delay = self.initial_delay.as_millis() as f64
            * self
                .multiplier
                .powi(i32::try_from(attempt - 1).unwrap_or(0));

        let delay_ms = base_delay.min(self.max_delay.as_millis() as f64);

        let final_delay = if self.jitter {
            // Add up to 25% jitter
            let jitter_factor = rand::random::<f64>().mul_add(0.25, 1.0);
            (delay_ms * jitter_factor) as u64
        } else {
            delay_ms as u64
        };

        Duration::from_millis(final_delay)
    }

    /// Execute operation with retries
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut last_error = None;

        for attempt in 0..self.max_attempts {
            if attempt > 0 {
                let delay = self.calculate_delay(attempt);
                debug!(
                    "Retry attempt {}/{}, delay: {:?}",
                    attempt + 1,
                    self.max_attempts,
                    delay
                );
                tokio::time::sleep(delay).await;
            }

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt < self.max_attempts - 1 {
                        debug!(
                            "Operation failed (attempt {}/{}): {}",
                            attempt + 1,
                            self.max_attempts,
                            e
                        );
                    }
                    last_error = Some(e);
                }
            }
        }

        #[expect(
            clippy::expect_used,
            reason = "retry loop must have executed at least once"
        )]
        Err(last_error.expect("retry loop must have executed at least once"))
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::exponential(3, Duration::from_millis(100))
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,

    /// Circuit is open, requests fail immediately
    Open {
        /// When the circuit was opened
        opened_at: Instant,
        /// Number of consecutive failures that triggered the open
        failure_count: usize,
    },

    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker for preventing cascade failures
pub struct CircuitBreaker {
    /// Current state
    state: Arc<RwLock<CircuitState>>,

    /// Number of failures before opening circuit
    failure_threshold: usize,

    /// Duration to keep circuit open before testing recovery
    timeout: Duration,

    /// Current failure count (in closed state)
    failure_count: Arc<RwLock<usize>>,

    /// Current success count (in half-open state)
    success_count: Arc<RwLock<usize>>,

    /// Number of successes needed to close circuit from half-open
    success_threshold: usize,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_threshold,
            timeout,
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            success_threshold: 2, // Default: 2 successes to close
        }
    }

    /// Builder: Set success threshold for half-open → closed transition
    pub fn with_success_threshold(mut self, threshold: usize) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Get current circuit state
    pub async fn state(&self) -> CircuitState {
        self.state.read().await.clone()
    }

    /// Check if circuit is open
    pub async fn is_open(&self) -> bool {
        matches!(*self.state.read().await, CircuitState::Open { .. })
    }

    /// Check if circuit should transition from open to half-open
    async fn should_attempt_reset(&self) -> bool {
        let state = self.state.read().await;
        if let CircuitState::Open { opened_at, .. } = *state {
            Instant::now().duration_since(opened_at) >= self.timeout
        } else {
            false
        }
    }

    /// Record a successful call
    async fn record_success(&self) {
        let mut state = self.state.write().await;

        match *state {
            CircuitState::Closed => {
                // Reset failure count on success
                *self.failure_count.write().await = 0;
            }
            CircuitState::HalfOpen => {
                let mut success_count = self.success_count.write().await;
                *success_count += 1;

                if *success_count >= self.success_threshold {
                    // Enough successes, close the circuit
                    *state = CircuitState::Closed;
                    *self.failure_count.write().await = 0;
                    *success_count = 0;
                    debug!("Circuit breaker closed (service recovered)");
                }
            }
            CircuitState::Open { .. } => {
                // Shouldn't happen, but reset to closed if we get a success
                *state = CircuitState::Closed;
                *self.failure_count.write().await = 0;
            }
        }
    }

    /// Record a failed call
    async fn record_failure(&self) {
        let mut state = self.state.write().await;

        match *state {
            CircuitState::Closed => {
                let mut failure_count = self.failure_count.write().await;
                *failure_count += 1;

                if *failure_count >= self.failure_threshold {
                    // Too many failures, open the circuit
                    *state = CircuitState::Open {
                        opened_at: Instant::now(),
                        failure_count: *failure_count,
                    };
                    warn!("Circuit breaker opened ({} failures)", *failure_count);
                }
            }
            CircuitState::HalfOpen => {
                // Failure in half-open, go back to open
                *state = CircuitState::Open {
                    opened_at: Instant::now(),
                    failure_count: self.failure_threshold,
                };
                *self.success_count.write().await = 0;
                warn!("Circuit breaker re-opened (half-open test failed)");
            }
            CircuitState::Open { .. } => {
                // Already open, nothing to do
            }
        }
    }

    /// Execute operation through circuit breaker
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T, BirdSongError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, BirdSongError>>,
    {
        self.execute(operation).await
    }

    /// Execute an operation through the circuit breaker with generic error type.
    ///
    /// Like [`CircuitBreaker::call`], this manages the full circuit state machine: checks for
    /// open state, handles half-open recovery, and records success/failure.
    /// Unlike `call`, the operation can return any error type convertible from
    /// [`RetryError`], making it compatible with `anyhow::Error` and other
    /// common error types.
    pub async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: From<RetryError>,
    {
        // Check if we should attempt reset
        if self.should_attempt_reset().await {
            let mut state = self.state.write().await;
            if matches!(*state, CircuitState::Open { .. }) {
                *state = CircuitState::HalfOpen;
                debug!("Circuit breaker half-open (testing recovery)");
            }
        }

        // Check if circuit is open
        {
            let state = self.state.read().await;
            if let CircuitState::Open {
                opened_at,
                failure_count,
            } = *state
            {
                let elapsed = Instant::now().duration_since(opened_at);
                return Err(E::from(RetryError::CircuitBreakerOpen(format!(
                    "Circuit open for {elapsed:?} ({failure_count} failures, timeout: {:?})",
                    self.timeout
                ))));
            }
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_exponential() {
        let policy = RetryPolicy::exponential(3, Duration::from_millis(100));
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.initial_delay, Duration::from_millis(100));
    }

    #[test]
    fn test_retry_policy_delay_calculation() {
        let policy = RetryPolicy::exponential(5, Duration::from_millis(100)).with_jitter(false);

        let delay0 = policy.calculate_delay(0);
        let delay1 = policy.calculate_delay(1);
        let delay2 = policy.calculate_delay(2);
        let delay3 = policy.calculate_delay(3);

        assert_eq!(delay0, Duration::from_millis(0));
        assert_eq!(delay1, Duration::from_millis(100));
        assert_eq!(delay2, Duration::from_millis(200));
        assert_eq!(delay3, Duration::from_millis(400));
    }

    #[test]
    fn test_retry_policy_max_delay() {
        let policy = RetryPolicy::exponential(10, Duration::from_millis(100))
            .with_max_delay(Duration::from_millis(500))
            .with_jitter(false);

        let delay5 = policy.calculate_delay(5);
        let delay10 = policy.calculate_delay(10);

        assert!(delay5 <= Duration::from_millis(500));
        assert!(delay10 <= Duration::from_millis(500));
    }

    #[test]
    fn test_retry_error_display() {
        let err = RetryError::RetryExhausted("test".to_string());
        assert!(err.to_string().contains("retries"));
        let err2 = RetryError::CircuitBreakerOpen("open".to_string());
        assert!(err2.to_string().contains("Circuit breaker"));
    }

    #[test]
    fn test_retry_policy_fixed() {
        let policy = RetryPolicy::fixed(5, Duration::from_millis(50));
        assert_eq!(policy.initial_delay, Duration::from_millis(50));
        assert_eq!(policy.max_delay, Duration::from_millis(50));
    }

    #[test]
    fn test_retry_policy_no_retry() {
        let policy = RetryPolicy::no_retry();
        assert_eq!(policy.max_attempts, 1);
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
    }

    #[test]
    fn test_circuit_state_equality() {
        assert_eq!(CircuitState::Closed, CircuitState::Closed);
        assert_eq!(CircuitState::HalfOpen, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_retry_policy_execute_success() {
        let policy = RetryPolicy::exponential(3, Duration::from_millis(10));
        let mut attempts = 0;

        let result = policy
            .execute(|| {
                attempts += 1;
                async move {
                    if attempts < 2 {
                        Err("transient error")
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert_eq!(result, Ok("success"));
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn test_retry_policy_execute_all_fail() {
        let policy = RetryPolicy::exponential(3, Duration::from_millis(10));
        let mut attempts = 0;

        let result = policy
            .execute(|| {
                attempts += 1;
                async move { Err::<(), _>("permanent error") }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempts, 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closed_to_open() {
        let breaker = CircuitBreaker::new(3, Duration::from_secs(1));

        // First 2 failures should keep circuit closed
        for _ in 0..2 {
            let _ = breaker
                .call(|| async {
                    Err::<(), _>(BirdSongError::RetryExhausted("test failure".to_string()))
                })
                .await;
        }

        assert!(!breaker.is_open().await);

        // 3rd failure should open circuit
        let _ = breaker
            .call(|| async {
                Err::<(), _>(BirdSongError::RetryExhausted("test failure".to_string()))
            })
            .await;

        assert!(breaker.is_open().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_rejects() {
        let breaker = CircuitBreaker::new(2, Duration::from_secs(60));

        // Open the circuit
        for _ in 0..2 {
            let _ = breaker
                .call(|| async {
                    Err::<(), _>(BirdSongError::RetryExhausted("test failure".to_string()))
                })
                .await;
        }

        // Next call should fail immediately
        let result = breaker
            .call(|| async { Ok::<_, BirdSongError>("should not reach here") })
            .await;

        assert!(matches!(result, Err(BirdSongError::CircuitBreakerOpen(_))));
    }

    #[tokio::test]
    async fn test_circuit_breaker_execute_generic_error() {
        let breaker = CircuitBreaker::new(3, Duration::from_secs(1));

        let result: Result<String, anyhow::Error> =
            breaker.execute(|| async { Ok("hello".to_string()) }).await;

        assert_eq!(result.unwrap(), "hello");
    }

    #[tokio::test]
    async fn test_circuit_breaker_execute_opens_on_failures() {
        let breaker = CircuitBreaker::new(2, Duration::from_secs(60));

        for _ in 0..2 {
            let _: Result<(), anyhow::Error> = breaker
                .execute(|| async { Err(anyhow::anyhow!("boom")) })
                .await;
        }

        assert!(breaker.is_open().await);

        let result: Result<(), anyhow::Error> = breaker.execute(|| async { Ok(()) }).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circuit"));
    }

    #[tokio::test(start_paused = true)]
    async fn test_circuit_breaker_execute_half_open_recovery() {
        let breaker = CircuitBreaker::new(2, Duration::from_millis(100)).with_success_threshold(1);

        for _ in 0..2 {
            let _: Result<(), anyhow::Error> = breaker
                .execute(|| async { Err(anyhow::anyhow!("fail")) })
                .await;
        }

        assert!(breaker.is_open().await);

        tokio::time::advance(Duration::from_millis(150)).await;

        let result: Result<&str, anyhow::Error> =
            breaker.execute(|| async { Ok("recovered") }).await;

        assert_eq!(result.unwrap(), "recovered");
        let state = breaker.state().await;
        assert_eq!(state, CircuitState::Closed);
    }

    #[tokio::test(start_paused = true)]
    async fn test_circuit_breaker_half_open_recovery() {
        let breaker = CircuitBreaker::new(2, Duration::from_millis(100)).with_success_threshold(2);

        // Open the circuit
        for _ in 0..2 {
            let _ = breaker
                .call(|| async { Err::<(), _>(BirdSongError::RetryExhausted("test".to_string())) })
                .await;
        }

        assert!(breaker.is_open().await);

        tokio::time::advance(Duration::from_millis(150)).await;

        // First success in half-open
        let _ = breaker
            .call(|| async { Ok::<_, BirdSongError>("success") })
            .await;

        // Should still be half-open (need 2 successes)
        let state = breaker.state().await;
        assert_eq!(state, CircuitState::HalfOpen);

        // Second success should close circuit
        let _ = breaker
            .call(|| async { Ok::<_, BirdSongError>("success") })
            .await;

        let state = breaker.state().await;
        assert_eq!(state, CircuitState::Closed);
    }
}
