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
        .call(|| async { Err::<(), _>(BirdSongError::RetryExhausted("test failure".to_string())) })
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

    let result: Result<&str, anyhow::Error> = breaker.execute(|| async { Ok("recovered") }).await;

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

#[test]
fn calculate_delay_custom_multiplier_without_jitter() {
    let policy = RetryPolicy::exponential(5, Duration::from_millis(10))
        .with_multiplier(3.0)
        .with_jitter(false);
    assert_eq!(policy.calculate_delay(2), Duration::from_millis(30));
    assert_eq!(policy.calculate_delay(3), Duration::from_millis(90));
}

#[test]
fn calculate_delay_with_jitter_stays_bounded() {
    let policy = RetryPolicy::exponential(5, Duration::from_millis(100)).with_jitter(true);
    for attempt in 1..=4 {
        let d = policy.calculate_delay(attempt);
        assert!(d <= Duration::from_secs(60));
    }
}

#[test]
fn fixed_policy_zero_initial_delay_for_positive_attempts() {
    let policy = RetryPolicy::fixed(3, Duration::from_millis(0));
    assert_eq!(policy.calculate_delay(1), Duration::from_millis(0));
}

#[test]
fn no_retry_single_attempt_delay_zero() {
    let policy = RetryPolicy::no_retry();
    assert_eq!(policy.max_attempts, 1);
    assert_eq!(policy.calculate_delay(1), Duration::from_millis(0));
}

#[test]
fn retry_policy_builder_chain_caps_at_max_delay() {
    let policy = RetryPolicy::exponential(20, Duration::from_millis(100))
        .with_max_delay(Duration::from_millis(200))
        .with_jitter(false)
        .with_multiplier(2.0);
    assert_eq!(policy.calculate_delay(50), Duration::from_millis(200));
}
