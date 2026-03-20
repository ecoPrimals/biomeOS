// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Extraction of metadata from tower.toml configuration.
//!
//! Reads spore ID, family ID, and other tower-specific metadata
//! from the spore's tower.toml file.

use std::path::Path;

use crate::error::SporeResult;

/// Extract spore ID from spore path or config
///
/// Tries to read from tower.toml meta.node_id, falls back to directory name.
pub fn extract_spore_id(spore_path: &Path) -> SporeResult<String> {
    let tower_toml_path = spore_path.join("tower.toml");

    if tower_toml_path.exists() {
        let content = std::fs::read_to_string(&tower_toml_path)?;

        // Parse TOML and extract node_id from meta section
        if let Ok(config) = toml::from_str::<toml::Value>(&content)
            && let Some(meta) = config.get("meta")
            && let Some(node_id) = meta.get("node_id")
            && let Some(id) = node_id.as_str()
        {
            return Ok(id.to_string());
        }
    }

    // Fallback: use directory name or UUID
    Ok(spore_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string())
}

/// Extract family_id from tower.toml
///
/// Tries tower.family first, then meta.family_id, falls back to "unknown".
pub fn extract_family_id(spore_path: &Path) -> Result<String, anyhow::Error> {
    let tower_toml_path = spore_path.join("tower.toml");
    let content = std::fs::read_to_string(&tower_toml_path)?;

    if let Ok(config) = toml::from_str::<toml::Value>(&content) {
        if let Some(tower) = config.get("tower")
            && let Some(family) = tower.get("family")
            && let Some(family_str) = family.as_str()
        {
            return Ok(family_str.to_string());
        }

        // Fallback: check meta.family_id
        if let Some(meta) = config.get("meta")
            && let Some(family_id) = meta.get("family_id")
            && let Some(id) = family_id.as_str()
        {
            return Ok(id.to_string());
        }
    }

    Ok("unknown".to_string())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn extract_spore_id_from_tower_toml_meta_node_id() {
        let temp = TempDir::new().unwrap();
        let tower_toml = temp.path().join("tower.toml");
        fs::write(
            &tower_toml,
            r#"
[meta]
node_id = "spore-abc-123"
"#,
        )
        .unwrap();

        let id = extract_spore_id(temp.path()).unwrap();
        assert_eq!(id, "spore-abc-123");
    }

    #[test]
    fn extract_spore_id_fallback_to_dir_name_when_no_tower_toml() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("my-spore-dir");
        fs::create_dir_all(&subdir).unwrap();

        let id = extract_spore_id(&subdir).unwrap();
        assert_eq!(id, "my-spore-dir");
    }

    #[test]
    fn extract_spore_id_fallback_when_tower_toml_missing_meta() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("tower.toml"),
            r#"
[tower]
family = "test"
"#,
        )
        .unwrap();

        let id = extract_spore_id(temp.path()).unwrap();
        assert_eq!(id, temp.path().file_name().unwrap().to_str().unwrap());
    }

    #[test]
    fn extract_spore_id_fallback_when_node_id_not_string() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("tower.toml"),
            r"
[meta]
node_id = 42
",
        )
        .unwrap();

        let id = extract_spore_id(temp.path()).unwrap();
        assert_eq!(id, temp.path().file_name().unwrap().to_str().unwrap());
    }

    #[test]
    fn extract_family_id_from_tower_family() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("tower.toml"),
            r#"
[tower]
family = "beardog-clan"
"#,
        )
        .unwrap();

        let family = extract_family_id(temp.path()).unwrap();
        assert_eq!(family, "beardog-clan");
    }

    #[test]
    fn extract_family_id_from_meta_family_id() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("tower.toml"),
            r#"
[meta]
family_id = "meta-family-xyz"
"#,
        )
        .unwrap();

        let family = extract_family_id(temp.path()).unwrap();
        assert_eq!(family, "meta-family-xyz");
    }

    #[test]
    fn extract_family_id_fallback_to_unknown() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("tower.toml"), "[other]\nkey = \"value\"").unwrap();

        let family = extract_family_id(temp.path()).unwrap();
        assert_eq!(family, "unknown");
    }

    #[test]
    fn extract_family_id_tower_takes_precedence_over_meta() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("tower.toml"),
            r#"
[tower]
family = "tower-family"
[meta]
family_id = "meta-family"
"#,
        )
        .unwrap();

        let family = extract_family_id(temp.path()).unwrap();
        assert_eq!(family, "tower-family");
    }

    #[test]
    fn extract_family_id_errors_when_tower_toml_missing() {
        let temp = TempDir::new().unwrap();
        let result = extract_family_id(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn extract_spore_id_fallback_unknown_for_root_path() {
        let id = extract_spore_id(Path::new("/")).unwrap();
        assert_eq!(id, "unknown");
    }
}
