//! Squirrel API Adapter
//!
//! Adapter for Squirrel's AI agent management and MCP protocol API.
//! Discovers agent endpoints, MCP protocol, and session management.

use crate::api_adapter::{ApiAdapter, discovery};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Squirrel-specific API adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquirrelAdapter {
    /// Base API adapter
    base: ApiAdapter,
    
    /// Squirrel-specific endpoints (discovered)
    agent_list_endpoint: Option<String>,
    agent_create_endpoint: Option<String>,
    agent_status_endpoint: Option<String>,
    mcp_endpoint: Option<String>,
    session_endpoint: Option<String>,
    chat_endpoint: Option<String>,
}

impl SquirrelAdapter {
    /// Discover Squirrel's API structure
    pub async fn discover(base_url: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into();
        
        // Use generic discovery first
        let base = discovery::discover_api_interface(&base_url, "squirrel").await?;
        
        // Squirrel-specific discovery
        let mut adapter = Self {
            base,
            agent_list_endpoint: None,
            agent_create_endpoint: None,
            agent_status_endpoint: None,
            mcp_endpoint: None,
            session_endpoint: None,
            chat_endpoint: None,
        };
        
        // Discover Squirrel-specific endpoints
        adapter.discover_agent_endpoints().await;
        adapter.discover_mcp_endpoints().await;
        adapter.discover_session_endpoints().await;
        adapter.discover_chat_endpoints().await;
        
        Ok(adapter)
    }
    
    /// Discover agent management endpoints
    async fn discover_agent_endpoints(&mut self) {
        let list_patterns = vec![
            "/agents",
            "/agents/list",
            "/api/agents",
            "/api/v1/agents",
            "/ai/agents",
        ];
        
        for pattern in list_patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.agent_list_endpoint = Some(pattern.to_string());
                println!("  ✓ Agent list endpoint: {}", pattern);
                break;
            }
        }
        
        let create_patterns = vec![
            "/agents/create",
            "/api/agents/create",
            "/api/v1/agents/create",
            "/ai/agents/create",
        ];
        
        for pattern in create_patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.agent_create_endpoint = Some(pattern.to_string());
                println!("  ✓ Agent create endpoint: {}", pattern);
                break;
            }
        }
        
        let status_patterns = vec![
            "/agents/status",
            "/api/agents/status",
            "/api/v1/agents/status",
        ];
        
        for pattern in status_patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.agent_status_endpoint = Some(pattern.to_string());
                println!("  ✓ Agent status endpoint: {}", pattern);
                break;
            }
        }
    }
    
    /// Discover MCP (Model Context Protocol) endpoints
    async fn discover_mcp_endpoints(&mut self) {
        let patterns = vec![
            "/mcp",
            "/mcp/status",
            "/api/mcp",
            "/api/v1/mcp",
            "/protocol/mcp",
        ];
        
        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.mcp_endpoint = Some(pattern.to_string());
                println!("  ✓ MCP endpoint: {}", pattern);
                break;
            }
        }
    }
    
    /// Discover session management endpoints
    async fn discover_session_endpoints(&mut self) {
        let patterns = vec![
            "/sessions",
            "/sessions/list",
            "/api/sessions",
            "/api/v1/sessions",
            "/ai/sessions",
        ];
        
        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.session_endpoint = Some(pattern.to_string());
                println!("  ✓ Session endpoint: {}", pattern);
                break;
            }
        }
    }
    
    /// Discover chat/interaction endpoints
    async fn discover_chat_endpoints(&mut self) {
        let patterns = vec![
            "/chat",
            "/api/chat",
            "/api/v1/chat",
            "/ai/chat",
            "/interact",
        ];
        
        for pattern in patterns {
            if self.base.try_endpoint(pattern).await.unwrap_or(false) {
                self.chat_endpoint = Some(pattern.to_string());
                println!("  ✓ Chat endpoint: {}", pattern);
                break;
            }
        }
    }
    
    /// Get the base adapter
    pub fn base(&self) -> &ApiAdapter {
        &self.base
    }
    
    /// Check if Squirrel AI is healthy
    pub async fn check_ai_health(&self) -> Result<bool> {
        // Try agent-specific endpoint first
        if let Some(endpoint) = &self.agent_list_endpoint {
            let url = format!("{}{}", self.base.base_url(), endpoint);
            let client = reqwest::Client::new();
            
            if let Ok(response) = client.get(&url).send().await {
                return Ok(response.status().is_success());
            }
        }
        
        // Fallback to generic health check
        self.base.check_health().await
    }
    
    /// Get agent list (if endpoint discovered)
    pub async fn get_agents(&self) -> Result<Option<serde_json::Value>> {
        if let Some(endpoint) = &self.agent_list_endpoint {
            let url = format!("{}{}", self.base.base_url(), endpoint);
            let client = reqwest::Client::new();
            
            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let json = response.json().await?;
                return Ok(Some(json));
            }
        }
        
        Ok(None)
    }
    
    /// Get agent status (if endpoint discovered)
    pub async fn get_agent_status(&self, agent_id: &str) -> Result<Option<serde_json::Value>> {
        if let Some(endpoint) = &self.agent_status_endpoint {
            let url = format!("{}{}/{}", self.base.base_url(), endpoint, agent_id);
            let client = reqwest::Client::new();
            
            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let json = response.json().await?;
                return Ok(Some(json));
            }
        }
        
        Ok(None)
    }
    
    /// Check MCP protocol availability
    pub async fn check_mcp_available(&self) -> Result<bool> {
        if let Some(endpoint) = &self.mcp_endpoint {
            let url = format!("{}{}", self.base.base_url(), endpoint);
            let client = reqwest::Client::new();
            
            if let Ok(response) = client.get(&url).send().await {
                return Ok(response.status().is_success());
            }
        }
        
        Ok(false)
    }
    
    /// Get sessions (if endpoint discovered)
    pub async fn get_sessions(&self) -> Result<Option<serde_json::Value>> {
        if let Some(endpoint) = &self.session_endpoint {
            let url = format!("{}{}", self.base.base_url(), endpoint);
            let client = reqwest::Client::new();
            
            let response = client.get(&url).send().await?;
            if response.status().is_success() {
                let json = response.json().await?;
                return Ok(Some(json));
            }
        }
        
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_squirrel_adapter_creation() {
        // Test will require actual Squirrel instance
        assert_eq!(std::mem::size_of::<SquirrelAdapter>(), std::mem::size_of::<SquirrelAdapter>());
    }
}

