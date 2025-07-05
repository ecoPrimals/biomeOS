//! BYOB - Build Your Own Biome CLI Tool

use std::collections::HashMap;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use biomeos_manifest::niche_templates::*;
use biomeos_manifest::*;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "byob")]
#[command(about = "Build Your Own Biome - Create, customize, and share biomes")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available niche templates
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Show detailed information about a specific template
    Info {
        /// Template ID
        template_id: String,
    },
    
    /// Generate a new biome from a template
    Generate {
        /// Template ID to use
        template_id: String,
        
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Interactive mode (prompt for parameters)
        #[arg(short, long)]
        interactive: bool,
    },
    
    /// Validate a biome configuration
    Validate {
        /// Biome file to validate
        biome_file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let registry = NicheTemplateRegistry::new();
    
    match &cli.command {
        Commands::List { verbose } => {
            list_templates(&registry, *verbose)?;
        }
        Commands::Info { template_id } => {
            show_template_info(&registry, template_id)?;
        }
        Commands::Generate { template_id, output, interactive } => {
            generate_biome(&registry, template_id, output, *interactive)?;
        }
        Commands::Validate { biome_file } => {
            validate_biome(biome_file)?;
        }
    }
    
    Ok(())
}

fn list_templates(registry: &NicheTemplateRegistry, verbose: bool) -> Result<()> {
    let templates = registry.list_templates();
    
    if templates.is_empty() {
        println!("No templates found.");
        return Ok(());
    }
    
    println!("Available Biome Templates:");
    println!("{}", "=".repeat(50));
    
    for template in templates {
        if verbose {
            print_template_detailed(template);
        } else {
            print_template_summary(template);
        }
    }
    
    Ok(())
}

fn print_template_summary(template: &NicheTemplate) {
    println!("📦 {} ({})", template.metadata.name, template.metadata.id);
    println!("   {}", template.metadata.description);
    println!("   Category: {:?} | Difficulty: {:?} | Audience: {:?}", 
             template.metadata.category, template.metadata.difficulty, template.metadata.audience);
    println!("   Setup Time: {} | Resources: {}C/{}GB/{}GB", 
             template.metadata.setup_time, 
             template.metadata.min_resources.cpu_cores,
             template.metadata.min_resources.memory_gb,
             template.metadata.min_resources.storage_gb);
    println!();
}

fn print_template_detailed(template: &NicheTemplate) {
    println!("📦 {} ({})", template.metadata.name, template.metadata.id);
    println!("   Description: {}", template.metadata.description);
    println!("   Category: {:?}", template.metadata.category);
    println!("   Difficulty: {:?}", template.metadata.difficulty);
    println!("   Audience: {:?}", template.metadata.audience);
    println!("   Setup Time: {}", template.metadata.setup_time);
    println!("   Author: {}", template.metadata.author);
    println!("   Version: {}", template.metadata.version);
    println!("   Tags: {}", template.metadata.tags.join(", "));
    
    println!("   Resource Requirements:");
    println!("     CPU: {} cores", template.metadata.min_resources.cpu_cores);
    println!("     Memory: {} GB", template.metadata.min_resources.memory_gb);
    println!("     Storage: {} GB", template.metadata.min_resources.storage_gb);
    
    if !template.parameters.is_empty() {
        println!("   Parameters:");
        for param in &template.parameters {
            println!("     • {} ({}): {}", param.name, format!("{:?}", param.param_type), param.description);
        }
    }
    
    println!();
}

fn show_template_info(registry: &NicheTemplateRegistry, template_id: &str) -> Result<()> {
    let template = registry.get_template(template_id)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;
    
    print_template_detailed(template);
    
    if !template.examples.is_empty() {
        println!("Example Configurations:");
        println!("{}", "-".repeat(30));
        
        for example in &template.examples {
            println!("📋 {}", example.name);
            println!("   {}", example.description);
            println!("   Parameters:");
            for (key, value) in &example.parameters {
                println!("     {}: {}", key, value);
            }
            println!();
        }
    }
    
    Ok(())
}

fn generate_biome(
    registry: &NicheTemplateRegistry,
    template_id: &str,
    output: &Option<PathBuf>,
    interactive: bool,
) -> Result<()> {
    let template = registry.get_template(template_id)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;
    
    let parameters = if interactive {
        collect_parameters_interactively(&template.parameters)?
    } else {
        // Use default parameters
        template.parameters.iter()
            .filter_map(|p| p.default.clone().map(|v| (p.name.clone(), v)))
            .collect()
    };
    
    let biome = registry.generate_biome(template_id, parameters)
        .map_err(|e| anyhow::anyhow!("Failed to generate biome: {}", e))?;
    
    let output_path = output.clone().unwrap_or_else(|| {
        PathBuf::from(format!("{}.biome.yaml", biome.metadata.name.replace(" ", "-").to_lowercase()))
    });
    
    let yaml_content = serde_yaml::to_string(&biome)?;
    std::fs::write(&output_path, yaml_content)?;
    
    println!("✅ Generated biome: {}", output_path.display());
    println!("   Name: {}", biome.metadata.name);
    println!("   Description: {}", biome.metadata.description.unwrap_or_default());
    
    // Handle the specialization option properly
    if let Some(spec) = &biome.metadata.specialization {
        println!("   Specialization: {:?}", spec);
    }
    
    println!("\nNext steps:");
    println!("1. Review the generated biome.yaml file");
    println!("2. Customize the configuration as needed");
    println!("3. Deploy with: biomeos-manager deploy {}", output_path.display());
    
    Ok(())
}

fn collect_parameters_interactively(params: &[TemplateParameter]) -> Result<HashMap<String, serde_json::Value>> {
    let mut parameter_values = HashMap::new();
    
    println!("🔧 Interactive Parameter Collection");
    println!("{}", "=".repeat(40));
    
    for param in params {
        println!();
        println!("Parameter: {}", param.name);
        println!("Description: {}", param.description);
        println!("Type: {:?}", param.param_type);
        
        if let Some(default) = &param.default {
            println!("Default: {}", default);
        }
        
        if param.required {
            println!("⚠️  Required parameter");
        }
        
        print!("Enter value");
        if !param.required {
            print!(" (or press Enter for default)");
        }
        print!(": ");
        
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            if param.required {
                println!("❌ Required parameter cannot be empty");
                return Err(anyhow::anyhow!("Required parameter '{}' not provided", param.name));
            } else if let Some(default) = &param.default {
                parameter_values.insert(param.name.clone(), default.clone());
            }
        } else {
            let value = match param.param_type {
                ParameterType::String => serde_json::Value::String(input.to_string()),
                ParameterType::Number => {
                    let num: f64 = input.parse()
                        .map_err(|_| anyhow::anyhow!("Invalid number: {}", input))?;
                    serde_json::Value::Number(serde_json::Number::from_f64(num).unwrap())
                }
                ParameterType::Boolean => {
                    let bool_val = input.to_lowercase() == "true" || input == "1" || input.to_lowercase() == "yes";
                    serde_json::Value::Bool(bool_val)
                }
                _ => serde_json::Value::String(input.to_string()),
            };
            parameter_values.insert(param.name.clone(), value);
        }
    }
    
    Ok(parameter_values)
}

fn validate_biome(biome_file: &PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(biome_file)?;
    let biome: BiomeManifest = serde_yaml::from_str(&content)?;
    
    // Simple validation - just check that it parses correctly
    println!("✅ Biome validation successful");
    println!("   Name: {}", biome.metadata.name);
    println!("   Version: {}", biome.metadata.version);
    println!("   Primals: {}", biome.primals.len());
    println!("   Services: {}", biome.services.len());
    
    // Check for basic required fields
    if biome.metadata.name.is_empty() {
        return Err(anyhow::anyhow!("Biome name cannot be empty"));
    }
    
    if biome.metadata.version.is_empty() {
        return Err(anyhow::anyhow!("Biome version cannot be empty"));
    }
    
    Ok(())
} 