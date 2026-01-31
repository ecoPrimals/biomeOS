//! Helper functions for graph execution
//!
//! **TRUE ecoBin v2.0:** Runtime discovery, zero hardcoding.
//!
//! Common utilities used across node executors:
//! - Environment variable substitution
//! - Primal socket discovery (capability-based)
//! - Configuration parsing

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Substitute environment variables in a string
///
/// Replaces `${VAR_NAME}` with values from the environment map.
///
/// # Examples
///
/// ```ignore
/// let env = HashMap::from([("HOME".to_string(), "/home/user".to_string())]);
/// let result = substitute_env("${HOME}/config.toml", &env);
/// assert_eq!(result, "/home/user/config.toml");
/// ```
pub fn substitute_env(input: &str, env: &HashMap<String, String>) -> String {
    let mut result = input.to_string();

    for (key, value) in env {
        let placeholder = format!("${{{}}}", key);
        result = result.replace(&placeholder, value);
    }

    result
}

/// Discover BearDog socket path
///
/// **TRUE ecoBin v2.0:** Uses runtime discovery, NO hardcoded paths.
///
/// Priority order:
/// 1. BEARDOG_SOCKET environment variable
/// 2. Family-scoped socket (deterministic nucleation)
/// 3. XDG_RUNTIME_DIR pattern
/// 4. Common patterns (as fallback)
///
/// # Errors
///
/// Returns an error if BearDog socket cannot be discovered.
pub fn discover_beardog_socket(env: &HashMap<String, String>) -> Result<String> {
    // Priority 1: Explicit environment variable (from graph env or system)
    if let Some(socket) = env.get("BEARDOG_SOCKET") {
        debug!("BearDog socket from graph env: {}", socket);
        return Ok(socket.clone());
    }
    if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
        debug!("BearDog socket from system env: {}", socket);
        return Ok(socket);
    }

    // Priority 2: Family-based socket path (deterministic nucleation)
    // This supports federated deployments where each family has unique sockets
    if let Some(family_id) = env
        .get("FAMILY_ID")
        .or_else(|| std::env::var("BIOMEOS_FAMILY_ID").ok().as_ref())
    {
        let socket = format!("/tmp/beardog-{}.sock", family_id);
        debug!("BearDog socket from family ID: {}", socket);
        return Ok(socket);
    }

    // Priority 3: XDG runtime directory pattern (preferred on Linux)
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let socket = format!("{}/biomeos/beardog.sock", runtime_dir);
        if Path::new(&socket).exists() {
            debug!("BearDog socket from XDG_RUNTIME_DIR: {}", socket);
            return Ok(socket);
        }
    }

    // Priority 4: Common patterns (fallback)
    let patterns = ["/tmp/beardog.sock", "/run/biomeos/beardog.sock"];
    for pattern in &patterns {
        if Path::new(pattern).exists() {
            debug!("BearDog socket from common pattern: {}", pattern);
            return Ok((*pattern).to_string());
        }
    }

    anyhow::bail!(
        "BearDog socket not found. Set BEARDOG_SOCKET or ensure BearDog is running. \
         Checked: BEARDOG_SOCKET env, family-scoped, XDG_RUNTIME_DIR, common patterns"
    )
}

/// Discover any primal socket by name
///
/// **TRUE ecoBin v2.0:** Generic runtime discovery for any primal.
///
/// # Examples
///
/// ```ignore
/// let socket = discover_primal_socket("songbird", &env)?;
/// // Returns: /run/user/1000/biomeos/songbird.sock
/// ```
pub fn discover_primal_socket(
    primal_name: &str,
    env: &HashMap<String, String>,
) -> Result<String> {
    // Priority 1: Primal-specific env var (e.g., SONGBIRD_SOCKET)
    let env_var = format!("{}_SOCKET", primal_name.to_uppercase());
    if let Some(socket) = env.get(&env_var) {
        debug!("{} socket from graph env: {}", primal_name, socket);
        return Ok(socket.clone());
    }
    if let Ok(socket) = std::env::var(&env_var) {
        debug!("{} socket from system env: {}", primal_name, socket);
        return Ok(socket);
    }

    // Priority 2: Family-scoped socket
    if let Some(family_id) = env
        .get("FAMILY_ID")
        .or_else(|| std::env::var("BIOMEOS_FAMILY_ID").ok().as_ref())
    {
        let socket = format!("/tmp/{}-{}.sock", primal_name, family_id);
        debug!("{} socket from family ID: {}", primal_name, socket);
        return Ok(socket);
    }

    // Priority 3: XDG runtime directory
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let socket = format!("{}/biomeos/{}.sock", runtime_dir, primal_name);
        if Path::new(&socket).exists() {
            debug!("{} socket from XDG_RUNTIME_DIR: {}", primal_name, socket);
            return Ok(socket);
        }
    }

    // Priority 4: Common pattern
    let socket = format!("/tmp/{}.sock", primal_name);
    if Path::new(&socket).exists() {
        debug!("{} socket from common pattern: {}", primal_name, socket);
        return Ok(socket);
    }

    anyhow::bail!(
        "{} socket not found. Set {} or ensure {} is running.",
        primal_name,
        env_var,
        primal_name
    )
}

/// Parse a node config value as a specific type
///
/// Extracts and deserializes a configuration value from a node's config JSON.
///
/// # Examples
///
/// ```ignore
/// let path: String = parse_config(&node.config, "path")?;
/// let count: usize = parse_config(&node.config, "retry_count")?;
/// ```
pub fn parse_config<T>(node_config: &serde_json::Value, key: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let value = node_config
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("Missing required config key: {}", key))?;

    serde_json::from_value(value.clone())
        .context(format!("Failed to parse config key '{}' as type", key))
}

/// Parse an optional config value
///
/// Like `parse_config`, but returns `None` if the key doesn't exist.
pub fn parse_config_optional<T>(
    node_config: &serde_json::Value,
    key: &str,
) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    match node_config.get(key) {
        Some(value) => {
            let parsed = serde_json::from_value(value.clone())
                .context(format!("Failed to parse optional config key '{}'", key))?;
            Ok(Some(parsed))
        }
        None => Ok(None),
    }
}

/// Validate that a path exists
///
/// Returns the path as a PathBuf if it exists, error otherwise.
pub fn validate_path_exists(path: &str) -> Result<PathBuf> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        anyhow::bail!("Path does not exist: {}", path_buf.display());
    }
    Ok(path_buf)
}

/// Create directory if it doesn't exist
///
/// Returns the path as a PathBuf. Creates parent directories as needed.
pub fn ensure_dir_exists(path: &str) -> Result<PathBuf> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        std::fs::create_dir_all(&path_buf)
            .context(format!("Failed to create directory: {}", path_buf.display()))?;
    }
    Ok(path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_env() {
        let env = HashMap::from([
            ("HOME".to_string(), "/home/user".to_string()),
            ("NAME".to_string(), "test".to_string()),
        ]);

        assert_eq!(
            substitute_env("${HOME}/config.toml", &env),
            "/home/user/config.toml"
        );

        assert_eq!(
            substitute_env("Hello ${NAME}!", &env),
            "Hello test!"
        );

        // No substitution if not found
        assert_eq!(
            substitute_env("${UNKNOWN}/path", &env),
            "${UNKNOWN}/path"
        );
    }

    #[test]
    fn test_parse_config() {
        let config = serde_json::json!({
            "path": "/tmp/test",
            "count": 42,
            "enabled": true
        });

        assert_eq!(
            parse_config::<String>(&config, "path").unwrap(),
            "/tmp/test"
        );
        assert_eq!(parse_config::<u32>(&config, "count").unwrap(), 42);
        assert_eq!(parse_config::<bool>(&config, "enabled").unwrap(), true);
    }

    #[test]
    fn test_parse_config_missing_key() {
        let config = serde_json::json!({});
        let result = parse_config::<String>(&config, "missing");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing required config key"));
    }

    #[test]
    fn test_parse_config_optional() {
        let config = serde_json::json!({
            "existing": "value"
        });

        assert_eq!(
            parse_config_optional::<String>(&config, "existing")
                .unwrap()
                .unwrap(),
            "value"
        );
        assert!(parse_config_optional::<String>(&config, "missing")
            .unwrap()
            .is_none());
    }
}
