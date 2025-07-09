//! # biomeOS Universal UI
//!
//! AI-first, API-driven user interface for the entire biomeOS ecosystem.
//! This UI works seamlessly across all Primals and provides a unified
//! experience for managing biomes, services, and ecosystem coordination.

pub mod api;
pub mod components;
pub mod state;
pub mod ai;
pub mod primal_adapters;
pub mod real_time;
pub mod config;
pub mod universal_ui;

// Re-export main types
pub use api::{BiomeOSApiClient, UniversalApiClient};
pub use state::{UIState, PrimalState, BiomeState};
pub use ai::{AIAssistant, UICommand, AIResponse};
pub use primal_adapters::{PrimalAdapter, UniversalPrimalInterface};
pub use universal_ui::{
    UniversalUIManager, UniversalUIConfig, UIMode, DiscoveredPrimal, 
    PrimalUIConfig, UIFeatures, SystemStatus
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Main UI application structure
#[derive(Debug)]
pub struct BiomeOSUI {
    /// API client for ecosystem communication
    pub api_client: UniversalApiClient,
    
    /// Current UI state
    pub state: UIState,
    
    /// AI assistant for natural language interaction
    pub ai_assistant: AIAssistant,
    
    /// Real-time event stream
    pub event_stream: real_time::EventStream,
    
    /// Configuration
    pub config: config::UIConfig,
}

impl BiomeOSUI {
    /// Create a new biomeOS UI instance
    pub async fn new(config: config::UIConfig) -> Result<Self> {
        let api_client = UniversalApiClient::new(&config.api_endpoints).await?;
        let state = UIState::new();
        let ai_assistant = AIAssistant::new(&config.ai_config).await?;
        let event_stream = real_time::EventStream::new(&config.websocket_endpoints).await?;
        
        Ok(Self {
            api_client,
            state,
            ai_assistant,
            event_stream,
            config,
        })
    }
    
    /// Start the UI application
    pub async fn run(&mut self) -> Result<()> {
        // Initialize connections to all available Primals
        self.discover_and_connect_primals().await?;
        
        // Start real-time event monitoring
        self.start_event_monitoring().await?;
        
        // Launch the appropriate UI mode
        match self.config.ui_mode {
            config::UIMode::Desktop => self.run_desktop_ui().await,
            config::UIMode::Terminal => self.run_terminal_ui().await,
            config::UIMode::Web => self.run_web_ui().await,
        }
    }
    
    /// Discover and connect to all available Primals
    async fn discover_and_connect_primals(&mut self) -> Result<()> {
        tracing::info!("Discovering available Primals in the ecosystem...");
        
        let discovered_primals = self.api_client.discover_primals().await?;
        
        for primal_info in discovered_primals {
            tracing::info!("Connecting to {}: {}", primal_info.name, primal_info.endpoint);
            
            match self.api_client.connect_to_primal(&primal_info).await {
                Ok(adapter) => {
                    self.state.add_primal(primal_info.name.clone(), adapter);
                    tracing::info!("✅ Connected to {}", primal_info.name);
                }
                Err(e) => {
                    tracing::warn!("⚠️  Failed to connect to {}: {}", primal_info.name, e);
                    // Continue with other Primals - graceful degradation
                }
            }
        }
        
        tracing::info!("Connected to {} Primals", self.state.connected_primals_count());
        Ok(())
    }
    
    /// Start monitoring real-time events from all Primals
    async fn start_event_monitoring(&mut self) -> Result<()> {
        self.event_stream.start_monitoring(&self.state.get_connected_primals()).await
    }
    
    /// Run desktop UI using Tauri
    #[cfg(feature = "desktop-ui")]
    async fn run_desktop_ui(&mut self) -> Result<()> {
        use tauri::{Builder, generate_context};
        
        let context = generate_context!();
        
        Builder::default()
            .setup(|app| {
                // Initialize desktop UI
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                // Desktop UI command handlers
                get_ecosystem_status,
                deploy_biome,
                get_primal_status,
                execute_ai_command,
                get_real_time_events
            ])
            .run(context)
            .expect("error while running tauri application");
            
        Ok(())
    }
    
    /// Run terminal UI using ratatui
    #[cfg(feature = "terminal-ui")]
    async fn run_terminal_ui(&mut self) -> Result<()> {
        use ratatui::{
            prelude::*,
            widgets::*,
        };
        use crossterm::{
            event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
            execute,
            terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        };
        
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Main terminal UI loop
        loop {
            terminal.draw(|f| {
                self.render_terminal_ui(f);
            })?;
            
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('h') => self.show_help(),
                    KeyCode::Char('s') => self.show_status().await?,
                    KeyCode::Char('d') => self.deploy_biome_interactive().await?,
                    KeyCode::Char('a') => self.ai_assistant_mode().await?,
                    _ => {}
                }
            }
        }
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        Ok(())
    }
    
    /// Run web UI using wry
    #[cfg(feature = "web-ui")]
    async fn run_web_ui(&mut self) -> Result<()> {
        // Web UI implementation using wry
        Ok(())
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub ui_mode: config::UIMode,
    pub api_endpoints: HashMap<String, String>,
    pub websocket_endpoints: HashMap<String, String>,
    pub ai_config: ai::AIConfig,
    pub theme: config::Theme,
    pub auto_refresh_interval: u64,
}

impl Default for UIConfig {
    fn default() -> Self {
        let mut api_endpoints = HashMap::new();
        api_endpoints.insert("songbird".to_string(), "http://localhost:8080".to_string());
        api_endpoints.insert("nestgate".to_string(), "http://localhost:8082".to_string());
        api_endpoints.insert("toadstool".to_string(), "http://localhost:8084".to_string());
        api_endpoints.insert("beardog".to_string(), "http://localhost:9000".to_string());
        api_endpoints.insert("squirrel".to_string(), "http://localhost:5000".to_string());
        
        let mut websocket_endpoints = HashMap::new();
        websocket_endpoints.insert("songbird".to_string(), "ws://localhost:8080/ws".to_string());
        websocket_endpoints.insert("events".to_string(), "ws://localhost:8080/events".to_string());
        
        Self {
            ui_mode: config::UIMode::Desktop,
            api_endpoints,
            websocket_endpoints,
            ai_config: ai::AIConfig::default(),
            theme: config::Theme::Dark,
            auto_refresh_interval: 5000, // 5 seconds
        }
    }
}

// Desktop UI command handlers for Tauri
#[cfg(feature = "desktop-ui")]
mod desktop_commands {
    use super::*;
    use tauri::command;
    
    #[command]
    pub async fn get_ecosystem_status() -> Result<serde_json::Value, String> {
        // Implementation for getting ecosystem status
        Ok(serde_json::json!({
            "status": "healthy",
            "primals": []
        }))
    }
    
    #[command]
    pub async fn deploy_biome(manifest: String) -> Result<String, String> {
        // Implementation for deploying a biome
        Ok("deployment-id".to_string())
    }
    
    #[command]
    pub async fn get_primal_status(primal_name: String) -> Result<serde_json::Value, String> {
        // Implementation for getting Primal status
        Ok(serde_json::json!({
            "name": primal_name,
            "status": "healthy"
        }))
    }
    
    #[command]
    pub async fn execute_ai_command(command: String) -> Result<String, String> {
        // Implementation for AI command execution
        Ok("AI response".to_string())
    }
    
    #[command]
    pub async fn get_real_time_events() -> Result<Vec<serde_json::Value>, String> {
        // Implementation for getting real-time events
        Ok(vec![])
    }
}

#[cfg(feature = "desktop-ui")]
pub use desktop_commands::*;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information
pub const BUILD_INFO: &str = concat!(
    "biomeOS UI v", env!("CARGO_PKG_VERSION"),
    " - Universal AI-first interface"
); 