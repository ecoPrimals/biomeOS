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
        println!(
            "❌ Chimera definitions directory not found: {:?}",
            definitions_dir
        );
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
            println!("❌ Failed to load chimera registry: {}", e);
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
                println!("     {}", line);
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
            println!("❌ Chimera not found: {}", id);
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
            println!("🔨 Building chimera: {}", id);

            let builder = ChimeraBuilder::new(Arc::clone(&def))
                .output_dir("bin/chimeras")
                .primals_dir("bin/primals");

            // Check primals first
            match builder.check_primals() {
                Ok(paths) => {
                    println!("   ✅ Found {} required primal binaries", paths.len());
                }
                Err(e) => {
                    println!("   ❌ Missing primals: {}", e);
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
                        println!("   ⚠️  {}", warning);
                    }
                }
                Err(e) => {
                    println!("   ❌ Build failed: {}", e);
                }
            }
        }
        None => {
            println!("❌ Chimera not found: {}", id);
        }
    }

    Ok(())
}
