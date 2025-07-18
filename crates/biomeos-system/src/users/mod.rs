//! biomeOS User Manager
//!
//! Manages user accounts, authentication, and permissions for biomeOS.
//! **FULLY INTEGRATED** with BearDog security for keys, secrets, and authentication.

pub mod types;
pub mod manager;
pub mod keys;
pub mod config;
pub mod tests;

// Re-export commonly used types and structures
pub use types::*;

// Re-export the main manager and configuration
pub use types::UserManager;
pub use types::UserConfig;

// Re-export key types for external use
pub use types::{
    User, UserSession, UserGroup, UserAuthRequest, UserAuthMethod,
    Permission, PermissionScope, PermissionLevel, UserStatus, SessionStatus,
    BeardogSecurityProvider, BeardogIntegrationConfig, BeardogAuthMethod,
    BeardogSecurityContext, BeardogKeyManagement, KeyType
};

// Re-export configuration utilities
pub use types::UserConfig as Config; 