// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Manifest Management
//!
//! Handles finding, validating, and deploying federation manifests

use anyhow::{Context, Result};
use biomeos_types::JsonRpcRequest;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

use super::config::FederationConfig;

/// Parsed federation manifest (YAML) describing identity, gates, trust, and shared settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationManifest {
    /// Federation identity (name, family).
    #[serde(default)]
    pub federation: FederationIdentity,
    /// Gate endpoints and roles.
    #[serde(default)]
    pub gates: Vec<GateManifest>,
    /// Directed trust edges between gates (must not contain cycles).
    #[serde(default)]
    pub trust: Vec<TrustEdge>,
    /// Shared discovery, networking, and other cross-gate configuration.
    #[serde(default)]
    pub shared: serde_json::Value,
}

/// Federation identity carried in the manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FederationIdentity {
    /// Human-readable federation name.
    #[serde(default)]
    pub name: String,
    /// Optional stable family identifier for lineage.
    #[serde(default)]
    pub family_id: String,
}

/// One gate in the federation topology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateManifest {
    /// Stable gate id (referenced by trust edges).
    pub id: String,
    /// Neural API endpoint: absolute Unix path, `unix:…`, or `tcp://host:port` / `host:port`.
    pub endpoint: String,
    /// Optional roles (coordinator, worker, …).
    #[serde(default)]
    pub roles: Vec<String>,
}

/// Directed trust from one gate to another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone)]
enum GateEndpoint {
    Unix(PathBuf),
    Tcp(String),
}

/// Find a manifest by name in configured directories
pub fn find_manifest(config: &FederationConfig, manifest: &str) -> Result<PathBuf> {
    // First, check if it's a direct file path
    let direct_path = PathBuf::from(manifest);
    if direct_path.exists() {
        return Ok(direct_path);
    }

    // Search in template directories
    for template_dir in &config.manifests.template_dirs {
        let manifest_path = template_dir.join(format!("{}.yaml", manifest));
        if manifest_path.exists() {
            return Ok(manifest_path);
        }
    }

    // Search in custom directory
    if let Some(custom_dir) = &config.manifests.custom_dir {
        let manifest_path = custom_dir.join(format!("{}.yaml", manifest));
        if manifest_path.exists() {
            return Ok(manifest_path);
        }
    }

    Err(anyhow::anyhow!("Manifest '{}' not found in any configured directory", manifest))
}

/// Validate a manifest file
pub fn validate_manifest(manifest_path: &PathBuf) -> Result<()> {
    if !manifest_path.exists() {
        return Err(anyhow::anyhow!("Manifest file does not exist: {}", manifest_path.display()));
    }

    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;

    // Basic YAML validation
    let _: serde_json::Value = serde_yaml::from_str(&content)
        .with_context(|| format!("Invalid YAML in manifest: {}", manifest_path.display()))?;

    info!("✓ Manifest validation passed: {}", manifest_path.display());
    Ok(())
}

/// Deploy a manifest using kubectl or similar tool
pub fn deploy_manifest(
    config: &FederationConfig,
    manifest: &str,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let manifest_path = find_manifest(config, manifest)?;

    info!("Deploying manifest: {}", manifest_path.display());

    // Validate first
    validate_manifest(&manifest_path)?;

    if dry_run {
        info!("✓ Dry run completed successfully for {}", manifest_path.display());
        return Ok(());
    }

    // Determine deployment method based on manifest content
    let content = fs::read_to_string(&manifest_path)?;

    if content.contains("apiVersion:") {
        // Kubernetes manifest
        deploy_kubernetes_manifest(&manifest_path, force)
    } else {
        // Custom federation manifest
        deploy_federation_manifest(config, &manifest_path, force)
    }
}

fn deploy_kubernetes_manifest(manifest_path: &PathBuf, force: bool) -> Result<()> {
    let mut cmd = Command::new("kubectl");
    cmd.arg("apply").arg("-f").arg(manifest_path);

    if force {
        cmd.arg("--force");
    }

    let output = cmd
        .output()
        .with_context(|| "Failed to execute kubectl - ensure it's installed and configured")?;

    if output.status.success() {
        info!("✓ Kubernetes manifest deployed successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("kubectl failed: {}", stderr))
    }
}

fn deploy_federation_manifest(config: &FederationConfig, manifest_path: &PathBuf, force: bool) -> Result<()> {
    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read federation manifest: {}", manifest_path.display()))?;

    let manifest: FederationManifest = serde_yaml::from_str(&content).with_context(|| {
        format!(
            "Failed to parse federation manifest as structured YAML: {}",
            manifest_path.display()
        )
    })?;

    validate_federation_topology(&manifest)?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("failed to start async runtime for federation deploy: {e}"))?;

    rt.block_on(deploy_federation_manifest_async(config, &manifest, force))
}

fn validate_federation_topology(manifest: &FederationManifest) -> Result<()> {
    if manifest.gates.is_empty() {
        return Err(anyhow::anyhow!("federation manifest defines no gates"));
    }
    if manifest.federation.name.trim().is_empty() {
        return Err(anyhow::anyhow!("federation.name is required for federation manifests"));
    }

    let mut ids = HashSet::new();
    for gate in &manifest.gates {
        if gate.id.trim().is_empty() {
            return Err(anyhow::anyhow!("each gate must have a non-empty id"));
        }
        if !ids.insert(gate.id.clone()) {
            return Err(anyhow::anyhow!("duplicate gate id: {}", gate.id));
        }
        if gate.endpoint.trim().is_empty() {
            return Err(anyhow::anyhow!("gate {} has an empty endpoint", gate.id));
        }
        parse_gate_endpoint(&gate.endpoint)
            .with_context(|| format!("invalid endpoint for gate {}", gate.id))?;
    }

    for edge in &manifest.trust {
        if !ids.contains(&edge.from) {
            return Err(anyhow::anyhow!(
                "trust edge references unknown gate (from): {}",
                edge.from
            ));
        }
        if !ids.contains(&edge.to) {
            return Err(anyhow::anyhow!(
                "trust edge references unknown gate (to): {}",
                edge.to
            ));
        }
    }

    ensure_trust_acyclic(&ids, &manifest.trust)?;
    Ok(())
}

/// Returns an error if the directed trust graph contains a cycle.
fn ensure_trust_acyclic(gate_ids: &HashSet<String>, trust: &[TrustEdge]) -> Result<()> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for id in gate_ids {
        adj.insert(id.clone(), Vec::new());
    }
    for edge in trust {
        adj.entry(edge.from.clone()).or_default().push(edge.to.clone());
    }

    // 0 = unvisited, 1 = on recursion stack, 2 = finished
    let mut color: HashMap<String, u8> = gate_ids.iter().map(|id| (id.clone(), 0)).collect();

    fn visit(
        u: &str,
        adj: &HashMap<String, Vec<String>>,
        color: &mut HashMap<String, u8>,
    ) -> Result<()> {
        let u_entry = color
            .get_mut(u)
            .ok_or_else(|| anyhow::anyhow!("internal: missing gate {u} in color map"))?;
        *u_entry = 1;

        if let Some(neighbors) = adj.get(u) {
            for v in neighbors {
                match color.get(v).copied().unwrap_or(0) {
                    1 => {
                        anyhow::bail!(
                            "trust relationship graph contains a cycle (edge {} -> {})",
                            u,
                            v
                        );
                    }
                    0 => visit(v, adj, color)?,
                    2 => {}
                    _ => {}
                }
            }
        }

        let u_done = color
            .get_mut(u)
            .ok_or_else(|| anyhow::anyhow!("internal: missing gate {u} in color map"))?;
        *u_done = 2;
        Ok(())
    }

    for id in gate_ids {
        if color.get(id).copied().unwrap_or(0) == 0 {
            visit(id, &adj, &mut color)?;
        }
    }
    Ok(())
}

fn parse_gate_endpoint(raw: &str) -> Result<GateEndpoint> {
    let s = raw.trim();
    if s.is_empty() {
        return Err(anyhow::anyhow!("endpoint string is empty"));
    }
    if let Some(rest) = s.strip_prefix("unix:") {
        let path = rest.trim();
        if path.is_empty() {
            return Err(anyhow::anyhow!("unix: endpoint has empty path"));
        }
        return Ok(GateEndpoint::Unix(PathBuf::from(path)));
    }
    if let Some(rest) = s.strip_prefix("tcp://") {
        let addr = rest.trim();
        if addr.is_empty() {
            return Err(anyhow::anyhow!("tcp:// endpoint has empty address"));
        }
        return Ok(GateEndpoint::Tcp(addr.to_string()));
    }
    if s.starts_with('/') {
        return Ok(GateEndpoint::Unix(PathBuf::from(s)));
    }
    if s.contains(':') {
        return Ok(GateEndpoint::Tcp(s.to_string()));
    }
    Err(anyhow::anyhow!(
        "cannot parse neural API endpoint (use absolute unix path, unix:path, tcp://host:port, or host:port): {s}"
    ))
}

async fn send_jsonrpc_newline(endpoint: &GateEndpoint, request: &JsonRpcRequest) -> Result<serde_json::Value> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let payload = serde_json::to_string(request).context("serialize JSON-RPC request")?;

    match endpoint {
        GateEndpoint::Unix(path) => {
            let stream = tokio::net::UnixStream::connect(path)
                .await
                .with_context(|| format!("connect Unix neural API at {}", path.display()))?;
            let (read, mut write) = stream.into_split();
            write
                .write_all(payload.as_bytes())
                .await
                .context("write JSON-RPC request bytes")?;
            write.write_all(b"\n").await.context("write newline after JSON-RPC")?;
            write.flush().await.context("flush JSON-RPC request")?;
            let mut reader = BufReader::new(read);
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .await
                .context("read JSON-RPC response line")?;
            serde_json::from_str(line.trim()).context("parse JSON-RPC response as JSON")
        }
        GateEndpoint::Tcp(addr) => {
            let stream = tokio::net::TcpStream::connect(addr.as_str())
                .await
                .with_context(|| format!("connect TCP neural API at {addr}"))?;
            let (read, mut write) = stream.into_split();
            write
                .write_all(payload.as_bytes())
                .await
                .context("write JSON-RPC request bytes")?;
            write.write_all(b"\n").await.context("write newline after JSON-RPC")?;
            write.flush().await.context("flush JSON-RPC request")?;
            let mut reader = BufReader::new(read);
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .await
                .context("read JSON-RPC response line")?;
            serde_json::from_str(line.trim()).context("parse JSON-RPC response as JSON")
        }
    }
}

fn jsonrpc_check_ok(response: &serde_json::Value, method: &str, gate_id: &str) -> Result<()> {
    if let Some(err) = response.get("error") {
        let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(-32603);
        let message = err
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        anyhow::bail!("JSON-RPC {method} failed on gate {gate_id}: [{code}] {message}");
    }
    Ok(())
}

async fn deploy_federation_manifest_async(
    config: &FederationConfig,
    manifest: &FederationManifest,
    force: bool,
) -> Result<()> {
    let mut gates: Vec<&GateManifest> = manifest.gates.iter().collect();
    gates.sort_by(|a, b| a.id.cmp(&b.id));

    let all_peer_summaries: Vec<serde_json::Value> = manifest
        .gates
        .iter()
        .map(|g| {
            serde_json::json!({
                "id": g.id,
                "endpoint": g.endpoint,
                "roles": g.roles,
            })
        })
        .collect();

    let orchestrator_context = serde_json::json!({
        "federation_domain": config.federation.domain,
        "federation_port": config.federation.port,
        "ssl_enabled": config.is_ssl_enabled(),
        "tower_deployment_path": config.tower.deployment_path,
    });

    info!(
        "Deploying federation {:?} across {} gate(s)",
        manifest.federation.name,
        gates.len()
    );

    for gate in gates {
        let endpoint = match parse_gate_endpoint(&gate.endpoint) {
            Ok(e) => e,
            Err(e) => {
                warn!(
                    gate_id = %gate.id,
                    error = %e,
                    "skipping gate with invalid endpoint"
                );
                continue;
            }
        };

        let configure_params = serde_json::json!({
            "federation": {
                "name": manifest.federation.name,
                "family_id": manifest.federation.family_id,
            },
            "gate": {
                "id": gate.id,
                "endpoint": gate.endpoint,
                "roles": gate.roles,
            },
            "peers": all_peer_summaries,
            "trust": manifest.trust,
            "shared": manifest.shared,
            "orchestrator": orchestrator_context,
            "force": force,
        });

        let configure_req = JsonRpcRequest::new("federation.configure", configure_params);

        match send_jsonrpc_newline(&endpoint, &configure_req).await {
            Ok(resp) => {
                if let Err(e) = jsonrpc_check_ok(&resp, "federation.configure", &gate.id) {
                    warn!(gate_id = %gate.id, error = %e, "federation.configure reported an error; continuing with other gates");
                } else {
                    info!(gate_id = %gate.id, "federation.configure completed");
                }
            }
            Err(e) => {
                warn!(
                    gate_id = %gate.id,
                    error = %e,
                    "gate unreachable for federation.configure; continuing with other gates"
                );
            }
        }

        let outgoing_trust: Vec<String> = manifest
            .trust
            .iter()
            .filter(|e| e.from == gate.id)
            .map(|e| e.to.clone())
            .collect();

        let join_params = serde_json::json!({
            "federation": {
                "name": manifest.federation.name,
                "family_id": manifest.federation.family_id,
            },
            "gate_id": gate.id,
            "trust_peers": outgoing_trust,
            "peers": all_peer_summaries,
            "force": force,
        });

        let join_req = JsonRpcRequest::new("federation.join", join_params);

        match send_jsonrpc_newline(&endpoint, &join_req).await {
            Ok(resp) => {
                if let Err(e) = jsonrpc_check_ok(&resp, "federation.join", &gate.id) {
                    warn!(gate_id = %gate.id, error = %e, "federation.join reported an error; continuing with other gates");
                } else {
                    info!(gate_id = %gate.id, "federation.join completed");
                }
            }
            Err(e) => {
                warn!(
                    gate_id = %gate.id,
                    error = %e,
                    "gate unreachable for federation.join; continuing with other gates"
                );
            }
        }
    }

    // Health verification: each gate should report federation visibility.
    let expected_ids: Vec<String> = manifest.gates.iter().map(|g| g.id.clone()).collect();
    let mut health_ok = 0_u32;
    for gate in gates {
        let endpoint = match parse_gate_endpoint(&gate.endpoint) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let health_params = serde_json::json!({
            "federation_name": manifest.federation.name,
            "family_id": manifest.federation.family_id,
            "expected_gate_ids": expected_ids,
        });
        let health_req = JsonRpcRequest::new("federation.health_check", health_params);
        match send_jsonrpc_newline(&endpoint, &health_req).await {
            Ok(resp) => {
                if jsonrpc_check_ok(&resp, "federation.health_check", &gate.id).is_ok() {
                    health_ok += 1;
                    debug!(gate_id = %gate.id, "federation.health_check succeeded");
                } else if let Some(err) = resp.get("error") {
                    let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(-32603);
                    if code == -32601 {
                        debug!(
                            gate_id = %gate.id,
                            "federation.health_check not implemented on gate; skipping"
                        );
                    } else {
                        let message = err
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("error");
                        warn!(
                            gate_id = %gate.id,
                            code,
                            message,
                            "federation.health_check reported an error"
                        );
                    }
                }
            }
            Err(e) => {
                warn!(
                    gate_id = %gate.id,
                    error = %e,
                    "gate unreachable for federation.health_check"
                );
            }
        }
    }

    if health_ok == 0 && !manifest.gates.is_empty() {
        warn!(
            "federation health check did not succeed on any gate (gates may be down or methods not implemented)"
        );
    } else {
        info!(
            "federation health check completed on {}/{} gate(s)",
            health_ok,
            manifest.gates.len()
        );
    }

    info!(
        "✓ Federation manifest deployment finished for {:?}",
        manifest.federation.name
    );
    Ok(())
}

/// List available manifests in configured directories
pub fn list_manifests(config: &FederationConfig, detailed: bool) -> Result<()> {
    info!("Available Federation Manifests:");
    info!("================================");

    for template_dir in &config.manifests.template_dirs {
        if template_dir.exists() {
            list_manifests_in_dir(template_dir, "Template", detailed)?;
        }
    }

    if let Some(custom_dir) = &config.manifests.custom_dir {
        if custom_dir.exists() {
            list_manifests_in_dir(custom_dir, "Custom", detailed)?;
        }
    }

    Ok(())
}

fn list_manifests_in_dir(dir: &PathBuf, category: &str, detailed: bool) -> Result<()> {
    info!("\n{} Manifests ({}/):", category, dir.display());

    let entries = fs::read_dir(dir).with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    let mut count = 0;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            count += 1;
            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("<invalid>");

            if detailed {
                let metadata = fs::metadata(&path)?;
                let size = metadata.len();
                let modified = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();

                info!("  {} ({} bytes, modified: {})", name, size, modified);
            } else {
                info!("  {}", name);
            }
        }
    }

    if count == 0 {
        info!("  (no manifests found)");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_endpoint_unix_absolute() {
        let e = parse_gate_endpoint("/run/biome/neural.sock").expect("parse");
        match e {
            GateEndpoint::Unix(p) => assert_eq!(p, Path::new("/run/biome/neural.sock")),
            GateEndpoint::Tcp(_) => panic!("expected unix"),
        }
    }

    #[test]
    fn parse_endpoint_tcp() {
        let e = parse_gate_endpoint("tcp://127.0.0.1:9443").expect("parse");
        match e {
            GateEndpoint::Tcp(a) => assert_eq!(a, "127.0.0.1:9443"),
            GateEndpoint::Unix(_) => panic!("expected tcp"),
        }
    }

    #[test]
    fn trust_cycle_detected() {
        let mut ids = HashSet::new();
        ids.insert("a".into());
        ids.insert("b".into());
        let trust = vec![
            TrustEdge {
                from: "a".into(),
                to: "b".into(),
            },
            TrustEdge {
                from: "b".into(),
                to: "a".into(),
            },
        ];
        assert!(ensure_trust_acyclic(&ids, &trust).is_err());
    }

    #[test]
    fn trust_dag_ok() {
        let mut ids = HashSet::new();
        ids.insert("a".into());
        ids.insert("b".into());
        ids.insert("c".into());
        let trust = vec![
            TrustEdge {
                from: "a".into(),
                to: "b".into(),
            },
            TrustEdge {
                from: "b".into(),
                to: "c".into(),
            },
        ];
        assert!(ensure_trust_acyclic(&ids, &trust).is_ok());
    }
}
