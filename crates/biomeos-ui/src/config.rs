//! Configuration management for biomeOS UI

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIMode {
    Desktop,
    Terminal,
    Web,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub ui_mode: UIMode,
    pub api_endpoints: HashMap<String, String>,
    pub websocket_endpoints: HashMap<String, String>,
    pub ai_config: crate::ai::AIConfig,
    pub theme: Theme,
    pub auto_refresh_interval: u64,
} 