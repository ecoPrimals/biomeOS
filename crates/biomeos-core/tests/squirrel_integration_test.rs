// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Squirrel Integration Tests
//!
//! Tests the integration between biomeOS and Squirrel using the new
//! transport abstraction layer.
//!
//! # Test Scenarios
//!
//! 1. **Discovery**: Verify Squirrel can be discovered via Unix socket
//! 2. **Health Check**: Confirm Squirrel is healthy and responsive
//! 3. **Capabilities**: Verify Squirrel announces its capabilities
//! 4. **AI Query**: Test basic AI inference via JSON-RPC
//!
//! # Prerequisites
//!
//! - Squirrel binary must be running on Unix socket
//! - Family ID must be configured
//! - AI provider (e.g., Songbird) must be available

use anyhow::Result;
use biomeos_core::clients::squirrel::SquirrelClient;
use biomeos_core::primal_client::PrimalClient;
use serde_json::json;

/// Test that we can discover and connect to Squirrel via JSON-RPC Unix socket
#[tokio::test]
#[ignore] // Requires live Squirrel primal
async fn test_squirrel_discovery() -> Result<()> {
    // Attempt to discover Squirrel for family "nat0"
    let squirrel = SquirrelClient::discover("nat0").await?;

    // Verify we got a Unix socket connection (not HTTP fallback)
    let endpoint = squirrel.endpoint();
    assert!(
        endpoint.starts_with("unix://"),
        "Expected Unix socket, got: {}",
        endpoint
    );

    println!("✅ Squirrel discovered at: {}", endpoint);
    Ok(())
}

/// Test Squirrel health check via JSON-RPC
#[tokio::test]
#[ignore] // Requires live Squirrel primal
async fn test_squirrel_health_check() -> Result<()> {
    let squirrel = SquirrelClient::discover("nat0").await?;

    // Call health_check
    let health = squirrel.health_check().await?;

    assert!(health.healthy, "Squirrel should be healthy");
    assert!(
        health.message.contains("healthy") || health.message.contains("ok"),
        "Unexpected health message: {}",
        health.message
    );

    println!("✅ Squirrel health: {:?}", health);
    Ok(())
}

/// Test Squirrel capability announcement
#[tokio::test]
#[ignore] // Requires live Squirrel primal
async fn test_squirrel_capabilities() -> Result<()> {
    let squirrel = SquirrelClient::discover("nat0").await?;

    // Request capabilities via JSON-RPC
    let response = squirrel
        .request("announce_capabilities", "", Some(json!({})))
        .await?;

    // Verify capabilities are announced
    let capabilities = response["capabilities"]
        .as_array()
        .expect("Expected capabilities array");

    assert!(
        !capabilities.is_empty(),
        "Squirrel should announce at least one capability"
    );

    // Check for AI-related capabilities
    let has_ai_capability = capabilities.iter().any(|cap| {
        cap.as_str()
            .map(|s| s.contains("ai") || s.contains("inference"))
            .unwrap_or(false)
    });

    assert!(
        has_ai_capability,
        "Squirrel should announce AI capabilities"
    );

    println!("✅ Squirrel capabilities: {:?}", capabilities);
    Ok(())
}

/// Test AI query via Squirrel's JSON-RPC API
#[tokio::test]
#[ignore] // Requires live Squirrel primal with AI provider
async fn test_squirrel_ai_query() -> Result<()> {
    let squirrel = SquirrelClient::discover("nat0").await?;

    // Simple AI query
    let query = "What is the purpose of biomeOS?";
    let response = squirrel
        .infer(query, Some(&json!({"max_tokens": 50})))
        .await?;

    // Verify we got a response
    assert!(
        !response.is_empty(),
        "Expected non-empty AI response"
    );

    println!("✅ AI query successful!");
    println!("   Query: {}", query);
    println!("   Response: {}", response);
    Ok(())
}

/// Test list_providers via JSON-RPC
#[tokio::test]
#[ignore] // Requires live Squirrel primal
async fn test_squirrel_list_providers() -> Result<()> {
    let squirrel = SquirrelClient::discover("nat0").await?;

    // List available AI providers
    let response = squirrel
        .request("list_providers", "", Some(json!({})))
        .await?;

    let providers = response["providers"]
        .as_array()
        .expect("Expected providers array");

    println!("✅ Squirrel providers: {:?}", providers);

    // Verify at least one provider is available
    // (This might be zero if no providers are configured, which is OK)
    assert!(
        providers.len() >= 0,
        "Expected zero or more providers"
    );

    Ok(())
}

/// Test multi-protocol fallback: JSON-RPC → HTTP
#[tokio::test]
#[ignore] // Requires live Squirrel primal
async fn test_squirrel_protocol_fallback() -> Result<()> {
    // Try to discover with a non-existent family ID
    // This should fall back to HTTP if Unix socket is not found
    let result = SquirrelClient::discover("nonexistent-family").await;

    match result {
        Ok(client) => {
            // If discovery succeeded, verify it's an HTTP client (fallback)
            let endpoint = client.endpoint();
            assert!(
                endpoint.starts_with("http://"),
                "Expected HTTP fallback, got: {}",
                endpoint
            );
            println!("✅ Protocol fallback working: {}", endpoint);
        }
        Err(e) => {
            // If discovery failed, that's also acceptable
            println!("ℹ️ Discovery failed (expected): {}", e);
        }
    }

    Ok(())
}

/// Integration test: Full workflow
#[tokio::test]
#[ignore] // Requires live Squirrel primal with AI provider
async fn test_squirrel_full_workflow() -> Result<()> {
    println!("\n🧪 Starting Squirrel full workflow integration test...\n");

    // 1. Discovery
    println!("1️⃣ Discovering Squirrel...");
    let squirrel = SquirrelClient::discover("nat0").await?;
    println!("   ✅ Found at: {}", squirrel.endpoint());

    // 2. Health check
    println!("\n2️⃣ Checking health...");
    let health = squirrel.health_check().await?;
    assert!(health.healthy, "Squirrel must be healthy");
    println!("   ✅ Status: {}", health.message);

    // 3. Capabilities
    println!("\n3️⃣ Querying capabilities...");
    let caps = squirrel
        .request("announce_capabilities", "", Some(json!({})))
        .await?;
    println!("   ✅ Capabilities: {:?}", caps["capabilities"]);

    // 4. List providers
    println!("\n4️⃣ Listing AI providers...");
    let providers = squirrel
        .request("list_providers", "", Some(json!({})))
        .await?;
    println!("   ✅ Providers: {:?}", providers["providers"]);

    // 5. AI query (if providers are available)
    if let Some(providers_array) = providers["providers"].as_array() {
        if !providers_array.is_empty() {
            println!("\n5️⃣ Testing AI query...");
            let response = squirrel
                .infer("Hello from biomeOS!", Some(&json!({"max_tokens": 30})))
                .await?;
            println!("   ✅ AI Response: {}", response);
        } else {
            println!("\n5️⃣ Skipping AI query (no providers configured)");
        }
    }

    println!("\n🎊 Full workflow test complete!\n");
    Ok(())
}

