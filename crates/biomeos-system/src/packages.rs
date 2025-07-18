//! # biomeOS Package Manager
//!
//! Manages installation, updates, and removal of biomeOS components and applications.
//! Integrates with Toadstool for runtime management and BearDog for security.

use biomeos_core::BiomeResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use std::time::Duration;

/// Package manager for biomeOS
pub struct PackageManager {
    /// Configuration
    pub config: PackageConfig,
    /// Installed packages
    pub packages: RwLock<HashMap<String, Package>>,
    /// Package repositories
    pub repositories: RwLock<HashMap<String, Repository>>,
    /// Package cache
    pub cache: RwLock<HashMap<String, CachedPackage>>,
}

/// Package configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Package installation directory
    pub install_dir: PathBuf,
    /// Package cache directory
    pub cache_dir: PathBuf,
    /// Package database path
    pub db_path: PathBuf,
    /// Default repositories
    pub repositories: Vec<RepositoryConfig>,
    /// Auto-update enabled
    pub auto_update: bool,
    /// Update check interval in seconds
    pub update_interval_seconds: u64,
    /// Package verification enabled
    pub verify_packages: bool,
    /// Allow untrusted packages
    pub allow_untrusted: bool,
}

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Repository name
    pub name: String,
    /// Repository URL
    pub url: String,
    /// Repository type
    pub repo_type: RepositoryType,
    /// Repository priority
    pub priority: u32,
    /// Repository enabled
    pub enabled: bool,
    /// Repository trust level
    pub trust_level: TrustLevel,
}

/// Repository type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepositoryType {
    /// Official biomeOS repository
    Official,
    /// Community repository
    Community,
    /// Private repository
    Private,
    /// Local repository
    Local,
}

/// Trust level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Fully trusted
    Trusted,
    /// Partially trusted
    Partial,
    /// Untrusted
    Untrusted,
}

/// Package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package description
    pub description: String,
    /// Package author
    pub author: String,
    /// Package license
    pub license: String,
    /// Package category
    pub category: PackageCategory,
    /// Package type
    pub package_type: PackageType,
    /// Package dependencies
    pub dependencies: Vec<Dependency>,
    /// Package files
    pub files: Vec<PackageFile>,
    /// Package metadata
    pub metadata: HashMap<String, String>,
    /// Package status
    pub status: PackageStatus,
    /// Installation time
    pub installed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Installation directory
    pub install_path: PathBuf,
    /// Package size in bytes
    pub size: u64,
    /// Package checksum
    pub checksum: String,
    /// Package signature
    pub signature: Option<String>,
}

/// Package category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageCategory {
    /// System components
    System,
    /// Primal components
    Primal,
    /// Applications
    Application,
    /// Libraries
    Library,
    /// Development tools
    Development,
    /// Games
    Games,
    /// Multimedia
    Multimedia,
    /// Office
    Office,
    /// Science
    Science,
    /// Other
    Other,
}

/// Package type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageType {
    /// Binary package
    Binary,
    /// Source package
    Source,
    /// Container image
    Container,
    /// WebAssembly module
    Wasm,
    /// Biome template
    BiomeTemplate,
    /// Primal extension
    PrimalExtension,
}

/// Package dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency name
    pub name: String,
    /// Version constraint
    pub version: VersionConstraint,
    /// Dependency type
    pub dep_type: DependencyType,
    /// Optional dependency
    pub optional: bool,
}

/// Version constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionConstraint {
    /// Exact version
    Exact(String),
    /// Minimum version
    Min(String),
    /// Maximum version
    Max(String),
    /// Version range
    Range(String, String),
    /// Any version
    Any,
}

/// Dependency type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Runtime dependency
    Runtime,
    /// Build dependency
    Build,
    /// Optional dependency
    Optional,
}

/// Package file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFile {
    /// File path
    pub path: PathBuf,
    /// File size
    pub size: u64,
    /// File checksum
    pub checksum: String,
    /// File permissions
    pub permissions: u32,
    /// File type
    pub file_type: FileType,
}

/// File type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    /// Regular file
    Regular,
    /// Directory
    Directory,
    /// Symbolic link
    Symlink,
    /// Executable
    Executable,
    /// Configuration file
    Config,
    /// Data file
    Data,
}

/// Package status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageStatus {
    /// Package state
    pub state: PackageState,
    /// Package health
    pub health: PackageHealth,
    /// Last update check
    pub last_update_check: Option<chrono::DateTime<chrono::Utc>>,
    /// Available updates
    pub available_updates: Vec<String>,
    /// Installation errors
    pub errors: Vec<String>,
}

/// Package state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageState {
    /// Package is installed
    Installed,
    /// Package is not installed
    NotInstalled,
    /// Package is installing
    Installing,
    /// Package is updating
    Updating,
    /// Package is removing
    Removing,
    /// Package installation failed
    Failed,
}

/// Package health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageHealth {
    /// Package is healthy
    Healthy,
    /// Package has issues
    Degraded,
    /// Package is broken
    Broken,
    /// Package health unknown
    Unknown,
}

/// Repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository configuration
    pub config: RepositoryConfig,
    /// Repository status
    pub status: RepositoryStatus,
    /// Available packages
    pub packages: HashMap<String, RepositoryPackage>,
    /// Last update time
    pub last_update: Option<chrono::DateTime<chrono::Utc>>,
}

/// Repository status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepositoryStatus {
    /// Repository is available
    Available,
    /// Repository is unavailable
    Unavailable,
    /// Repository is updating
    Updating,
    /// Repository has errors
    Error(String),
}

/// Repository package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryPackage {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package description
    pub description: String,
    /// Package download URL
    pub download_url: String,
    /// Package size
    pub size: u64,
    /// Package checksum
    pub checksum: String,
    /// Package metadata
    pub metadata: HashMap<String, String>,
}

/// Repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    /// Metadata version
    pub version: String,
    /// Available packages
    pub packages: Vec<PackageInfo>,
}

/// Package information in repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package description
    pub description: String,
    /// Package filename
    pub filename: String,
    /// Package checksum
    pub checksum: String,
    /// Package signature
    pub signature: Option<String>,
    /// Package size in bytes
    pub size: u64,
    /// Package dependencies
    pub dependencies: Vec<String>,
}

/// Cached package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPackage {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Path to cached file
    pub file_path: PathBuf,
    /// Package checksum
    pub checksum: String,
    /// Package size
    pub size: u64,
    /// Verification status
    pub verified: bool,
}

impl PackageManager {
    /// Create new package manager
    pub fn new(config: PackageConfig) -> Self {
        Self {
            config,
            packages: RwLock::new(HashMap::new()),
            repositories: RwLock::new(HashMap::new()),
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize package manager
    pub async fn initialize(&self) -> BiomeResult<()> {
        tracing::info!("Initializing package manager");

        // Create directories
        if let Err(e) = tokio::fs::create_dir_all(&self.config.install_dir).await {
            tracing::warn!("Failed to create install directory: {}", e);
        }
        if let Err(e) = tokio::fs::create_dir_all(&self.config.cache_dir).await {
            tracing::warn!("Failed to create cache directory: {}", e);
        }

        // Initialize repositories
        self.initialize_repositories().await?;

        // Load installed packages
        self.load_installed_packages().await?;

        tracing::info!("Package manager initialized");
        Ok(())
    }

    /// Initialize repositories
    async fn initialize_repositories(&self) -> BiomeResult<()> {
        let mut repositories = self.repositories.write().await;

        for repo_config in &self.config.repositories {
            let repository = Repository {
                config: repo_config.clone(),
                status: RepositoryStatus::Available,
                packages: HashMap::new(),
                last_update: None,
            };
            repositories.insert(repo_config.name.clone(), repository);
        }

        Ok(())
    }

    /// Load installed packages
    async fn load_installed_packages(&self) -> BiomeResult<()> {
        tracing::info!("Loading installed packages from database");

        // Load packages from database file
        if !self.config.db_path.exists() {
            tracing::info!("Package database does not exist, creating new one");
            self.create_package_database().await?;
            return Ok(());
        }

        // Read package database
        let db_content = tokio::fs::read_to_string(&self.config.db_path).await
            .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to read package database: {}", e)))?;

        // Parse database content (JSON format)
        let packages: HashMap<String, Package> = serde_json::from_str(&db_content)
            .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to parse package database: {}", e)))?;

        // Load packages into memory
        {
            let mut package_store = self.packages.write().await;
            for (name, package) in packages {
                package_store.insert(name, package);
            }
        }

        tracing::info!("Loaded {} packages from database", packages.len());
        Ok(())
    }

    /// Create new package database
    async fn create_package_database(&self) -> BiomeResult<()> {
        tracing::info!("Creating new package database");

        // Create database directory if it doesn't exist
        if let Some(parent) = self.config.db_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to create database directory: {}", e)))?;
        }

        // Create empty database
        let empty_db = HashMap::<String, Package>::new();
        let db_content = serde_json::to_string_pretty(&empty_db)
            .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to serialize empty database: {}", e)))?;

        tokio::fs::write(&self.config.db_path, db_content).await
            .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to write package database: {}", e)))?;

        Ok(())
    }

    /// Update repositories
    pub async fn update_repositories(&self) -> BiomeResult<()> {
        tracing::info!("Updating package repositories");

        let repo_names: Vec<String> = {
            let repositories = self.repositories.read().await;
            repositories.keys().cloned().collect()
        };

        for repo_name in repo_names {
            if let Err(e) = self.update_repository(&repo_name).await {
                tracing::error!("Failed to update repository {}: {}", repo_name, e);
            }
        }

        tracing::info!("Repository update complete");
        Ok(())
    }

    /// Update a specific repository
    async fn update_repository(&self, repo_name: &str) -> BiomeResult<()> {
        tracing::info!("Updating repository: {}", repo_name);

        let repo_config = {
            let repositories = self.repositories.read().await;
            repositories.get(repo_name)
                .map(|r| r.config.clone())
                .ok_or_else(|| biomeos_core::BiomeError::Generic(format!("Repository not found: {}", repo_name)))?
        };

        // Download repository metadata
        let metadata_url = format!("{}/metadata.json", repo_config.url);
        let metadata_content = self.download_file(&metadata_url).await?;

        // Parse repository metadata
        let repo_metadata: RepositoryMetadata = serde_json::from_str(&metadata_content)
            .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to parse repository metadata: {}", e)))?;

        // Update repository packages
        {
            let mut repositories = self.repositories.write().await;
            if let Some(repo) = repositories.get_mut(repo_name) {
                repo.packages.clear();
                
                for package_info in repo_metadata.packages {
                    let repo_package = RepositoryPackage {
                        name: package_info.name.clone(),
                        version: package_info.version,
                        description: package_info.description,
                        download_url: format!("{}/packages/{}", repo_config.url, package_info.filename),
                        checksum: package_info.checksum,
                        signature: package_info.signature,
                        size: package_info.size,
                        metadata: HashMap::new(), // No metadata in RepositoryPackage for now
                    };
                    
                    repo.packages.insert(package_info.name, repo_package);
                }
                
                repo.last_update = Some(chrono::Utc::now());
                repo.status = RepositoryStatus::Available;
            }
        }

        tracing::info!("Repository {} updated with {} packages", repo_name, repo_metadata.packages.len());
        Ok(())
    }

    /// Download a file from URL
    async fn download_file(&self, url: &str) -> BiomeResult<String> {
        tracing::debug!("Downloading file from: {}", url);

        // In a real implementation, this would use a proper HTTP client
        // For now, simulate the download
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Mock metadata content
        let mock_metadata = RepositoryMetadata {
            version: "1.0.0".to_string(),
            packages: vec![
                PackageInfo {
                    name: "biomeos-core".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Core biomeOS components".to_string(),
                    filename: "biomeos-core-1.0.0.tar.gz".to_string(),
                    checksum: "sha256:abcdef123456789".to_string(),
                    signature: Some("signature_data".to_string()),
                    size: 1024000,
                    dependencies: vec![],
                },
                PackageInfo {
                    name: "biomeos-utils".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Utility tools for biomeOS".to_string(),
                    filename: "biomeos-utils-1.0.0.tar.gz".to_string(),
                    checksum: "sha256:123456789abcdef".to_string(),
                    signature: Some("signature_data".to_string()),
                    size: 512000,
                    dependencies: vec!["biomeos-core".to_string()],
                },
            ],
        };

        let content = serde_json::to_string(&mock_metadata)
            .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to serialize mock metadata: {}", e)))?;

        Ok(content)
    }

    /// Find package in repositories
    async fn find_package(
        &self,
        package_name: &str,
        version: Option<&str>,
    ) -> BiomeResult<RepositoryPackage> {
        let repositories = self.repositories.read().await;

        for repo in repositories.values() {
            if let Some(package) = repo.packages.get(package_name) {
                // Check version constraints if specified
                if let Some(required_version) = version {
                    if package.version != required_version {
                        continue;
                    }
                }
                return Ok(package.clone());
            }
        }

        Err(biomeos_core::BiomeError::Generic(format!(
            "Package not found: {}",
            package_name
        )))
    }

    /// Download package
    async fn download_package(&self, repo_package: &RepositoryPackage) -> BiomeResult<CachedPackage> {
        tracing::info!("Downloading package: {}", repo_package.name);

        // Create cache directory if it doesn't exist
        tokio::fs::create_dir_all(&self.config.cache_dir).await
            .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to create cache directory: {}", e)))?;

        // Generate cache file path
        let cache_file = self.config.cache_dir.join(format!("{}-{}.tar.gz", repo_package.name, repo_package.version));

        // Check if package is already cached
        if cache_file.exists() {
            tracing::debug!("Package already cached: {}", cache_file.display());
            
            // Verify cached package
            if self.verify_cached_package(&cache_file, &repo_package.checksum).await? {
                return Ok(CachedPackage {
                    name: repo_package.name.clone(),
                    version: repo_package.version.clone(),
                    file_path: cache_file,
                    checksum: repo_package.checksum.clone(),
                    size: repo_package.size,
                    verified: true,
                });
            } else {
                tracing::warn!("Cached package checksum mismatch, re-downloading");
                tokio::fs::remove_file(&cache_file).await.ok();
            }
        }

        // Download package file
        tracing::debug!("Downloading from: {}", repo_package.download_url);
        
        // In a real implementation, this would download the actual file
        // For now, create a mock package file
        let mock_content = format!("Mock package content for {}-{}", repo_package.name, repo_package.version);
        tokio::fs::write(&cache_file, mock_content).await
            .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to write package file: {}", e)))?;

        let cached_package = CachedPackage {
            name: repo_package.name.clone(),
            version: repo_package.version.clone(),
            file_path: cache_file,
            checksum: repo_package.checksum.clone(),
            size: repo_package.size,
            verified: false,
        };

        tracing::info!("Package downloaded: {}", repo_package.name);
        Ok(cached_package)
    }

    /// Verify cached package
    async fn verify_cached_package(&self, file_path: &std::path::Path, expected_checksum: &str) -> BiomeResult<bool> {
        tracing::debug!("Verifying cached package: {}", file_path.display());

        // In a real implementation, this would calculate the actual checksum
        // For now, simulate verification
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Mock verification - assume cached files are valid
        Ok(true)
    }

    /// Verify package
    async fn verify_package(&self, cached_package: &CachedPackage) -> BiomeResult<()> {
        tracing::info!("Verifying package: {}", cached_package.name);

        // Calculate file checksum
        let file_checksum = self.calculate_checksum(&cached_package.file_path).await?;

        // Verify checksum
        if file_checksum != cached_package.checksum {
            return Err(biomeos_core::BiomeError::Generic(format!(
                "Package checksum mismatch for {}: expected {}, got {}",
                cached_package.name, cached_package.checksum, file_checksum
            )));
        }

        tracing::info!("Package verification successful: {}", cached_package.name);
        Ok(())
    }

    /// Calculate file checksum
    async fn calculate_checksum(&self, file_path: &std::path::Path) -> BiomeResult<String> {
        tracing::debug!("Calculating checksum for: {}", file_path.display());

        // In a real implementation, this would calculate SHA256 or similar
        // For now, return a mock checksum
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok("sha256:abcdef123456789".to_string())
    }

    /// Install package files
    async fn install_package_files(&self, cached_package: &CachedPackage) -> BiomeResult<()> {
        tracing::info!("Installing package files: {}", cached_package.name);

        // Extract package to installation directory
        let install_path = self.config.install_dir.join(&cached_package.name);
        tokio::fs::create_dir_all(&install_path).await
            .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to create install directory: {}", e)))?;

        // In a real implementation, this would extract the tar.gz file
        // For now, create some mock files
        let mock_files = vec![
            "bin/main",
            "lib/libmain.so",
            "share/doc/README.md",
            "share/man/main.1",
        ];

        for file_path in mock_files {
            let full_path = install_path.join(file_path);
            if let Some(parent) = full_path.parent() {
                tokio::fs::create_dir_all(parent).await.ok();
            }
            
            let content = format!("Mock file content for {}/{}", cached_package.name, file_path);
            tokio::fs::write(&full_path, content).await
                .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to write file {}: {}", full_path.display(), e)))?;
        }

        tracing::info!("Package files installed: {}", cached_package.name);
        Ok(())
    }

    /// Register installed package
    async fn register_installed_package(&self, cached_package: &CachedPackage) -> BiomeResult<()> {
        tracing::info!("Registering installed package: {}", cached_package.name);

        let package = Package {
            name: cached_package.name.clone(),
            version: cached_package.version.clone(),
            description: format!("Package {}", cached_package.name),
            author: "Unknown".to_string(),
            license: "Unknown".to_string(),
            category: PackageCategory::System,
            package_type: PackageType::Binary,
            dependencies: vec![],
            files: vec![],
            metadata: HashMap::new(),
            status: PackageStatus::Installed,
            installed_at: Some(chrono::Utc::now()),
            install_path: self.config.install_dir.join(&cached_package.name),
            size: cached_package.size,
            checksum: cached_package.checksum.clone(),
            signature: None,
        };

        // Add to installed packages
        {
            let mut packages = self.packages.write().await;
            packages.insert(cached_package.name.clone(), package);
        }

        // Save to database
        self.save_package_database().await?;

        tracing::info!("Package registered: {}", cached_package.name);
        Ok(())
    }

    /// Save package database
    async fn save_package_database(&self) -> BiomeResult<()> {
        tracing::debug!("Saving package database");

        let packages = self.packages.read().await;
        let db_content = serde_json::to_string_pretty(&*packages)
            .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to serialize package database: {}", e)))?;

        tokio::fs::write(&self.config.db_path, db_content).await
            .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to write package database: {}", e)))?;

        tracing::debug!("Package database saved");
        Ok(())
    }

    /// Remove package
    pub async fn remove_package(&self, package_name: &str) -> BiomeResult<()> {
        tracing::info!("Removing package: {}", package_name);

        // Check if package is installed
        if !self.is_package_installed(package_name).await? {
            return Err(biomeos_core::BiomeError::Generic(format!(
                "Package not installed: {}",
                package_name
            )));
        }

        // Remove package files
        self.remove_package_files(package_name).await?;

        // Remove from package database
        {
            let mut packages = self.packages.write().await;
            packages.remove(package_name);
        }

        tracing::info!("Package removed successfully: {}", package_name);
        Ok(())
    }

    /// Remove package files
    async fn remove_package_files(&self, package_name: &str) -> BiomeResult<()> {
        tracing::info!("Removing package files: {}", package_name);

        // Load the package to get its install path and files
        let package = self.packages.read().await.get(package_name).cloned().ok_or_else(|| {
            biomeos_core::BiomeError::Generic(format!("Package not found: {}", package_name))
        })?;

        // Remove installation directory
        if package.install_path.exists() {
            tokio::fs::remove_dir_all(&package.install_path).await
                .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to remove install directory: {}", e)))?;
        }

        // Remove individual files if specified
        for file in &package.files {
            let file_path = &file.path;
            if file_path.exists() {
                if file_path.is_file() {
                    tokio::fs::remove_file(file_path).await
                        .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to remove file {}: {}", file_path.display(), e)))?;
                } else if file_path.is_dir() {
                    tokio::fs::remove_dir_all(file_path).await
                        .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to remove directory {}: {}", file_path.display(), e)))?;
                }
            }
        }

        // Remove cached package file
        let cache_file = self.config.cache_dir.join(format!("{}-{}.tar.gz", package.name, package.version));
        if cache_file.exists() {
            tokio::fs::remove_file(&cache_file).await
                .map_err(|e| biomeos_core::BiomeError::Generic(format!("Failed to remove cached file: {}", e)))?;
        }

        tracing::info!("Package files removed: {}", package.name);
        Ok(())
    }

    /// Get installed packages
    pub async fn get_installed_packages(&self) -> HashMap<String, Package> {
        self.packages.read().await.clone()
    }

    /// Get package info
    pub async fn get_package_info(&self, package_name: &str) -> Option<Package> {
        let packages = self.packages.read().await;
        packages.get(package_name).cloned()
    }

    /// Search packages
    pub async fn search_packages(&self, query: &str) -> Vec<RepositoryPackage> {
        let mut results = Vec::new();
        let repositories = self.repositories.read().await;

        for repo in repositories.values() {
            for package in repo.packages.values() {
                if package.name.contains(query) || package.description.contains(query) {
                    results.push(package.clone());
                }
            }
        }

        results
    }

    /// Shutdown package manager
    pub async fn shutdown(&self) -> BiomeResult<()> {
        tracing::info!("Shutting down package manager");

        // Save package database
        self.save_package_database().await?;

        tracing::info!("Package manager shutdown complete");
        Ok(())
    }
}

impl Default for PackageConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/var/lib"))
            .join("biomeos");

        Self {
            install_dir: PathBuf::from("/opt/biomeos/packages"),
            cache_dir: data_dir.join("cache/packages"),
            db_path: data_dir.join("packages.db"),
            repositories: vec![
                RepositoryConfig {
                    name: "official".to_string(),
                    url: "https://packages.biomeos.org/official".to_string(),
                    repo_type: RepositoryType::Official,
                    priority: 1,
                    enabled: true,
                    trust_level: TrustLevel::Trusted,
                },
                RepositoryConfig {
                    name: "community".to_string(),
                    url: "https://packages.biomeos.org/community".to_string(),
                    repo_type: RepositoryType::Community,
                    priority: 2,
                    enabled: true,
                    trust_level: TrustLevel::Partial,
                },
            ],
            auto_update: false,
            update_interval_seconds: 86400, // 24 hours
            verify_packages: true,
            allow_untrusted: false,
        }
    }
}
