// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Local filesystem handlers for the IDE.
//!
//! Provides read/write/list access to the local filesystem,
//! scoped to the Development directory for security.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::AppState;

/// Allowed root directories for filesystem access
fn allowed_roots() -> Vec<PathBuf> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    vec![
        home.join("Development"),
        home.join("Documents"),
    ]
}

/// Check if a path is within allowed roots (prevent directory traversal)
fn is_safe_path(path: &Path) -> bool {
    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };

    allowed_roots().iter().any(|root| {
        root.canonicalize()
            .map_or(false, |r| canonical.starts_with(&r))
    })
}

/// Query parameters for file read
#[derive(Deserialize)]
pub struct ReadParams {
    /// Absolute or relative path to the file
    path: String,
}

/// Read a file from the local filesystem
pub async fn read_file(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<ReadParams>,
) -> impl IntoResponse {
    let path = PathBuf::from(&params.path);

    if !is_safe_path(&path) {
        warn!("Blocked read attempt: {}", params.path);
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Path not within allowed directories"
        }))).into_response();
    }

    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Json(serde_json::json!({
            "path": params.path,
            "content": content,
            "lines": content.lines().count(),
            "size": content.len(),
        })).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": format!("Cannot read {}: {}", params.path, e)
        }))).into_response(),
    }
}

/// Request body for file write
#[derive(Deserialize)]
pub struct WriteRequest {
    /// Absolute path to the file
    path: String,
    /// File content to write
    content: String,
}

/// Write a file to the local filesystem
pub async fn write_file(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<WriteRequest>,
) -> impl IntoResponse {
    let path = PathBuf::from(&req.path);

    if !is_safe_path(&path) {
        warn!("Blocked write attempt: {}", req.path);
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Path not within allowed directories"
        }))).into_response();
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": format!("Cannot create directory: {e}")
            }))).into_response();
        }
    }

    match tokio::fs::write(&path, &req.content).await {
        Ok(()) => Json(serde_json::json!({
            "path": req.path,
            "written": req.content.len(),
            "ok": true,
        })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Cannot write {}: {}", req.path, e)
        }))).into_response(),
    }
}

/// Query parameters for directory listing
#[derive(Deserialize)]
pub struct ListParams {
    /// Directory path to list
    path: String,
    /// Max depth (default 1) — reserved for future recursive listing
    #[allow(dead_code)]
    depth: Option<u32>,
}

/// File entry in a directory listing
#[derive(Serialize)]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
}

/// List files in a directory
pub async fn list_dir(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let path = PathBuf::from(&params.path);

    if !is_safe_path(&path) {
        warn!("Blocked list attempt: {}", params.path);
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Path not within allowed directories"
        }))).into_response();
    }

    let mut entries = Vec::new();
    let mut dir = match tokio::fs::read_dir(&path).await {
        Ok(d) => d,
        Err(e) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": format!("Cannot read directory: {e}")
            }))).into_response();
        }
    };

    while let Ok(Some(entry)) = dir.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip hidden files and common non-essential dirs
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        let meta = entry.metadata().await.unwrap_or_else(|_| {
            // Fallback — shouldn't happen but satisfy the type
            std::fs::metadata(entry.path()).unwrap_or_else(|_| std::fs::metadata(".").unwrap_or_else(|e| panic!("Cannot stat '.': {e}")))
        });
        entries.push(FileEntry {
            path: entry.path().display().to_string(),
            name,
            is_dir: meta.is_dir(),
            size: meta.len(),
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Json(serde_json::json!({
        "path": params.path,
        "entries": entries,
        "count": entries.len(),
    })).into_response()
}
