// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! `manifest.gate_profile` handler — resolves the local gate's repo profile
//! from `ecosystem_manifest.toml` at runtime.
//!
//! The ecosystem manifest (maintained in `wateringHole`) defines which repos
//! each gate cares about. This handler reads the manifest, identifies the
//! local gate, and returns an enriched profile that signal graphs
//! (`ecosystem.check`, `ecosystem.pull`) use to drive per-gate operations.

use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

use biomeos_types::env_config::vars;

/// Top-level structure of `ecosystem_manifest.toml`.
#[derive(Deserialize)]
struct EcosystemManifest {
    meta: ManifestMeta,
    sync: Option<SyncConfig>,
    #[serde(default)]
    repos: HashMap<String, RepoEntry>,
    #[serde(default)]
    gates: HashMap<String, GateProfile>,
}

#[derive(Deserialize)]
struct ManifestMeta {
    version: String,
    #[serde(default)]
    wave: u32,
    #[serde(default)]
    total_repos: u32,
    #[serde(default)]
    generated: String,
}

#[derive(Deserialize)]
struct SyncConfig {
    #[serde(default)]
    forgejo_base_url: String,
    #[serde(default)]
    forgejo_ssh: String,
    #[serde(default)]
    forgejo_host: String,
    #[serde(default)]
    default_source: String,
    #[serde(default)]
    default_branch: String,
}

#[derive(Deserialize)]
struct RepoEntry {
    #[serde(default)]
    org: String,
    #[serde(default)]
    local_path: String,
    #[serde(default)]
    membrane: String,
    #[serde(default)]
    sync_priority: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    github_repo: String,
    #[serde(default)]
    forgejo_repo: String,
    #[serde(default)]
    default_branch: String,
    #[serde(default)]
    sync_source: String,
}

#[derive(Deserialize)]
struct GateProfile {
    repos: Vec<String>,
}

/// Resolve the path to `ecosystem_manifest.toml`.
///
/// Priority:
/// 1. `BIOMEOS_ECOSYSTEM_MANIFEST` env var
/// 2. `params.manifest_path` from the JSON-RPC call
/// 3. Walk up from CWD looking for `infra/wateringHole/ecosystem_manifest.toml`
fn resolve_manifest_path(params: &Option<Value>) -> Option<PathBuf> {
    if let Ok(p) = std::env::var(vars::ECOSYSTEM_MANIFEST_PATH) {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }

    if let Some(p) = params
        .as_ref()
        .and_then(|v| v.get("manifest_path"))
        .and_then(|v| v.as_str())
    {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }

    walk_up_for_manifest()
}

/// Walk up from CWD looking for `infra/wateringHole/ecosystem_manifest.toml`.
fn walk_up_for_manifest() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..8 {
        let candidate = dir.join("infra/wateringHole/ecosystem_manifest.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Resolve the local gate ID.
///
/// Priority:
/// 1. `params.gate_id` from JSON-RPC call
/// 2. `BIOMEOS_GATE_ID` env var
fn resolve_gate_id(params: &Option<Value>) -> Option<String> {
    if let Some(id) = params
        .as_ref()
        .and_then(|v| v.get("gate_id"))
        .and_then(|v| v.as_str())
    {
        return Some(id.to_string());
    }

    std::env::var(vars::GATE_ID).ok()
}

/// Handle `manifest.gate_profile` JSON-RPC method.
///
/// Params (all optional):
///   - `gate_id`: override gate identity (default: `$BIOMEOS_GATE_ID`)
///   - `manifest_path`: override manifest path (default: auto-discovered)
///
/// Returns:
///   - `gate_id`: resolved gate name
///   - `manifest_version`: manifest format version
///   - `wave`: wave number from manifest meta
///   - `repos`: array of enriched repo objects for this gate
///   - `repo_count`: number of repos in this profile
pub async fn handle_gate_profile(params: &Option<Value>) -> Result<Value, anyhow::Error> {
    let manifest_path = resolve_manifest_path(params).context(
        "ecosystem_manifest.toml not found — set BIOMEOS_ECOSYSTEM_MANIFEST or pass manifest_path",
    )?;

    let gate_id = resolve_gate_id(params)
        .context("gate ID not resolved — set BIOMEOS_GATE_ID or pass gate_id param")?;

    debug!(gate = %gate_id, manifest = %manifest_path.display(), "resolving gate profile");

    load_and_resolve(&manifest_path, &gate_id)
}

fn load_and_resolve(manifest_path: &Path, gate_id: &str) -> Result<Value, anyhow::Error> {
    let raw = std::fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let manifest: EcosystemManifest =
        toml::from_str(&raw).context("failed to parse ecosystem_manifest.toml")?;

    let profile = manifest.gates.get(gate_id).with_context(|| {
        format!(
            "gate '{gate_id}' not found in manifest (available: {:?})",
            manifest.gates.keys().collect::<Vec<_>>()
        )
    })?;

    let default_branch = manifest
        .sync
        .as_ref()
        .map(|s| s.default_branch.as_str())
        .unwrap_or("main");

    let enriched: Vec<Value> = profile
        .repos
        .iter()
        .map(|repo_key| {
            if let Some(repo) = manifest.repos.get(repo_key) {
                serde_json::json!({
                    "name": repo_key,
                    "org": repo.org,
                    "local_path": repo.local_path,
                    "membrane": repo.membrane,
                    "sync_priority": repo.sync_priority,
                    "category": repo.category,
                    "description": repo.description,
                    "github_repo": repo.github_repo,
                    "forgejo_repo": repo.forgejo_repo,
                    "default_branch": if repo.default_branch.is_empty() {
                        default_branch
                    } else {
                        &repo.default_branch
                    },
                    "sync_source": if repo.sync_source.is_empty() {
                        manifest.sync.as_ref().map_or("github", |s| {
                            if s.default_source.is_empty() { "github" } else { s.default_source.as_str() }
                        })
                    } else {
                        repo.sync_source.as_str()
                    },
                })
            } else {
                serde_json::json!({
                    "name": repo_key,
                    "error": "repo not found in manifest"
                })
            }
        })
        .collect();

    let sync_meta = manifest.sync.as_ref().map(|s| {
        serde_json::json!({
            "forgejo_base_url": s.forgejo_base_url,
            "forgejo_ssh": s.forgejo_ssh,
            "forgejo_host": s.forgejo_host,
        })
    });

    Ok(serde_json::json!({
        "gate_id": gate_id,
        "manifest_version": manifest.meta.version,
        "wave": manifest.meta.wave,
        "generated": manifest.meta.generated,
        "total_repos_in_manifest": manifest.meta.total_repos,
        "repo_count": enriched.len(),
        "repos": enriched,
        "sync": sync_meta,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_test_manifest(dir: &Path) -> PathBuf {
        let path = dir.join("ecosystem_manifest.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"
[meta]
version = "2.0.0"
wave = 60
total_repos = 3
generated = "2026-05-29"

[sync]
forgejo_base_url = "https://git.primals.eco"
forgejo_ssh = "ssh://git@git.primals.eco:2222"
forgejo_host = "vps"
default_source = "github"
default_branch = "main"

[repos.biomeOS]
org = "ecoPrimals"
local_path = "primals/biomeOS"
membrane = "trailing-mirror"
sync_priority = "high"
category = "primal"
description = "Orchestration layer"
github_repo = "ecoPrimals/biomeOS"
forgejo_repo = "ecoPrimals/biomeOS"

[repos.bearDog]
org = "ecoPrimals"
local_path = "primals/bearDog"
membrane = "trailing-mirror"
sync_priority = "high"
category = "primal"
description = "Security, crypto, BTSP"
github_repo = "ecoPrimals/bearDog"
forgejo_repo = "ecoPrimals/bearDog"

[repos.songbird]
org = "ecoPrimals"
local_path = "primals/songBird"
membrane = "trailing-mirror"
sync_priority = "high"
category = "primal"
description = "Discovery, routing"
github_repo = "ecoPrimals/songBird"
forgejo_repo = "ecoPrimals/songBird"

[gates.testGate]
repos = ["biomeOS", "bearDog"]

[gates.fullGate]
repos = ["biomeOS", "bearDog", "songbird"]
"#
        )
        .unwrap();
        path
    }

    #[test]
    fn resolve_known_gate() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = write_test_manifest(dir.path());

        let result = load_and_resolve(&manifest_path, "testGate").unwrap();
        assert_eq!(result["gate_id"], "testGate");
        assert_eq!(result["repo_count"], 2);
        assert_eq!(result["manifest_version"], "2.0.0");
        assert_eq!(result["wave"], 60);

        let repos = result["repos"].as_array().unwrap();
        assert_eq!(repos[0]["name"], "biomeOS");
        assert_eq!(repos[0]["category"], "primal");
        assert_eq!(repos[0]["default_branch"], "main");
        assert_eq!(repos[1]["name"], "bearDog");
    }

    #[test]
    fn resolve_full_gate() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = write_test_manifest(dir.path());

        let result = load_and_resolve(&manifest_path, "fullGate").unwrap();
        assert_eq!(result["repo_count"], 3);
    }

    #[test]
    fn unknown_gate_errors() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = write_test_manifest(dir.path());

        let err = load_and_resolve(&manifest_path, "unknownGate").unwrap_err();
        assert!(err.to_string().contains("unknownGate"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn missing_repo_ref_returns_error_marker() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.toml");
        std::fs::write(
            &path,
            r#"
[meta]
version = "1.0.0"

[gates.broken]
repos = ["nonexistent"]
"#,
        )
        .unwrap();

        let result = load_and_resolve(&path, "broken").unwrap();
        let repos = result["repos"].as_array().unwrap();
        assert_eq!(repos[0]["name"], "nonexistent");
        assert_eq!(repos[0]["error"], "repo not found in manifest");
    }

    #[test]
    fn sync_metadata_present() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = write_test_manifest(dir.path());

        let result = load_and_resolve(&manifest_path, "testGate").unwrap();
        let sync = &result["sync"];
        assert_eq!(sync["forgejo_base_url"], "https://git.primals.eco");
        assert_eq!(sync["forgejo_host"], "vps");
    }
}
