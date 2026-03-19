// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! CLI commands for spore incubation and local node management

use anyhow::Result;
use biomeos_spore::incubation::{SporeIncubator, list_local_nodes};
use clap::Args;
use std::path::PathBuf;
use tracing::info;

/// Arguments for spore incubation
#[derive(Args, Debug)]
pub struct IncubateArgs {
    /// Path to the USB spore
    #[arg(long)]
    pub spore: PathBuf,

    /// Computer name (optional, uses hostname if not provided)
    #[arg(long)]
    pub computer_name: Option<String>,

    /// Also create local deployment in /tmp
    #[arg(long, default_value = "false")]
    pub deploy_local: bool,
}

/// Arguments for listing locally incubated nodes
#[derive(Args, Debug)]
pub struct ListLocalArgs {
    /// Filter by spore ID
    #[arg(long)]
    pub spore_id: Option<String>,

    /// Show detailed information
    #[arg(long, default_value = "false")]
    pub detailed: bool,
}

/// Handle `biomeos spore incubate` command
pub async fn handle_spore_incubate(args: &IncubateArgs) -> Result<()> {
    info!("Incubating spore from: {}", args.spore.display());

    let incubator = SporeIncubator::new(&args.spore)?;
    let incubated = incubator
        .incubate(args.computer_name.as_deref(), args.deploy_local)
        .await?;

    println!("\n🌱 Spore Incubated Successfully!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Node ID:           {}", incubated.node_id);
    println!("  Spore ID:          {}", incubated.spore_id);
    println!(
        "  Incubated At:      {}",
        incubated.incubated_at.to_rfc3339()
    );
    println!("  Entropy Hash:      {}...", &incubated.entropy_hash[..16]);
    println!(
        "  Deployed Seed:     {}...",
        &incubated.deployed_seed_hash[..16]
    );
    println!(
        "  Config Path:       {}",
        incubated.local_config_path.display()
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n✅ Local configuration created. You can now:");
    println!("   1. Eject the USB spore");
    println!("   2. Use it on another computer");
    println!("   3. Both deployments will federate (genetic lineage)");

    Ok(())
}

/// Handle `biomeos node list-local` command
pub async fn handle_node_list_local(args: &ListLocalArgs) -> Result<()> {
    let nodes = list_local_nodes().await?;

    if nodes.is_empty() {
        println!("No locally incubated nodes found.");
        return Ok(());
    }

    // Filter if requested
    let filtered: Vec<_> = nodes
        .iter()
        .filter(|n| {
            args.spore_id
                .as_ref()
                .is_none_or(|id| n.node.spore_id.contains(id))
        })
        .collect();

    if filtered.is_empty() {
        println!("No nodes found matching criteria.");
        return Ok(());
    }

    println!("\n📊 Locally Incubated Nodes:");
    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    );
    println!(
        "{:<25} {:<25} {:<30} {:<20}",
        "NODE_ID", "SPORE_ID", "DEPLOYED_AT", "FAMILY_ID"
    );
    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    );

    for node in &filtered {
        println!(
            "{:<25} {:<25} {:<30} {:<20}",
            truncate(&node.node.node_id, 24),
            truncate(&node.node.spore_id, 24),
            node.node
                .deployed_at
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            truncate(&node.federation.family_id, 19),
        );

        if args.detailed {
            println!("  Computer:         {}", node.node.computer_name);
            println!("  Entropy Hash:     {}...", &node.node.entropy_hash[..16]);
            println!(
                "  Deployed Seed:    {}...",
                &node.lineage.deployed_seed_hash[..16]
            );

            if let Some(ref path) = node.spore.original_path {
                println!("  Spore Path:       {}", path.display());
            }

            if !node.federation.sub_federations.is_empty() {
                println!(
                    "  Sub-Federations:  {}",
                    node.federation.sub_federations.join(", ")
                );
            }

            println!();
        }
    }

    println!(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    );
    println!("Total: {} node(s)", filtered.len());

    Ok(())
}

/// Truncate string to max length
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("abcdefghij", 6), "abc...");
    }

    #[test]
    fn test_truncate_boundary() {
        assert_eq!(truncate("abc", 3), "abc");
        assert_eq!(truncate("abcd", 4), "abcd");
        assert_eq!(truncate("abcde", 5), "abcde");
        assert_eq!(truncate("abcdef", 6), "abcdef");
        assert_eq!(truncate("abcdefg", 6), "abc...");
    }

    #[test]
    fn test_incubate_args_debug() {
        let args = IncubateArgs {
            spore: PathBuf::from("/tmp/spore"),
            computer_name: Some("test-pc".to_string()),
            deploy_local: true,
        };
        let _ = format!("{args:?}");
    }

    #[test]
    fn test_list_local_args_debug() {
        let args = ListLocalArgs {
            spore_id: Some("spore-1".to_string()),
            detailed: true,
        };
        let _ = format!("{args:?}");
    }

    #[test]
    fn test_incubate_args_default_deploy_local() {
        let args = IncubateArgs {
            spore: PathBuf::from("/tmp/spore"),
            computer_name: None,
            deploy_local: false,
        };
        assert!(!args.deploy_local);
    }

    #[test]
    fn test_list_local_args_filter_none() {
        let args = ListLocalArgs {
            spore_id: None,
            detailed: false,
        };
        assert!(args.spore_id.is_none());
        assert!(!args.detailed);
    }

    #[test]
    fn test_truncate_to_short_max() {
        // When max < 4, we get "..." (max.saturating_sub(3) = 0)
        assert_eq!(truncate("hello", 3), "...");
    }

    #[test]
    fn test_truncate_zero_max() {
        assert_eq!(truncate("hello", 0), "...");
    }

    #[tokio::test]
    async fn test_handle_spore_incubate_nonexistent_path() {
        let args = IncubateArgs {
            spore: PathBuf::from("/nonexistent/spore/path/12345"),
            computer_name: None,
            deploy_local: false,
        };
        let result = handle_spore_incubate(&args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_node_list_local_empty() {
        let args = ListLocalArgs {
            spore_id: None,
            detailed: false,
        };
        let result = handle_node_list_local(&args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_node_list_local_with_filter_no_match() {
        let args = ListLocalArgs {
            spore_id: Some("nonexistent-spore-id-xyz".to_string()),
            detailed: false,
        };
        let result = handle_node_list_local(&args).await;
        assert!(result.is_ok());
    }
}
