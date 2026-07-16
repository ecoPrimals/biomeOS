// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{JsonRpcNotification, *};
use std::sync::Arc;

#[tokio::test]
async fn test_process_events_handler_error_continues() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::sync::oneshot;

    let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));
    let handler = RealTimeEventHandler::new(subscriber.clone());

    let processed = Arc::new(AtomicU32::new(0));
    let errored = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (tx, rx) = oneshot::channel::<()>();

    let p = processed.clone();
    let e = errored.clone();
    let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));
    let tx_clone = tx.clone();
    let mut h = handler;
    let handle = tokio::spawn(async move {
        h.process_events(move |event| {
            p.fetch_add(1, Ordering::SeqCst);
            if matches!(event, RealTimeEvent::Heartbeat { .. }) {
                e.store(true, Ordering::SeqCst);
                let value = tx_clone.blocking_lock().take();
                if let Some(sender) = value {
                    let _ = sender.send(());
                }
                Err(anyhow::anyhow!("simulated handler error"))
            } else {
                Ok(())
            }
        })
        .await
    });

    subscriber.send_event(RealTimeEvent::Heartbeat {
        timestamp: 1,
        primals_count: 1,
        healthy_count: 1,
    });
    subscriber.send_event(RealTimeEvent::Heartbeat {
        timestamp: 2,
        primals_count: 2,
        healthy_count: 2,
    });

    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), rx).await;
    assert!(
        processed.load(Ordering::SeqCst) >= 1,
        "handler should have processed at least one event"
    );
    assert!(
        errored.load(Ordering::SeqCst),
        "handler should have seen the error path"
    );
    handle.abort();
}
