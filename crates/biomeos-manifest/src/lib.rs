//! biomeOS Manifest Management

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BiomeManifest {
    pub name: String,
    pub version: String,
    pub description: String,
}
