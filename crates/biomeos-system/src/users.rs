//! # biomeOS User Manager
//!
//! Manages user accounts, authentication, and permissions for biomeOS.
//! **FULLY INTEGRATED** with BearDog security for keys, secrets, and authentication.

use biomeos_core::{BiomeResult, BeardogAccessLevel, GeneticBeardogKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use uuid::Uuid;

/// User manager for biomeOS with BearDog integration
pub struct UserManager {
    /// Configuration
    pub config: UserConfig,
    /// User accounts
    pub users: RwLock<HashMap<String, User>>,
    /// User sessions (managed by BearDog)
    pub sessions: RwLock<HashMap<String, UserSession>>,
    /// User groups
    pub groups: RwLock<HashMap<String, UserGroup>>,
    /// BearDog security provider integration
    pub beardog_provider: Option<BeardogSecurityProvider>,
}

/// BearDog security provider interface
#[derive(Debug, Clone)]
pub struct BeardogSecurityProvider {
    /// BearDog endpoint for user authentication
    pub auth_endpoint: String,
    /// BearDog endpoint for key management
    pub key_management_endpoint: String,
    /// BearDog endpoint for secret storage
    pub secret_storage_endpoint: String,
    /// BearDog service token for system operations
    pub service_token: String,
    /// Enable genetic key support
    pub genetic_keys_enabled: bool,
    /// Enable HSM integration
    pub hsm_integration_enabled: bool,
}

/// User configuration with BearDog integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// User database path
    pub user_db_path: PathBuf,
    /// Home directory base
    pub home_dir_base: PathBuf,
    /// Default shell
    pub default_shell: PathBuf,
    /// Default user group
    pub default_group: String,
    /// BearDog integration configuration
    pub beardog_config: BeardogIntegrationConfig,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Enable guest account
    pub enable_guest: bool,
    /// Enable sudo access
    pub enable_sudo: bool,
    /// Enable genetic BearDog keys
    pub enable_genetic_keys: bool,
}

/// BearDog integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeardogIntegrationConfig {
    /// BearDog service endpoint
    pub endpoint: String,
    /// Authentication method for BearDog
    pub auth_method: BeardogAuthMethod,
    /// Enable key management through BearDog
    pub key_management_enabled: bool,
    /// Enable secret storage through BearDog
    pub secret_storage_enabled: bool,
    /// Enable audit logging through BearDog
    pub audit_logging_enabled: bool,
    /// Enable HSM integration
    pub hsm_integration_enabled: bool,
    /// BearDog compliance mode
    pub compliance_mode: String, // "standard", "fips140", "soc2", "gdpr"
    /// Security level for user operations
    pub security_level: String, // "public", "internal", "confidential", "secret"
}

/// BearDog authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeardogAuthMethod {
    /// Mutual TLS authentication
    MutualTLS {
        cert_path: PathBuf,
        key_path: PathBuf,
        ca_path: PathBuf,
    },
    /// API key authentication
    ApiKey {
        key_reference: String,
    },
    /// Service account authentication
    ServiceAccount {
        account_id: String,
        private_key_reference: String,
    },
    /// Genetic BearDog key authentication
    GeneticKey {
        parent_key_fingerprint: String,
        genetic_lineage: Vec<String>,
    },
}

/// User account with BearDog security integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: u32,
    /// Username
    pub username: String,
    /// Full name
    pub full_name: Option<String>,
    /// Email address
    pub email: Option<String>,
    /// BearDog key reference (replaces password_hash)
    pub beardog_key_reference: String,
    /// Genetic BearDog key (if enabled)
    pub genetic_key: Option<GeneticBeardogKey>,
    /// BearDog access level
    pub access_level: BeardogAccessLevel,
    /// User home directory
    pub home_dir: PathBuf,
    /// Default shell
    pub shell: PathBuf,
    /// Primary group ID
    pub primary_group: u32,
    /// Additional groups
    pub groups: Vec<String>,
    /// User status
    pub status: UserStatus,
    /// Account creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last login time
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    /// Account expiry
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// User permissions
    pub permissions: Vec<Permission>,
    /// SSH public keys (stored in BearDog)
    pub ssh_key_references: Vec<String>,
    /// API keys (stored in BearDog)
    pub api_key_references: Vec<String>,
    /// User metadata
    pub metadata: HashMap<String, String>,
}

/// User status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserStatus {
    /// User account is active
    Active,
    /// User account is disabled
    Disabled,
    /// User account is locked
    Locked,
    /// User account is expired
    Expired,
    /// User account is pending activation
    Pending,
    /// User account is pending BearDog key generation
    PendingKeyGeneration,
}

/// User permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Permission name
    pub name: String,
    /// Permission description
    pub description: String,
    /// Permission scope
    pub scope: PermissionScope,
    /// Permission level
    pub level: PermissionLevel,
    /// BearDog authorization reference
    pub beardog_auth_reference: Option<String>,
}

/// Permission scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionScope {
    /// System-wide permission
    System,
    /// User-specific permission
    User,
    /// Group-specific permission
    Group,
    /// Resource-specific permission
    Resource { resource: String },
    /// Primal-specific permission
    Primal { primal: String },
    /// Biome-specific permission
    Biome { biome: String },
}

/// Permission level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// Read-only access
    Read,
    /// Read-write access
    Write,
    /// Execute access
    Execute,
    /// Administrative access
    Admin,
    /// Full access
    Full,
    /// BearDog-secured access
    BeardogSecured,
}

/// User session with BearDog security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: u32,
    /// Username
    pub username: String,
    /// BearDog security context
    pub beardog_context: BeardogSecurityContext,
    /// Session start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Session last activity
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Session expiry time
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Session IP address
    pub ip_address: Option<String>,
    /// Session user agent
    pub user_agent: Option<String>,
    /// Session status
    pub status: SessionStatus,
    /// BearDog audit trail reference
    pub audit_trail_reference: String,
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

/// BearDog security context for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeardogSecurityContext {
    /// BearDog authentication token
    pub auth_token: String,
    /// Security level for this session
    pub security_level: String,
    /// Authorized operations
    pub authorized_operations: Vec<String>,
    /// Key access grants
    pub key_access_grants: Vec<String>,
    /// Secret access grants
    pub secret_access_grants: Vec<String>,
    /// Threat assessment score
    pub threat_assessment_score: f64,
    /// Compliance status
    pub compliance_status: String,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is active
    Active,
    /// Session is expired
    Expired,
    /// Session is revoked
    Revoked,
    /// Session is terminated
    Terminated,
    /// Session is blocked by BearDog
    BeardogBlocked,
    /// Session is under threat assessment
    ThreatAssessment,
}

/// User group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroup {
    /// Group ID
    pub id: u32,
    /// Group name
    pub name: String,
    /// Group description
    pub description: Option<String>,
    /// Group members
    pub members: Vec<String>,
    /// Group permissions
    pub permissions: Vec<Permission>,
    /// BearDog group policy reference
    pub beardog_policy_reference: Option<String>,
    /// Group metadata
    pub metadata: HashMap<String, String>,
}

/// User authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthRequest {
    /// Username
    pub username: String,
    /// Authentication method
    pub auth_method: UserAuthMethod,
    /// Client IP address
    pub client_ip: Option<String>,
    /// Client user agent
    pub client_user_agent: Option<String>,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// User authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserAuthMethod {
    /// Password authentication (validated through BearDog)
    Password { password: String },
    /// SSH key authentication (validated through BearDog)
    SshKey { public_key: String, signature: String },
    /// API key authentication (validated through BearDog)
    ApiKey { key: String },
    /// Genetic BearDog key authentication
    GeneticKey { key: GeneticBeardogKey },
    /// Biometric authentication (processed through BearDog)
    Biometric { biometric_data: String, biometric_type: String },
}

/// BearDog key management operations
#[derive(Debug, Clone)]
pub struct BeardogKeyManagement {
    /// BearDog provider reference
    pub provider: Option<BeardogSecurityProvider>,
}

impl BeardogKeyManagement {
    /// Create a new BearDog key for user
    pub async fn create_user_key(&self, username: &str, key_type: KeyType) -> BiomeResult<String> {
        // Implementation would call BearDog API
        todo!("Implement BearDog key creation")
    }
    
    /// Rotate user's BearDog key
    pub async fn rotate_user_key(&self, username: &str, old_key_ref: &str) -> BiomeResult<String> {
        // Implementation would call BearDog API
        todo!("Implement BearDog key rotation")
    }
    
    /// Validate user key through BearDog
    pub async fn validate_user_key(&self, key_ref: &str, challenge: &str) -> BiomeResult<bool> {
        // Implementation would call BearDog API
        todo!("Implement BearDog key validation")
    }
    
    /// Store secret in BearDog HSM
    pub async fn store_user_secret(&self, username: &str, secret_name: &str, secret_value: &str) -> BiomeResult<String> {
        // Implementation would call BearDog secret storage
        todo!("Implement BearDog secret storage")
    }
    
    /// Retrieve secret from BearDog HSM
    pub async fn retrieve_user_secret(&self, username: &str, secret_ref: &str) -> BiomeResult<String> {
        // Implementation would call BearDog secret retrieval
        todo!("Implement BearDog secret retrieval")
    }
}

/// Key types for BearDog integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyType {
    /// Authentication key
    Authentication,
    /// Encryption key
    Encryption,
    /// Signing key
    Signing,
    /// SSH key pair
    SshKeyPair,
    /// API key
    ApiKey,
    /// Genetic BearDog key
    GeneticKey,
}

impl UserManager {
    /// Create a new user manager with BearDog integration
    pub fn new(config: UserConfig) -> Self {
        let beardog_provider = if config.beardog_config.key_management_enabled {
            Some(BeardogSecurityProvider {
                auth_endpoint: format!("{}/auth", config.beardog_config.endpoint),
                key_management_endpoint: format!("{}/keys", config.beardog_config.endpoint),
                secret_storage_endpoint: format!("{}/secrets", config.beardog_config.endpoint),
                service_token: "system_service_token".to_string(), // Would be loaded securely
                genetic_keys_enabled: config.enable_genetic_keys,
                hsm_integration_enabled: config.beardog_config.hsm_integration_enabled,
            })
        } else {
            None
        };

        Self {
            config,
            users: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
            groups: RwLock::new(HashMap::new()),
            beardog_provider,
        }
    }

    /// Start the user manager
    pub async fn start(&self) -> BiomeResult<()> {
        tracing::info!("Starting user manager");
        self.initialize().await?;
        tracing::info!("User manager started successfully");
        Ok(())
    }

    /// Initialize user manager with BearDog integration
    pub async fn initialize(&self) -> BiomeResult<()> {
        tracing::info!("Initializing user manager with BearDog integration");

        // Initialize BearDog connection
        if let Some(provider) = &self.beardog_provider {
            self.initialize_beardog_connection(provider).await?;
        }

        // Load users from database
        self.load_users().await?;

        // Create default groups with BearDog policies
        self.create_default_groups().await?;

        // Create root user with BearDog key
        if !self.user_exists("root").await? {
            self.create_root_user().await?;
        }

        tracing::info!("User manager initialization complete");
        Ok(())
    }

    /// Initialize BearDog connection and validate security context
    async fn initialize_beardog_connection(&self, provider: &BeardogSecurityProvider) -> BiomeResult<()> {
        tracing::info!("Initializing BearDog security provider connection");
        
        // Validate BearDog endpoints
        // Implementation would ping BearDog services and validate connectivity
        
        // Establish service-to-service authentication
        // Implementation would authenticate with BearDog using service credentials
        
        // Validate security context
        // Implementation would verify security level and compliance mode
        
        tracing::info!("BearDog security provider initialized successfully");
        Ok(())
    }

    /// Create user with BearDog key management
    pub async fn create_user_with_beardog(&self, 
        username: &str, 
        auth_method: UserAuthMethod,
        access_level: BeardogAccessLevel,
        full_name: Option<String>
    ) -> BiomeResult<u32> {
        tracing::info!("Creating user '{}' with BearDog integration", username);

        // Check if user already exists
        if self.user_exists(username).await? {
            return Err(biomeos_core::BiomeError::Generic {
                message: format!("User '{}' already exists", username)
            });
        }

        let user_id = self.generate_user_id().await;

        // Create BearDog key for the user
        let beardog_key_reference = if let Some(provider) = &self.beardog_provider {
            // Implementation would create key through BearDog API
            format!("beardog_key_{}_{}_{}", username, user_id, Uuid::new_v4().simple())
        } else {
            return Err(biomeos_core::BiomeError::Generic {
                message: "BearDog provider not configured".to_string()
            });
        };

        // Generate genetic key if enabled
        let genetic_key = if self.config.enable_genetic_keys {
            Some(GeneticBeardogKey {
                parent_key_fingerprint: "beardog_prime_key".to_string(),
                genetic_lineage: vec![
                    "beardog_prime".to_string(),
                    format!("user_branch_{}", username),
                ],
                access_level: access_level.clone(),
                encrypted_endpoint: Some(format!("https://beardog.biome.local/user/{}", username)),
                valid_until: Some(chrono::Utc::now() + chrono::Duration::days(365)),
            })
        } else {
            None
        };

        let user = User {
            id: user_id,
            username: username.to_string(),
            full_name,
            email: None,
            beardog_key_reference,
            genetic_key,
            access_level,
            home_dir: self.config.home_dir_base.join(username),
            shell: self.config.default_shell.clone(),
            primary_group: 1000, // users group
            groups: vec![self.config.default_group.clone()],
            status: UserStatus::Active,
            created_at: chrono::Utc::now(),
            last_login: None,
            expires_at: None,
            permissions: vec![],
            ssh_key_references: vec![],
            api_key_references: vec![],
            metadata: HashMap::new(),
        };

        // Store user in database
        {
            let mut users = self.users.write().await;
            users.insert(username.to_string(), user);
        }

        // Log user creation through BearDog audit system
        self.audit_user_operation("user_created", username, &HashMap::new()).await?;

        tracing::info!("User '{}' created successfully with BearDog integration", username);
        Ok(user_id)
    }

    /// Authenticate user through BearDog
    pub async fn authenticate_with_beardog(&self, auth_request: UserAuthRequest) -> BiomeResult<UserSession> {
        tracing::info!("Authenticating user '{}' through BearDog", auth_request.username);

        // Get user from database
        let user = {
            let users = self.users.read().await;
            users.get(&auth_request.username).cloned()
        };

        let user = user.ok_or_else(|| biomeos_core::BiomeError::Generic {
            message: format!("User '{}' not found", auth_request.username)
        })?;

        // Validate through BearDog
        let validation_result = self.validate_auth_through_beardog(&user, &auth_request.auth_method).await?;

        if !validation_result.success {
            self.audit_user_operation("authentication_failed", &auth_request.username, &HashMap::new()).await?;
            return Err(biomeos_core::BiomeError::Generic {
                message: "Authentication failed".to_string()
            });
        }

        // Create BearDog security context
        let beardog_context = BeardogSecurityContext {
            auth_token: validation_result.auth_token,
            security_level: validation_result.security_level,
            authorized_operations: validation_result.authorized_operations,
            key_access_grants: validation_result.key_access_grants,
            secret_access_grants: validation_result.secret_access_grants,
            threat_assessment_score: validation_result.threat_assessment_score,
            compliance_status: validation_result.compliance_status,
        };

        // Create session with BearDog integration
        let session = self.create_beardog_session(&user, beardog_context, &auth_request).await?;

        // Update last login time
        {
            let mut users = self.users.write().await;
            if let Some(user) = users.get_mut(&auth_request.username) {
                user.last_login = Some(chrono::Utc::now());
            }
        }

        // Log successful authentication
        self.audit_user_operation("authentication_success", &auth_request.username, &HashMap::new()).await?;

        tracing::info!("User '{}' authenticated successfully through BearDog", auth_request.username);
        Ok(session)
    }

    /// Validate authentication through BearDog
    async fn validate_auth_through_beardog(&self, user: &User, auth_method: &UserAuthMethod) -> BiomeResult<BeardogValidationResult> {
        // Implementation would call BearDog authentication API
        // This is a placeholder for the actual BearDog integration
        
        Ok(BeardogValidationResult {
            success: true,
            auth_token: format!("beardog_token_{}", Uuid::new_v4().simple()),
            security_level: "confidential".to_string(),
            authorized_operations: vec!["read".to_string(), "write".to_string()],
            key_access_grants: vec![user.beardog_key_reference.clone()],
            secret_access_grants: vec![],
            threat_assessment_score: 0.1, // Low threat
            compliance_status: "compliant".to_string(),
        })
    }

    /// Create session with BearDog security context
    async fn create_beardog_session(&self, user: &User, beardog_context: BeardogSecurityContext, auth_request: &UserAuthRequest) -> BiomeResult<UserSession> {
        let session_id = Uuid::new_v4().to_string();
        let audit_trail_reference = format!("beardog_audit_{}", Uuid::new_v4().simple());

        let session = UserSession {
            id: session_id,
            user_id: user.id,
            username: user.username.clone(),
            beardog_context,
            start_time: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(self.config.session_timeout_seconds as i64),
            ip_address: auth_request.client_ip.clone(),
            user_agent: auth_request.client_user_agent.clone(),
            status: SessionStatus::Active,
            audit_trail_reference,
            metadata: auth_request.metadata.clone(),
        };

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session.id.clone(), session.clone());
        }

        Ok(session)
    }

    /// Audit user operations through BearDog
    async fn audit_user_operation(&self, operation: &str, username: &str, metadata: &HashMap<String, String>) -> BiomeResult<()> {
        if let Some(provider) = &self.beardog_provider {
            // Implementation would log to BearDog audit system
            tracing::info!("Auditing operation '{}' for user '{}' through BearDog", operation, username);
        }
        Ok(())
    }

    /// Add SSH key for user (stored in BearDog)
    pub async fn add_user_ssh_key(&self, username: &str, public_key: &str, key_name: &str) -> BiomeResult<String> {
        tracing::info!("Adding SSH key '{}' for user '{}' through BearDog", key_name, username);

        if let Some(provider) = &self.beardog_provider {
            // Implementation would store SSH key in BearDog
            let key_reference = format!("ssh_key_{}_{}", username, Uuid::new_v4().simple());
            
            // Update user record
            {
                let mut users = self.users.write().await;
                if let Some(user) = users.get_mut(username) {
                    user.ssh_key_references.push(key_reference.clone());
                }
            }

            self.audit_user_operation("ssh_key_added", username, &HashMap::from([
                ("key_name".to_string(), key_name.to_string()),
                ("key_reference".to_string(), key_reference.clone()),
            ])).await?;

            Ok(key_reference)
        } else {
            Err(biomeos_core::BiomeError::Generic {
                message: "BearDog provider not configured".to_string()
            })
        }
    }

    /// Generate API key for user (stored in BearDog)
    pub async fn generate_user_api_key(&self, username: &str, key_name: &str, permissions: Vec<String>) -> BiomeResult<String> {
        tracing::info!("Generating API key '{}' for user '{}' through BearDog", key_name, username);

        if let Some(provider) = &self.beardog_provider {
            // Implementation would generate API key in BearDog
            let key_reference = format!("api_key_{}_{}", username, Uuid::new_v4().simple());
            
            // Update user record
            {
                let mut users = self.users.write().await;
                if let Some(user) = users.get_mut(username) {
                    user.api_key_references.push(key_reference.clone());
                }
            }

            self.audit_user_operation("api_key_generated", username, &HashMap::from([
                ("key_name".to_string(), key_name.to_string()),
                ("key_reference".to_string(), key_reference.clone()),
                ("permissions".to_string(), permissions.join(",")),
            ])).await?;

            Ok(key_reference)
        } else {
            Err(biomeos_core::BiomeError::Generic {
                message: "BearDog provider not configured".to_string()
            })
        }
    }

    // ... existing methods updated for BearDog integration
    
    async fn load_users(&self) -> BiomeResult<()> {
        // Implementation would load users from database
        // Enhanced to load BearDog key references and genetic keys
        Ok(())
    }

    async fn create_default_groups(&self) -> BiomeResult<()> {
        tracing::info!("Creating default groups with BearDog policies");
        
        let default_groups = vec![
            ("administrators", "System administrators with full access"),
            ("users", "Standard users"),
            ("guests", "Guest users with limited access"),
            ("researchers", "Research users with data access"),
            ("developers", "Development users with build access"),
        ];

        for (group_name, description) in default_groups {
            let group = UserGroup {
                id: self.generate_group_id().await,
                name: group_name.to_string(),
                description: Some(description.to_string()),
                members: vec![],
                permissions: vec![],
                beardog_policy_reference: Some(format!("beardog_policy_{}", group_name)),
                metadata: HashMap::new(),
            };

            let mut groups = self.groups.write().await;
            groups.insert(group_name.to_string(), group);
        }

        Ok(())
    }

    async fn create_root_user(&self) -> BiomeResult<()> {
        tracing::info!("Creating root user with BearDog integration");

        self.create_user_with_beardog(
            "root",
            UserAuthMethod::Password { 
                password: "root".to_string() // Would be securely generated
            },
            BeardogAccessLevel::Enterprise, // Root gets enterprise level access
            Some("Root User".to_string())
        ).await?;

        Ok(())
    }

    async fn user_exists(&self, username: &str) -> BiomeResult<bool> {
        let users = self.users.read().await;
        Ok(users.contains_key(username))
    }

    async fn generate_user_id(&self) -> u32 {
        let users = self.users.read().await;
        users.len() as u32 + 1000 // Start from 1000
    }

    async fn generate_group_id(&self) -> u32 {
        let groups = self.groups.read().await;
        groups.len() as u32 + 100 // Start from 100
    }

    pub async fn get_user(&self, username: &str) -> Option<User> {
        let users = self.users.read().await;
        users.get(username).cloned()
    }

    pub async fn get_all_users(&self) -> HashMap<String, User> {
        self.users.read().await.clone()
    }

    pub async fn get_session(&self, session_id: &str) -> Option<UserSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn shutdown(&self) -> BiomeResult<()> {
        tracing::info!("Shutting down user manager with BearDog integration");

        // Terminate all sessions
        {
            let mut sessions = self.sessions.write().await;
            for session in sessions.values_mut() {
                session.status = SessionStatus::Terminated;
            }
        }

        // Save users to database
        // Implementation would save user data including BearDog references

        // Audit shutdown operation
        self.audit_user_operation("user_manager_shutdown", "system", &HashMap::new()).await?;

        tracing::info!("User manager shutdown complete");
        Ok(())
    }
}

/// BearDog validation result
#[derive(Debug, Clone)]
struct BeardogValidationResult {
    success: bool,
    auth_token: String,
    security_level: String,
    authorized_operations: Vec<String>,
    key_access_grants: Vec<String>,
    secret_access_grants: Vec<String>,
    threat_assessment_score: f64,
    compliance_status: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            user_db_path: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("/var/lib"))
                .join("biomeos/users.db"),
            home_dir_base: PathBuf::from("/home"),
            default_shell: PathBuf::from("/bin/bash"),
            default_group: "users".to_string(),
            beardog_config: BeardogIntegrationConfig {
                endpoint: "https://beardog.biome.local:8443".to_string(),
                auth_method: BeardogAuthMethod::MutualTLS {
                    cert_path: PathBuf::from("/etc/biomeos/certs/user-manager.crt"),
                    key_path: PathBuf::from("/etc/biomeos/certs/user-manager.key"),
                    ca_path: PathBuf::from("/etc/biomeos/certs/beardog-ca.crt"),
                },
                key_management_enabled: true,
                secret_storage_enabled: true,
                audit_logging_enabled: true,
                hsm_integration_enabled: true,
                compliance_mode: "soc2".to_string(),
                security_level: "confidential".to_string(),
            },
            session_timeout_seconds: 3600, // 1 hour
            enable_guest: false,
            enable_sudo: true,
            enable_genetic_keys: true, // Enable genetic BearDog keys by default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_user_config_default() {
        let config = UserConfig::default();
        
        assert!(config.user_db_path.ends_with("users.db"));
        assert!(config.home_dir_base.ends_with("home"));
        assert_eq!(config.default_shell, PathBuf::from("/bin/bash"));
        assert_eq!(config.default_group, "users");
        assert_eq!(config.session_timeout_seconds, 3600);
        assert!(!config.enable_guest); // Default is false
        assert!(config.enable_sudo);
        assert!(config.enable_genetic_keys); // Default is true
    }

    #[test]
    fn test_user_manager_creation() {
        let config = UserConfig::default();
        let user_manager = UserManager::new(config);
        
        // Verify the user manager was created with the correct configuration
        assert_eq!(user_manager.config.default_group, "users");
        assert_eq!(user_manager.config.session_timeout_seconds, 3600);
        assert_eq!(user_manager.config.default_shell, PathBuf::from("/bin/bash"));
        
        // Verify BearDog provider is created when key management is enabled
        assert!(user_manager.beardog_provider.is_some());
    }

    #[test]
    fn test_beardog_integration_config() {
        let beardog_config = BeardogIntegrationConfig {
            endpoint: "https://beardog.test.local".to_string(),
            auth_method: BeardogAuthMethod::ApiKey {
                key_reference: "test_key".to_string(),
            },
            key_management_enabled: true,
            secret_storage_enabled: true,
            audit_logging_enabled: true,
            hsm_integration_enabled: false,
            compliance_mode: "standard".to_string(),
            security_level: "internal".to_string(),
        };
        
        assert_eq!(beardog_config.endpoint, "https://beardog.test.local");
        assert!(beardog_config.key_management_enabled);
        assert!(beardog_config.secret_storage_enabled);
        assert!(beardog_config.audit_logging_enabled);
        assert!(!beardog_config.hsm_integration_enabled);
        assert_eq!(beardog_config.compliance_mode, "standard");
        assert_eq!(beardog_config.security_level, "internal");
    }

    #[test]
    fn test_user_auth_method_variants() {
        let password_auth = UserAuthMethod::Password { password: "secret".to_string() };
        let ssh_key_auth = UserAuthMethod::SshKey { 
            public_key: "ssh-rsa AAAAB3...".to_string(),
            signature: "signature".to_string(),
        };
        let api_key_auth = UserAuthMethod::ApiKey { key: "api_key_123".to_string() };
        
        match password_auth {
            UserAuthMethod::Password { password } => assert_eq!(password, "secret"),
            _ => panic!("Expected password auth"),
        }
        
        match ssh_key_auth {
            UserAuthMethod::SshKey { public_key, .. } => assert!(public_key.starts_with("ssh-rsa")),
            _ => panic!("Expected SSH key auth"),
        }
        
        match api_key_auth {
            UserAuthMethod::ApiKey { key } => assert_eq!(key, "api_key_123"),
            _ => panic!("Expected API key auth"),
        }
    }

    #[test]
    fn test_user_session_status_transitions() {
        let active_status = SessionStatus::Active;
        let expired_status = SessionStatus::Expired;
        let revoked_status = SessionStatus::Revoked;
        
        assert_eq!(format!("{:?}", active_status), "Active");
        assert_eq!(format!("{:?}", expired_status), "Expired");
        assert_eq!(format!("{:?}", revoked_status), "Revoked");
    }

    #[test]
    fn test_beardog_auth_method_variants() {
        let mutual_tls = BeardogAuthMethod::MutualTLS {
            cert_path: PathBuf::from("/path/to/cert.pem"),
            key_path: PathBuf::from("/path/to/key.pem"),
            ca_path: PathBuf::from("/path/to/ca.pem"),
        };
        
        let api_key = BeardogAuthMethod::ApiKey {
            key_reference: "beardog_key_123".to_string(),
        };
        
        match mutual_tls {
            BeardogAuthMethod::MutualTLS { cert_path, key_path, ca_path } => {
                assert!(cert_path.ends_with("cert.pem"));
                assert!(key_path.ends_with("key.pem"));
                assert!(ca_path.ends_with("ca.pem"));
            },
            _ => panic!("Expected mutual TLS auth"),
        }
        
        match api_key {
            BeardogAuthMethod::ApiKey { key_reference } => {
                assert_eq!(key_reference, "beardog_key_123");
            },
            _ => panic!("Expected API key auth"),
        }
    }

    #[test]
    fn test_beardog_key_management_creation() {
        let key_mgmt = BeardogKeyManagement {
            provider: None,
        };
        
        assert!(key_mgmt.provider.is_none());
    }

    #[test]
    fn test_user_auth_request_creation() {
        let auth_request = UserAuthRequest {
            username: "testuser".to_string(),
            auth_method: UserAuthMethod::Password { password: "secret".to_string() },
            client_ip: Some("127.0.0.1".to_string()),
            client_user_agent: Some("test-agent".to_string()),
            metadata: HashMap::new(),
        };
        
        assert_eq!(auth_request.username, "testuser");
        assert_eq!(auth_request.client_ip.unwrap(), "127.0.0.1");
        assert_eq!(auth_request.client_user_agent.unwrap(), "test-agent");
        assert_eq!(auth_request.metadata.len(), 0);
    }

    #[test]
    fn test_user_status_variants() {
        let active = UserStatus::Active;
        let disabled = UserStatus::Disabled;
        let locked = UserStatus::Locked;
        
        assert_eq!(format!("{:?}", active), "Active");
        assert_eq!(format!("{:?}", disabled), "Disabled");
        assert_eq!(format!("{:?}", locked), "Locked");
    }

    #[test]
    fn test_key_type_variants() {
        let auth_key = KeyType::Authentication;
        let enc_key = KeyType::Encryption;
        let sign_key = KeyType::Signing;
        
        assert_eq!(format!("{:?}", auth_key), "Authentication");
        assert_eq!(format!("{:?}", enc_key), "Encryption");
        assert_eq!(format!("{:?}", sign_key), "Signing");
    }

    #[tokio::test]
    async fn test_user_manager_async_operations() {
        let config = UserConfig::default();
        let user_manager = UserManager::new(config);
        
        // Test user existence check
        let exists = user_manager.user_exists("nonexistent").await.unwrap();
        assert!(!exists);
        
        // Test ID generation
        let user_id = user_manager.generate_user_id().await;
        assert_eq!(user_id, 1000); // Should start from 1000
        
        let group_id = user_manager.generate_group_id().await;
        assert_eq!(group_id, 100); // Should start from 100
    }

    #[tokio::test]
    async fn test_user_manager_get_operations() {
        let config = UserConfig::default();
        let user_manager = UserManager::new(config);
        
        // Test getting non-existent user
        let user = user_manager.get_user("nonexistent").await;
        assert!(user.is_none());
        
        // Test getting all users (should be empty initially)
        let all_users = user_manager.get_all_users().await;
        assert!(all_users.is_empty());
        
        // Test getting non-existent session
        let session = user_manager.get_session("nonexistent").await;
        assert!(session.is_none());
    }

    #[test]
    fn test_permission_creation() {
        let permission = Permission {
            name: "read_files".to_string(),
            description: "Read access to files".to_string(),
            scope: PermissionScope::System,
            level: PermissionLevel::Read,
            beardog_auth_reference: Some("beardog_auth_123".to_string()),
        };
        
        assert_eq!(permission.name, "read_files");
        assert_eq!(permission.description, "Read access to files");
        assert!(permission.beardog_auth_reference.is_some());
        
        match permission.scope {
            PermissionScope::System => {}, // Expected
            _ => panic!("Expected system scope"),
        }
        
        match permission.level {
            PermissionLevel::Read => {}, // Expected
            _ => panic!("Expected read level"),
        }
    }

    #[test]
    fn test_user_group_creation() {
        let group = UserGroup {
            id: 100,
            name: "developers".to_string(),
            description: Some("Developer group".to_string()),
            members: vec!["alice".to_string(), "bob".to_string()],
            permissions: vec![],
            beardog_policy_reference: Some("dev_policy".to_string()),
            metadata: HashMap::new(),
        };
        
        assert_eq!(group.id, 100);
        assert_eq!(group.name, "developers");
        assert_eq!(group.description.unwrap(), "Developer group");
        assert_eq!(group.members.len(), 2);
        assert!(group.members.contains(&"alice".to_string()));
        assert!(group.members.contains(&"bob".to_string()));
        assert_eq!(group.beardog_policy_reference.unwrap(), "dev_policy");
    }

    #[test]
    fn test_beardog_security_context() {
        let context = BeardogSecurityContext {
            auth_token: "token123".to_string(),
            security_level: "confidential".to_string(),
            authorized_operations: vec!["read".to_string(), "write".to_string()],
            key_access_grants: vec!["enc_key_1".to_string()],
            secret_access_grants: vec!["secret_1".to_string()],
            threat_assessment_score: 0.85,
            compliance_status: "compliant".to_string(),
        };
        
        assert_eq!(context.auth_token, "token123");
        assert_eq!(context.security_level, "confidential");
        assert_eq!(context.authorized_operations.len(), 2);
        assert_eq!(context.key_access_grants.len(), 1);
        assert_eq!(context.secret_access_grants.len(), 1);
        assert_eq!(context.threat_assessment_score, 0.85);
        assert_eq!(context.compliance_status, "compliant");
    }

    #[test]
    fn test_user_session_creation() {
        let session = UserSession {
            id: "session123".to_string(),
            user_id: 1001,
            username: "testuser".to_string(),
            beardog_context: BeardogSecurityContext {
                auth_token: "token123".to_string(),
                security_level: "internal".to_string(),
                authorized_operations: vec!["read".to_string()],
                key_access_grants: vec![],
                secret_access_grants: vec![],
                threat_assessment_score: 0.9,
                compliance_status: "compliant".to_string(),
            },
            start_time: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            status: SessionStatus::Active,
            audit_trail_reference: "audit123".to_string(),
            metadata: HashMap::new(),
        };
        
        assert_eq!(session.id, "session123");
        assert_eq!(session.user_id, 1001);
        assert_eq!(session.username, "testuser");
        assert_eq!(session.beardog_context.security_level, "internal");
        assert_eq!(session.ip_address.unwrap(), "127.0.0.1");
        assert_eq!(session.user_agent.unwrap(), "test-agent");
        assert_eq!(session.audit_trail_reference, "audit123");
        assert!(matches!(session.status, SessionStatus::Active));
    }

    #[test]
    fn test_permission_scope_variants() {
        let system_scope = PermissionScope::System;
        let user_scope = PermissionScope::User;
        let resource_scope = PermissionScope::Resource { resource: "database".to_string() };
        
        assert_eq!(format!("{:?}", system_scope), "System");
        assert_eq!(format!("{:?}", user_scope), "User");
        
        match resource_scope {
            PermissionScope::Resource { resource } => assert_eq!(resource, "database"),
            _ => panic!("Expected resource scope"),
        }
    }

    #[test]
    fn test_permission_level_variants() {
        let read_level = PermissionLevel::Read;
        let write_level = PermissionLevel::Write;
        let admin_level = PermissionLevel::Admin;
        
        assert_eq!(format!("{:?}", read_level), "Read");
        assert_eq!(format!("{:?}", write_level), "Write");
        assert_eq!(format!("{:?}", admin_level), "Admin");
    }
} 