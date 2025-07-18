use biomeos::{UserAuthMethod, UserAuthRequest, UserConfig, UserManager};
use biomeos_core::{BeardogAccessLevel, GeneticBeardogKey};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐕 biomeOS User Management with BearDog Integration Demo");
    println!("==================================================");
    println!();

    // Initialize user manager with BearDog integration
    let config = UserConfig::default(); // BearDog enabled by default
    let user_manager = UserManager::new(config);
    user_manager.initialize().await?;

    println!("✅ User manager initialized with BearDog integration");
    println!();

    // Demo 1: Create users with different access levels
    println!("📝 DEMO 1: Creating users with BearDog keys");
    println!("=========================================");

    // Power user (individual)
    let alice_id = user_manager
        .create_user_with_beardog(
            "alice",
            UserAuthMethod::Password {
                password: "secure_password".to_string(),
            },
            BeardogAccessLevel::PowerUser,
            Some("Alice Cooper".to_string()),
        )
        .await?;

    println!("✅ Created PowerUser 'alice' with ID: {}", alice_id);

    // Small business user (cost-effective)
    let bob_id = user_manager
        .create_user_with_beardog(
            "bob",
            UserAuthMethod::Password {
                password: "business_password".to_string(),
            },
            BeardogAccessLevel::SmallBusiness,
            Some("Bob's Small Business".to_string()),
        )
        .await?;

    println!("✅ Created SmallBusiness 'bob' with ID: {}", bob_id);

    // Research user (humanitarian access)
    let charlie_id = user_manager
        .create_user_with_beardog(
            "charlie",
            UserAuthMethod::Password {
                password: "research_password".to_string(),
            },
            BeardogAccessLevel::Research,
            Some("Charlie Research".to_string()),
        )
        .await?;

    println!("✅ Created Research 'charlie' with ID: {}", charlie_id);
    println!();

    // Demo 2: Authentication through BearDog
    println!("🔐 DEMO 2: Authentication through BearDog");
    println!("=======================================");

    let auth_request = UserAuthRequest {
        username: "alice".to_string(),
        auth_method: UserAuthMethod::Password {
            password: "secure_password".to_string(),
        },
        timestamp: std::time::SystemTime::now(),
        client_ip: Some("192.168.1.100".to_string()),
        client_user_agent: Some("biomeOS-demo/1.0".to_string()),
        metadata: HashMap::new(),
    };

    let session = user_manager.authenticate_with_beardog(auth_request).await?;
    println!("✅ Alice authenticated successfully");
    println!("   Session ID: {}", session.id);
    println!(
        "   Security Level: {}",
        session
            .beardog_context
            .as_ref()
            .and_then(|c| c.get("security_level"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
    );
    println!(
        "   Threat Score: {:.2}",
        session
            .beardog_context
            .as_ref()
            .and_then(|c| c.get("threat_assessment_score"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    );
    println!(
        "   Compliance: {}",
        session
            .beardog_context
            .as_ref()
            .and_then(|c| c.get("compliance_status"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
    );
    println!();

    // Demo 3: SSH Key Management through BearDog
    println!("🔑 DEMO 3: SSH Key Management through BearDog");
    println!("============================================");

    let ssh_key_ref = user_manager
        .add_user_ssh_key(
            "alice",
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQ...",
            "alice_workstation",
        )
        .await?;

    println!("✅ Added SSH key for Alice");
    println!("   Key Reference: {}", ssh_key_ref);
    println!();

    // Demo 4: API Key Generation through BearDog
    println!("🔧 DEMO 4: API Key Generation through BearDog");
    println!("===========================================");

    let api_key_ref = user_manager
        .generate_user_api_key(
            "alice",
            "alice_api_key",
            vec![
                "read".to_string(),
                "write".to_string(),
                "execute".to_string(),
            ],
        )
        .await?;

    println!("✅ Generated API key for Alice");
    println!("   Key Reference: {}", api_key_ref);
    println!();

    // Demo 5: Genetic BearDog Key Authentication
    println!("🧬 DEMO 5: Genetic BearDog Key Authentication");
    println!("===========================================");

    let genetic_key = GeneticBeardogKey {
        parent_key_fingerprint: "beardog_prime_key_abc123".to_string(),
        genetic_lineage: vec![
            "beardog_prime".to_string(),
            "research_branch".to_string(),
            "alice_research_leaf".to_string(),
        ],
        access_level: BeardogAccessLevel::Research,
        encrypted_endpoint: Some("https://beardog.biome.local/research/alice".to_string()),
        valid_until: Some(chrono::Utc::now() + chrono::Duration::days(365)),
    };

    let genetic_auth_request = UserAuthRequest {
        username: "alice".to_string(),
        auth_method: UserAuthMethod::GeneticKey {
            key: genetic_key.parent_key_fingerprint,
        },
        timestamp: std::time::SystemTime::now(),
        client_ip: Some("192.168.1.100".to_string()),
        client_user_agent: Some("research-client/1.0".to_string()),
        metadata: HashMap::new(),
    };

    let genetic_session = user_manager
        .authenticate_with_beardog(genetic_auth_request)
        .await?;
    println!("✅ Alice authenticated with genetic BearDog key");
    println!("   Session ID: {}", genetic_session.id);
    println!(
        "   Security Level: {}",
        genetic_session
            .beardog_context
            .as_ref()
            .and_then(|c| c.get("security_level"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
    );
    println!();

    // Demo 6: User Information Display
    println!("👤 DEMO 6: User Information Display");
    println!("================================");

    let alice = user_manager.get_user("alice").await?.unwrap();
    println!("User: {}", alice.username);
    println!("  Full Name: {}", alice.full_name.unwrap_or_default());
    println!("  Access Level: {:?}", alice.access_level);
    println!(
        "  BearDog Key: {}",
        alice.beardog_key_reference.unwrap_or_default()
    );
    println!("  SSH Keys: {} references", alice.ssh_key_references.len());
    println!("  API Keys: {} references", alice.api_key_references.len());
    println!(
        "  Genetic Key: {}",
        if alice.genetic_key.is_some() {
            "Yes"
        } else {
            "No"
        }
    );
    println!();

    // Demo 7: All Users Summary
    println!("📋 DEMO 7: All Users Summary");
    println!("==========================");

    let all_users = user_manager.get_all_users().await?;
    println!("Total users: {}", all_users.len());

    for user in all_users.iter() {
        println!(
            "  {} - {:?} - {}",
            user.username,
            user.access_level,
            user.full_name.as_deref().unwrap_or("No name")
        );
    }
    println!();

    // Cleanup
    user_manager.shutdown().await?;
    println!("✅ Demo completed successfully!");

    Ok(())
}
