// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Readiness signaling for concurrent test infrastructure.
//!
//! Replaces `tokio::time::sleep(Duration::from_millis(50))` patterns with
//! proper synchronization. A spawned server signals when it's bound and
//! ready — the test waits on that signal instead of racing with a sleep.

use tokio::sync::oneshot;

/// Sender half — the server calls `signal()` once it is ready to accept.
pub struct ReadySender(Option<oneshot::Sender<()>>);

/// Receiver half — the test awaits `wait()` before connecting.
pub struct ReadyReceiver(oneshot::Receiver<()>);

/// Create a linked `(sender, receiver)` pair.
#[must_use]
pub fn ready_signal() -> (ReadySender, ReadyReceiver) {
    let (tx, rx) = oneshot::channel();
    (ReadySender(Some(tx)), ReadyReceiver(rx))
}

impl ReadySender {
    /// Signal that the server is ready. Idempotent — second call is a no-op.
    pub fn signal(&mut self) {
        if let Some(tx) = self.0.take() {
            let _ = tx.send(());
        }
    }
}

impl ReadyReceiver {
    /// Wait until the server signals readiness.
    ///
    /// Returns `Ok(())` on signal, `Err` if the sender was dropped (server crashed).
    pub async fn wait(self) -> Result<(), &'static str> {
        self.0
            .await
            .map_err(|_| "server dropped ReadySender without signaling")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn signal_then_wait() {
        let (mut tx, rx) = ready_signal();
        tx.signal();
        assert!(rx.wait().await.is_ok());
    }

    #[tokio::test]
    async fn drop_sender_returns_err() {
        let (tx, rx) = ready_signal();
        drop(tx);
        assert!(rx.wait().await.is_err());
    }

    #[tokio::test]
    async fn double_signal_is_noop() {
        let (mut tx, rx) = ready_signal();
        tx.signal();
        tx.signal();
        assert!(rx.wait().await.is_ok());
    }
}
