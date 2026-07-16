// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Runtime launch-set discovery from `ecosystem_manifest.toml`.
//!
//! Bootstrap launch order in [`super::NucleusMode::bootstrap_launch_order`] is the
//! cold-start fallback when no manifest is available. Manifest compositions are
//! bootstrap hints for startup ordering — not capability routing dependencies.

use super::NucleusMode;
use biomeos_types::env_config::vars;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Deserialize)]
struct EcosystemManifestCompositions {
    #[serde(default)]
    compositions: HashMap<String, CompositionProfile>,
}

#[derive(Deserialize)]
struct CompositionProfile {
    #[serde(default)]
    primals: Vec<String>,
}

/// Composition keys in `ecosystem_manifest.toml` used for each nucleus mode.
fn composition_keys_for_mode(mode: NucleusMode) -> &'static [&'static str] {
    match mode {
        NucleusMode::Tower => &["tower"],
        NucleusMode::Node => &["tower", "compute"],
        NucleusMode::Nest => &["tower", "nest"],
        // Legacy 5-primal profile — no manifest composition; bootstrap only.
        NucleusMode::Core => &[],
        NucleusMode::Full => &["full"],
    }
}

/// Normalize manifest primal identifiers to lowercase canonical names.
fn normalize_primal_name(name: &str) -> String {
    name.chars().flat_map(char::to_lowercase).collect()
}

/// Resolve path to `ecosystem_manifest.toml` when present.
fn resolve_manifest_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var(vars::ECOSYSTEM_MANIFEST_PATH) {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    walk_up_for_manifest()
}

/// Walk up from CWD looking for `infra/wateringHole/ecosystem_manifest.toml`.
fn walk_up_for_manifest() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..8 {
        let candidate = dir.join("infra/wateringHole/ecosystem_manifest.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Load and union primal names from manifest composition profiles for `mode`.
fn discover_primals_from_manifest(mode: NucleusMode, manifest_path: &Path) -> Option<Vec<String>> {
    let keys = composition_keys_for_mode(mode);
    if keys.is_empty() {
        return None;
    }

    let raw = std::fs::read_to_string(manifest_path).ok()?;
    let manifest: EcosystemManifestCompositions = toml::from_str(&raw).ok()?;

    let mut discovered = HashSet::new();
    for key in keys {
        if let Some(profile) = manifest.compositions.get(*key) {
            for name in &profile.primals {
                let normalized = normalize_primal_name(name);
                if normalized != "biomeos" {
                    discovered.insert(normalized);
                }
            }
        }
    }

    if discovered.is_empty() {
        return None;
    }

    Some(discovered.into_iter().collect())
}

/// Merge manifest-discovered primals with bootstrap cold-start order.
///
/// Bootstrap order is always preserved for known mode requirements; manifest
/// entries refine ordering and may append additional primals at the end.
pub(super) fn merge_discovered_with_bootstrap(
    bootstrap: &[&str],
    discovered: Vec<String>,
) -> Vec<String> {
    let discovered_set: HashSet<String> = discovered.into_iter().collect();

    let mut result = Vec::with_capacity(bootstrap.len());
    for name in bootstrap {
        if discovered_set.contains(*name) {
            result.push((*name).to_string());
        }
    }
    for name in bootstrap {
        if !discovered_set.contains(*name) {
            result.push((*name).to_string());
        }
    }
    for name in &discovered_set {
        if !bootstrap.contains(&name.as_str()) {
            result.push(name.clone());
        }
    }
    result
}

/// Attempt runtime discovery from `ecosystem_manifest.toml`.
pub(super) fn try_discover_launch_set(mode: NucleusMode, bootstrap: &[&str]) -> Vec<String> {
    let Some(manifest_path) = resolve_manifest_path() else {
        return bootstrap.iter().map(|p| (*p).to_string()).collect();
    };

    let Some(discovered) = discover_primals_from_manifest(mode, &manifest_path) else {
        debug!(
            manifest = %manifest_path.display(),
            mode = ?mode,
            "ecosystem manifest found but no composition profile matched — using bootstrap order"
        );
        return bootstrap.iter().map(|p| (*p).to_string()).collect();
    };

    debug!(
        manifest = %manifest_path.display(),
        mode = ?mode,
        count = discovered.len(),
        "resolved launch set from ecosystem manifest compositions"
    );
    merge_discovered_with_bootstrap(bootstrap, discovered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_manifest(dir: &Path, extra: &str) -> PathBuf {
        let path = dir.join("ecosystem_manifest.toml");
        let mut file = std::fs::File::create(&path).expect("create manifest");
        write!(
            file,
            r#"
[compositions.tower]
primals = ["bearDog", "songBird", "skunkBat"]

[compositions.compute]
primals = ["toadStool", "barraCuda", "coralReef", "biomeOS"]

[compositions.nest]
primals = ["nestGate", "rhizoCrypt", "loamSpine", "sweetGrass"]

[compositions.full]
primals = ["bearDog", "songBird", "skunkBat", "toadStool", "barraCuda", "coralReef", "nestGate", "rhizoCrypt", "loamSpine", "sweetGrass", "squirrel", "petalTongue", "biomeOS"]
{extra}
"#
        )
        .expect("write manifest");
        path
    }

    #[test]
    fn merge_preserves_bootstrap_order_for_nest_with_missing_manifest_entry() {
        let bootstrap = NucleusMode::Nest.bootstrap_launch_order();
        let discovered = vec![
            "beardog".to_string(),
            "songbird".to_string(),
            "skunkbat".to_string(),
            "nestgate".to_string(),
            "rhizocrypt".to_string(),
            "loamspine".to_string(),
            "sweetgrass".to_string(),
        ];
        let merged = merge_discovered_with_bootstrap(&bootstrap, discovered);
        assert_eq!(merged, bootstrap.iter().map(|p| (*p).to_string()).collect::<Vec<_>>());
    }

    #[test]
    fn discover_tower_from_manifest_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_manifest(dir.path(), "");
        let bootstrap = NucleusMode::Tower.bootstrap_launch_order();
        let discovered =
            discover_primals_from_manifest(NucleusMode::Tower, &path).expect("discover");
        assert_eq!(discovered.len(), 3);
        let merged = merge_discovered_with_bootstrap(&bootstrap, discovered);
        assert_eq!(merged, vec!["beardog", "songbird", "skunkbat"]);
    }

    #[test]
    fn discover_node_unions_tower_and_compute_excluding_biomeos() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_manifest(dir.path(), "");
        let bootstrap = NucleusMode::Node.bootstrap_launch_order();
        let discovered =
            discover_primals_from_manifest(NucleusMode::Node, &path).expect("discover");
        let merged = merge_discovered_with_bootstrap(&bootstrap, discovered);
        assert_eq!(
            merged,
            vec![
                "beardog", "songbird", "skunkbat", "toadstool", "coralreef", "barracuda"
            ]
        );
    }

    #[test]
    fn core_mode_has_no_manifest_composition_keys() {
        assert!(composition_keys_for_mode(NucleusMode::Core).is_empty());
    }
}
