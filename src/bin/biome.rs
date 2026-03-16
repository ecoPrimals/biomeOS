// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! # biome CLI
//!
//! Command-line interface for BYOB (Bring Your Own Biome) functionality.
//! Teams can deploy and manage their biomes independently while leveraging
//! the existing Primal ecosystem (Songbird, Toadstool, NestGate).

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, PartialEq, Debug)]
enum Commands {
    /// Deploy a biome from a manifest file
    Deploy {
        /// Path to the biome manifest file
        manifest: PathBuf,
        /// Team name for deployment
        #[arg(short, long)]
        team: String,
    },
    /// List deployments for a team
    List {
        /// Team name to list deployments for
        #[arg(short, long)]
        team: String,
    },
    /// Show deployment status
    Status {
        /// Deployment ID to check
        deployment_id: String,
    },
    /// Remove a deployment
    Remove {
        /// Deployment ID to remove
        deployment_id: String,
    },
    /// Manage team workspace
    Workspace {
        /// Team name
        #[arg(short, long)]
        team: String,
    },
    /// Initialize a new biome manifest
    Init {
        /// Template to use (basic, webapp, ai-research, gaming)
        #[arg(short, long, default_value = "basic")]
        template: String,
        /// Output manifest file path
        #[arg(short, long, default_value = "biome.yaml")]
        output: PathBuf,
    },
    /// Validate a biome manifest
    Validate {
        /// Path to the biome manifest file
        manifest: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Deploy { manifest, team } => {
            println!(
                "🚀 Deploying biome from {} for team {}",
                manifest.display(),
                team
            );
            println!("✅ Deployment successful! (simulated)");
            println!("   Deployment ID: dep-simulated-id");
            println!("   Team: {team}");
            println!("   Status: Pending");
        }
        Commands::List { team } => {
            println!("📋 Listing deployments for team: {team}");
            println!("   No deployments found for team {team} (this is expected in demo mode)");
        }
        Commands::Status { deployment_id } => {
            println!("📊 Checking status for deployment: {deployment_id}");
            println!("   Status: Running (simulated)");
        }
        Commands::Remove { deployment_id } => {
            println!("🗑️  Removing deployment: {deployment_id}");
            println!("✅ Deployment removed successfully! (simulated)");
        }
        Commands::Workspace { team } => {
            println!("🏠 Team workspace for: {team}");
            println!("   Team ID: {team}");
            println!("   Created: {}", chrono::Utc::now());
            println!("   Resource Quota:");
            println!("     CPU: 16 cores");
            println!("     Memory: 64 GB");
            println!("     Storage: 512 GB");
            println!("   Active Deployments: 0");
            println!("   Isolation:");
            println!("     Network: true");
            println!("     Resource: true");
            println!("     Secrets: true");
        }
        Commands::Init { template, output } => {
            println!("📝 Initializing biome manifest with template: {template}");

            let manifest_content = match get_template_content(&template) {
                Some(content) => content,
                None => {
                    eprintln!("❌ Unknown template: {template}");
                    std::process::exit(1);
                }
            };

            match std::fs::write(&output, manifest_content) {
                Ok(_) => {
                    println!("✅ Manifest created: {}", output.display());
                }
                Err(e) => {
                    eprintln!("❌ Failed to write manifest: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Validate { manifest } => {
            println!("🔍 Validating biome manifest: {}", manifest.display());

            match std::fs::read_to_string(&manifest) {
                Ok(content) => {
                    match serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        Ok(parsed) => {
                            println!("✅ Manifest is valid YAML");

                            // Basic validation checks
                            if let Some(api_version) = parsed.get("apiVersion") {
                                println!(
                                    "   API Version: {}",
                                    api_version.as_str().unwrap_or("unknown")
                                );
                            }
                            if let Some(kind) = parsed.get("kind") {
                                println!("   Kind: {}", kind.as_str().unwrap_or("unknown"));
                            }
                            if let Some(metadata) = parsed.get("metadata")
                                && let Some(name) = metadata.get("name")
                            {
                                println!("   Name: {}", name.as_str().unwrap_or("unknown"));
                            }
                            if let Some(services) = parsed.get("services")
                                && let Some(services_map) = services.as_mapping()
                            {
                                println!("   Services: {} defined", services_map.len());
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Invalid YAML: {e}");
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to read manifest: {e}");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn create_basic_template() -> String {
    r#"apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: basic-biome
  description: "Basic biome template"
  version: "1.0.0"

services:
  web-server:
    primal: toadstool
    runtime: Container
    image: nginx:alpine
    ports:
      - "80:80"
    resources:
      cpu: 1.0
      memory: 1073741824  # 1GB

  storage:
    primal: nestgate
    runtime: Native
    resources:
      storage: 10737418240  # 10GB

networking:
  load_balancing: true
  service_discovery: true

security:
  network_policies: true
  resource_quotas: true
"#
    .to_string()
}

fn create_webapp_template() -> String {
    r#"apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: webapp-biome
  description: "Web application template"
  version: "1.0.0"

services:
  frontend:
    primal: toadstool
    runtime: Container
    image: node:18-alpine
    ports:
      - "3000:3000"
    resources:
      cpu: 2.0
      memory: 4294967296  # 4GB

  api-gateway:
    primal: songbird
    runtime: Native
    ports:
      - "8080:8080"
    resources:
      cpu: 1.0
      memory: 1073741824  # 1GB

  database:
    primal: nestgate
    runtime: Container
    image: postgres:15
    resources:
      cpu: 1.0
      memory: 2147483648  # 2GB
      storage: 53687091200  # 50GB

networking:
  load_balancing: true
  cdn_integration: true

scaling:
  min_replicas: 2
  max_replicas: 10
"#
    .to_string()
}

fn create_ai_research_template() -> String {
    r#"apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: ai-research-biome
  description: "AI research template"
  version: "1.0.0"

services:
  gpu-trainer:
    primal: toadstool
    runtime: GPU
    image: pytorch/pytorch:latest
    resources:
      cpu: 16.0
      memory: 68719476736  # 64GB
      gpu: 4

  data-storage:
    primal: nestgate
    runtime: Native
    resources:
      storage: 1099511627776  # 1TB

  coordinator:
    primal: songbird
    runtime: Native
    resources:
      cpu: 4.0
      memory: 17179869184  # 16GB

networking:
  high_bandwidth: true
  multi_node: true

security:
  model_encryption: true
  access_control: "rbac"
"#
    .to_string()
}

fn create_gaming_template() -> String {
    r#"apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: gaming-biome
  description: "Gaming tournament template"
  version: "1.0.0"

services:
  game-server:
    primal: toadstool
    runtime: Native
    ports:
      - "7777:7777"
    resources:
      cpu: 8.0
      memory: 8589934592  # 8GB

  matchmaking:
    primal: songbird
    runtime: Native
    ports:
      - "8080:8080"
    resources:
      cpu: 4.0
      memory: 4294967296  # 4GB

  leaderboard:
    primal: nestgate
    runtime: Native
    resources:
      storage: 10737418240  # 10GB

networking:
  low_latency: true
  anti_cheat: true

scaling:
  auto_scaling: true
  min_game_servers: 2
  max_game_servers: 20
"#
    .to_string()
}

/// Returns template content for a given template name, or None if unknown.
/// Extracted for testability.
pub fn get_template_content(template: &str) -> Option<String> {
    match template {
        "basic" => Some(create_basic_template()),
        "webapp" => Some(create_webapp_template()),
        "ai-research" => Some(create_ai_research_template()),
        "gaming" => Some(create_gaming_template()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_valid_yaml_and_contains(content: &str, expected_substrings: &[&str]) {
        // Verify it parses as valid YAML
        let parsed: serde_yaml::Value =
            serde_yaml::from_str(content).expect("template should be valid YAML");
        assert_eq!(
            parsed.get("apiVersion").and_then(|v| v.as_str()),
            Some("biomeOS/v1")
        );
        assert_eq!(parsed.get("kind").and_then(|v| v.as_str()), Some("Biome"));
        assert!(parsed.get("metadata").is_some());
        assert!(parsed.get("services").is_some());

        for substr in expected_substrings {
            assert!(
                content.contains(*substr),
                "template should contain '{}'",
                substr
            );
        }
    }

    #[test]
    fn test_create_basic_template() {
        let content = create_basic_template();
        assert_valid_yaml_and_contains(
            &content,
            &[
                "basic-biome",
                "web-server",
                "nginx:alpine",
                "nestgate",
                "load_balancing",
            ],
        );
    }

    #[test]
    fn test_create_webapp_template() {
        let content = create_webapp_template();
        assert_valid_yaml_and_contains(
            &content,
            &[
                "webapp-biome",
                "frontend",
                "node:18-alpine",
                "postgres:15",
                "cdn_integration",
            ],
        );
    }

    #[test]
    fn test_create_ai_research_template() {
        let content = create_ai_research_template();
        assert_valid_yaml_and_contains(
            &content,
            &[
                "ai-research-biome",
                "gpu-trainer",
                "pytorch/pytorch:latest",
                "model_encryption",
            ],
        );
    }

    #[test]
    fn test_create_gaming_template() {
        let content = create_gaming_template();
        assert_valid_yaml_and_contains(
            &content,
            &[
                "gaming-biome",
                "game-server",
                "matchmaking",
                "anti_cheat",
                "min_game_servers",
            ],
        );
    }

    #[test]
    fn test_get_template_content_known_templates() {
        assert!(get_template_content("basic").is_some());
        assert!(get_template_content("webapp").is_some());
        assert!(get_template_content("ai-research").is_some());
        assert!(get_template_content("gaming").is_some());
    }

    #[test]
    fn test_get_template_content_unknown_template() {
        assert!(get_template_content("unknown").is_none());
        assert!(get_template_content("").is_none());
    }

    #[test]
    fn test_cli_parse_deploy() {
        let cli = Cli::try_parse_from(["biome", "deploy", "manifest.yaml", "-t", "team1"])
            .expect("deploy should parse");
        assert_eq!(
            cli.command,
            Commands::Deploy {
                manifest: PathBuf::from("manifest.yaml"),
                team: "team1".to_string(),
            }
        );
    }

    #[test]
    fn test_cli_parse_list() {
        let cli = Cli::try_parse_from(["biome", "list", "-t", "team1"]).expect("list should parse");
        assert_eq!(
            cli.command,
            Commands::List {
                team: "team1".to_string(),
            }
        );
    }

    #[test]
    fn test_cli_parse_status() {
        let cli = Cli::try_parse_from(["biome", "status", "dep-123"]).expect("status should parse");
        assert_eq!(
            cli.command,
            Commands::Status {
                deployment_id: "dep-123".to_string(),
            }
        );
    }

    #[test]
    fn test_cli_parse_remove() {
        let cli = Cli::try_parse_from(["biome", "remove", "dep-123"]).expect("remove should parse");
        assert_eq!(
            cli.command,
            Commands::Remove {
                deployment_id: "dep-123".to_string(),
            }
        );
    }

    #[test]
    fn test_cli_parse_init() {
        let cli = Cli::try_parse_from(["biome", "init", "-t", "webapp", "-o", "out.yaml"])
            .expect("init should parse");
        assert_eq!(
            cli.command,
            Commands::Init {
                template: "webapp".to_string(),
                output: PathBuf::from("out.yaml"),
            }
        );
    }

    #[test]
    fn test_cli_parse_validate() {
        let cli = Cli::try_parse_from(["biome", "validate", "manifest.yaml"])
            .expect("validate should parse");
        assert_eq!(
            cli.command,
            Commands::Validate {
                manifest: PathBuf::from("manifest.yaml"),
            }
        );
    }

    #[test]
    fn test_cli_parse_workspace() {
        let cli = Cli::try_parse_from(["biome", "workspace", "-t", "team1"])
            .expect("workspace should parse");
        assert_eq!(
            cli.command,
            Commands::Workspace {
                team: "team1".to_string(),
            }
        );
    }

    #[test]
    fn test_template_basic_is_valid_yaml() {
        let content = create_basic_template();
        let parsed: serde_yaml::Value =
            serde_yaml::from_str(&content).expect("basic template should parse as YAML");
        assert_eq!(
            parsed.get("apiVersion").and_then(|v| v.as_str()),
            Some("biomeOS/v1")
        );
        assert_eq!(parsed.get("kind").and_then(|v| v.as_str()), Some("Biome"));
        assert!(parsed.get("metadata").is_some());
        assert!(parsed.get("services").is_some());
    }

    #[test]
    fn test_all_templates_have_required_fields() {
        let templates = [
            ("basic", create_basic_template()),
            ("webapp", create_webapp_template()),
            ("ai-research", create_ai_research_template()),
            ("gaming", create_gaming_template()),
        ];
        for (name, content) in templates {
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .unwrap_or_else(|_| panic!("{name} template should parse as YAML"));
            assert!(
                parsed.get("apiVersion").is_some(),
                "{name} template should have apiVersion"
            );
            assert!(
                parsed.get("kind").is_some(),
                "{name} template should have kind"
            );
            assert!(
                parsed.get("metadata").is_some(),
                "{name} template should have metadata"
            );
            assert!(
                parsed.get("services").is_some(),
                "{name} template should have services"
            );
        }
    }
}
