//! Core types for biomeOS primal SDK

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrimalMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrimalConfig {
    pub endpoint: String,
    pub port: u16,
    pub metadata: PrimalMetadata,
}
