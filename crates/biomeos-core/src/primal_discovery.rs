// Primal Auto-Discovery
//
// Platform-agnostic primal discovery and capability querying

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info};

/// Primal metadata discovered from binary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetadata {
    /// Binary path
    pub binary: PathBuf,

    /// Primal ID (from query or filename)
    pub id: String,

    /// Capabilities provided
    pub provides: Vec<String>,

    /// Capabilities required
    pub requires: Vec<String>,

    /// Version (if available)
    pub version: Option<String>,

    /// Name (if available)
    pub name: Option<String>,
}

/// Scan directory for executable primals
pub async fn discover_primals(dir: &Path) -> Result<Vec<PrimalMetadata>> {
    info!("🔍 Scanning directory for primals: {}", dir.display());

    let mut primals = Vec::new();
    let mut entries = tokio::fs::read_dir(dir)
        .await
        .context("Failed to read directory")?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Skip non-files and hidden files
        if !path.is_file()
            || path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        {
            continue;
        }

        // Check if executable
        if !is_executable(&path).await {
            continue;
        }

        debug!("📦 Found executable: {}", path.display());

        // Try to query metadata
        match query_primal_metadata(&path).await {
            Ok(metadata) => {
                info!(
                    "✅ Discovered primal: {} (provides: {:?})",
                    metadata.id, metadata.provides
                );
                primals.push(metadata);
            }
            Err(e) => {
                debug!("⚠️  Could not query {}: {}", path.display(), e);
                // Still add it with basic metadata
                if let Some(id) = path.file_stem().and_then(|s| s.to_str()) {
                    primals.push(PrimalMetadata {
                        binary: path.clone(),
                        id: id.to_string(),
                        provides: vec![],
                        requires: vec![],
                        version: None,
                        name: None,
                    });
                }
            }
        }
    }

    info!("🎯 Discovered {} primals", primals.len());
    Ok(primals)
}

/// Query primal binary for its capabilities
///
/// Calls: ./primal --biomeos-capabilities
/// Expects JSON: {"provides": ["Security"], "requires": ["Discovery"]}
pub async fn query_primal_metadata(binary: &Path) -> Result<PrimalMetadata> {
    debug!("🔍 Querying primal metadata: {}", binary.display());

    let output = Command::new(binary)
        .arg("--biomeos-capabilities")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to execute binary")?;

    if !output.status.success() {
        anyhow::bail!(
            "Binary returned error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[derive(Deserialize)]
    struct CapabilityResponse {
        provides: Vec<String>,
        requires: Vec<String>,
        #[serde(default)]
        version: Option<String>,
        #[serde(default)]
        name: Option<String>,
    }

    let response: CapabilityResponse =
        serde_json::from_slice(&output.stdout).context("Failed to parse capabilities JSON")?;

    let id = binary
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(PrimalMetadata {
        binary: binary.to_path_buf(),
        id,
        provides: response.provides,
        requires: response.requires,
        version: response.version,
        name: response.name,
    })
}

/// Check if file is executable (platform-agnostic)
#[cfg(unix)]
async fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    use tokio::fs;

    if let Ok(metadata) = fs::metadata(path).await {
        metadata.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

#[cfg(windows)]
async fn is_executable(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("exe"))
        .unwrap_or(false)
}

#[cfg(not(any(unix, windows)))]
async fn is_executable(path: &Path) -> bool {
    // Bare metal / WASM: rely on naming convention (no extension)
    path.extension().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_discover_empty_dir() {
        let dir = TempDir::new().unwrap();
        let primals = discover_primals(dir.path()).await.unwrap();
        assert_eq!(primals.len(), 0);
    }

    #[tokio::test]
    async fn test_skip_hidden_files() {
        let dir = TempDir::new().unwrap();
        let hidden = dir.path().join(".hidden");
        fs::write(&hidden, "#!/bin/bash\necho test").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hidden).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hidden, perms).unwrap();
        }

        let primals = discover_primals(dir.path()).await.unwrap();
        assert_eq!(primals.len(), 0); // Hidden files skipped
    }

    #[tokio::test]
    async fn test_discover_skips_non_executable() {
        let dir = TempDir::new().unwrap();
        let regular_file = dir.path().join("not_executable");
        fs::write(&regular_file, "#!/bin/bash\necho test").unwrap();
        // No chmod - file is not executable

        let primals = discover_primals(dir.path()).await.unwrap();
        assert_eq!(primals.len(), 0, "Non-executable files should be skipped");
    }

    #[tokio::test]
    async fn test_discover_skips_directories() {
        let dir = TempDir::new().unwrap();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let primals = discover_primals(dir.path()).await.unwrap();
        assert_eq!(primals.len(), 0);
    }

    #[test]
    fn test_primal_metadata_serialization() {
        let meta = PrimalMetadata {
            binary: PathBuf::from("/bin/squirrel"),
            id: "squirrel".to_string(),
            provides: vec!["ai".to_string(), "mcp".to_string()],
            requires: vec!["compute".to_string()],
            version: Some("1.0.0".to_string()),
            name: Some("Squirrel".to_string()),
        };
        let json = serde_json::to_string(&meta).expect("serialize");
        let restored: PrimalMetadata = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(meta.id, restored.id);
        assert_eq!(meta.provides, restored.provides);
        assert_eq!(meta.version, restored.version);
    }

    #[tokio::test]
    async fn test_query_primal_metadata_nonexistent() {
        let result = query_primal_metadata(Path::new("/nonexistent/binary")).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Failed to execute") || err.to_string().contains("No such"),
            "Expected execution error, got: {}",
            err
        );
    }
}
