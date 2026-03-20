// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Authorization configuration.
//!
//! Contains configuration for RBAC, ABAC, and authorization policies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    /// Default authorization policy
    pub default_policy: AuthorizationPolicy,

    /// Role-based access control
    pub rbac: Option<RbacConfig>,

    /// Attribute-based access control
    pub abac: Option<AbacConfig>,
}

impl Default for AuthorizationConfig {
    fn default() -> Self {
        Self {
            default_policy: AuthorizationPolicy::Allow,
            rbac: None,
            abac: None,
        }
    }
}

/// Authorization policies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationPolicy {
    /// Allow all requests by default
    Allow,
    /// Deny all requests by default
    Deny,
    /// Use role-based access control
    Rbac,
    /// Use attribute-based access control
    Abac,
    /// Custom authorization handler
    Custom(String),
}

/// RBAC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    /// Roles definition
    pub roles: HashMap<String, Vec<String>>,

    /// Role hierarchy
    pub hierarchy: Option<HashMap<String, Vec<String>>>,
}

/// ABAC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacConfig {
    /// Policy rules
    pub rules: Vec<AbacRule>,

    /// Attribute sources
    pub attribute_sources: HashMap<String, String>,
}

/// ABAC rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacRule {
    /// Rule name
    pub name: String,

    /// Rule condition
    pub condition: String,

    /// Rule action
    pub action: AuthorizationAction,
}

/// Authorization actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationAction {
    /// Allow the request
    Allow,
    /// Deny the request
    Deny,
    /// Log the request but allow it
    Log,
    /// Challenge the user for additional credentials
    Challenge,
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_config_default() {
        let config = AuthorizationConfig::default();
        assert!(matches!(config.default_policy, AuthorizationPolicy::Allow));
        assert!(config.rbac.is_none());
        assert!(config.abac.is_none());
    }

    #[test]
    fn test_authorization_policy_serialization() {
        for policy in [
            AuthorizationPolicy::Allow,
            AuthorizationPolicy::Deny,
            AuthorizationPolicy::Rbac,
            AuthorizationPolicy::Abac,
            AuthorizationPolicy::Custom("opa".to_string()),
        ] {
            let json = serde_json::to_string(&policy).expect("serialize");
            let _: AuthorizationPolicy = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_authorization_action_serialization() {
        for action in [
            AuthorizationAction::Allow,
            AuthorizationAction::Deny,
            AuthorizationAction::Log,
            AuthorizationAction::Challenge,
        ] {
            let json = serde_json::to_string(&action).expect("serialize");
            let _: AuthorizationAction = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_rbac_config_with_hierarchy() {
        let mut roles = HashMap::new();
        roles.insert(
            "admin".to_string(),
            vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
            ],
        );
        roles.insert("user".to_string(), vec!["read".to_string()]);

        let mut hierarchy = HashMap::new();
        hierarchy.insert("admin".to_string(), vec!["user".to_string()]);

        let config = RbacConfig {
            roles,
            hierarchy: Some(hierarchy),
        };

        assert!(config.roles.contains_key("admin"));
        assert!(config.hierarchy.is_some());
    }

    #[test]
    fn test_abac_config_with_rules() {
        let config = AbacConfig {
            rules: vec![
                AbacRule {
                    name: "department-match".to_string(),
                    condition: "user.department == resource.department".to_string(),
                    action: AuthorizationAction::Allow,
                },
                AbacRule {
                    name: "time-based".to_string(),
                    condition: "time.hour >= 9 && time.hour <= 17".to_string(),
                    action: AuthorizationAction::Allow,
                },
            ],
            attribute_sources: {
                let mut sources = HashMap::new();
                sources.insert("user".to_string(), "ldap://directory.local".to_string());
                sources
            },
        };

        assert_eq!(config.rules.len(), 2);
        assert!(config.attribute_sources.contains_key("user"));
    }
}
