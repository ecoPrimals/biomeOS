// biomeos-cli/src/commands/genome.rs
// Genome Factory CLI commands
//
// Deep Debt: CLI for genomeBin operations

use anyhow::{Context, Result};
use biomeos_genome_factory::create::GenomeMetadata;
use biomeos_genome_factory::{
    GenomeComposeRequest, GenomeCreateRequest, GenomeFactory,
};
use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

/// Create genome arguments
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Primal name (e.g., beardog, songbird)
    pub name: String,

    /// Binary path (format: arch=path, e.g., x86_64=/path/to/binary)
    #[arg(short, long, value_parser = parse_binary_mapping)]
    pub binary: Vec<(String, PathBuf)>,

    /// Version
    #[arg(short, long)]
    pub version: Option<String>,

    /// Description
    #[arg(short, long)]
    pub description: Option<String>,
}

/// Compose genome arguments
#[derive(Debug, Args)]
pub struct ComposeArgs {
    /// Atomic name (e.g., tower, node, nest)
    pub name: String,

    /// Nucleus type (TOWER, NODE, NEST, NUCLEUS)
    #[arg(short = 't', long)]
    pub nucleus_type: String,

    /// Genome names to compose
    #[arg(short, long)]
    pub genome: Vec<String>,
}

/// Verify genome arguments
#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Genome name
    pub name: String,
}

/// Handle genome create command
pub fn handle_genome_create(args: CreateArgs) -> Result<()> {
    println!("🧬 Creating genomeBin: {}", args.name);

    let factory = GenomeFactory::default()
        .context("Failed to initialize genome factory")?;

    let mut binaries = HashMap::new();
    for (arch, path) in args.binary {
        binaries.insert(arch, path);
    }

    if binaries.is_empty() {
        anyhow::bail!("At least one binary must be specified (use --binary arch=path)");
    }

    let request = GenomeCreateRequest {
        name: args.name.clone(),
        binaries,
        metadata: GenomeMetadata {
            version: args.version,
            description: args.description,
            nucleus_atomic: None,
            capabilities: vec![],
        },
    };

    let response = factory.create_genome(request)
        .context("Failed to create genomeBin")?;

    println!("✅ genomeBin created!");
    println!("   ID: {}", response.genome_id);
    println!("   Path: {}", response.path.display());
    println!("   Size: {} bytes ({:.2} MB)", response.size, response.size as f64 / 1_048_576.0);
    println!("   Architectures: {}", response.architectures.join(", "));

    Ok(())
}

/// Handle genome compose command
pub fn handle_genome_compose(args: ComposeArgs) -> Result<()> {
    println!("🧬 Composing {} atomic: {}", args.nucleus_type, args.name);

    let factory = GenomeFactory::default()
        .context("Failed to initialize genome factory")?;

    if args.genome.is_empty() {
        anyhow::bail!("At least one genome must be specified (use --genome name)");
    }

    let request = GenomeComposeRequest {
        name: args.name.clone(),
        nucleus_type: args.nucleus_type.clone(),
        genomes: args.genome,
    };

    let response = factory.compose_genome(request)
        .context("Failed to compose genomeBin")?;

    println!("✅ Atomic genomeBin composed!");
    println!("   ID: {}", response.genome_id);
    println!("   Type: {}", response.nucleus_type);
    println!("   Path: {}", response.path.display());
    println!("   Size: {} bytes ({:.2} MB)", response.size, response.size as f64 / 1_048_576.0);
    println!("   Embedded: {}", response.embedded_genomes.join(", "));

    Ok(())
}

/// Handle genome self-replicate command
pub fn handle_genome_self_replicate() -> Result<()> {
    println!("🧬 Self-replication: biomeOS creating its own genomeBin");

    let factory = GenomeFactory::default()
        .context("Failed to initialize genome factory")?;

    let response = factory.self_replicate()
        .context("Failed to self-replicate")?;

    println!("✅ Self-replication complete!");
    println!("   ID: {}", response.genome_id);
    println!("   Path: {}", response.path.display());
    println!("   Size: {} bytes ({:.2} MB)", response.size, response.size as f64 / 1_048_576.0);
    println!("   Architectures: {}", response.architectures.join(", "));
    println!("");
    println!("🎊 biomeOS can now reproduce itself autonomously!");

    Ok(())
}

/// Handle genome list command
pub fn handle_genome_list() -> Result<()> {
    let factory = GenomeFactory::default()
        .context("Failed to initialize genome factory")?;

    let genomes = factory.list_genomes()
        .context("Failed to list genomes")?;

    if genomes.is_empty() {
        println!("📦 No genomes found in storage");
        return Ok(());
    }

    println!("📦 Available genomeBins ({}):", genomes.len());
    println!("");

    for genome_name in genomes {
        let path = factory.genome_path(&genome_name);
        
        if let Ok(metadata) = std::fs::metadata(&path) {
            let size_mb = metadata.len() as f64 / 1_048_576.0;
            println!("  • {} ({:.2} MB)", genome_name, size_mb);
            println!("    Path: {}", path.display());
        } else {
            println!("  • {} (metadata unavailable)", genome_name);
        }
    }

    Ok(())
}

/// Handle genome verify command
pub fn handle_genome_verify(args: VerifyArgs) -> Result<()> {
    println!("🔍 Verifying genomeBin: {}", args.name);

    let factory = GenomeFactory::default()
        .context("Failed to initialize genome factory")?;

    let result = factory.verify_genome(&args.name)
        .context("Failed to verify genomeBin")?;

    if result.valid {
        println!("✅ Verification PASSED");
        println!("   All checksums valid: {}/{}", 
            result.checksums.values().filter(|c| c.valid).count(),
            result.checksums.len()
        );
        
        if result.embedded_count > 0 {
            println!("   Embedded genomes verified: {}", result.embedded_count);
        }
    } else {
        println!("❌ Verification FAILED");
        println!("   Invalid checksums detected!");
        
        for (key, checksum) in result.checksums {
            if !checksum.valid {
                println!("   ❌ {}: checksum mismatch", key);
                println!("      Expected: {}", checksum.expected);
                println!("      Actual:   {}", checksum.actual);
            }
        }
    }

    Ok(())
}

/// Parse binary mapping (arch=path)
fn parse_binary_mapping(s: &str) -> Result<(String, PathBuf)> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    
    if parts.len() != 2 {
        anyhow::bail!("Invalid format. Expected: arch=path (e.g., x86_64=/path/to/binary)");
    }

    Ok((parts[0].to_string(), PathBuf::from(parts[1])))
}
