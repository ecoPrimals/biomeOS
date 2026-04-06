// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Concurrent Test Synchronization Helpers
//!
//! Proper async synchronization primitives to replace sleep() anti-patterns.
//! 
//! # Philosophy
//! 
//! "No sleeps in tests - only proper concurrency!"
//! - Wait for actual events, not arbitrary time
//! - Use channels, notify, and watch for coordination
//! - Test concurrent behavior, not timing assumptions

use anyhow::Result;
use std::future::Future;
use std::time::Duration;
use tokio::sync::{Notify, watch};
use std::sync::Arc;

/// Ready signal for coordination between tasks
///
/// Use this instead of `sleep()` to wait for initialization.
///
/// # Example
///
/// ```rust
/// use tests::helpers::sync::ReadySignal;
///
/// #[tokio::test(flavor = "multi_thread")]
/// async fn test_with_proper_sync() {
///     let ready = ReadySignal::new();
///     let ready_clone = ready.clone();
///     
///     // Spawn background task
///     tokio::spawn(async move {
///         // Do initialization
///         setup().await;
///         // Signal ready
///         ready_clone.signal();
///     });
///     
///     // Wait for ready signal (not sleep!)
///     ready.wait().await;
///     
///     // Now test
///     assert!(is_ready());
/// }
/// ```
#[derive(Clone)]
pub struct ReadySignal {
    notify: Arc<Notify>,
}

impl ReadySignal {
    /// Create a new ready signal
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
        }
    }
    
    /// Wait for the signal
    pub async fn wait(&self) {
        self.notify.notified().await
    }
    
    /// Wait for signal with timeout
    pub async fn wait_timeout(&self, timeout: Duration) -> Result<()> {
        tokio::time::timeout(timeout, self.wait())
            .await
            .map_err(|_| anyhow::anyhow!("Timeout waiting for ready signal"))?;
        Ok(())
    }
    
    /// Send the signal
    pub fn signal(&self) {
        self.notify.notify_waiters();
    }
}

impl Default for ReadySignal {
    fn default() -> Self {
        Self::new()
    }
}

/// State watcher for monitoring async state changes
///
/// Use this instead of polling with sleep() to wait for state changes.
///
/// # Example
///
/// ```rust
/// use tests::helpers::sync::StateWatcher;
///
/// #[tokio::test(flavor = "multi_thread")]
/// async fn test_state_change() {
///     let watcher = StateWatcher::new(false);
///     let mut rx = watcher.subscribe();
///     
///     // Spawn task that changes state
///     let watcher_clone = watcher.clone();
///     tokio::spawn(async move {
///         do_work().await;
///         watcher_clone.update(true);
///     });
///     
///     // Wait for state to become true
///     rx.wait_for(|&v| v).await.unwrap();
///     
///     assert_eq!(*watcher.get(), true);
/// }
/// ```
#[derive(Clone)]
pub struct StateWatcher<T: Clone> {
    tx: Arc<watch::Sender<T>>,
}

impl<T: Clone + Send + Sync + 'static> StateWatcher<T> {
    /// Create a new state watcher with initial value
    pub fn new(initial: T) -> Self {
        let (tx, _) = watch::channel(initial);
        Self {
            tx: Arc::new(tx),
        }
    }
    
    /// Update the state
    pub fn update(&self, value: T) {
        let _ = self.tx.send(value);
    }
    
    /// Get current state
    pub fn get(&self) -> watch::Ref<'_, T> {
        self.tx.borrow()
    }
    
    /// Subscribe to state changes
    pub fn subscribe(&self) -> StateReceiver<T> {
        StateReceiver {
            rx: self.tx.subscribe(),
        }
    }
}

/// Receiver for state changes
pub struct StateReceiver<T> {
    rx: watch::Receiver<T>,
}

impl<T: Clone> StateReceiver<T> {
    /// Wait for a condition to be true
    pub async fn wait_for<F>(&mut self, mut condition: F) -> Result<()>
    where
        F: FnMut(&T) -> bool,
    {
        // Check current value first
        if condition(&*self.rx.borrow()) {
            return Ok(());
        }
        
        // Wait for changes
        while self.rx.changed().await.is_ok() {
            if condition(&*self.rx.borrow()) {
                return Ok(());
            }
        }
        
        Err(anyhow::anyhow!("Watch channel closed"))
    }
    
    /// Wait for condition with timeout
    pub async fn wait_for_timeout<F>(&mut self, condition: F, timeout: Duration) -> Result<()>
    where
        F: FnMut(&T) -> bool + Send,
    {
        tokio::time::timeout(timeout, self.wait_for(condition))
            .await
            .map_err(|_| anyhow::anyhow!("Timeout waiting for condition"))?
    }
}

/// Wait for an async condition to become true
///
/// This is a last resort - prefer ReadySignal or StateWatcher.
/// Only use when you truly need to poll an external state.
///
/// # Example
///
/// ```rust
/// use tests::helpers::sync::wait_for_condition;
/// use std::time::Duration;
///
/// #[tokio::test(flavor = "multi_thread")]
/// async fn test_external_state() {
///     start_service().await;
///     
///     // Wait for external service to be ready
///     wait_for_condition(
///         || async { service_is_ready().await },
///         Duration::from_secs(5)
///     ).await.unwrap();
///     
///     // Now test
///     test_service().await;
/// }
/// ```
pub async fn wait_for_condition<F, Fut>(mut check: F, timeout: Duration) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    tokio::time::timeout(timeout, async {
        while !check().await {
            // Short sleep only for polling external state
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .map_err(|_| anyhow::anyhow!("Timeout waiting for condition"))?;
    
    Ok(())
}

/// Barrier for coordinating multiple tasks
///
/// Use when you need N tasks to reach a synchronization point.
///
/// # Example
///
/// ```rust
/// use tests::helpers::sync::Barrier;
/// use std::sync::Arc;
///
/// #[tokio::test(flavor = "multi_thread")]
/// async fn test_coordinated_start() {
///     let barrier = Arc::new(Barrier::new(3));
///     
///     for i in 0..3 {
///         let barrier = barrier.clone();
///         tokio::spawn(async move {
///             setup(i).await;
///             // Wait for all tasks to be ready
///             barrier.wait().await;
///             // Now all start together
///             run_test(i).await;
///         });
///     }
///     
///     // Wait for all to complete
///     tokio::time::sleep(Duration::from_secs(1)).await;
/// }
/// ```
pub struct Barrier {
    notify: Notify,
    count: Arc<std::sync::atomic::AtomicUsize>,
    target: usize,
}

impl Barrier {
    /// Create a barrier for N tasks
    pub fn new(n: usize) -> Self {
        Self {
            notify: Notify::new(),
            count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            target: n,
        }
    }
    
    /// Wait at the barrier
    pub async fn wait(&self) {
        use std::sync::atomic::Ordering;
        
        let old = self.count.fetch_add(1, Ordering::SeqCst);
        
        if old + 1 >= self.target {
            // Last one to arrive
            self.notify.notify_waiters();
        } else {
            // Wait for others
            self.notify.notified().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test(flavor = "multi_thread")]
    async fn test_ready_signal() {
        let ready = ReadySignal::new();
        let ready_clone = ready.clone();
        
        let handle = tokio::spawn(async move {
            for _ in 0..5 {
                tokio::task::yield_now().await;
            }
            ready_clone.signal();
        });
        
        // This should complete when signal is sent, not timeout
        ready.wait_timeout(Duration::from_secs(1)).await.unwrap();
        
        handle.await.unwrap();
    }
    
    #[tokio::test(flavor = "multi_thread")]
    async fn test_state_watcher() {
        let watcher = StateWatcher::new(0);
        let mut rx = watcher.subscribe();
        
        let watcher_clone = watcher.clone();
        tokio::spawn(async move {
            for i in 1..=5 {
                tokio::task::yield_now().await;
                watcher_clone.update(i);
            }
        });
        
        // Wait for value to reach 5
        rx.wait_for_timeout(|&v| v == 5, Duration::from_secs(1))
            .await
            .unwrap();
        
        assert_eq!(*watcher.get(), 5);
    }
    
    #[tokio::test(flavor = "multi_thread")]
    async fn test_barrier() {
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = vec![];
        
        for i in 0..3 {
            let barrier = barrier.clone();
            let handle = tokio::spawn(async move {
                let start = tokio::time::Instant::now();
                barrier.wait().await;
                let elapsed = start.elapsed();
                (i, elapsed)
            });
            handles.push(handle);
        }
        
        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        
        // All should complete within ~10ms of each other
        let max_elapsed = results.iter().map(|(_, e)| *e).max().unwrap();
        let min_elapsed = results.iter().map(|(_, e)| *e).min().unwrap();
        assert!(max_elapsed - min_elapsed < Duration::from_millis(50));
    }
}

