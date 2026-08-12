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
    #[serde(default)]
    boot_order: Option<BootOrderConfig>,
}

#[derive(Deserialize)]
struct CompositionProfile {
    #[serde(default)]
    primals: Vec<String>,
}

/// cellMembrane `boot_order` configuration (shipped b7707ee).
///
/// Declares the authoritative startup sequence for compositions. When present,
/// this overrides the static `bootstrap_launch_order()` to ensure biomeOS
/// respects cellMembrane's deployment-proven ordering.
///
/// Format in `ecosystem_manifest.toml`:
/// ```toml
/// [boot_order]
/// sequence = ["beardog", "songbird", "skunkbat", "nestgate", ...]
/// strategy = "sequential"  # or "phased"
/// ```
#[derive(Deserialize, Clone)]
struct BootOrderConfig {
    sequence: Vec<String>,
    #[serde(default = "default_boot_strategy")]
    strategy: String,
}

fn default_boot_strategy() -> String {
    "sequential".to_string()
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

/// Extract `boot_order` from the ecosystem manifest if present.
///
/// cellMembrane ships `boot_order.sequence` (commit b7707ee) declaring the
/// deployment-proven startup ordering. When available, this takes precedence
/// over both static bootstrap hints and composition-profile discovery.
fn extract_boot_order(manifest_path: &Path) -> Option<Vec<String>> {
    let raw = std::fs::read_to_string(manifest_path).ok()?;
    let manifest: EcosystemManifestCompositions = toml::from_str(&raw).ok()?;

    let boot_order = manifest.boot_order?;
    if boot_order.sequence.is_empty() {
        return None;
    }

    let ordered: Vec<String> = boot_order
        .sequence
        .iter()
        .map(|name| normalize_primal_name(name))
        .filter(|name| name != "biomeos")
        .collect();

    if ordered.is_empty() {
        return None;
    }

    debug!(
        strategy = %boot_order.strategy,
        count = ordered.len(),
        "cellMembrane boot_order consumed — overrides bootstrap launch order"
    );

    Some(ordered)
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
///
/// Resolution priority:
/// 1. `[boot_order]` section (cellMembrane-authoritative, b7707ee) — ordered sequence
/// 2. `[compositions.*]` profiles — unordered set merged with bootstrap hints
/// 3. Static `bootstrap_launch_order()` — cold-start fallback
pub(super) fn try_discover_launch_set(mode: NucleusMode, bootstrap: &[&str]) -> Vec<String> {
    let Some(manifest_path) = resolve_manifest_path() else {
        return bootstrap.iter().map(|p| (*p).to_string()).collect();
    };

    // Priority 1: cellMembrane boot_order (authoritative deployment ordering)
    if let Some(boot_order) = extract_boot_order(&manifest_path) {
        debug!(
            manifest = %manifest_path.display(),
            mode = ?mode,
            count = boot_order.len(),
            "using cellMembrane boot_order as authoritative launch sequence"
        );
        return filter_boot_order_for_mode(mode, &boot_order, bootstrap);
    }

    // Priority 2: composition profiles (unordered set, merged with bootstrap)
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

/// Filter `boot_order` sequence to only include primals relevant to `mode`.
///
/// The full `boot_order` sequence declares ordering for the entire NUCLEUS.
/// For partial modes (Tower, Node, Nest), we preserve the order but filter
/// to only the primals that belong to that composition tier.
fn filter_boot_order_for_mode(
    _mode: NucleusMode,
    boot_order: &[String],
    bootstrap: &[&str],
) -> Vec<String> {
    let required: HashSet<&str> = bootstrap.iter().copied().collect();

    // Keep boot_order items that are in the required set for this mode
    let mut result: Vec<String> = boot_order
        .iter()
        .filter(|name| required.contains(name.as_str()))
        .cloned()
        .collect();

    // Append any required primals not in boot_order (safety net)
    for name in bootstrap {
        if !result.iter().any(|r| r == name) {
            result.push((*name).to_string());
        }
    }

    result
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
primals = ["bearDog", "songBird", "skunkBat", "swarmVine"]

[compositions.compute]
primals = ["toadStool", "barraCuda", "coralReef", "biomeOS"]

[compositions.nest]
primals = ["nestGate", "rhizoCrypt", "loamSpine", "sweetGrass"]

[compositions.full]
primals = ["bearDog", "songBird", "skunkBat", "swarmVine", "toadStool", "barraCuda", "coralReef", "nestGate", "rhizoCrypt", "loamSpine", "sweetGrass", "squirrel", "petalTongue", "biomeOS"]
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
            "swarmvine".to_string(),
            "nestgate".to_string(),
            "rhizocrypt".to_string(),
            "loamspine".to_string(),
            "sweetgrass".to_string(),
        ];
        let merged = merge_discovered_with_bootstrap(&bootstrap, discovered);
        assert_eq!(
            merged,
            bootstrap
                .iter()
                .map(|p| (*p).to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn discover_tower_from_manifest_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_manifest(dir.path(), "");
        let bootstrap = NucleusMode::Tower.bootstrap_launch_order();
        let discovered =
            discover_primals_from_manifest(NucleusMode::Tower, &path).expect("discover");
        assert_eq!(discovered.len(), 4);
        let merged = merge_discovered_with_bootstrap(&bootstrap, discovered);
        assert_eq!(merged, vec!["beardog", "songbird", "skunkbat", "swarmvine"]);
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
                "beardog",
                "songbird",
                "skunkbat",
                "swarmvine",
                "toadstool",
                "coralreef",
                "barracuda"
            ]
        );
    }

    #[test]
    fn core_mode_has_no_manifest_composition_keys() {
        assert!(composition_keys_for_mode(NucleusMode::Core).is_empty());
    }

    #[test]
    fn boot_order_overrides_composition_profiles() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_manifest(
            dir.path(),
            r#"
[boot_order]
sequence = ["bearDog", "songBird", "skunkBat", "swarmVine", "nestGate", "rhizoCrypt", "loamSpine", "sweetGrass", "toadStool", "coralReef", "barraCuda", "squirrel", "petalTongue"]
strategy = "sequential"
"#,
        );

        let boot_order = extract_boot_order(&path).expect("boot_order present");
        assert_eq!(boot_order.len(), 13);
        assert_eq!(boot_order[0], "beardog");
        assert_eq!(boot_order[3], "swarmvine");
    }

    #[test]
    fn boot_order_filters_for_tower_mode() {
        let boot_order = vec![
            "beardog".to_string(),
            "songbird".to_string(),
            "skunkbat".to_string(),
            "swarmvine".to_string(),
            "nestgate".to_string(),
            "toadstool".to_string(),
        ];
        let bootstrap = NucleusMode::Tower.bootstrap_launch_order();
        let result = filter_boot_order_for_mode(NucleusMode::Tower, &boot_order, &bootstrap);
        assert_eq!(result, vec!["beardog", "songbird", "skunkbat", "swarmvine"]);
    }

    #[test]
    fn boot_order_filters_for_node_mode_preserves_order() {
        let boot_order = vec![
            "beardog".to_string(),
            "songbird".to_string(),
            "skunkbat".to_string(),
            "swarmvine".to_string(),
            "nestgate".to_string(),
            "toadstool".to_string(),
            "coralreef".to_string(),
            "barracuda".to_string(),
        ];
        let bootstrap = NucleusMode::Node.bootstrap_launch_order();
        let result = filter_boot_order_for_mode(NucleusMode::Node, &boot_order, &bootstrap);
        assert_eq!(
            result,
            vec![
                "beardog",
                "songbird",
                "skunkbat",
                "swarmvine",
                "toadstool",
                "coralreef",
                "barracuda"
            ]
        );
    }

    #[test]
    fn boot_order_excludes_biomeos() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_manifest(
            dir.path(),
            r#"
[boot_order]
sequence = ["bearDog", "songBird", "biomeOS", "skunkBat"]
strategy = "sequential"
"#,
        );
        let boot_order = extract_boot_order(&path).expect("boot_order present");
        assert!(!boot_order.contains(&"biomeos".to_string()));
        assert_eq!(boot_order.len(), 3);
    }

    #[test]
    fn boot_order_appends_missing_required_primals() {
        let boot_order = vec!["beardog".to_string(), "songbird".to_string()];
        let bootstrap = NucleusMode::Tower.bootstrap_launch_order();
        let result = filter_boot_order_for_mode(NucleusMode::Tower, &boot_order, &bootstrap);
        assert_eq!(result, vec!["beardog", "songbird", "skunkbat", "swarmvine"]);
    }

    #[test]
    fn empty_boot_order_returns_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_manifest(
            dir.path(),
            "
[boot_order]
sequence = []
",
        );
        assert!(extract_boot_order(&path).is_none());
    }
}
