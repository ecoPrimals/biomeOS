//! AI Assistant for biomeOS UI
//!
//! Provides natural language interaction with the biomeOS ecosystem,
//! allowing users to manage biomes, deploy services, and monitor the
//! ecosystem using conversational AI.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tracing::{info, warn};

use crate::api::{UniversalApiClient, BiomeManifest, AICommand};

/// AI Assistant for biomeOS ecosystem management
#[derive(Debug)]
pub struct AIAssistant {
    /// AI service configuration
    config: AIConfig,
    
    /// Conversation history
    conversation_history: Vec<ConversationEntry>,
    
    /// Current ecosystem context
    ecosystem_context: EcosystemContext,
    
    /// Command templates for common operations
    command_templates: HashMap<String, CommandTemplate>,
}

impl AIAssistant {
    /// Create a new AI assistant
    pub async fn new(config: &AIConfig) -> Result<Self> {
        let command_templates = Self::initialize_command_templates();
        
        Ok(Self {
            config: config.clone(),
            conversation_history: Vec::new(),
            ecosystem_context: EcosystemContext::default(),
            command_templates,
        })
    }
    
    /// Process a natural language command
    pub async fn process_command(
        &mut self, 
        user_input: &str, 
        api_client: &UniversalApiClient
    ) -> Result<AIResponse> {
        info!("Processing AI command: {}", user_input);
        
        // Add user input to conversation history
        self.add_to_conversation("user", user_input);
        
        // Update ecosystem context
        self.update_ecosystem_context(api_client).await?;
        
        // Parse and understand the command
        let parsed_command = self.parse_natural_language_command(user_input).await?;
        
        // Execute the command
        let execution_result = self.execute_parsed_command(parsed_command, api_client).await?;
        
        // Generate AI response
        let ai_response = self.generate_response(execution_result).await?;
        
        // Add AI response to conversation history
        self.add_to_conversation("assistant", &ai_response.message);
        
        Ok(ai_response)
    }
    
    /// Parse natural language into executable commands
    async fn parse_natural_language_command(&self, input: &str) -> Result<ParsedCommand> {
        let input_lower = input.to_lowercase();
        
        // Intent recognition patterns
        let intent = if input_lower.contains("deploy") || input_lower.contains("create") {
            if input_lower.contains("biome") {
                CommandIntent::DeployBiome
            } else {
                CommandIntent::DeployService
            }
        } else if input_lower.contains("status") || input_lower.contains("health") {
            if input_lower.contains("ecosystem") || input_lower.contains("all") {
                CommandIntent::GetEcosystemStatus
            } else {
                CommandIntent::GetPrimalStatus
            }
        } else if input_lower.contains("list") || input_lower.contains("show") {
            if input_lower.contains("biome") {
                CommandIntent::ListBiomes
            } else if input_lower.contains("service") {
                CommandIntent::ListServices
            } else {
                CommandIntent::ListPrimals
            }
        } else if input_lower.contains("stop") || input_lower.contains("terminate") {
            CommandIntent::StopService
        } else if input_lower.contains("scale") || input_lower.contains("resize") {
            CommandIntent::ScaleService
        } else if input_lower.contains("logs") || input_lower.contains("log") {
            CommandIntent::GetLogs
        } else if input_lower.contains("help") || input_lower.contains("?") {
            CommandIntent::Help
        } else {
            CommandIntent::General
        };
        
        // Extract entities (names, numbers, etc.)
        let entities = self.extract_entities(input);
        
        // Generate parameters based on intent and entities
        let parameters = self.generate_parameters(&intent, &entities, input);
        
        Ok(ParsedCommand {
            intent,
            entities,
            parameters,
            original_input: input.to_string(),
        })
    }
    
    /// Execute a parsed command
    async fn execute_parsed_command(
        &self,
        command: ParsedCommand,
        api_client: &UniversalApiClient
    ) -> Result<CommandExecutionResult> {
        match command.intent {
            CommandIntent::DeployBiome => {
                self.execute_deploy_biome_command(command, api_client).await
            }
            CommandIntent::DeployService => {
                self.execute_deploy_service_command(command, api_client).await
            }
            CommandIntent::GetEcosystemStatus => {
                self.execute_get_ecosystem_status_command(api_client).await
            }
            CommandIntent::GetPrimalStatus => {
                self.execute_get_primal_status_command(command, api_client).await
            }
            CommandIntent::ListBiomes => {
                self.execute_list_biomes_command(api_client).await
            }
            CommandIntent::ListServices => {
                self.execute_list_services_command(command, api_client).await
            }
            CommandIntent::ListPrimals => {
                self.execute_list_primals_command(api_client).await
            }
            CommandIntent::StopService => {
                self.execute_stop_service_command(command, api_client).await
            }
            CommandIntent::ScaleService => {
                self.execute_scale_service_command(command, api_client).await
            }
            CommandIntent::GetLogs => {
                self.execute_get_logs_command(command, api_client).await
            }
            CommandIntent::Help => {
                self.execute_help_command().await
            }
            CommandIntent::General => {
                self.execute_general_command(command, api_client).await
            }
        }
    }
    
    /// Deploy biome command execution
    async fn execute_deploy_biome_command(
        &self,
        command: ParsedCommand,
        api_client: &UniversalApiClient
    ) -> Result<CommandExecutionResult> {
        // Extract biome name and configuration
        let biome_name = command.entities.get("biome_name")
            .cloned()
            .unwrap_or_else(|| format!("biome-{}", chrono::Utc::now().timestamp()));
        
        // Generate or use provided manifest
        let manifest = if let Some(manifest_path) = command.entities.get("manifest_path") {
            // Load manifest from file/path
            self.load_manifest_from_path(manifest_path).await?
        } else {
            // Generate manifest from command parameters
            self.generate_manifest_from_command(&command).await?
        };
        
        // Deploy the biome
        match api_client.deploy_biome(manifest).await {
            Ok(deployment) => {
                Ok(CommandExecutionResult::BiomeDeployment {
                    deployment_id: deployment.deployment_id,
                    status: deployment.status,
                    message: format!("Successfully deployed biome '{}'", biome_name),
                    details: serde_json::to_value(deployment)?,
                })
            }
            Err(e) => {
                Ok(CommandExecutionResult::Error {
                    message: format!("Failed to deploy biome '{}': {}", biome_name, e),
                    error_type: "deployment_failed".to_string(),
                })
            }
        }
    }
    
    /// Get ecosystem status command execution
    async fn execute_get_ecosystem_status_command(
        &self,
        api_client: &UniversalApiClient
    ) -> Result<CommandExecutionResult> {
        match api_client.get_ecosystem_status().await {
            Ok(status) => {
                Ok(CommandExecutionResult::EcosystemStatus {
                    overall_health: status.overall_health,
                    total_primals: status.total_primals,
                    healthy_primals: status.healthy_primals,
                    details: serde_json::to_value(status)?,
                })
            }
            Err(e) => {
                Ok(CommandExecutionResult::Error {
                    message: format!("Failed to get ecosystem status: {}", e),
                    error_type: "status_failed".to_string(),
                })
            }
        }
    }
    
    /// List Primals command execution
    async fn execute_list_primals_command(
        &self,
        api_client: &UniversalApiClient
    ) -> Result<CommandExecutionResult> {
        match api_client.discover_primals().await {
            Ok(primals) => {
                Ok(CommandExecutionResult::PrimalList {
                    primals: primals.into_iter()
                        .map(|p| format!("{} ({})", p.name, p.health))
                        .collect(),
                    count: primals.len(),
                })
            }
            Err(e) => {
                Ok(CommandExecutionResult::Error {
                    message: format!("Failed to list Primals: {}", e),
                    error_type: "list_failed".to_string(),
                })
            }
        }
    }
    
    /// Help command execution
    async fn execute_help_command(&self) -> Result<CommandExecutionResult> {
        let help_text = r#"
🤖 biomeOS AI Assistant Help

Available Commands:
• "Deploy a biome called [name]" - Deploy a new biome
• "Show ecosystem status" - Get overall ecosystem health
• "List all Primals" - Show available Primals
• "Deploy a web service" - Deploy a service
• "Stop service [name]" - Stop a running service
• "Scale [service] to [number] instances" - Scale a service
• "Show logs for [service]" - Get service logs
• "What's the status of [Primal]?" - Get Primal status

Examples:
• "Deploy a biome called my-app with a web service and database"
• "Show me the status of all Primals in the ecosystem"
• "Scale the web-frontend service to 3 instances"
• "What are the logs for the api-gateway service?"

The AI assistant understands natural language and can help you manage
your entire biomeOS ecosystem across all Primals.
"#;

        Ok(CommandExecutionResult::Help {
            message: help_text.to_string(),
        })
    }
    
    /// General command execution (fallback)
    async fn execute_general_command(
        &self,
        command: ParsedCommand,
        api_client: &UniversalApiClient
    ) -> Result<CommandExecutionResult> {
        // For general commands, try to route to appropriate Primals
        let ai_command = AICommand {
            command: command.original_input.clone(),
            context: Some(format!("Ecosystem context: {:?}", self.ecosystem_context)),
            parameters: command.parameters,
        };
        
        match api_client.execute_ai_command(ai_command).await {
            Ok(result) => {
                Ok(CommandExecutionResult::General {
                    message: "Command executed across relevant Primals".to_string(),
                    results: result.results,
                })
            }
            Err(e) => {
                Ok(CommandExecutionResult::Error {
                    message: format!("Failed to execute command: {}", e),
                    error_type: "general_command_failed".to_string(),
                })
            }
        }
    }
    
    /// Generate AI response from execution result
    async fn generate_response(&self, result: CommandExecutionResult) -> Result<AIResponse> {
        let (message, success, data) = match result {
            CommandExecutionResult::BiomeDeployment { deployment_id, status, message, details } => {
                let response_msg = format!(
                    "✅ {}\nDeployment ID: {}\nStatus: {}",
                    message, deployment_id, status
                );
                (response_msg, status == "success", Some(details))
            }
            CommandExecutionResult::EcosystemStatus { overall_health, total_primals, healthy_primals, details } => {
                let health_emoji = match overall_health.as_str() {
                    "healthy" => "✅",
                    "degraded" => "⚠️",
                    _ => "❌",
                };
                let response_msg = format!(
                    "{} Ecosystem Status: {}\n📊 Primals: {}/{} healthy",
                    health_emoji, overall_health, healthy_primals, total_primals
                );
                (response_msg, overall_health == "healthy", Some(details))
            }
            CommandExecutionResult::PrimalList { primals, count } => {
                let response_msg = format!(
                    "🔧 Available Primals ({}):\n{}",
                    count,
                    primals.join("\n• ")
                );
                (response_msg, true, None)
            }
            CommandExecutionResult::Help { message } => {
                (message, true, None)
            }
            CommandExecutionResult::General { message, results } => {
                (message, true, Some(serde_json::to_value(results)?))
            }
            CommandExecutionResult::Error { message, error_type: _ } => {
                (format!("❌ {}", message), false, None)
            }
        };
        
        Ok(AIResponse {
            message,
            success,
            data,
            timestamp: Utc::now(),
            suggestions: self.generate_suggestions(&result).await,
        })
    }
    
    /// Generate suggestions for next actions
    async fn generate_suggestions(&self, result: &CommandExecutionResult) -> Vec<String> {
        match result {
            CommandExecutionResult::BiomeDeployment { .. } => vec![
                "Check deployment status".to_string(),
                "View deployment logs".to_string(),
                "Scale services if needed".to_string(),
            ],
            CommandExecutionResult::EcosystemStatus { .. } => vec![
                "Deploy a new biome".to_string(),
                "Check individual Primal status".to_string(),
                "View recent events".to_string(),
            ],
            CommandExecutionResult::Error { .. } => vec![
                "Check ecosystem status".to_string(),
                "View error logs".to_string(),
                "Try alternative approach".to_string(),
            ],
            _ => vec![
                "Ask for help".to_string(),
                "Check ecosystem status".to_string(),
            ],
        }
    }
    
    /// Extract entities from natural language input
    fn extract_entities(&self, input: &str) -> HashMap<String, String> {
        let mut entities = HashMap::new();
        
        // Simple entity extraction patterns
        // In a real implementation, you'd use NLP libraries or AI services
        
        // Extract quoted strings as names
        if let Some(captures) = regex::Regex::new(r#""([^"]+)""#).unwrap().captures(input) {
            if let Some(name) = captures.get(1) {
                entities.insert("name".to_string(), name.as_str().to_string());
            }
        }
        
        // Extract numbers
        if let Some(captures) = regex::Regex::new(r"\b(\d+)\b").unwrap().captures(input) {
            if let Some(number) = captures.get(1) {
                entities.insert("number".to_string(), number.as_str().to_string());
            }
        }
        
        // Extract service types
        for service_type in &["web", "api", "database", "cache", "queue"] {
            if input.to_lowercase().contains(service_type) {
                entities.insert("service_type".to_string(), service_type.to_string());
            }
        }
        
        entities
    }
    
    /// Generate parameters for command execution
    fn generate_parameters(
        &self,
        intent: &CommandIntent,
        entities: &HashMap<String, String>,
        input: &str
    ) -> HashMap<String, serde_json::Value> {
        let mut parameters = HashMap::new();
        
        // Add extracted entities as parameters
        for (key, value) in entities {
            parameters.insert(key.clone(), serde_json::Value::String(value.clone()));
        }
        
        // Add intent-specific parameters
        match intent {
            CommandIntent::DeployBiome => {
                parameters.insert("auto_generate".to_string(), serde_json::Value::Bool(true));
                if input.contains("with") {
                    parameters.insert("include_services".to_string(), serde_json::Value::Bool(true));
                }
            }
            CommandIntent::ScaleService => {
                if let Some(number) = entities.get("number") {
                    if let Ok(replicas) = number.parse::<u32>() {
                        parameters.insert("replicas".to_string(), serde_json::Value::Number(replicas.into()));
                    }
                }
            }
            _ => {}
        }
        
        parameters
    }
    
    /// Update ecosystem context from current state
    async fn update_ecosystem_context(&mut self, api_client: &UniversalApiClient) -> Result<()> {
        if let Ok(status) = api_client.get_ecosystem_status().await {
            self.ecosystem_context.last_status = Some(status);
        }
        
        if let Ok(events) = api_client.get_real_time_events().await {
            self.ecosystem_context.recent_events = events.into_iter().take(10).collect();
        }
        
        Ok(())
    }
    
    /// Add entry to conversation history
    fn add_to_conversation(&mut self, role: &str, content: &str) {
        self.conversation_history.push(ConversationEntry {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
        });
        
        // Keep only last 20 entries to manage memory
        if self.conversation_history.len() > 20 {
            self.conversation_history.drain(0..self.conversation_history.len() - 20);
        }
    }
    
    /// Initialize command templates
    fn initialize_command_templates() -> HashMap<String, CommandTemplate> {
        let mut templates = HashMap::new();
        
        templates.insert("deploy_biome".to_string(), CommandTemplate {
            pattern: "deploy.*biome".to_string(),
            intent: CommandIntent::DeployBiome,
            required_params: vec!["name".to_string()],
            optional_params: vec!["services".to_string(), "config".to_string()],
        });
        
        templates.insert("ecosystem_status".to_string(), CommandTemplate {
            pattern: "(status|health).*ecosystem".to_string(),
            intent: CommandIntent::GetEcosystemStatus,
            required_params: vec![],
            optional_params: vec![],
        });
        
        templates
    }
    
    /// Load manifest from path (placeholder implementation)
    async fn load_manifest_from_path(&self, _path: &str) -> Result<BiomeManifest> {
        // In a real implementation, this would load from file system or URL
        Err(anyhow!("Manifest loading not implemented"))
    }
    
    /// Generate manifest from command parameters
    async fn generate_manifest_from_command(&self, command: &ParsedCommand) -> Result<BiomeManifest> {
        use crate::api::{BiomeMetadata, PrimalConfig, ServiceConfig, ResourceConfig};
        
        let biome_name = command.entities.get("name")
            .cloned()
            .unwrap_or_else(|| format!("ai-generated-biome-{}", Utc::now().timestamp()));
        
        let mut primals = HashMap::new();
        let mut services = HashMap::new();
        
        // Add default Primals
        primals.insert("songbird".to_string(), PrimalConfig {
            enabled: true,
            endpoint: Some("http://songbird:8080".to_string()),
            capabilities: vec!["orchestration".to_string()],
            config: HashMap::new(),
        });
        
        primals.insert("toadstool".to_string(), PrimalConfig {
            enabled: true,
            endpoint: Some("http://toadstool:8084".to_string()),
            capabilities: vec!["compute".to_string()],
            config: HashMap::new(),
        });
        
        // Add services based on detected service types
        if command.entities.contains_key("service_type") || command.original_input.contains("service") {
            services.insert("web-service".to_string(), ServiceConfig {
                primal: "toadstool".to_string(),
                runtime: "container".to_string(),
                image: Some("nginx:alpine".to_string()),
                resources: ResourceConfig {
                    cpu: 1.0,
                    memory: "1Gi".to_string(),
                    storage: None,
                },
            });
        }
        
        Ok(BiomeManifest {
            metadata: BiomeMetadata {
                name: biome_name,
                version: "1.0.0".to_string(),
                description: Some("AI-generated biome".to_string()),
            },
            primals,
            services,
        })
    }
    
    // Placeholder implementations for other command executions
    async fn execute_deploy_service_command(&self, _command: ParsedCommand, _api_client: &UniversalApiClient) -> Result<CommandExecutionResult> {
        Ok(CommandExecutionResult::Error {
            message: "Service deployment not yet implemented".to_string(),
            error_type: "not_implemented".to_string(),
        })
    }
    
    async fn execute_get_primal_status_command(&self, _command: ParsedCommand, _api_client: &UniversalApiClient) -> Result<CommandExecutionResult> {
        Ok(CommandExecutionResult::Error {
            message: "Primal status not yet implemented".to_string(),
            error_type: "not_implemented".to_string(),
        })
    }
    
    async fn execute_list_biomes_command(&self, _api_client: &UniversalApiClient) -> Result<CommandExecutionResult> {
        Ok(CommandExecutionResult::Error {
            message: "Biome listing not yet implemented".to_string(),
            error_type: "not_implemented".to_string(),
        })
    }
    
    async fn execute_list_services_command(&self, _command: ParsedCommand, _api_client: &UniversalApiClient) -> Result<CommandExecutionResult> {
        Ok(CommandExecutionResult::Error {
            message: "Service listing not yet implemented".to_string(),
            error_type: "not_implemented".to_string(),
        })
    }
    
    async fn execute_stop_service_command(&self, _command: ParsedCommand, _api_client: &UniversalApiClient) -> Result<CommandExecutionResult> {
        Ok(CommandExecutionResult::Error {
            message: "Service stopping not yet implemented".to_string(),
            error_type: "not_implemented".to_string(),
        })
    }
    
    async fn execute_scale_service_command(&self, _command: ParsedCommand, _api_client: &UniversalApiClient) -> Result<CommandExecutionResult> {
        Ok(CommandExecutionResult::Error {
            message: "Service scaling not yet implemented".to_string(),
            error_type: "not_implemented".to_string(),
        })
    }
    
    async fn execute_get_logs_command(&self, _command: ParsedCommand, _api_client: &UniversalApiClient) -> Result<CommandExecutionResult> {
        Ok(CommandExecutionResult::Error {
            message: "Log retrieval not yet implemented".to_string(),
            error_type: "not_implemented".to_string(),
        })
    }
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: AIProvider,
    pub api_key: Option<String>,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            provider: AIProvider::Local,
            api_key: None,
            model: "local-assistant".to_string(),
            max_tokens: 1000,
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProvider {
    OpenAI,
    Anthropic,
    Local,
}

#[derive(Debug, Clone)]
struct ConversationEntry {
    role: String,
    content: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
struct EcosystemContext {
    last_status: Option<crate::api::EcosystemStatus>,
    recent_events: Vec<crate::api::EcosystemEvent>,
}

#[derive(Debug, Clone)]
struct ParsedCommand {
    intent: CommandIntent,
    entities: HashMap<String, String>,
    parameters: HashMap<String, serde_json::Value>,
    original_input: String,
}

#[derive(Debug, Clone)]
enum CommandIntent {
    DeployBiome,
    DeployService,
    GetEcosystemStatus,
    GetPrimalStatus,
    ListBiomes,
    ListServices,
    ListPrimals,
    StopService,
    ScaleService,
    GetLogs,
    Help,
    General,
}

#[derive(Debug, Clone)]
struct CommandTemplate {
    pattern: String,
    intent: CommandIntent,
    required_params: Vec<String>,
    optional_params: Vec<String>,
}

#[derive(Debug, Clone)]
enum CommandExecutionResult {
    BiomeDeployment {
        deployment_id: String,
        status: String,
        message: String,
        details: serde_json::Value,
    },
    EcosystemStatus {
        overall_health: String,
        total_primals: usize,
        healthy_primals: usize,
        details: serde_json::Value,
    },
    PrimalList {
        primals: Vec<String>,
        count: usize,
    },
    Help {
        message: String,
    },
    General {
        message: String,
        results: HashMap<String, serde_json::Value>,
    },
    Error {
        message: String,
        error_type: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub message: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UICommand {
    pub command: String,
    pub context: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
} 