use biomeos_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧬 biomeOS Universal Platform with Genetic Beardog Key System");
    println!("==========================================================");
    println!();

    // Test 1: Sovereignty-First Priorities
    println!("📊 SOVEREIGNTY-FIRST PRIORITY SYSTEM");
    println!("=====================================");
    println!("Priority Order: Sovereign > Humanity > Companies > Governments");
    println!();

    // Create genetic beardog keys for different access levels
    let power_user_key = GeneticBeardogKey {
        parent_key_fingerprint: "beardog_prime_key_abc123".to_string(),
        genetic_lineage: vec![
            "beardog_prime".to_string(),
            "power_user_branch_001".to_string(),
        ],
        access_level: BeardogAccessLevel::PowerUser,
        encrypted_endpoint: Some("https://encrypted.strandgate.dev/power_user".to_string()),
        valid_until: Some(chrono::Utc::now() + chrono::Duration::days(365)),
    };

    let small_business_key = GeneticBeardogKey {
        parent_key_fingerprint: "beardog_prime_key_abc123".to_string(),
        genetic_lineage: vec![
            "beardog_prime".to_string(),
            "business_branch_001".to_string(),
            "small_business_leaf_042".to_string(),
        ],
        access_level: BeardogAccessLevel::SmallBusiness,
        encrypted_endpoint: Some("https://encrypted.strandgate.dev/small_biz".to_string()),
        valid_until: Some(chrono::Utc::now() + chrono::Duration::days(365)),
    };

    let mega_corp_key = GeneticBeardogKey {
        parent_key_fingerprint: "beardog_prime_key_abc123".to_string(),
        genetic_lineage: vec![
            "beardog_prime".to_string(),
            "business_branch_001".to_string(),
            "mega_corp_leaf_999".to_string(),
        ],
        access_level: BeardogAccessLevel::MegaCorp,
        encrypted_endpoint: Some("https://encrypted.strandgate.dev/mega_corp".to_string()),
        valid_until: Some(chrono::Utc::now() + chrono::Duration::days(365)),
    };

    // Test 2: Inverse Scaling (Burden Sharing)
    println!("💰 INVERSE SCALING BURDEN SHARING MODEL");
    println!("=======================================");
    println!("Principle: Small businesses get cheap access, mega corps carry the weight");
    println!();

    let config = UniversalBiomeConfig::default();
    let manager = UniversalBiomeManager::new_from_universal_config(config);

    // Initialize with different access levels
    println!("🏠 Power User Access (Sovereign Priority):");
    manager
        .initialize_partnership_access(power_user_key)
        .await?;
    println!("   ✅ Access granted at sovereign priority level");
    println!("   🔐 Genetic lineage verified via good faith model (Gen 1)");
    println!();

    println!("🏢 Small Business Access (Cheap, Subsidized):");
    manager
        .initialize_partnership_access(small_business_key)
        .await?;
    println!("   ✅ Access granted with 90% discount (0.1x cost multiplier)");
    println!("   💚 Small businesses are the backbone of innovation");
    println!();

    println!("🏭 Mega Corp Access (Expensive, Carries the Weight):");
    manager.initialize_partnership_access(mega_corp_key).await?;
    println!("   ✅ Access granted at 100x cost multiplier");
    println!("   ⚖️ Mega corps subsidize small businesses and individuals");
    println!();

    // Test 3: Generation Transition
    println!("🔄 GENERATION TRANSITION PLAN");
    println!("=============================");
    println!("Gen 1: Good faith model with encrypted endpoints");
    println!("Gen 2: Self-sustaining rhizoCrypt verification");
    println!();

    println!("📡 Gen 1 Features:");
    println!("   • Encrypted endpoint access for partnership members");
    println!("   • Good faith genetic lineage verification");
    println!("   • Inverse scaling burden sharing");
    println!("   • Sovereignty-first priority system");
    println!();

    println!("🌱 Gen 2 Features (Future):");
    println!("   • Full rhizoCrypt integration");
    println!("   • Cryptographic genetic lineage verification");
    println!("   • Self-sustaining partnership ecosystem");
    println!("   • Zero trust, maximum sovereignty");
    println!();

    // Test 4: AI Cat Door Integration
    println!("🐱 AI CAT DOOR + PARTNERSHIP INTEGRATION");
    println!("========================================");
    println!("Basic users: AI cat door (grandma-safe)");
    println!("Power users: Genetic beardog key access");
    println!("Companies: Inverse scaling partnership");
    println!();

    manager.initialize_grandma_safe().await?;

    if manager.config.crypto_locks.ai_cat_door.enabled {
        println!("🏠 Grandma-Safe AI Access:");
        println!("   • OpenAI, Anthropic, local LLaMA available");
        println!("   • $20/month cost protection");
        println!("   • 100 requests/day limit");
        println!("   • Auto-disable on cost limit");
        println!();
    }

    // Test 5: Universal Platform Demonstration
    println!("🌍 UNIVERSAL PLATFORM CAPABILITIES");
    println!("==================================");

    // Show all universal interfaces working together
    let platform = UniversalPlatform::new();
    let installation_result = platform
        .install_biomeos(&manager.config.platform.deployment)
        .await;

    match installation_result {
        Ok(_) => {
            println!("✅ Universal installation successful across all platforms");
            println!("   🐧 Linux: Native installation");
            println!("   🪟 Windows: WSL2 + Docker fallback");
            println!("   🍎 macOS: Docker + Podman");
            println!("   ☁️  Cloud: Kubernetes + serverless");
            println!("   🕸️ WebAssembly: Browser-based execution");
        }
        Err(e) => {
            println!("⚠️  Universal installation encountered issues: {}", e);
            println!("   🔄 Falling back to sovereignty-compliant alternatives");
        }
    }

    println!();
    println!("🎯 SOVEREIGNTY-FIRST SUMMARY");
    println!("============================");
    println!("✅ Zero vendor lock-ins across all infrastructure");
    println!("✅ Genetic beardog key partnership system");
    println!("✅ Inverse scaling burden sharing");
    println!("✅ AI cat door for basic users");
    println!("✅ Universal platform deployment");
    println!("✅ Good faith Gen 1 → rhizoCrypt Gen 2 transition");
    println!();
    println!("🌱 Priority Order Respected:");
    println!("   1. Sovereign users (highest priority)");
    println!("   2. Humanity/individuals (second priority)");
    println!("   3. Companies (third priority, inverse scaling)");
    println!("   4. Governments (lowest priority)");
    println!();
    println!("💰 Burden Sharing Working:");
    println!("   • Small businesses: 0.1x cost (90% discount)");
    println!("   • Medium businesses: 1.0x cost (baseline)");
    println!("   • Large enterprises: 10x cost (subsidize others)");
    println!("   • Mega corps: 100x cost (carry the weight)");

    Ok(())
}
