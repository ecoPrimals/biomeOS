// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use std::sync::Arc;

#[tokio::test]
async fn test_process_events_receives_and_processes() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::sync::oneshot;

    let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));
    let handler = RealTimeEventHandler::new(subscriber.clone());

    let event_count = Arc::new(AtomicU32::new(0));
    let (tx, rx) = oneshot::channel::<()>();
    let ec = event_count.clone();
    let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));
    let tx_clone = tx.clone();
    let mut h = handler;
    let _handle = tokio::spawn(async move {
        h.process_events(move |_| {
            ec.fetch_add(1, Ordering::SeqCst);
            let value = tx_clone.blocking_lock().take();
            if let Some(sender) = value {
                let _ = sender.send(());
            }
            Ok(())
        })
        .await
    });

    subscriber.send_event(RealTimeEvent::Heartbeat {
        timestamp: 1,
        primals_count: 1,
        healthy_count: 1,
    });

    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), rx).await;
    assert!(
        event_count.load(Ordering::SeqCst) >= 1,
        "should have processed at least one event"
    );
}

#[tokio::test]
async fn test_subscription_multiple_receivers_independent() {
    let subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    let mut rx1 = subscriber.subscribe();
    let mut rx2 = subscriber.subscribe();

    let event = RealTimeEvent::Heartbeat {
        timestamp: 1,
        primals_count: 1,
        healthy_count: 1,
    };
    subscriber.send_event(event);

    let e1 = rx1.try_recv();
    let e2 = rx2.try_recv();
    assert!(e1.is_ok());
    assert!(e2.is_ok());
    assert!(rx1.try_recv().is_err());
    assert!(rx2.try_recv().is_err());
}

#[tokio::test]
async fn test_event_handler_receives_multiple_event_types() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::sync::oneshot;

    let subscriber = Arc::new(RealTimeEventSubscriber::new("family".to_string()));
    let handler = RealTimeEventHandler::new(subscriber.clone());
    let count = Arc::new(AtomicU32::new(0));
    let (tx, rx) = oneshot::channel::<()>();
    let c = count.clone();
    let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));
    let tx_clone = tx.clone();
    let mut h = handler;
    let _handle = tokio::spawn(async move {
        h.process_events(move |_| {
            let n = c.fetch_add(1, Ordering::SeqCst);
            if n >= 1 {
                let maybe_tx = tx_clone.blocking_lock().take();
                if let Some(sender) = maybe_tx {
                    let _ = sender.send(());
                }
            }
            Ok(())
        })
        .await
    });

    subscriber.send_event(RealTimeEvent::DeviceAdded {
        device_id: "d1".to_string(),
        device_type: "gpu".to_string(),
        capabilities: vec![],
    });
    subscriber.send_event(RealTimeEvent::DeviceRemoved {
        device_id: "d1".to_string(),
    });

    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), rx).await;
    assert!(
        count.load(Ordering::SeqCst) >= 2,
        "should process multiple event types"
    );
}

#[tokio::test]
async fn test_process_events_stops_when_broadcast_closed() {
    let sub = RealTimeEventSubscriber::new("test_family".to_string());
    let rx = sub.subscribe();
    drop(sub);
    let mut handler = RealTimeEventHandler::from_receiver_for_test(rx);
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        handler.process_events(|_| Ok(())),
    )
    .await;
    assert!(
        result.is_ok(),
        "process_events should finish when channel is closed"
    );
    assert!(result.unwrap().is_ok());
}
