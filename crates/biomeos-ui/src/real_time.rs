//! Real-time event streaming for biomeOS UI

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{info, warn, error};

#[derive(Debug)]
pub struct EventStream {
    /// WebSocket endpoints for each Primal
    endpoints: HashMap<String, String>,
    
    /// Event sender channel
    event_sender: Option<mpsc::UnboundedSender<EcosystemEvent>>,
    
    /// Event receiver channel
    event_receiver: Option<mpsc::UnboundedReceiver<EcosystemEvent>>,
}

impl EventStream {
    pub async fn new(endpoints: &HashMap<String, String>) -> Result<Self> {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            endpoints: endpoints.clone(),
            event_sender: Some(sender),
            event_receiver: Some(receiver),
        })
    }
    
    pub async fn start_monitoring(&mut self, primals: &[String]) -> Result<()> {
        info!("Starting real-time event monitoring for {} Primals", primals.len());
        
        if let Some(sender) = self.event_sender.take() {
            for primal_name in primals {
                if let Some(endpoint) = self.endpoints.get(primal_name) {
                    let primal_name = primal_name.clone();
                    let endpoint = endpoint.clone();
                    let sender = sender.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::monitor_primal_events(primal_name, endpoint, sender).await {
                            error!("Event monitoring failed: {}", e);
                        }
                    });
                }
            }
        }
        
        Ok(())
    }
    
    async fn monitor_primal_events(
        primal_name: String,
        endpoint: String,
        sender: mpsc::UnboundedSender<EcosystemEvent>
    ) -> Result<()> {
        info!("Connecting to event stream: {} -> {}", primal_name, endpoint);
        
        let (ws_stream, _) = connect_async(&endpoint).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // Send subscription message
        let subscription = serde_json::json!({
            "type": "subscribe",
            "events": ["deployment", "service", "health", "error"]
        });
        
        ws_sender.send(Message::Text(subscription.to_string())).await?;
        
        while let Some(msg) = ws_receiver.next().await {
            match msg? {
                Message::Text(text) => {
                    if let Ok(event) = serde_json::from_str::<PrimalEvent>(&text) {
                        let ecosystem_event = EcosystemEvent {
                            source_primal: primal_name.clone(),
                            event_type: event.event_type,
                            data: event.data,
                            timestamp: event.timestamp,
                        };
                        
                        if sender.send(ecosystem_event).is_err() {
                            warn!("Event receiver dropped, stopping monitoring for {}", primal_name);
                            break;
                        }
                    }
                }
                Message::Close(_) => {
                    info!("WebSocket closed for {}", primal_name);
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    pub async fn get_next_event(&mut self) -> Option<EcosystemEvent> {
        if let Some(receiver) = &mut self.event_receiver {
            receiver.recv().await
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemEvent {
    pub source_primal: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
} 