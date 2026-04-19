// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Secret manifest types (`SecretSpec`, `SecretData`, external providers).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Secret specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretSpec {
    /// Secret metadata
    pub metadata: SecretMetadata,

    /// Secret type
    pub secret_type: SecretType,

    /// Secret data
    pub data: HashMap<String, SecretData>,

    /// Immutable
    pub immutable: bool,
}

/// Secret metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    /// Secret name
    pub name: String,

    /// Secret description
    pub description: Option<String>,

    /// Secret labels
    pub labels: HashMap<String, String>,

    /// Secret annotations
    pub annotations: HashMap<String, String>,
}

/// Secret types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretType {
    /// Opaque secret
    Opaque,
    /// Service account token
    ServiceAccountToken,
    /// Docker config
    DockerConfig,
    /// Docker config JSON
    DockerConfigJson,
    /// Basic auth
    BasicAuth,
    /// SSH auth
    SshAuth,
    /// TLS secret
    Tls,
    /// Bootstrap token
    BootstrapToken,
    /// Custom type
    Custom(String),
}

/// Secret data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretData {
    /// Base64 encoded data
    Base64(String),
    /// Plain text data
    Text(String),
    /// File reference
    File(String),
    /// External reference
    External(ExternalSecretRef),
}

/// External secret reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSecretRef {
    /// Provider
    pub provider: SecretProvider,
    /// Key
    pub key: String,
    /// Version
    pub version: Option<String>,
}

/// Secret providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretProvider {
    /// AWS Secrets Manager
    AwsSecretsManager {
        /// AWS region
        region: String,
    },
    /// Azure Key Vault
    AzureKeyVault {
        /// Vault URL
        vault_url: String,
    },
    /// Google Secret Manager
    GoogleSecretManager {
        /// GCP project ID
        project_id: String,
    },
    /// `HashiCorp` Vault
    Vault {
        /// Vault server address
        address: String,
        /// Secret path within vault
        path: String,
    },
    /// Kubernetes secret
    Kubernetes {
        /// Kubernetes namespace
        namespace: String,
        /// Secret name
        name: String,
    },
    /// Custom provider
    Custom {
        /// Provider name
        provider_name: String,
        /// Provider configuration
        config: HashMap<String, String>,
    },
}

impl Default for SecretSpec {
    fn default() -> Self {
        Self {
            metadata: SecretMetadata::default(),
            secret_type: SecretType::Opaque,
            data: HashMap::new(),
            immutable: false,
        }
    }
}

impl Default for SecretMetadata {
    fn default() -> Self {
        Self {
            name: "default-secret".to_string(),
            description: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
        }
    }
}
