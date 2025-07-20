//! biomeOS System Integration

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub kernel: String,
    pub arch: String,
}
