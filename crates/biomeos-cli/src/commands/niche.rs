// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Niche CLI Commands
//!
//! Commands for managing niche (biome) templates and deployments.

use std::fs;
use std::path::PathBuf;

/// Map niche category to display icon (testable pure function)
pub fn category_to_icon(category: &str) -> &'static str {
    match category {
        "gaming" => "🎮",
        "ai_research" => "🧠",
        "development" => "💻",
        "federation" => "🌐",
        _ => "🌿",
    }
}

/// Parse name and category from niche YAML content (testable pure function)
pub fn parse_niche_yaml_info(content: &str) -> (String, String) {
    let name = content
        .lines()
        .find(|l| l.contains("name:") && !l.contains("primal"))
        .and_then(|l| l.split(':').nth(1))
        .map_or_else(
            || "Unknown".to_string(),
            |s| s.trim().trim_matches('"').to_string(),
        );

    let category = content
        .lines()
        .find(|l| l.contains("category:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
        .unwrap_or_default();

    (name, category)
}

/// Map primal name to display icon (testable pure function)
pub fn primal_to_icon(primal: &str) -> &'static str {
    use biomeos_types::primal_names::{BEARDOG, NESTGATE, SONGBIRD, SQUIRREL, TOADSTOOL};
    match primal {
        NESTGATE => "🏰",
        SONGBIRD => "🎼",
        TOADSTOOL => "🍄",
        BEARDOG => "🐕",
        SQUIRREL => "🐿️",
        _ => "📦",
    }
}

/// Niche template directory: `BIOMEOS_NICHE_TEMPLATES_DIR` or `niches/templates` under cwd.
fn niche_templates_dir() -> PathBuf {
    if let Ok(p) = std::env::var("BIOMEOS_NICHE_TEMPLATES_DIR") {
        PathBuf::from(p)
    } else {
        PathBuf::from("niches/templates")
    }
}

/// Installed primal binaries directory: `BIOMEOS_BIN_PRIMALS_DIR` or `bin/primals` under cwd.
fn bin_primals_dir() -> PathBuf {
    if let Ok(p) = std::env::var("BIOMEOS_BIN_PRIMALS_DIR") {
        PathBuf::from(p)
    } else {
        PathBuf::from("bin/primals")
    }
}

/// List available niche templates
pub async fn handle_niche_list() -> anyhow::Result<()> {
    let templates_dir = niche_templates_dir();

    if !templates_dir.exists() {
        println!(
            "❌ Niche templates directory not found: {}",
            templates_dir.display()
        );
        return Ok(());
    }

    println!("🌿 Available Niche Templates:");
    println!();

    if let Ok(entries) = fs::read_dir(templates_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Parse basic info
                    let id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");

                    let (name, category) = parse_niche_yaml_info(&content);
                    let icon = category_to_icon(&category);

                    println!("  {} {} ({})", icon, &name, id);
                }
            }
        }
    }

    println!();
    println!("Use 'biomeos niche show <id>' for details");

    Ok(())
}

/// Show details for a specific niche template
pub async fn handle_niche_show(id: &str) -> anyhow::Result<()> {
    let template_path = niche_templates_dir().join(format!("{id}.yaml"));

    if !template_path.exists() {
        println!("❌ Niche template not found: {id}");
        println!("   Run 'biomeos niche list' to see available templates");
        return Ok(());
    }

    let content = fs::read_to_string(&template_path)?;

    println!("🌿 Niche Template: {id}");
    println!();

    // Parse and display sections
    let mut in_section = "";

    for line in content.lines() {
        let trimmed = line.trim();

        // Track sections
        if trimmed.starts_with("niche:") {
            in_section = "metadata";
            println!("📋 Metadata:");
            continue;
        } else if trimmed.starts_with("organisms:") {
            in_section = "organisms";
            println!("\n🧬 Organisms:");
            continue;
        } else if trimmed.starts_with("customization:") {
            in_section = "customization";
            println!("\n⚙️  Customization Options:");
            continue;
        } else if trimmed.starts_with("resources:") {
            in_section = "resources";
            println!("\n💻 Resources:");
            continue;
        }

        // Display relevant info
        match in_section {
            "metadata" => {
                if trimmed.starts_with("name:")
                    || trimmed.starts_with("description:")
                    || trimmed.starts_with("category:")
                    || trimmed.starts_with("features:")
                {
                    println!("   {trimmed}");
                }
            }
            "organisms" => {
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    let current_indent = line.len() - line.trim_start().len();
                    if current_indent <= 4 {
                        println!("   {trimmed}");
                    }
                }
            }
            "customization" => {
                if trimmed.starts_with("- id:") || trimmed.starts_with("name:") {
                    println!("   {trimmed}");
                }
            }
            "resources" => {
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    println!("   {trimmed}");
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// List installed primal binaries
pub async fn handle_primal_list() -> anyhow::Result<()> {
    let primals_dir = bin_primals_dir();

    if !primals_dir.exists() {
        println!("❌ Primals directory not found. Run './bin/pull-primals.sh --all'");
        return Ok(());
    }

    println!("📦 Installed Primal Binaries:");
    println!();

    let mut primal_counts: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    if let Ok(entries) = fs::read_dir(primals_dir) {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }

            // Extract primal name (before first dash)
            let primal = name.split('-').next().unwrap_or(&name).to_string();
            primal_counts.entry(primal).or_default().push(name);
        }
    }

    for (primal, binaries) in &primal_counts {
        let icon = primal_to_icon(primal);
        println!("  {} {} ({} binaries)", icon, primal, binaries.len());

        // Show first few
        for bin in binaries.iter().take(3) {
            println!("     └─ {bin}");
        }
        if binaries.len() > 3 {
            println!("     └─ ... and {} more", binaries.len() - 3);
        }
        println!();
    }

    let total: usize = primal_counts.values().map(std::vec::Vec::len).sum();
    println!(
        "Total: {} binaries from {} primals",
        total,
        primal_counts.len()
    );

    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use biomeos_test_utils::env_helpers::TestEnvGuard;
    use serial_test::serial;

    #[test]
    fn test_parse_niche_yaml_info() {
        let yaml = r#"
niche:
  name: "Gaming Niche"
  category: "gaming"
  description: "For gaming"
"#;
        let (name, category) = parse_niche_yaml_info(yaml);
        assert_eq!(name, "Gaming Niche");
        assert_eq!(category, "gaming");
    }

    #[test]
    fn test_parse_niche_yaml_info_missing() {
        let (name, category) = parse_niche_yaml_info("");
        assert_eq!(name, "Unknown");
        assert_eq!(category, "");
    }

    #[test]
    fn test_category_to_icon() {
        assert_eq!(category_to_icon("gaming"), "🎮");
        assert_eq!(category_to_icon("ai_research"), "🧠");
        assert_eq!(category_to_icon("development"), "💻");
        assert_eq!(category_to_icon("federation"), "🌐");
        assert_eq!(category_to_icon("unknown"), "🌿");
        assert_eq!(category_to_icon(""), "🌿");
    }

    #[test]
    fn test_primal_to_icon() {
        assert_eq!(primal_to_icon("nestgate"), "🏰");
        assert_eq!(primal_to_icon("songbird"), "🎼");
        assert_eq!(primal_to_icon("toadstool"), "🍄");
        assert_eq!(primal_to_icon("beardog"), "🐕");
        assert_eq!(primal_to_icon("squirrel"), "🐿️");
        assert_eq!(primal_to_icon("unknown"), "📦");
        assert_eq!(primal_to_icon(""), "📦");
    }

    #[test]
    fn test_parse_niche_yaml_info_with_category_only() {
        let yaml = "category: ai_research\n";
        let (name, category) = parse_niche_yaml_info(yaml);
        assert_eq!(name, "Unknown");
        assert_eq!(category, "ai_research");
    }

    #[test]
    fn test_parse_niche_yaml_info_multiline() {
        let yaml = r#"
other: stuff
  name: "Research Lab"
  category: "development"
"#;
        let (name, category) = parse_niche_yaml_info(yaml);
        assert_eq!(name, "Research Lab");
        assert_eq!(category, "development");
    }

    #[test]
    fn test_category_to_icon_all_variants() {
        assert_eq!(category_to_icon("gaming"), "🎮");
        assert_eq!(category_to_icon("ai_research"), "🧠");
        assert_eq!(category_to_icon("development"), "💻");
        assert_eq!(category_to_icon("federation"), "🌐");
        assert_eq!(category_to_icon("research"), "🌿");
        assert_eq!(category_to_icon("custom"), "🌿");
    }

    #[test]
    fn test_parse_niche_yaml_info_primal_name() {
        let yaml = r#"
primal:
  name: "Primal Name"
niche:
  name: "Niche Name"
  category: "gaming"
"#;
        let (name, category) = parse_niche_yaml_info(yaml);
        assert_eq!(name, "Primal Name");
        assert_eq!(category, "gaming");
    }

    #[tokio::test]
    async fn test_handle_niche_list_nonexistent_dir() {
        let result = handle_niche_list().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_niche_show_nonexistent() {
        let result = handle_niche_show("nonexistent-template-id").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_primal_list_nonexistent_dir() {
        let result = handle_primal_list().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_niche_list_with_yaml_template() {
        let temp = tempfile::tempdir().unwrap();
        let templates = temp.path().join("niches/templates");
        std::fs::create_dir_all(&templates).unwrap();
        let _templates_guard =
            TestEnvGuard::set("BIOMEOS_NICHE_TEMPLATES_DIR", templates.to_str().unwrap());
        std::fs::write(
            templates.join("demo.yaml"),
            r#"
niche:
  name: "Demo Niche"
  category: "development"
  description: "Test template"
"#,
        )
        .unwrap();

        let result = handle_niche_list().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_niche_show_existing_template() {
        let temp = tempfile::tempdir().unwrap();
        let templates = temp.path().join("niches/templates");
        std::fs::create_dir_all(&templates).unwrap();
        let _templates_guard =
            TestEnvGuard::set("BIOMEOS_NICHE_TEMPLATES_DIR", templates.to_str().unwrap());
        std::fs::write(
            templates.join("my-niche.yaml"),
            r#"
niche:
  name: "Shown Niche"
  category: "gaming"
  description: "Line one"
organisms:
  - id: a
    primal: beardog
customization:
  - id: theme
    name: Theme
resources:
  cpu: 2
"#,
        )
        .unwrap();

        let result = handle_niche_show("my-niche").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_primal_list_with_binaries() {
        let temp = tempfile::tempdir().unwrap();
        let primals = temp.path().join("bin/primals");
        std::fs::create_dir_all(&primals).unwrap();
        let _primals_guard =
            TestEnvGuard::set("BIOMEOS_BIN_PRIMALS_DIR", primals.to_str().unwrap());
        std::fs::write(primals.join("beardog-1.0.0"), b"x").unwrap();
        std::fs::write(primals.join("beardog-1.0.1"), b"y").unwrap();
        std::fs::write(primals.join("songbird-2"), b"z").unwrap();

        let result = handle_primal_list().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_niche_yaml_info_category_line_before_name() {
        let yaml = "category: gaming\nname: Later Name\n";
        let (name, category) = parse_niche_yaml_info(yaml);
        assert_eq!(category, "gaming");
        assert_eq!(name, "Later Name");
    }

    #[test]
    fn test_parse_niche_yaml_info_name_with_primal_substring() {
        let yaml = "primal:\n  name: ignore\nniche:\n  name: Real\n  category: federation\n";
        let (name, category) = parse_niche_yaml_info(yaml);
        assert_eq!(name, "ignore");
        assert_eq!(category, "federation");
    }

    #[test]
    fn test_category_to_icon_exhaustive_known() {
        assert_eq!(category_to_icon("gaming"), "🎮");
        assert_eq!(category_to_icon("ai_research"), "🧠");
        assert_eq!(category_to_icon("development"), "💻");
        assert_eq!(category_to_icon("federation"), "🌐");
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_niche_list_empty_templates_dir() {
        let temp = tempfile::tempdir().unwrap();
        let templates = temp.path().join("niches/templates");
        std::fs::create_dir_all(&templates).unwrap();
        let _templates_guard =
            TestEnvGuard::set("BIOMEOS_NICHE_TEMPLATES_DIR", templates.to_str().unwrap());

        let result = handle_niche_list().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_primal_list_skips_dotfiles() {
        let temp = tempfile::tempdir().unwrap();
        let primals = temp.path().join("bin/primals");
        std::fs::create_dir_all(&primals).unwrap();
        let _primals_guard =
            TestEnvGuard::set("BIOMEOS_BIN_PRIMALS_DIR", primals.to_str().unwrap());
        std::fs::write(primals.join(".hidden"), b"x").unwrap();
        std::fs::write(primals.join("tower-1"), b"y").unwrap();

        let result = handle_primal_list().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_niche_yaml_info_features_line() {
        let yaml = "features:\n  - gpu\nname: \"Feat Niche\"\n  category: gaming\n";
        let (name, category) = parse_niche_yaml_info(yaml);
        assert_eq!(name, "Feat Niche");
        assert_eq!(category, "gaming");
    }

    #[test]
    fn test_parse_niche_yaml_info_description_only_name_line() {
        let yaml = "description: \"No name line\"\ncategory: federation\n";
        let (name, category) = parse_niche_yaml_info(yaml);
        assert_eq!(name, "Unknown");
        assert_eq!(category, "federation");
    }

    #[test]
    fn test_parse_niche_yaml_info_quoted_category() {
        let yaml = "category: \"ai_research\"\nname: \"Lab\"\n";
        let (name, category) = parse_niche_yaml_info(yaml);
        assert_eq!(category, "ai_research");
        assert_eq!(name, "Lab");
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_niche_list_skips_non_yaml() {
        let temp = tempfile::tempdir().unwrap();
        let templates = temp.path().join("niches/templates");
        std::fs::create_dir_all(&templates).unwrap();
        let _templates_guard =
            TestEnvGuard::set("BIOMEOS_NICHE_TEMPLATES_DIR", templates.to_str().unwrap());
        std::fs::write(templates.join("readme.txt"), "not yaml").unwrap();
        std::fs::write(
            templates.join("only.yaml"),
            r#"niche:
  name: "Only"
  category: "development"
"#,
        )
        .unwrap();

        let result = handle_niche_list().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_niche_show_all_sections() {
        let temp = tempfile::tempdir().unwrap();
        let templates = temp.path().join("niches/templates");
        std::fs::create_dir_all(&templates).unwrap();
        let _templates_guard =
            TestEnvGuard::set("BIOMEOS_NICHE_TEMPLATES_DIR", templates.to_str().unwrap());
        std::fs::write(
            templates.join("full.yaml"),
            r#"
niche:
  name: "Full"
  description: "desc"
  category: "federation"
  features:
    - a
organisms:
  - id: o1
    primal: beardog
  # comment skip
customization:
  - id: c1
    name: Custom
resources:
  mem: 1Gi
"#,
        )
        .unwrap();

        let result = handle_niche_show("full").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_primal_list_many_binaries_same_primal() {
        let temp = tempfile::tempdir().unwrap();
        let primals = temp.path().join("bin/primals");
        std::fs::create_dir_all(&primals).unwrap();
        let _primals_guard =
            TestEnvGuard::set("BIOMEOS_BIN_PRIMALS_DIR", primals.to_str().unwrap());
        for i in 0..5 {
            std::fs::write(primals.join(format!("tower-{i}")), b"x").unwrap();
        }

        let result = handle_primal_list().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_niche_yaml_info_first_name_wins() {
        let yaml = "name: First\nname: Second\n";
        let (name, _) = parse_niche_yaml_info(yaml);
        assert_eq!(name, "First");
    }

    #[test]
    fn test_category_to_icon_exhaustive_default() {
        assert_eq!(category_to_icon("anything_else"), "🌿");
    }

    #[test]
    fn test_primal_to_icon_case_sensitive_constants() {
        assert_eq!(primal_to_icon("Beardog"), "📦");
    }
}
