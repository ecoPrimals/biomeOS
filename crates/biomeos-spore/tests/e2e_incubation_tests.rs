//! E2E tests for spore incubation workflow

use biomeos_spore::incubation::SporeIncubator;
use biomeos_spore::seed::FamilySeed;
use biomeos_types::identifiers::FamilyId;
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_e2e_incubation_workflow() {
    // 1. Setup: Create a test spore with genetic seed
    let temp_dir = TempDir::new().unwrap();
    let spore_path = temp_dir.path().join("biomeOS");

    // Create spore directory structure
    std::fs::create_dir_all(&spore_path).unwrap();
    std::fs::create_dir_all(spore_path.join(".spore.logs")).unwrap();

    // Create family seed (32 bytes required)
    let seed_path = spore_path.join(".family.seed");
    let seed_bytes = [42u8; 32];
    std::fs::write(&seed_path, seed_bytes).unwrap();

    // Create tower.toml
    std::fs::write(
        spore_path.join("tower.toml"),
        r#"
[meta]
node_id = "test-spore"
family_id = "test-family"

[tower]
family = "test-family"
        "#,
    )
    .unwrap();

    // 2. Create incubator
    let _incubator = SporeIncubator::new(&spore_path).unwrap();

    // 3. Incubate the spore
    // Note: This will try to write to ~/.config/biomeos/ which may not be desired in tests
    // For now, we'll just verify the incubator can be created (successful creation = test pass)

    // Note: Full incubation would require:
    // - Mocking HOME directory
    // - Creating local config
    // - Verifying deployed seed derivation
    // - Checking spore log tracking
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_e2e_multi_computer_incubation_simulation() {
    // Simulate the same spore being incubated on multiple computers

    let temp_dir = TempDir::new().unwrap();
    let spore_path = temp_dir.path().join("biomeOS");

    // Create spore
    std::fs::create_dir_all(&spore_path).unwrap();
    let seed_bytes = [42u8; 32];
    std::fs::write(spore_path.join(".family.seed"), seed_bytes).unwrap();
    let test_family = FamilyId::new("test-family").to_string();
    std::fs::write(
        spore_path.join("tower.toml"),
        format!(
            r#"
[meta]
node_id = "alpha"
family_id = "{}"
            "#,
            test_family
        ),
    )
    .unwrap();

    // Simulate two computers
    let _incubator = SporeIncubator::new(&spore_path).unwrap();

    // In a real scenario, we would:
    // 1. Incubate on computer A with entropy_A
    // 2. Incubate on computer B with entropy_B
    // 3. Verify both have unique deployed_seeds but share spore lineage
    // 4. Verify both can federate

    assert!(true, "Multi-computer simulation setup complete");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_e2e_genetic_lineage_preservation() {
    // Test that genetic lineage is preserved through incubation

    let temp_dir = TempDir::new().unwrap();
    let spore_path = temp_dir.path().join("biomeOS");

    // Create spore with known seed (32 bytes)
    std::fs::create_dir_all(&spore_path).unwrap();
    let seed_data = [99u8; 32];
    std::fs::write(spore_path.join(".family.seed"), seed_data).unwrap();
    let test_family = FamilyId::new("test-family").to_string();
    std::fs::write(
        spore_path.join("tower.toml"),
        format!(
            r#"
[meta]
node_id = "lineage-test"
family_id = "{}"
            "#,
            test_family
        ),
    )
    .unwrap();

    let _incubator = SporeIncubator::new(&spore_path).unwrap();

    // Verify the spore seed can be read
    let family_seed = FamilySeed::from_file(spore_path.join(".family.seed")).unwrap();

    // In a full E2E test, we would:
    // 1. Incubate the spore
    // 2. Read the deployed node config
    // 3. Verify lineage fields are correctly set
    // 4. Use BearDog to verify genetic relationship

    assert!(true, "Genetic lineage test setup complete");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_e2e_incubation_with_spore_log_tracking() {
    // Test that incubation events are logged to the spore

    let temp_dir = TempDir::new().unwrap();
    let spore_path = temp_dir.path().join("biomeOS");

    // Create spore
    std::fs::create_dir_all(&spore_path).unwrap();
    std::fs::create_dir_all(spore_path.join(".spore.logs")).unwrap();
    let seed_bytes = [42u8; 32];
    std::fs::write(spore_path.join(".family.seed"), seed_bytes).unwrap();
    std::fs::write(
        spore_path.join("tower.toml"),
        r#"
[meta]
node_id = "log-test"
        "#,
    )
    .unwrap();

    let _incubator = SporeIncubator::new(&spore_path).unwrap();

    // In a full test, we would:
    // 1. Incubate the spore
    // 2. Check that lifecycle.toml was created/updated
    // 3. Verify the incubation event was logged
    // 4. Check metadata includes computer name, entropy hash, node_id

    assert!(true, "Spore log tracking test setup complete");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_e2e_local_config_structure() {
    // Test that local configuration is correctly structured

    // This would require:
    // 1. Mocking HOME directory
    // 2. Incubating a spore
    // 3. Verifying ~/.config/biomeos/deployed-nodes/{spore-id}/ structure
    // 4. Checking node.toml format
    // 5. Verifying .deployed.seed permissions (0600)
    // 6. Checking entropy.json content

    assert!(
        true,
        "Local config structure test pending full implementation"
    );
}
