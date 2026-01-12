//! Integration tests for Interactive UI
//!
//! Tests the full UI workflow including:
//! - Phase 4: Real-time event streaming
//! - Phase 5: AI suggestions
//! - Phase 6: Error handling and polish

use biomeos_ui::*;
use anyhow::Result;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_realtime_event_subscriber_lifecycle() -> Result<()> {
    // Create subscriber
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    
    // Discover endpoints
    subscriber.discover_endpoints().await?;
    
    // Subscribe to events
    let mut rx = subscriber.subscribe();
    
    // Test that we can receive events
    // (In real environment, events would come from WebSocket)
    
    Ok(())
}

#[tokio::test]
async fn test_ai_suggestion_manager_local_fallback() -> Result<()> {
    // Create manager
    let mut manager = AISuggestionManager::new("test_family".to_string());
    
    // Create context with unassigned device
    let context = SuggestionContext {
        assignments: std::collections::HashMap::new(),
        available_devices: vec![
            suggestions::DeviceInfo {
                id: "device1".to_string(),
                device_type: "gpu".to_string(),
                capabilities: vec!["compute".to_string()],
                current_assignment: None,
            },
        ],
        running_primals: vec![
            suggestions::PrimalInfo {
                id: "toadstool1".to_string(),
                name: "ToadStool".to_string(),
                primal_type: "compute".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                load: Some(0.5),
            },
        ],
        recent_events: None,
        preferences: None,
    };
    
    // Request suggestions (should use local fallback)
    let suggestions = manager.request_suggestions(context).await?;
    
    // Should have at least one suggestion
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0].suggestion_type, suggestions::SuggestionType::DeviceAssignment);
    
    Ok(())
}

#[tokio::test]
async fn test_ai_suggestion_feedback_cycle() -> Result<()> {
    // Create manager
    let mut manager = AISuggestionManager::new("test_family".to_string());
    
    // Create a simple context
    let context = SuggestionContext {
        assignments: std::collections::HashMap::new(),
        available_devices: vec![],
        running_primals: vec![
            suggestions::PrimalInfo {
                id: "toadstool1".to_string(),
                name: "ToadStool".to_string(),
                primal_type: "compute".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                load: Some(0.9), // Overloaded
            },
        ],
        recent_events: None,
        preferences: None,
    };
    
    // Request suggestions
    let suggestions = manager.request_suggestions(context).await?;
    assert!(!suggestions.is_empty());
    
    let suggestion_id = &suggestions[0].id;
    
    // Send accepted feedback
    manager.send_feedback(suggestion_id, SuggestionFeedback::Accepted).await?;
    
    // Should be removed from active suggestions
    assert!(!manager.get_active_suggestions().iter().any(|s| &s.id == suggestion_id));
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_event_streaming() -> Result<()> {
    use std::sync::Arc;
    
    // Create subscriber
    let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));
    
    // Create multiple receivers
    let mut rx1 = subscriber.subscribe();
    let mut rx2 = subscriber.subscribe();
    let mut rx3 = subscriber.subscribe();
    
    // Send test event
    let event = RealTimeEvent::Heartbeat {
        timestamp: 12345,
        primals_count: 5,
        healthy_count: 5,
    };
    
    let _ = subscriber.event_tx.send(event);
    
    // All receivers should get the event
    assert!(rx1.try_recv().is_ok());
    assert!(rx2.try_recv().is_ok());
    assert!(rx3.try_recv().is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_suggestion_serialization() -> Result<()> {
    // Create a suggestion
    let suggestion = AISuggestion {
        id: "test_suggestion".to_string(),
        suggestion_type: suggestions::SuggestionType::DeviceAssignment,
        confidence: 0.85,
        explanation: "Test explanation".to_string(),
        action: suggestions::SuggestedAction::AssignDevice {
            device_id: "device1".to_string(),
            primal_id: "primal1".to_string(),
            reason: "Test reason".to_string(),
        },
        impact: suggestions::Impact {
            performance_improvement: Some(15.0),
            cost_change: None,
            affected_primals: vec!["primal1".to_string()],
            risk_level: "low".to_string(),
        },
    };
    
    // Serialize to JSON
    let json = serde_json::to_string(&suggestion)?;
    
    // Deserialize back
    let deserialized: AISuggestion = serde_json::from_str(&json)?;
    
    // Verify
    assert_eq!(deserialized.id, "test_suggestion");
    assert_eq!(deserialized.confidence, 0.85);
    
    Ok(())
}

#[tokio::test]
async fn test_event_filtering() -> Result<()> {
    use std::sync::Arc;
    
    let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));
    let mut rx = subscriber.subscribe();
    
    // Send various events
    let events = vec![
        RealTimeEvent::Heartbeat { timestamp: 1, primals_count: 5, healthy_count: 5 },
        RealTimeEvent::PrimalDiscovered {
            primal_id: "primal1".to_string(),
            name: "Primal1".to_string(),
            primal_type: "test".to_string(),
            capabilities: vec![],
        },
        RealTimeEvent::DeviceAdded {
            device_id: "device1".to_string(),
            device_type: "gpu".to_string(),
            capabilities: vec![],
        },
    ];
    
    for event in events {
        let _ = subscriber.event_tx.send(event);
    }
    
    // Receive all events
    let mut received_count = 0;
    while rx.try_recv().is_ok() {
        received_count += 1;
    }
    
    assert_eq!(received_count, 3);
    
    Ok(())
}

#[tokio::test]
async fn test_graceful_degradation_no_squirrel() -> Result<()> {
    // Create manager without Squirrel connection
    let mut manager = AISuggestionManager::new("test_family".to_string());
    
    // Don't call discover_squirrel()
    
    // Create context
    let context = SuggestionContext {
        assignments: std::collections::HashMap::new(),
        available_devices: vec![
            suggestions::DeviceInfo {
                id: "device1".to_string(),
                device_type: "gpu".to_string(),
                capabilities: vec!["compute".to_string()],
                current_assignment: None,
            },
        ],
        running_primals: vec![
            suggestions::PrimalInfo {
                id: "toadstool1".to_string(),
                name: "ToadStool".to_string(),
                primal_type: "compute".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                load: Some(0.5),
            },
        ],
        recent_events: None,
        preferences: None,
    };
    
    // Should still work with local fallback
    let suggestions = manager.request_suggestions(context).await?;
    assert!(!suggestions.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_suggestion_types() -> Result<()> {
    let mut manager = AISuggestionManager::new("test_family".to_string());
    
    // Create context with multiple issues
    let context = SuggestionContext {
        assignments: std::collections::HashMap::new(),
        available_devices: vec![
            suggestions::DeviceInfo {
                id: "device1".to_string(),
                device_type: "gpu".to_string(),
                capabilities: vec!["compute".to_string()],
                current_assignment: None,
            },
        ],
        running_primals: vec![
            suggestions::PrimalInfo {
                id: "toadstool1".to_string(),
                name: "ToadStool".to_string(),
                primal_type: "compute".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                load: Some(0.9), // Overloaded
            },
        ],
        recent_events: None,
        preferences: None,
    };
    
    // Should get both assignment and reallocation suggestions
    let suggestions = manager.request_suggestions(context).await?;
    assert_eq!(suggestions.len(), 2);
    
    // Check types
    let types: Vec<_> = suggestions.iter().map(|s| &s.suggestion_type).collect();
    assert!(types.contains(&&suggestions::SuggestionType::DeviceAssignment));
    assert!(types.contains(&&suggestions::SuggestionType::ResourceReallocation));
    
    Ok(())
}

#[tokio::test]
async fn test_event_handler_processing() -> Result<()> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));
    let mut handler = RealTimeEventHandler::new(subscriber.clone());
    
    let event_count = Arc::new(AtomicUsize::new(0));
    let event_count_clone = event_count.clone();
    
    // Start processing in background
    let process_task = tokio::spawn(async move {
        let _ = timeout(Duration::from_millis(100), handler.process_events(move |_event| {
            event_count_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })).await;
    });
    
    // Send some events
    for i in 0..5 {
        let event = RealTimeEvent::Heartbeat {
            timestamp: i,
            primals_count: 5,
            healthy_count: 5,
        };
        let _ = subscriber.event_tx.send(event);
    }
    
    // Wait a bit for processing
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Should have processed events
    assert!(event_count.load(Ordering::SeqCst) > 0);
    
    // Clean up
    process_task.abort();
    
    Ok(())
}

