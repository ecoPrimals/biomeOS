// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Chimera CLI Commands
//!
//! Commands for managing chimera definitions and builds.

use biomeos_chimera::{ChimeraBuilder, ChimeraRegistry};
use std::path::Path;
use std::sync::Arc;

/// List all available chimera definitions
pub async fn handle_chimera_list() -> anyhow::Result<()> {
    let definitions_dir = Path::new("chimeras/definitions");

    if !definitions_dir.exists() {
        println!("❌ Chimera definitions directory not found: {definitions_dir:?}");
        println!("   Run from biomeOS root directory");
        return Ok(());
    }

    match ChimeraRegistry::from_directory(definitions_dir) {
        Ok(registry) => {
            println!("🧬 Available Chimeras ({}):", registry.len());
            println!();

            for (id, summary) in registry.summary() {
                println!("  {} {}", if summary.uses_arrays { "🔄" } else { "🧬" }, id);
                println!("     Name: {}", summary.name);
                println!("     Version: {}", summary.version);
                println!("     Primals: {}", summary.primals.join(" + "));
                if summary.uses_arrays {
                    println!("     Arrays: ✅ (supports multiple instances)");
                }
                println!();
            }
        }
        Err(e) => {
            println!("❌ Failed to load chimera registry: {e}");
        }
    }

    Ok(())
}

/// Show details for a specific chimera
pub async fn handle_chimera_show(id: &str) -> anyhow::Result<()> {
    let definitions_dir = Path::new("chimeras/definitions");

    let registry = ChimeraRegistry::from_directory(definitions_dir)?;

    match registry.get(id) {
        Some(def) => {
            println!("🧬 Chimera: {}", def.chimera.id);
            println!("   Name: {}", def.chimera.name);
            println!("   Version: {}", def.chimera.version);
            println!();
            println!("   Description:");
            for line in def.chimera.description.lines() {
                println!("     {line}");
            }
            println!();

            println!("   Components:");
            for (name, component) in &def.components {
                println!("     📦 {} ({})", name, component.version);
                for module in &component.modules {
                    println!("        └─ {}: {}", module.name, module.description);
                }
            }
            println!();

            println!("   Fusion Bindings:");
            for (name, binding) in &def.fusion.bindings {
                let provider = binding.provider.as_deref().unwrap_or("(none)");
                println!("     🔗 {}: {} → {:?}", name, provider, binding.consumers);
            }
            println!();

            println!("   API Endpoints:");
            for endpoint in &def.fusion.api.endpoints {
                println!(
                    "     📡 {}({}) -> {}",
                    endpoint.name,
                    endpoint.params.join(", "),
                    endpoint.returns
                );
            }
        }
        None => {
            println!("❌ Chimera not found: {id}");
            println!("   Run 'biomeos chimera list' to see available chimeras");
        }
    }

    Ok(())
}

/// Build a chimera
pub async fn handle_chimera_build(id: &str) -> anyhow::Result<()> {
    let definitions_dir = Path::new("chimeras/definitions");
    let registry = ChimeraRegistry::from_directory(definitions_dir)?;

    match registry.get(id) {
        Some(def) => {
            println!("🔨 Building chimera: {id}");

            let builder = ChimeraBuilder::new(Arc::clone(&def))
                .output_dir("bin/chimeras")
                .primals_dir("bin/primals");

            // Check primals first
            match builder.check_primals() {
                Ok(paths) => {
                    println!("   ✅ Found {} required primal binaries", paths.len());
                }
                Err(e) => {
                    println!("   ❌ Missing primals: {e}");
                    println!("   Run './bin/pull-primals.sh --all' first");
                    return Ok(());
                }
            }

            // Build
            match builder.build() {
                Ok(result) => {
                    println!("   ✅ Built in {:?}", result.duration);
                    println!("   📦 Output: {:?}", result.binary_path);
                    for warning in &result.warnings {
                        println!("   ⚠️  {warning}");
                    }
                }
                Err(e) => {
                    println!("   ❌ Build failed: {e}");
                }
            }
        }
        None => {
            println!("❌ Chimera not found: {id}");
        }
    }

    Ok(())
}

/// Parse chimera ID from YAML content (testable pure function).
#[cfg(test)]
pub(crate) fn parse_chimera_id_from_yaml(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("id:") {
            return trimmed
                .split(':')
                .nth(1)
                .map(|s| s.trim().trim_matches('"').to_string());
        }
    }
    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::await_holding_lock)] // SAFETY: CWD_TEST_LOCK serializes cwd-changing tests; run with --test-threads=1
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::tempdir;

    struct RestoreCwd(std::path::PathBuf);
    impl Drop for RestoreCwd {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.0);
        }
    }

    static CWD_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_parse_chimera_id_from_yaml() {
        let yaml = r#"
chimera:
  id: my-chimera
  name: Test Chimera
  version: "1.0.0"
"#;
        assert_eq!(
            parse_chimera_id_from_yaml(yaml),
            Some("my-chimera".to_string())
        );
    }

    #[test]
    fn test_parse_chimera_id_from_yaml_quoted() {
        let yaml = r#"chimera:
  id: "quoted-id"
  name: Test"#;
        assert_eq!(
            parse_chimera_id_from_yaml(yaml),
            Some("quoted-id".to_string())
        );
    }

    #[test]
    fn test_parse_chimera_id_from_yaml_missing() {
        let yaml = "chimera:\n  name: No ID";
        assert_eq!(parse_chimera_id_from_yaml(yaml), None);
    }

    #[test]
    fn test_parse_chimera_id_from_yaml_first_line() {
        let yaml = "id: top-level-id\nname: Other";
        assert_eq!(
            parse_chimera_id_from_yaml(yaml),
            Some("top-level-id".to_string())
        );
    }

    #[test]
    fn test_parse_chimera_id_from_yaml_empty_value() {
        let yaml = "chimera:\n  id: \n  name: Test";
        assert_eq!(parse_chimera_id_from_yaml(yaml), Some("".to_string()));
    }

    #[test]
    fn test_parse_chimera_id_from_yaml_whitespace_only() {
        // "id:   " -> nth(1)="   ", trim gives "" (whitespace trimmed)
        let yaml = "chimera:\n  id:   \n  name: Test";
        assert_eq!(parse_chimera_id_from_yaml(yaml), Some("".to_string()));
    }

    #[test]
    fn test_parse_chimera_id_from_yaml_simple_colon_value() {
        // id: "my-chimera" (no colon in value) works
        let yaml = r#"chimera:
  id: "my-chimera"
  name: Test"#;
        assert_eq!(
            parse_chimera_id_from_yaml(yaml),
            Some("my-chimera".to_string())
        );
    }

    #[test]
    fn test_parse_chimera_id_from_yaml_multiple_id_lines_takes_first() {
        let yaml = r#"id: first
other: stuff
id: second"#;
        assert_eq!(parse_chimera_id_from_yaml(yaml), Some("first".to_string()));
    }

    #[test]
    fn test_create_test_chimera_yaml_produces_valid_structure() {
        let temp = tempdir().unwrap();
        let path = create_test_chimera_yaml(temp.path(), "test-validate");
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            parse_chimera_id_from_yaml(&content),
            Some("test-validate".to_string())
        );
        assert!(content.contains("chimera:"));
        assert!(content.contains("components:"));
        assert!(content.contains("fusion:"));
    }

    fn create_test_chimera_yaml(dir: &Path, id: &str) -> std::path::PathBuf {
        let yaml = format!(
            r#"
chimera:
  id: {id}
  name: Test {id}
  version: "1.0.0"
  description: Test chimera description

components:
  beardog:
    source: primals/beardog
    version: ">=1.0.0"
    modules: []

fusion:
  bindings: {{}}
  api:
    endpoints: []
"#
        );
        let path = dir
            .join("chimeras")
            .join("definitions")
            .join(format!("{id}.yaml"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        path
    }

    #[tokio::test]
    async fn test_handle_chimera_list_nonexistent_dir() {
        // When definitions dir doesn't exist, should return Ok (graceful message)
        let result = handle_chimera_list().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_handle_chimera_list_with_definitions() {
        let _guard = CWD_TEST_LOCK.lock().unwrap();
        let temp = tempdir().unwrap();
        let _restore = RestoreCwd(std::env::current_dir().unwrap());
        std::env::set_current_dir(temp.path()).unwrap();
        std::fs::create_dir_all("chimeras/definitions").unwrap();
        create_test_chimera_yaml(temp.path(), "test-chimera");

        let result = handle_chimera_list().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_handle_chimera_show_not_found() {
        let _guard = CWD_TEST_LOCK.lock().unwrap();
        let temp = tempdir().unwrap();
        let defs_dir = temp.path().join("chimeras/definitions");
        std::fs::create_dir_all(&defs_dir).unwrap();
        let _restore = RestoreCwd(std::env::current_dir().unwrap());
        std::env::set_current_dir(temp.path()).unwrap();

        let result = handle_chimera_show("nonexistent-chimera").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_handle_chimera_show_found() {
        let _guard = CWD_TEST_LOCK.lock().unwrap();
        let temp = tempdir().unwrap();
        let _restore = RestoreCwd(std::env::current_dir().unwrap());
        std::env::set_current_dir(temp.path()).unwrap();
        std::fs::create_dir_all("chimeras/definitions").unwrap();
        create_test_chimera_yaml(temp.path(), "my-chimera");

        let result = handle_chimera_show("my-chimera").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_handle_chimera_show_missing_definitions_dir() {
        let _guard = CWD_TEST_LOCK.lock().unwrap();
        let temp = tempdir().unwrap();
        let _restore = RestoreCwd(std::env::current_dir().unwrap());
        std::env::set_current_dir(temp.path()).unwrap();
        // No chimeras/definitions - from_directory will fail

        let result = handle_chimera_show("any-id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_handle_chimera_build_not_found() {
        let _guard = CWD_TEST_LOCK.lock().unwrap();
        let temp = tempdir().unwrap();
        let defs_dir = temp.path().join("chimeras/definitions");
        std::fs::create_dir_all(&defs_dir).unwrap();
        let _restore = RestoreCwd(std::env::current_dir().unwrap());
        std::env::set_current_dir(temp.path()).unwrap();

        let result = handle_chimera_build("nonexistent").await;
        assert!(
            result.is_ok(),
            "build with nonexistent chimera should return Ok (prints message)"
        );
    }
}
