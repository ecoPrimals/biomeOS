//! CLI commands for sub-federation management

use anyhow::{Context, Result};
use biomeos_federation::capability::{Capability, CapabilitySet};
use biomeos_federation::subfederation::{IsolationLevel, SubFederationManager};
use clap::Args;
use std::path::PathBuf;
use tracing::info;

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
    #[arg(long, default_value = "/var/biomeos/federation")]
    pub config_dir: PathBuf,
}

#[derive(Args, Debug)]
pub struct ListSubfedsArgs {
    /// Configuration directory
    #[arg(long, default_value = "/var/biomeos/federation")]
    pub config_dir: PathBuf,
    
    /// Filter by family ID
    #[arg(long)]
    pub family: Option<String>,
    
    /// Show detailed information
    #[arg(long, default_value = "false")]
    pub detailed: bool,
}

#[derive(Args, Debug)]
pub struct JoinSubfedArgs {
    /// Sub-federation name
    #[arg(long)]
    pub name: String,
    
    /// Node ID to add
    #[arg(long)]
    pub node: String,
    
    /// Configuration directory
    #[arg(long, default_value = "/var/biomeos/federation")]
    pub config_dir: PathBuf,
}

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
    #[arg(long, default_value = "/var/biomeos/federation")]
    pub config_dir: PathBuf,
}

/// Handle `biomeos federation create-subfed` command
pub async fn handle_federation_create_subfed(args: &CreateSubfedArgs) -> Result<()> {
    info!("Creating sub-federation: {}", args.name);
    
    // Parse members
    let members: Vec<String> = args
        .members
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    // Parse capabilities
    let capabilities: Vec<Capability> = args
        .capabilities
        .split(',')
        .map(|s| Capability::from_str(s.trim()))
        .collect();
    let capability_set = CapabilitySet::from_vec(capabilities);
    
    // Parse isolation level
    let isolation_level = match args.isolation.to_lowercase().as_str() {
        "none" => IsolationLevel::None,
        "low" => IsolationLevel::Low,
        "medium" => IsolationLevel::Medium,
        "high" => IsolationLevel::High,
        "critical" => IsolationLevel::Critical,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid isolation level: {}. Must be one of: none, low, medium, high, critical",
                args.isolation
            ));
        }
    };
    
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
    println!("  Capabilities:      {}", format_capabilities(&capability_set));
    println!("  Isolation:         {:?}", isolation_level);
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
                .map_or(true, |f| sf.parent_family.contains(f))
        })
        .collect();
    
    if filtered.is_empty() {
        println!("No sub-federations found matching criteria.");
        return Ok(());
    }
    
    println!("\n🌐 Sub-Federations:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{:<20} {:<20} {:<10} {:<15} {:<20}", "NAME", "FAMILY", "MEMBERS", "ISOLATION", "CAPABILITIES");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
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
                println!("    - {}", member);
            }
            println!();
        }
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total: {} sub-federation(s)", filtered.len());
    
    Ok(())
}

/// Handle `biomeos federation join-subfed` command
pub async fn handle_federation_join_subfed(args: &JoinSubfedArgs) -> Result<()> {
    info!("Adding node {} to sub-federation {}", args.node, args.name);
    
    let mut manager = SubFederationManager::new(args.config_dir.clone());
    manager.load().await?;
    
    manager.add_member(&args.name, args.node.clone()).await?;
    
    println!("✅ Node {} added to sub-federation {}", args.node, args.name);
    
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
            .context(format!("Sub-federation '{}' not found", subfed_name))?;
        
        let has_access = subfed.has_capability(&args.node, &capability);
        
        println!("\n🔍 Access Check:");
        println!("  Node:              {}", args.node);
        println!("  Capability:        {}", capability);
        println!("  Sub-Federation:    {}", subfed_name);
        println!("  Access:            {}", if has_access { "✅ GRANTED" } else { "❌ DENIED" });
        
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
        println!("  Capability:        {}", capability);
        println!("  Access:            {}", if has_access { "✅ GRANTED (at least one sub-federation)" } else { "❌ DENIED (no sub-federations)" });
        
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
fn format_capabilities(caps: &CapabilitySet) -> String {
    caps.all()
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// Truncate string to max length
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

