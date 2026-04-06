// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! CLI commands for sub-federation management

use anyhow::{Context, Result};
use biomeos_federation::capability::{Capability, CapabilitySet};
use biomeos_federation::subfederation::{IsolationLevel, SubFederationManager};
use clap::Args;
use std::path::PathBuf;
use tracing::info;

fn default_federation_config_dir() -> PathBuf {
    biomeos_types::SystemPaths::new_lazy()
        .data_dir()
        .join("federation")
}

/// Parse comma-separated members string (testable pure function)
pub(crate) fn parse_members_from_comma_separated(s: &str) -> Vec<String> {
    s.split(',').map(|s| s.trim().to_string()).collect()
}

/// Parse isolation level string to enum (testable pure function)
pub(crate) fn parse_isolation_level(s: &str) -> Result<IsolationLevel> {
    match s.to_lowercase().as_str() {
        "none" => Ok(IsolationLevel::None),
        "low" => Ok(IsolationLevel::Low),
        "medium" => Ok(IsolationLevel::Medium),
        "high" => Ok(IsolationLevel::High),
        "critical" => Ok(IsolationLevel::Critical),
        _ => Err(anyhow::anyhow!(
            "Invalid isolation level: {s}. Must be one of: none, low, medium, high, critical"
        )),
    }
}

/// Arguments for creating a sub-federation
#[derive(Args, Debug)]
pub struct CreateSubfedArgs {
    /// Sub-federation name
    #[arg(long)]
    pub name: String,

    /// Parent family ID (genetic lineage)
    #[arg(long)]
    pub parent_family: String,

    /// Member node IDs (comma-separated, supports wildcards like "node-*")
    #[arg(long)]
    pub members: String,

    /// Capabilities (comma-separated: storage,compute,gaming,sync,voice,video,discovery)
    #[arg(long)]
    pub capabilities: String,

    /// Isolation level (none,low,medium,high,critical)
    #[arg(long, default_value = "low")]
    pub isolation: String,

    /// Configuration directory
    #[arg(long, default_value_os_t = default_federation_config_dir())]
    pub config_dir: PathBuf,
}

/// Arguments for listing sub-federations
#[derive(Args, Debug)]
pub struct ListSubfedsArgs {
    /// Configuration directory
    #[arg(long, default_value_os_t = default_federation_config_dir())]
    pub config_dir: PathBuf,

    /// Filter by family ID
    #[arg(long)]
    pub family: Option<String>,

    /// Show detailed information
    #[arg(long, default_value = "false")]
    pub detailed: bool,
}

/// Arguments for joining a sub-federation
#[derive(Args, Debug)]
pub struct JoinSubfedArgs {
    /// Sub-federation name
    #[arg(long)]
    pub name: String,

    /// Node ID to add
    #[arg(long)]
    pub node: String,

    /// Configuration directory
    #[arg(long, default_value_os_t = default_federation_config_dir())]
    pub config_dir: PathBuf,
}

/// Arguments for checking sub-federation access
#[derive(Args, Debug)]
pub struct CheckAccessArgs {
    /// Node ID
    #[arg(long)]
    pub node: String,

    /// Capability to check
    #[arg(long)]
    pub capability: String,

    /// Sub-federation name (optional, checks all if not provided)
    #[arg(long)]
    pub subfed: Option<String>,

    /// Configuration directory
    #[arg(long, default_value_os_t = default_federation_config_dir())]
    pub config_dir: PathBuf,
}

/// Handle `biomeos federation create-subfed` command
pub async fn handle_federation_create_subfed(args: &CreateSubfedArgs) -> Result<()> {
    info!("Creating sub-federation: {}", args.name);

    // Parse members
    let members = parse_members_from_comma_separated(&args.members);

    // Parse capabilities
    let capabilities: Vec<Capability> = args
        .capabilities
        .split(',')
        .map(|s| Capability::from_str(s.trim()))
        .collect();
    let capability_set = CapabilitySet::from_vec(capabilities);

    // Parse isolation level
    let isolation_level = parse_isolation_level(&args.isolation)?;

    // Create manager
    let mut manager = SubFederationManager::new(args.config_dir.clone());
    manager.load().await?;

    // Create sub-federation
    let subfed = manager
        .create(
            args.name.clone(),
            args.parent_family.clone(),
            members.clone(),
            capability_set.clone(),
            isolation_level.clone(),
        )
        .await?;

    println!("\n🌐 Sub-Federation Created!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Name:              {}", subfed.name);
    println!("  Parent Family:     {}", subfed.parent_family);
    println!("  Members:           {}", members.join(", "));
    println!(
        "  Capabilities:      {}",
        format_capabilities(&capability_set)
    );
    println!("  Isolation:         {isolation_level:?}");
    println!("  Created At:        {}", subfed.created_at.to_rfc3339());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n✅ Sub-federation ready! Members can now access granted capabilities.");

    Ok(())
}

/// Handle `biomeos federation list-subfeds` command
pub async fn handle_federation_list_subfeds(args: &ListSubfedsArgs) -> Result<()> {
    let mut manager = SubFederationManager::new(args.config_dir.clone());
    manager.load().await?;

    let subfeds = manager.all();

    if subfeds.is_empty() {
        println!("No sub-federations found.");
        return Ok(());
    }

    // Filter if requested
    let filtered: Vec<_> = subfeds
        .iter()
        .filter(|sf| {
            args.family
                .as_ref()
                .is_none_or(|f| sf.parent_family.contains(f))
        })
        .collect();

    if filtered.is_empty() {
        println!("No sub-federations found matching criteria.");
        return Ok(());
    }

    println!("\n🌐 Sub-Federations:");
    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    );
    println!(
        "{:<20} {:<20} {:<10} {:<15} {:<20}",
        "NAME", "FAMILY", "MEMBERS", "ISOLATION", "CAPABILITIES"
    );
    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    );

    for subfed in &filtered {
        println!(
            "{:<20} {:<20} {:<10} {:<15} {:<20}",
            truncate(&subfed.name, 19),
            truncate(&subfed.parent_family, 19),
            subfed.members.len(),
            format!("{:?}", subfed.isolation_level),
            truncate(&format_capabilities(&subfed.capabilities), 19),
        );

        if args.detailed {
            println!("  Members:");
            for member in &subfed.members {
                println!("    - {member}");
            }
            println!();
        }
    }

    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    );
    println!("Total: {} sub-federation(s)", filtered.len());

    Ok(())
}

/// Handle `biomeos federation join-subfed` command
pub async fn handle_federation_join_subfed(args: &JoinSubfedArgs) -> Result<()> {
    info!("Adding node {} to sub-federation {}", args.node, args.name);

    let mut manager = SubFederationManager::new(args.config_dir.clone());
    manager.load().await?;

    manager.add_member(&args.name, args.node.clone()).await?;

    println!(
        "✅ Node {} added to sub-federation {}",
        args.node, args.name
    );

    Ok(())
}

/// Handle `biomeos federation check-access` command
pub async fn handle_federation_check_access(args: &CheckAccessArgs) -> Result<()> {
    let mut manager = SubFederationManager::new(args.config_dir.clone());
    manager.load().await?;

    let capability = Capability::from_str(&args.capability);

    if let Some(ref subfed_name) = args.subfed {
        // Check specific sub-federation
        let subfed = manager
            .get(subfed_name)
            .context(format!("Sub-federation '{subfed_name}' not found"))?;

        let has_access = subfed.has_capability(&args.node, &capability);

        println!("\n🔍 Access Check:");
        println!("  Node:              {}", args.node);
        println!("  Capability:        {capability}");
        println!("  Sub-Federation:    {subfed_name}");
        println!(
            "  Access:            {}",
            if has_access {
                "✅ GRANTED"
            } else {
                "❌ DENIED"
            }
        );

        if !has_access {
            println!("\n  Reason:");
            if !subfed.is_member(&args.node) {
                println!("    - Node is not a member of this sub-federation");
            }
            if !subfed.capabilities.has(&capability) {
                println!("    - Capability not granted to this sub-federation");
            }
            if !subfed.isolation_level.allows_auto_approval() {
                println!("    - Isolation level requires manual approval");
            }
        }
    } else {
        // Check all sub-federations
        let has_access = manager.has_access(&args.node, &capability);

        println!("\n🔍 Access Check:");
        println!("  Node:              {}", args.node);
        println!("  Capability:        {capability}");
        println!(
            "  Access:            {}",
            if has_access {
                "✅ GRANTED (at least one sub-federation)"
            } else {
                "❌ DENIED (no sub-federations)"
            }
        );

        if has_access {
            println!("\n  Granted by:");
            for subfed in manager.all() {
                if subfed.has_capability(&args.node, &capability) {
                    println!("    - {}", subfed.name);
                }
            }
        }
    }

    Ok(())
}

/// Format capabilities for display
pub(crate) fn format_capabilities(caps: &CapabilitySet) -> String {
    caps.all()
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// Truncate string to max length
pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use biomeos_federation::capability::{Capability, CapabilitySet};
    use biomeos_federation::subfederation::IsolationLevel;

    #[test]
    fn test_parse_isolation_level() {
        assert!(matches!(
            parse_isolation_level("none").unwrap(),
            IsolationLevel::None
        ));
        assert!(matches!(
            parse_isolation_level("low").unwrap(),
            IsolationLevel::Low
        ));
        assert!(matches!(
            parse_isolation_level("LOW").unwrap(),
            IsolationLevel::Low
        ));
        assert!(matches!(
            parse_isolation_level("medium").unwrap(),
            IsolationLevel::Medium
        ));
        assert!(matches!(
            parse_isolation_level("high").unwrap(),
            IsolationLevel::High
        ));
        assert!(matches!(
            parse_isolation_level("critical").unwrap(),
            IsolationLevel::Critical
        ));
    }

    #[test]
    fn test_parse_isolation_level_invalid() {
        assert!(parse_isolation_level("invalid").is_err());
        assert!(parse_isolation_level("").is_err());
    }

    #[test]
    fn test_format_capabilities() {
        let caps = CapabilitySet::from_vec(vec![Capability::Storage, Capability::Compute]);
        let result = format_capabilities(&caps);
        assert!(result.contains("storage"));
        assert!(result.contains("compute"));
    }

    #[test]
    fn test_format_capabilities_empty() {
        let caps = CapabilitySet::new();
        let result = format_capabilities(&caps);
        assert_eq!(result, "");
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hi", 5), "hi");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("abcdefghij", 7), "abcd...");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_short_max() {
        // When max=3, prefix length is 0 (max-3), so result is "..."
        assert_eq!(truncate("hello", 3), "...");
    }

    #[test]
    fn test_truncate_max_zero() {
        // max=0: saturating_sub(3)=0, so "..."
        assert_eq!(truncate("hello", 0), "...");
    }

    #[test]
    fn test_truncate_empty_string() {
        // Empty string is always returned as-is when len <= max
        assert_eq!(truncate("", 10), "");
        // max=0: len 0 <= 0, so return as-is
        assert_eq!(truncate("", 0), "");
    }

    #[test]
    fn test_parse_members_from_comma_separated() {
        assert_eq!(
            parse_members_from_comma_separated("a,b,c"),
            vec!["a", "b", "c"]
        );
        assert_eq!(
            parse_members_from_comma_separated("node-1, node-2 , node-3"),
            vec!["node-1", "node-2", "node-3"]
        );
        assert_eq!(parse_members_from_comma_separated("single"), vec!["single"]);
        assert_eq!(parse_members_from_comma_separated(""), vec![""]);
    }

    #[test]
    fn test_parse_isolation_level_error_message() {
        let err = parse_isolation_level("invalid").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Invalid isolation level"));
        assert!(msg.contains("invalid"));
        assert!(msg.contains("none"));
        assert!(msg.contains("critical"));
    }

    #[test]
    fn test_create_subfed_args_debug() {
        let args = CreateSubfedArgs {
            name: "test".to_string(),
            parent_family: "fam1".to_string(),
            members: "n1,n2".to_string(),
            capabilities: "storage,compute".to_string(),
            isolation: "low".to_string(),
            config_dir: PathBuf::from("/tmp"),
        };
        let _ = format!("{args:?}");
    }

    #[test]
    fn test_list_subfeds_args_debug() {
        let args = ListSubfedsArgs {
            config_dir: PathBuf::from("/tmp"),
            family: Some("fam".to_string()),
            detailed: true,
        };
        let _ = format!("{args:?}");
    }

    #[test]
    fn test_join_subfed_args_debug() {
        let args = JoinSubfedArgs {
            name: "subfed".to_string(),
            node: "node-1".to_string(),
            config_dir: PathBuf::from("/tmp"),
        };
        let _ = format!("{args:?}");
    }

    #[test]
    fn test_check_access_args_debug() {
        let args = CheckAccessArgs {
            node: "n1".to_string(),
            capability: "storage".to_string(),
            subfed: Some("sf".to_string()),
            config_dir: PathBuf::from("/tmp"),
        };
        let _ = format!("{args:?}");
    }

    #[test]
    fn test_truncate_unicode() {
        assert_eq!(truncate("hello", 2), "...");
    }

    #[test]
    fn test_default_federation_config_dir_contains_federation() {
        let p = default_federation_config_dir();
        assert!(
            p.to_string_lossy().contains("federation"),
            "unexpected path: {}",
            p.display()
        );
    }

    #[tokio::test]
    async fn test_handle_federation_list_subfeds_empty_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let args = ListSubfedsArgs {
            config_dir: dir.path().to_path_buf(),
            family: None,
            detailed: false,
        };
        handle_federation_list_subfeds(&args).await.expect("list");
    }

    #[tokio::test]
    async fn test_handle_federation_create_list_and_check_access() {
        let dir = tempfile::tempdir().expect("tempdir");
        let create = CreateSubfedArgs {
            name: "cli-sf-test".to_string(),
            parent_family: "parent-fam-1894".to_string(),
            members: "node-a,node-b".to_string(),
            capabilities: "storage,compute".to_string(),
            isolation: "low".to_string(),
            config_dir: dir.path().to_path_buf(),
        };
        handle_federation_create_subfed(&create)
            .await
            .expect("create subfed");

        let list = ListSubfedsArgs {
            config_dir: dir.path().to_path_buf(),
            family: None,
            detailed: true,
        };
        handle_federation_list_subfeds(&list).await.expect("list");

        let join = JoinSubfedArgs {
            name: "cli-sf-test".to_string(),
            node: "node-c".to_string(),
            config_dir: dir.path().to_path_buf(),
        };
        handle_federation_join_subfed(&join).await.expect("join");

        let check = CheckAccessArgs {
            node: "node-xyz".to_string(),
            capability: "storage".to_string(),
            subfed: Some("cli-sf-test".to_string()),
            config_dir: dir.path().to_path_buf(),
        };
        handle_federation_check_access(&check)
            .await
            .expect("check specific");

        let check_all = CheckAccessArgs {
            node: "node-a".to_string(),
            capability: "storage".to_string(),
            subfed: None,
            config_dir: dir.path().to_path_buf(),
        };
        handle_federation_check_access(&check_all)
            .await
            .expect("check all");
    }
}
