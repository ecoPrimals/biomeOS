//! State management for biomeOS UI

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::api::{PrimalAdapter, EcosystemStatus, BiomeDeployment};

#[derive(Debug, Clone)]
pub struct UIState {
    /// Connected Primal adapters
    pub connected_primals: HashMap<String, PrimalAdapter>,
    
    /// Current ecosystem status
    pub ecosystem_status: Option<EcosystemStatus>,
    
    /// Active biome deployments
    pub active_deployments: HashMap<String, BiomeDeployment>,
    
    /// UI-specific state
    pub ui_state: UIComponentState,
    
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            connected_primals: HashMap::new(),
            ecosystem_status: None,
            active_deployments: HashMap::new(),
            ui_state: UIComponentState::default(),
            last_updated: Utc::now(),
        }
    }
    
    pub fn add_primal(&mut self, name: String, adapter: PrimalAdapter) {
        self.connected_primals.insert(name, adapter);
        self.last_updated = Utc::now();
    }
    
    pub fn connected_primals_count(&self) -> usize {
        self.connected_primals.len()
    }
    
    pub fn get_connected_primals(&self) -> Vec<String> {
        self.connected_primals.keys().cloned().collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct UIComponentState {
    pub current_view: String,
    pub selected_primal: Option<String>,
    pub selected_deployment: Option<String>,
    pub show_ai_assistant: bool,
    pub show_logs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalState {
    pub name: String,
    pub status: String,
    pub last_seen: DateTime<Utc>,
    pub service_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeState {
    pub name: String,
    pub status: String,
    pub deployment_id: String,
    pub created_at: DateTime<Utc>,
    pub services: Vec<ServiceState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    pub name: String,
    pub status: String,
    pub primal: String,
    pub endpoint: Option<String>,
} 