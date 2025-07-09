//! Universal UI System for biomeOS
//!
//! This module provides a configurable, API-driven UI system that can work with any primal
//! by discovering capabilities and adapting the interface accordingly.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::primal_adapters::{PrimalAdapter, UniversalPrimalInterface};

/// Universal UI configuration that adapts to available primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalUIConfig {
    /// UI mode (desktop, web, terminal, cli)
    pub ui_mode: UIMode,
    
    /// Theme configuration
    pub theme: UITheme,
    
    /// Auto-discovery settings
    pub auto_discovery: AutoDiscoveryConfig,
    
    /// Known primal endpoints
    pub primal_endpoints: HashMap<String, String>,
    
    /// Custom primal configurations
    pub custom_primals: HashMap<String, CustomPrimalConfig>,
    
    /// UI feature toggles
    pub features: UIFeatures,
    
    /// Real-time update settings
    pub real_time: RealTimeConfig,
    
    /// AI assistant configuration
    pub ai_config: AIConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIMode {
    Desktop,
    Web,
    Terminal,
    CLI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UITheme {
    pub name: String,
    pub colors: HashMap<String, String>,
    pub fonts: HashMap<String, String>,
    pub layout: LayoutConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub sidebar_width: f32,
    pub header_height: f32,
    pub panel_spacing: f32,
    pub responsive_breakpoints: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDiscoveryConfig {
    pub enabled: bool,
    pub discovery_interval_secs: u64,
    pub health_check_interval_secs: u64,
    pub discovery_ports: Vec<u16>,
    pub discovery_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPrimalConfig {
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub description: String,
    pub ui_config: PrimalUIConfig,
    pub auth_config: Option<AuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalUIConfig {
    pub display_name: String,
    pub icon: String,
    pub color: String,
    pub dashboard_widgets: Vec<WidgetConfig>,
    pub custom_actions: Vec<ActionConfig>,
    pub metrics_config: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub widget_type: String,
    pub title: String,
    pub api_endpoint: String,
    pub refresh_interval_secs: u64,
    pub display_config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfig {
    pub action_id: String,
    pub display_name: String,
    pub api_endpoint: String,
    pub method: String,
    pub parameters: Vec<ParameterConfig>,
    pub confirmation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConfig {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub validation: Option<ValidationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub allowed_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub metrics_endpoint: String,
    pub chart_types: Vec<String>,
    pub default_time_range: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: String,
    pub endpoint: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIFeatures {
    pub ai_assistant: bool,
    pub real_time_monitoring: bool,
    pub deployment_wizard: bool,
    pub service_management: bool,
    pub log_viewer: bool,
    pub metrics_dashboard: bool,
    pub custom_dashboards: bool,
    pub multi_primal_coordination: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeConfig {
    pub enabled: bool,
    pub websocket_endpoints: HashMap<String, String>,
    pub event_types: Vec<String>,
    pub buffer_size: usize,
    pub reconnect_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub enabled: bool,
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub context_window: usize,
    pub temperature: f32,
    pub custom_prompts: HashMap<String, String>,
}

/// Discovered primal information with dynamic capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    pub name: String,
    pub endpoint: String,
    pub api_version: String,
    pub capabilities: Vec<String>,
    pub health_status: String,
    pub ui_config: Option<PrimalUIConfig>,
    pub custom_endpoints: HashMap<String, String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

/// Universal UI state that adapts to available primals
#[derive(Debug)]
pub struct UniversalUIState {
    pub config: UniversalUIConfig,
    pub discovered_primals: Arc<RwLock<HashMap<String, DiscoveredPrimal>>>,
    pub active_connections: Arc<RwLock<HashMap<String, PrimalAdapter>>>,
    pub ui_components: Arc<RwLock<HashMap<String, UIComponent>>>,
    pub real_time_events: Arc<RwLock<Vec<RealTimeEvent>>>,
    pub ai_context: Arc<RwLock<AIContext>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIComponent {
    pub component_id: String,
    pub component_type: String,
    pub primal_name: String,
    pub config: serde_json::Value,
    pub state: serde_json::Value,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeEvent {
    pub event_id: Uuid,
    pub primal_name: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContext {
    pub conversation_history: Vec<AIMessage>,
    pub primal_contexts: HashMap<String, serde_json::Value>,
    pub user_preferences: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMessage {
    pub message_id: Uuid,
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: HashMap<String, serde_json::Value>,
}

/// Universal UI manager that orchestrates all primal interactions
pub struct UniversalUIManager {
    state: UniversalUIState,
    discovery_engine: PrimalDiscoveryEngine,
    ui_renderer: Box<dyn UIRenderer>,
    event_processor: EventProcessor,
}

/// Primal discovery engine that finds and configures primals
pub struct PrimalDiscoveryEngine {
    config: AutoDiscoveryConfig,
    client: reqwest::Client,
}

/// Event processor for real-time updates
pub struct EventProcessor {
    event_handlers: HashMap<String, Box<dyn EventHandler>>,
}

/// UI renderer trait for different UI modes
#[async_trait]
pub trait UIRenderer: Send + Sync {
    async fn render_dashboard(&self, primals: &HashMap<String, DiscoveredPrimal>) -> Result<()>;
    async fn render_primal_view(&self, primal: &DiscoveredPrimal) -> Result<()>;
    async fn render_deployment_wizard(&self, available_primals: &[String]) -> Result<()>;
    async fn render_ai_assistant(&self, context: &AIContext) -> Result<()>;
    async fn handle_user_input(&self, input: UserInput) -> Result<UIResponse>;
}

/// Event handler trait for processing real-time events
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_event(&self, event: &RealTimeEvent) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInput {
    pub input_type: String,
    pub data: serde_json::Value,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIResponse {
    pub response_type: String,
    pub data: serde_json::Value,
    pub actions: Vec<UIAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIAction {
    pub action_type: String,
    pub target: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

impl UniversalUIManager {
    pub async fn new(config: UniversalUIConfig) -> Result<Self> {
        let state = UniversalUIState {
            config: config.clone(),
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            ui_components: Arc::new(RwLock::new(HashMap::new())),
            real_time_events: Arc::new(RwLock::new(Vec::new())),
            ai_context: Arc::new(RwLock::new(AIContext {
                conversation_history: Vec::new(),
                primal_contexts: HashMap::new(),
                user_preferences: HashMap::new(),
            })),
        };

        let discovery_engine = PrimalDiscoveryEngine::new(config.auto_discovery.clone());
        let ui_renderer = Self::create_ui_renderer(&config.ui_mode)?;
        let event_processor = EventProcessor::new();

        Ok(Self {
            state,
            discovery_engine,
            ui_renderer,
            event_processor,
        })
    }

    fn create_ui_renderer(ui_mode: &UIMode) -> Result<Box<dyn UIRenderer>> {
        match ui_mode {
            UIMode::Desktop => Ok(Box::new(DesktopUIRenderer::new())),
            UIMode::Web => Ok(Box::new(WebUIRenderer::new())),
            UIMode::Terminal => Ok(Box::new(TerminalUIRenderer::new())),
            UIMode::CLI => Ok(Box::new(CLIRenderer::new())),
        }
    }

    /// Start the universal UI system
    pub async fn start(&mut self) -> Result<()> {
        // Discover available primals
        self.discover_primals().await?;
        
        // Initialize UI components based on discovered primals
        self.initialize_ui_components().await?;
        
        // Start real-time event processing
        self.start_event_processing().await?;
        
        // Start the UI renderer
        self.start_ui_renderer().await?;
        
        Ok(())
    }

    /// Discover available primals using the discovery engine
    async fn discover_primals(&mut self) -> Result<()> {
        let discovered = self.discovery_engine.discover_primals(&self.state.config.primal_endpoints).await?;
        
        let mut primals = self.state.discovered_primals.write().await;
        for primal in discovered {
            primals.insert(primal.name.clone(), primal);
        }
        
        Ok(())
    }

    /// Initialize UI components based on discovered primals
    async fn initialize_ui_components(&self) -> Result<()> {
        let primals = self.state.discovered_primals.read().await;
        let mut components = self.state.ui_components.write().await;
        
        for (name, primal) in primals.iter() {
            // Create dashboard widgets for each primal
            if let Some(ui_config) = &primal.ui_config {
                for widget in &ui_config.dashboard_widgets {
                    let component = UIComponent {
                        component_id: format!("{}-{}", name, widget.widget_type),
                        component_type: widget.widget_type.clone(),
                        primal_name: name.clone(),
                        config: serde_json::to_value(widget)?,
                        state: serde_json::Value::Object(serde_json::Map::new()),
                        last_updated: chrono::Utc::now(),
                    };
                    components.insert(component.component_id.clone(), component);
                }
            }
        }
        
        Ok(())
    }

    /// Start real-time event processing
    async fn start_event_processing(&self) -> Result<()> {
        // Implementation would set up WebSocket connections and event handlers
        Ok(())
    }

    /// Start the UI renderer
    async fn start_ui_renderer(&self) -> Result<()> {
        let primals = self.state.discovered_primals.read().await;
        self.ui_renderer.render_dashboard(&primals).await?;
        Ok(())
    }

    /// Handle user interactions
    pub async fn handle_user_input(&self, input: UserInput) -> Result<UIResponse> {
        self.ui_renderer.handle_user_input(input).await
    }

    /// Get current system status
    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        let primals = self.state.discovered_primals.read().await;
        let total_primals = primals.len();
        let healthy_primals = primals.values()
            .filter(|p| p.health_status == "healthy")
            .count();

        Ok(SystemStatus {
            total_primals,
            healthy_primals,
            ui_mode: self.state.config.ui_mode.clone(),
            last_discovery: primals.values()
                .map(|p| p.discovered_at)
                .max(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub total_primals: usize,
    pub healthy_primals: usize,
    pub ui_mode: UIMode,
    pub last_discovery: Option<chrono::DateTime<chrono::Utc>>,
}

impl PrimalDiscoveryEngine {
    pub fn new(config: AutoDiscoveryConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn discover_primals(&self, known_endpoints: &HashMap<String, String>) -> Result<Vec<DiscoveredPrimal>> {
        let mut discovered = Vec::new();

        // Check known endpoints
        for (name, endpoint) in known_endpoints {
            if let Ok(primal) = self.probe_primal(name, endpoint).await {
                discovered.push(primal);
            }
        }

        // Auto-discovery if enabled
        if self.config.enabled {
            let auto_discovered = self.auto_discover_primals().await?;
            discovered.extend(auto_discovered);
        }

        Ok(discovered)
    }

    async fn probe_primal(&self, name: &str, endpoint: &str) -> Result<DiscoveredPrimal> {
        let health_url = format!("{}/health", endpoint);
        let response = self.client.get(&health_url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Health check failed"));
        }

        let health_data: serde_json::Value = response.json().await?;
        
        // Try to get capabilities
        let capabilities = self.get_primal_capabilities(endpoint).await.unwrap_or_default();
        
        // Try to get UI configuration
        let ui_config = self.get_primal_ui_config(endpoint).await.ok();

        Ok(DiscoveredPrimal {
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            api_version: health_data.get("api_version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            capabilities,
            health_status: health_data.get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown")
                .to_string(),
            ui_config,
            custom_endpoints: HashMap::new(),
            metadata: HashMap::new(),
            discovered_at: chrono::Utc::now(),
            last_health_check: chrono::Utc::now(),
        })
    }

    async fn auto_discover_primals(&self) -> Result<Vec<DiscoveredPrimal>> {
        // Implementation would scan network for primals
        Ok(Vec::new())
    }

    async fn get_primal_capabilities(&self, endpoint: &str) -> Result<Vec<String>> {
        let capabilities_url = format!("{}/api/v1/capabilities", endpoint);
        let response = self.client.get(&capabilities_url).send().await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(caps) = data.get("capabilities").and_then(|c| c.as_array()) {
                return Ok(caps.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect());
            }
        }
        
        Ok(Vec::new())
    }

    async fn get_primal_ui_config(&self, endpoint: &str) -> Result<PrimalUIConfig> {
        let ui_config_url = format!("{}/api/v1/ui/config", endpoint);
        let response = self.client.get(&ui_config_url).send().await?;
        
        if response.status().is_success() {
            let config: PrimalUIConfig = response.json().await?;
            Ok(config)
        } else {
            Err(anyhow::anyhow!("UI config not available"))
        }
    }
}

impl EventProcessor {
    pub fn new() -> Self {
        Self {
            event_handlers: HashMap::new(),
        }
    }

    pub fn register_handler(&mut self, event_type: String, handler: Box<dyn EventHandler>) {
        self.event_handlers.insert(event_type, handler);
    }

    pub async fn process_event(&self, event: &RealTimeEvent) -> Result<()> {
        if let Some(handler) = self.event_handlers.get(&event.event_type) {
            handler.handle_event(event).await?;
        }
        Ok(())
    }
}

// UI Renderer implementations
pub struct DesktopUIRenderer;
pub struct WebUIRenderer;
pub struct TerminalUIRenderer;
pub struct CLIRenderer;

#[async_trait]
impl UIRenderer for DesktopUIRenderer {
    async fn render_dashboard(&self, primals: &HashMap<String, DiscoveredPrimal>) -> Result<()> {
        // Desktop-specific dashboard rendering
        Ok(())
    }

    async fn render_primal_view(&self, primal: &DiscoveredPrimal) -> Result<()> {
        // Desktop-specific primal view rendering
        Ok(())
    }

    async fn render_deployment_wizard(&self, available_primals: &[String]) -> Result<()> {
        // Desktop-specific deployment wizard
        Ok(())
    }

    async fn render_ai_assistant(&self, context: &AIContext) -> Result<()> {
        // Desktop-specific AI assistant
        Ok(())
    }

    async fn handle_user_input(&self, input: UserInput) -> Result<UIResponse> {
        // Desktop-specific input handling
        Ok(UIResponse {
            response_type: "success".to_string(),
            data: serde_json::Value::Null,
            actions: Vec::new(),
        })
    }
}

impl DesktopUIRenderer {
    pub fn new() -> Self {
        Self
    }
}

// Similar implementations for WebUIRenderer, TerminalUIRenderer, CLIRenderer...
#[async_trait]
impl UIRenderer for WebUIRenderer {
    async fn render_dashboard(&self, _primals: &HashMap<String, DiscoveredPrimal>) -> Result<()> {
        Ok(())
    }
    async fn render_primal_view(&self, _primal: &DiscoveredPrimal) -> Result<()> {
        Ok(())
    }
    async fn render_deployment_wizard(&self, _available_primals: &[String]) -> Result<()> {
        Ok(())
    }
    async fn render_ai_assistant(&self, _context: &AIContext) -> Result<()> {
        Ok(())
    }
    async fn handle_user_input(&self, _input: UserInput) -> Result<UIResponse> {
        Ok(UIResponse {
            response_type: "success".to_string(),
            data: serde_json::Value::Null,
            actions: Vec::new(),
        })
    }
}

impl WebUIRenderer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UIRenderer for TerminalUIRenderer {
    async fn render_dashboard(&self, _primals: &HashMap<String, DiscoveredPrimal>) -> Result<()> {
        Ok(())
    }
    async fn render_primal_view(&self, _primal: &DiscoveredPrimal) -> Result<()> {
        Ok(())
    }
    async fn render_deployment_wizard(&self, _available_primals: &[String]) -> Result<()> {
        Ok(())
    }
    async fn render_ai_assistant(&self, _context: &AIContext) -> Result<()> {
        Ok(())
    }
    async fn handle_user_input(&self, _input: UserInput) -> Result<UIResponse> {
        Ok(UIResponse {
            response_type: "success".to_string(),
            data: serde_json::Value::Null,
            actions: Vec::new(),
        })
    }
}

impl TerminalUIRenderer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UIRenderer for CLIRenderer {
    async fn render_dashboard(&self, _primals: &HashMap<String, DiscoveredPrimal>) -> Result<()> {
        Ok(())
    }
    async fn render_primal_view(&self, _primal: &DiscoveredPrimal) -> Result<()> {
        Ok(())
    }
    async fn render_deployment_wizard(&self, _available_primals: &[String]) -> Result<()> {
        Ok(())
    }
    async fn render_ai_assistant(&self, _context: &AIContext) -> Result<()> {
        Ok(())
    }
    async fn handle_user_input(&self, _input: UserInput) -> Result<UIResponse> {
        Ok(UIResponse {
            response_type: "success".to_string(),
            data: serde_json::Value::Null,
            actions: Vec::new(),
        })
    }
}

impl CLIRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UniversalUIConfig {
    fn default() -> Self {
        Self {
            ui_mode: UIMode::Desktop,
            theme: UITheme {
                name: "biomeOS".to_string(),
                colors: HashMap::new(),
                fonts: HashMap::new(),
                layout: LayoutConfig {
                    sidebar_width: 250.0,
                    header_height: 60.0,
                    panel_spacing: 10.0,
                    responsive_breakpoints: HashMap::new(),
                },
            },
            auto_discovery: AutoDiscoveryConfig {
                enabled: true,
                discovery_interval_secs: 30,
                health_check_interval_secs: 10,
                discovery_ports: vec![8080, 8081, 8082, 8083, 8084, 9000],
                discovery_paths: vec!["/health".to_string(), "/api/v1/health".to_string()],
            },
            primal_endpoints: HashMap::new(),
            custom_primals: HashMap::new(),
            features: UIFeatures {
                ai_assistant: true,
                real_time_monitoring: true,
                deployment_wizard: true,
                service_management: true,
                log_viewer: true,
                metrics_dashboard: true,
                custom_dashboards: true,
                multi_primal_coordination: true,
            },
            real_time: RealTimeConfig {
                enabled: true,
                websocket_endpoints: HashMap::new(),
                event_types: vec![
                    "service_started".to_string(),
                    "service_stopped".to_string(),
                    "health_changed".to_string(),
                    "deployment_completed".to_string(),
                ],
                buffer_size: 1000,
                reconnect_interval_secs: 5,
            },
            ai_config: AIConfig {
                enabled: true,
                provider: "local".to_string(),
                model: "biomeOS-assistant".to_string(),
                api_key: None,
                context_window: 4096,
                temperature: 0.7,
                custom_prompts: HashMap::new(),
            },
        }
    }
} 