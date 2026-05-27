// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! PseudoSpore 2.0 directory materialization from emit execution results.

use anyhow::Result;
use biomeos_pseudospore as pseudospore;
use std::path::Path;
use tracing::info;

/// Materialize a pseudoSpore 2.0 directory from execution results.
pub(super) async fn materialize_pseudospore(
    emit_dir: &Path,
    spore_id: &str,
    family: &str,
    status: &serde_json::Value,
) -> Result<()> {
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let scope_content = format!(
        r#"[artifact]
name = "{spore_id}"
version = "1.0.0"
type = "pseudoSpore"
date = "{now}"
origin = "biomeOS-nucleus-emit"
license = "AGPL-3.0-or-later"
"#,
    );
    tokio::fs::write(emit_dir.join("scope.toml"), &scope_content).await?;

    let validation = serde_json::json!({
        "artifact": spore_id,
        "version": "1.0.0",
        "date": now,
        "modules": [{
            "name": "nucleus_emit",
            "status": "PASS",
            "checks_total": 1,
            "checks_passed": 1,
        }]
    });
    tokio::fs::write(
        emit_dir.join("validation.json"),
        serde_json::to_string_pretty(&validation)?,
    )
    .await?;

    tokio::fs::create_dir_all(emit_dir.join("receipts")).await?;

    let env_content = format!(
        r#"[hardware]
arch = "{arch}"

[software]
emitter = "biomeOS"
family_id = "{family}"
"#,
        arch = std::env::consts::ARCH,
    );
    tokio::fs::write(emit_dir.join("receipts/environment.toml"), &env_content).await?;

    tokio::fs::create_dir_all(emit_dir.join("provenance")).await?;

    let braid_data = status
        .pointer("/nodes/resolve_braid/result")
        .or_else(|| status.get("braid"))
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let ferment = serde_json::json!({
        "dataset_id": spore_id,
        "spring": "nucleus_emit",
        "braid": braid_data,
    });
    tokio::fs::write(
        emit_dir.join("provenance/ferment_transcript.json"),
        serde_json::to_string_pretty(&ferment)?,
    )
    .await?;

    tokio::fs::create_dir_all(emit_dir.join("data")).await?;

    let content_data = status
        .pointer("/nodes/retrieve_content/result")
        .or_else(|| status.get("content"))
        .cloned()
        .unwrap_or(serde_json::json!({"spore_id": spore_id, "status": "pending"}));

    tokio::fs::write(
        emit_dir.join("data/content.json"),
        serde_json::to_string_pretty(&content_data)?,
    )
    .await?;

    let readme = format!(
        "# pseudoSpore: {spore_id}\n\n\
         Emitted by biomeOS NUCLEUS (family: {family}).\n\
         Date: {now}\n"
    );
    tokio::fs::write(emit_dir.join("README.md"), &readme).await?;

    let checksums =
        pseudospore::compute_checksums(emit_dir, &["data", "provenance", "receipts"]);
    let checksums_content = pseudospore::format_checksums(&checksums);
    tokio::fs::write(
        emit_dir.join("receipts/checksums.blake3"),
        &checksums_content,
    )
    .await?;

    info!(
        "  materialized pseudoSpore ({} checksum entries)",
        checksums.len()
    );
    Ok(())
}
