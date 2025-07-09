//! UI Components for biomeOS UI

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// UI component trait for different UI modes
pub trait UIComponent {
    fn render(&self) -> String;
    fn handle_input(&mut self, input: &str) -> ComponentResult;
}

/// Result of component input handling
#[derive(Debug, Clone)]
pub enum ComponentResult {
    None,
    StateChange(String),
    Command(String),
    Exit,
}

/// Dashboard component for ecosystem overview
#[derive(Debug, Clone)]
pub struct DashboardComponent {
    pub ecosystem_status: Option<crate::api::EcosystemStatus>,
    pub selected_primal: Option<String>,
}

impl UIComponent for DashboardComponent {
    fn render(&self) -> String {
        let mut output = String::new();
        
        output.push_str("🌍 biomeOS Ecosystem Dashboard\n");
        output.push_str("═══════════════════════════════\n\n");
        
        if let Some(status) = &self.ecosystem_status {
            let health_emoji = match status.overall_health.as_str() {
                "healthy" => "✅",
                "degraded" => "⚠️",
                _ => "❌",
            };
            
            output.push_str(&format!("{} Overall Health: {}\n", health_emoji, status.overall_health));
            output.push_str(&format!("📊 Primals: {}/{} healthy\n", status.healthy_primals, status.total_primals));
            output.push_str(&format!("🕐 Last Updated: {}\n\n", status.last_updated.format("%H:%M:%S")));
            
            output.push_str("🔧 Primal Status:\n");
            for (name, primal_status) in &status.primal_statuses {
                let primal_emoji = match primal_status.health.as_str() {
                    "healthy" => "✅",
                    "degraded" => "⚠️",
                    _ => "❌",
                };
                output.push_str(&format!("  {} {:<12} {} ({} services)\n", 
                    primal_emoji, name, primal_status.health, primal_status.service_count));
            }
        } else {
            output.push_str("⏳ Loading ecosystem status...\n");
        }
        
        output.push_str("\n💡 Commands: [s]tatus, [d]eploy, [l]ist, [a]i, [q]uit\n");
        
        output
    }
    
    fn handle_input(&mut self, input: &str) -> ComponentResult {
        match input.trim().to_lowercase().as_str() {
            "s" | "status" => ComponentResult::Command("status".to_string()),
            "d" | "deploy" => ComponentResult::Command("deploy".to_string()),
            "l" | "list" => ComponentResult::Command("list".to_string()),
            "a" | "ai" => ComponentResult::StateChange("ai_assistant".to_string()),
            "q" | "quit" | "exit" => ComponentResult::Exit,
            _ => ComponentResult::None,
        }
    }
}

/// AI Assistant component for natural language interaction
#[derive(Debug, Clone)]
pub struct AIAssistantComponent {
    pub conversation_history: Vec<ConversationEntry>,
    pub current_input: String,
    pub suggestions: Vec<String>,
}

impl UIComponent for AIAssistantComponent {
    fn render(&self) -> String {
        let mut output = String::new();
        
        output.push_str("🤖 biomeOS AI Assistant\n");
        output.push_str("═══════════════════════\n\n");
        
        // Show recent conversation
        for entry in self.conversation_history.iter().rev().take(5).rev() {
            let role_emoji = match entry.role.as_str() {
                "user" => "👤",
                "assistant" => "🤖",
                _ => "💬",
            };
            output.push_str(&format!("{} {}: {}\n", role_emoji, entry.role, entry.content));
        }
        
        if !self.suggestions.is_empty() {
            output.push_str("\n💡 Suggestions:\n");
            for suggestion in &self.suggestions {
                output.push_str(&format!("  • {}\n", suggestion));
            }
        }
        
        output.push_str("\n🎯 Ask me anything about your biomeOS ecosystem!\n");
        output.push_str("Examples: \"Deploy a biome\", \"Show ecosystem status\", \"Scale web service\"\n");
        output.push_str("Type 'back' to return to dashboard\n");
        
        output
    }
    
    fn handle_input(&mut self, input: &str) -> ComponentResult {
        if input.trim().to_lowercase() == "back" {
            ComponentResult::StateChange("dashboard".to_string())
        } else {
            self.current_input = input.to_string();
            ComponentResult::Command(format!("ai_command:{}", input))
        }
    }
}

/// Deployment component for managing biome deployments
#[derive(Debug, Clone)]
pub struct DeploymentComponent {
    pub active_deployments: HashMap<String, DeploymentInfo>,
    pub selected_deployment: Option<String>,
}

impl UIComponent for DeploymentComponent {
    fn render(&self) -> String {
        let mut output = String::new();
        
        output.push_str("🚀 Biome Deployments\n");
        output.push_str("═══════════════════\n\n");
        
        if self.active_deployments.is_empty() {
            output.push_str("📦 No active deployments\n");
            output.push_str("Use 'deploy' command to create a new biome deployment\n");
        } else {
            output.push_str("📋 Active Deployments:\n");
            for (id, deployment) in &self.active_deployments {
                let status_emoji = match deployment.status.as_str() {
                    "success" => "✅",
                    "failed" => "❌",
                    "pending" => "⏳",
                    _ => "❓",
                };
                output.push_str(&format!("  {} {} - {} ({})\n", 
                    status_emoji, deployment.name, deployment.status, id));
            }
        }
        
        output.push_str("\n💡 Commands: [n]ew deployment, [r]efresh, [b]ack\n");
        
        output
    }
    
    fn handle_input(&mut self, input: &str) -> ComponentResult {
        match input.trim().to_lowercase().as_str() {
            "n" | "new" => ComponentResult::Command("new_deployment".to_string()),
            "r" | "refresh" => ComponentResult::Command("refresh_deployments".to_string()),
            "b" | "back" => ComponentResult::StateChange("dashboard".to_string()),
            _ => ComponentResult::None,
        }
    }
}

/// Service management component
#[derive(Debug, Clone)]
pub struct ServiceComponent {
    pub services: HashMap<String, ServiceInfo>,
    pub selected_service: Option<String>,
}

impl UIComponent for ServiceComponent {
    fn render(&self) -> String {
        let mut output = String::new();
        
        output.push_str("⚙️  Service Management\n");
        output.push_str("═══════════════════\n\n");
        
        if self.services.is_empty() {
            output.push_str("📦 No services running\n");
        } else {
            output.push_str("🔧 Running Services:\n");
            for (id, service) in &self.services {
                let status_emoji = match service.status.as_str() {
                    "running" => "✅",
                    "stopped" => "❌",
                    "pending" => "⏳",
                    _ => "❓",
                };
                output.push_str(&format!("  {} {:<20} {} ({}x replicas)\n", 
                    status_emoji, service.name, service.status, service.replicas));
            }
        }
        
        output.push_str("\n💡 Commands: [s]cale, [l]ogs, [r]estart, [b]ack\n");
        
        output
    }
    
    fn handle_input(&mut self, input: &str) -> ComponentResult {
        match input.trim().to_lowercase().as_str() {
            "s" | "scale" => ComponentResult::Command("scale_service".to_string()),
            "l" | "logs" => ComponentResult::Command("service_logs".to_string()),
            "r" | "restart" => ComponentResult::Command("restart_service".to_string()),
            "b" | "back" => ComponentResult::StateChange("dashboard".to_string()),
            _ => ComponentResult::None,
        }
    }
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub name: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub primal_results: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub status: String,
    pub replicas: u32,
    pub primal: String,
    pub endpoint: Option<String>,
}

/// Terminal UI renderer for ratatui
#[cfg(feature = "terminal-ui")]
pub mod terminal {
    use ratatui::{
        prelude::*,
        widgets::*,
    };
    
    pub fn render_dashboard(frame: &mut Frame, area: Rect, component: &super::DashboardComponent) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);
        
        // Header
        let header = Paragraph::new("🌍 biomeOS Ecosystem Dashboard")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);
        
        // Main content
        let content = if let Some(status) = &component.ecosystem_status {
            let health_color = match status.overall_health.as_str() {
                "healthy" => Color::Green,
                "degraded" => Color::Yellow,
                _ => Color::Red,
            };
            
            format!(
                "Overall Health: {}\nPrimals: {}/{} healthy\nLast Updated: {}",
                status.overall_health,
                status.healthy_primals,
                status.total_primals,
                status.last_updated.format("%H:%M:%S")
            )
        } else {
            "Loading ecosystem status...".to_string()
        };
        
        let main_panel = Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Status"));
        frame.render_widget(main_panel, chunks[1]);
        
        // Footer
        let footer = Paragraph::new("Commands: [s]tatus, [d]eploy, [l]ist, [a]i, [q]uit")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, chunks[2]);
    }
    
    pub fn render_ai_assistant(frame: &mut Frame, area: Rect, component: &super::AIAssistantComponent) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(5),
            ])
            .split(area);
        
        // Header
        let header = Paragraph::new("🤖 biomeOS AI Assistant")
            .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);
        
        // Conversation history
        let history_text = component.conversation_history
            .iter()
            .map(|entry| {
                let role_emoji = match entry.role.as_str() {
                    "user" => "👤",
                    "assistant" => "🤖",
                    _ => "💬",
                };
                format!("{} {}: {}", role_emoji, entry.role, entry.content)
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        let conversation = Paragraph::new(history_text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Conversation"))
            .wrap(Wrap { trim: true });
        frame.render_widget(conversation, chunks[1]);
        
        // Input area
        let input_text = format!("Type your command... (current: {})", component.current_input);
        let input_area = Paragraph::new(input_text)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Input"));
        frame.render_widget(input_area, chunks[2]);
    }
} 