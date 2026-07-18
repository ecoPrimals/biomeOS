// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Stale Unix socket cleanup for runtime-triggered discovery hygiene.
//!
//! JSON-RPC method: `cleanup.sockets`

use anyhow::{Context, Result};
use biomeos_core::{TransportEndpoint, connect_transport_timed};
use biomeos_types::constants::timeouts;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use super::topology::TopologyHandler;

/// Handler for stale socket cleanup operations.
pub struct CleanupHandler;

struct CleanupScanResult {
    removed: Vec<String>,
    active: Vec<String>,
}

impl CleanupHandler {
    /// Scan socket directories and remove stale `.sock` files.
    ///
    /// JSON-RPC method: `cleanup.sockets`
    ///
    /// Optional params:
    /// - `socket_dir`: explicit directory to scan (defaults to discovered dirs)
    pub async fn cleanup_sockets(params: Option<&Value>) -> Result<Value> {
        let dirs = resolve_socket_dirs(params)?;
        let mut removed = Vec::new();
        let mut active = Vec::new();

        for socket_dir in dirs {
            let scan = scan_and_cleanup_directory(&socket_dir).await?;
            removed.extend(scan.removed);
            active.extend(scan.active);
        }

        Ok(json!({
            "removed": removed,
            "active": active,
        }))
    }
}

fn resolve_socket_dirs(params: Option<&Value>) -> Result<Vec<PathBuf>> {
    if let Some(dir) = params
        .and_then(|p| p.get("socket_dir"))
        .and_then(|v| v.as_str())
    {
        return Ok(vec![PathBuf::from(dir)]);
    }

    let dirs = TopologyHandler::get_socket_directories();
    if dirs.is_empty() {
        anyhow::bail!("No socket directory found");
    }
    Ok(dirs)
}

async fn scan_and_cleanup_directory(socket_dir: &Path) -> Result<CleanupScanResult> {
    let mut removed = Vec::new();
    let mut active = Vec::new();

    let entries = match std::fs::read_dir(socket_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CleanupScanResult { removed, active });
        }
        Err(e) => {
            return Err(e).with_context(|| {
                format!("Failed to read socket directory {}", socket_dir.display())
            });
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("sock") {
            continue;
        }

        let endpoint = TransportEndpoint::UnixSocket { path: path.clone() };
        let connect_ok = connect_transport_timed(&endpoint, timeouts::PROBE_TIMEOUT)
            .await
            .is_ok();

        let path_str = path.to_string_lossy().into_owned();
        if connect_ok {
            active.push(path_str);
        } else if std::fs::remove_file(&path).is_ok() {
            removed.push(path_str);
            info!("Removed stale socket: {}", path.display());
            let pid_path = path.with_extension("pid");
            let _ = std::fs::remove_file(&pid_path);
        } else {
            warn!("Failed to remove stale socket: {}", path.display());
        }
    }

    Ok(CleanupScanResult { removed, active })
}
