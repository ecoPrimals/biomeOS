//! Niche CLI Commands
//!
//! Commands for managing niche (biome) templates and deployments.

use std::path::Path;
use std::fs;

/// List available niche templates
pub async fn handle_niche_list() -> anyhow::Result<()> {
    let templates_dir = Path::new("niches/templates");
    
    if !templates_dir.exists() {
        println!("❌ Niche templates directory not found: {:?}", templates_dir);
        return Ok(());
    }

    println!("🌿 Available Niche Templates:");
    println!();
    
    if let Ok(entries) = fs::read_dir(templates_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "yaml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Parse basic info
                    let id = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    
                    let name = content.lines()
                        .find(|l| l.contains("name:") && !l.contains("primal"))
                        .and_then(|l| l.split(':').nth(1))
                        .map(|s| s.trim().trim_matches('"'))
                        .unwrap_or("Unknown");
                    
                    let category = content.lines()
                        .find(|l| l.contains("category:"))
                        .and_then(|l| l.split(':').nth(1))
                        .map(|s| s.trim().trim_matches('"'))
                        .unwrap_or("");
                    
                    let icon = match category {
                        "gaming" => "🎮",
                        "ai_research" => "🧠",
                        "development" => "💻",
                        "federation" => "🌐",
                        _ => "🌿",
                    };
                    
                    println!("  {} {} ({})", icon, name, id);
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
    let template_path = Path::new("niches/templates").join(format!("{}.yaml", id));
    
    if !template_path.exists() {
        println!("❌ Niche template not found: {}", id);
        println!("   Run 'biomeos niche list' to see available templates");
        return Ok(());
    }

    let content = fs::read_to_string(&template_path)?;
    
    println!("🌿 Niche Template: {}", id);
    println!();
    
    // Parse and display sections
    let mut in_section = "";
    let mut indent = 0;
    
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
                if trimmed.starts_with("name:") || 
                   trimmed.starts_with("description:") ||
                   trimmed.starts_with("category:") ||
                   trimmed.starts_with("features:") {
                    println!("   {}", trimmed);
                }
            }
            "organisms" => {
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    let current_indent = line.len() - line.trim_start().len();
                    if current_indent <= 4 {
                        println!("   {}", trimmed);
                    }
                }
            }
            "customization" => {
                if trimmed.starts_with("- id:") || trimmed.starts_with("name:") {
                    println!("   {}", trimmed);
                }
            }
            "resources" => {
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    println!("   {}", trimmed);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// List installed primal binaries
pub async fn handle_primal_list() -> anyhow::Result<()> {
    let primals_dir = Path::new("bin/primals");
    
    if !primals_dir.exists() {
        println!("❌ Primals directory not found. Run './bin/pull-primals.sh --all'");
        return Ok(());
    }

    println!("📦 Installed Primal Binaries:");
    println!();
    
    let mut primal_counts: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    if let Ok(entries) = fs::read_dir(primals_dir) {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue; }
            
            // Extract primal name (before first dash)
            let primal = name.split('-').next().unwrap_or(&name).to_string();
            primal_counts.entry(primal).or_default().push(name);
        }
    }

    for (primal, binaries) in &primal_counts {
        let icon = match primal.as_str() {
            "nestgate" => "🏰",
            "songbird" => "🎼",
            "toadstool" => "🍄",
            "beardog" => "🐕",
            "squirrel" => "🐿️",
            _ => "📦",
        };
        println!("  {} {} ({} binaries)", icon, primal, binaries.len());
        
        // Show first few
        for bin in binaries.iter().take(3) {
            println!("     └─ {}", bin);
        }
        if binaries.len() > 3 {
            println!("     └─ ... and {} more", binaries.len() - 3);
        }
        println!();
    }

    let total: usize = primal_counts.values().map(|v| v.len()).sum();
    println!("Total: {} binaries from {} primals", total, primal_counts.len());

    Ok(())
}

