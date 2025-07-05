//! Test Coverage Analyzer
//! 
//! Pure Rust test coverage analysis and reporting tool.
//! Ensures biomeOS meets 50%+ test coverage requirements.

use anyhow::Result;
use clap::{Parser, Subcommand};
use biomeos_tools::{
    testing::{TestConfig, run_all_tests, generate_coverage_report},
    print_section, print_success,
};

#[derive(Parser)]
#[command(name = "test-coverage")]
#[command(about = "biomeOS Test Coverage Analyzer - Ensuring quality through comprehensive testing")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Workspace root directory
    #[arg(short, long, default_value = "/home/strandgate/Development/biomeOS")]
    workspace: String,

    /// Coverage threshold percentage
    #[arg(short, long, default_value = "50.0")]
    threshold: f64,

    /// Test timeout in seconds
    #[arg(short = 'T', long, default_value = "300")]
    timeout: u64,

    /// Run tests in parallel
    #[arg(short = 'Q', long, default_value = "true")]
    parallel: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all tests and generate coverage
    All,
    /// Generate coverage report only
    Coverage,
    /// Run unit tests only
    Unit,
    /// Run integration tests only
    Integration,
    /// Run UI tests only
    Ui,
    /// Run benchmarks
    Bench,
    /// Check if coverage meets threshold
    Check,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let config = TestConfig {
        workspace_root: cli.workspace,
        coverage_threshold: cli.threshold,
        test_timeout: cli.timeout,
        parallel: cli.parallel,
    };

    match cli.command {
        Some(Commands::All) | None => {
            run_comprehensive_testing(&config).await?;
        }
        Some(Commands::Coverage) => {
            run_coverage_only(&config).await?;
        }
        Some(Commands::Unit) => {
            run_unit_tests_only(&config).await?;
        }
        Some(Commands::Integration) => {
            run_integration_tests_only(&config).await?;
        }
        Some(Commands::Ui) => {
            run_ui_tests_only(&config).await?;
        }
        Some(Commands::Bench) => {
            run_benchmarks_only(&config).await?;
        }
        Some(Commands::Check) => {
            check_coverage_threshold(&config).await?;
        }
    }

    Ok(())
}

/// Run comprehensive testing suite
async fn run_comprehensive_testing(config: &TestConfig) -> Result<()> {
    print_section("🧪 COMPREHENSIVE TESTING SUITE");
    biomeos_tools::print_info("Running all tests and generating coverage report...");
    
    // Run all tests
    run_all_tests(config).await?;
    
    // Generate coverage report
    let coverage = generate_coverage_report(config).await?;
    
    // Print final summary
    print_final_summary(coverage, config.coverage_threshold);
    
    Ok(())
}

/// Run coverage analysis only
async fn run_coverage_only(config: &TestConfig) -> Result<()> {
    print_section("📊 COVERAGE ANALYSIS ONLY");
    
    let coverage = generate_coverage_report(config).await?;
    
    print_coverage_summary(coverage, config.coverage_threshold);
    
    Ok(())
}

/// Run unit tests only
async fn run_unit_tests_only(config: &TestConfig) -> Result<()> {
    print_section("🔬 UNIT TESTS ONLY");
    
    let workspace_path = std::path::Path::new(&config.workspace_root);
    
    biomeos_tools::execute_command(
        "cargo",
        &["test", "--workspace", "--lib"],
        Some(&workspace_path)
    ).await?;
    
    print_success("Unit tests completed successfully");
    Ok(())
}

/// Run integration tests only
async fn run_integration_tests_only(config: &TestConfig) -> Result<()> {
    print_section("🔗 INTEGRATION TESTS ONLY");
    
    let workspace_path = std::path::Path::new(&config.workspace_root);
    
    biomeos_tools::execute_command(
        "cargo",
        &["test", "--workspace", "--test", "*"],
        Some(&workspace_path)
    ).await?;
    
    print_success("Integration tests completed successfully");
    Ok(())
}

/// Run UI tests only
async fn run_ui_tests_only(config: &TestConfig) -> Result<()> {
    print_section("🎨 UI TESTS ONLY");
    
    let workspace_path = std::path::Path::new(&config.workspace_root);
    
    // Test UI compilation and functionality
    biomeos_tools::execute_command(
        "cargo",
        &["test", "-p", "biomeos-ui"],
        Some(&workspace_path)
    ).await?;
    
    // Test UI builds
    biomeos_tools::execute_command(
        "cargo",
        &["build", "-p", "biomeos-ui"],
        Some(&workspace_path)
    ).await?;
    
    print_success("UI tests completed successfully");
    Ok(())
}

/// Run benchmarks only
async fn run_benchmarks_only(config: &TestConfig) -> Result<()> {
    print_section("⚡ BENCHMARKS ONLY");
    
    let workspace_path = std::path::Path::new(&config.workspace_root);
    
    biomeos_tools::execute_command(
        "cargo",
        &["bench", "--workspace"],
        Some(&workspace_path)
    ).await?;
    
    print_success("Benchmarks completed successfully");
    Ok(())
}

/// Check if coverage meets threshold
async fn check_coverage_threshold(config: &TestConfig) -> Result<()> {
    print_section("✅ COVERAGE THRESHOLD CHECK");
    
    let coverage = generate_coverage_report(config).await?;
    
    if coverage >= config.coverage_threshold {
        print_success(&format!(
            "🎯 PASSED: Coverage {:.1}% meets threshold of {:.1}%", 
            coverage, 
            config.coverage_threshold
        ));
        println!("\n✅ QUALITY GATE: PASSED");
    } else {
        biomeos_tools::print_error(&format!(
            "❌ FAILED: Coverage {:.1}% below threshold of {:.1}%", 
            coverage, 
            config.coverage_threshold
        ));
        println!("\n❌ QUALITY GATE: FAILED");
        println!("📋 ACTION REQUIRED: Add more tests to reach {:.1}% coverage", config.coverage_threshold);
        
        // Suggest specific areas needing tests
        suggest_testing_improvements();
        
        anyhow::bail!("Coverage threshold not met");
    }
    
    Ok(())
}

/// Print final testing summary
fn print_final_summary(coverage: f64, threshold: f64) {
    print_section("📈 TESTING SUMMARY");
    
    println!("🧪 Test Results:");
    println!("  ✅ Unit Tests: PASSED");
    println!("  ✅ Integration Tests: PASSED");
    println!("  ✅ UI Tests: PASSED");
    println!("  ✅ Build Tests: PASSED");
    
    print_coverage_summary(coverage, threshold);
    
    if coverage >= threshold {
        println!("\n🎉 SUCCESS: All tests passed and coverage target achieved!");
        println!("🚀 biomeOS is ready for production deployment");
    } else {
        println!("\n⚠️ WARNING: Tests passed but coverage below target");
        println!("📋 Recommendation: Add more tests before production");
    }
}

/// Print coverage summary
fn print_coverage_summary(coverage: f64, threshold: f64) {
    println!("\n📊 Coverage Analysis:");
    println!("  Current Coverage: {:.1}%", coverage);
    println!("  Target Threshold: {:.1}%", threshold);
    
    let status = if coverage >= threshold {
        "✅ PASSED"
    } else {
        "❌ NEEDS IMPROVEMENT"
    };
    
    println!("  Status: {}", status);
    
    // Coverage quality assessment
    let quality = match coverage {
        c if c >= 90.0 => "🟢 EXCELLENT",
        c if c >= 80.0 => "🟡 GOOD", 
        c if c >= 70.0 => "🟡 ACCEPTABLE",
        c if c >= 50.0 => "🟠 MINIMUM",
        _ => "🔴 POOR"
    };
    
    println!("  Quality: {}", quality);
}

/// Suggest testing improvements
fn suggest_testing_improvements() {
    println!("\n💡 TESTING IMPROVEMENT SUGGESTIONS:");
    println!("  📝 Add unit tests for core modules");
    println!("  🔗 Add integration tests for API endpoints");
    println!("  🎨 Add UI component tests");
    println!("  🧪 Add property-based tests");
    println!("  📊 Add performance regression tests");
    println!("  🔒 Add security-focused tests");
    println!("  🌐 Add cross-platform compatibility tests");
    
    println!("\n🎯 PRIORITY AREAS:");
    println!("  1. biomeOS core functionality");
    println!("  2. Crypto lock system");
    println!("  3. Genetic beardog keys");
    println!("  4. AI cat door protection");
    println!("  5. UI state management");
    println!("  6. API error handling");
} 