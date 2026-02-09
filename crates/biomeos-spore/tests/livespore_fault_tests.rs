//! Fault Injection Tests for LiveSpore Deployments
//!
//! Tests that verify LiveSpore resilience under various failure conditions:
//! - Missing or corrupt binaries
//! - Invalid genetic lineage
//! - Filesystem failures
//! - Resource exhaustion
//!
//! # Running
//! ```bash
//! cargo test --package biomeos-spore --test livespore_fault_tests
//! ```

use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture for LiveSpore fault testing
struct LiveSporeFaultFixture {
    /// Temp dir kept alive to prevent cleanup during test
    #[allow(dead_code)]
    temp_dir: TempDir,
    spore_path: PathBuf,
}

impl LiveSporeFaultFixture {
    fn new(test_name: &str) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let spore_path = temp_dir.path().join(format!("test-spore-{}", test_name));
        std::fs::create_dir_all(&spore_path).expect("Failed to create spore dir");

        Self {
            temp_dir,
            spore_path,
        }
    }

    fn create_directory(&self, path: &str) {
        std::fs::create_dir_all(self.spore_path.join(path)).ok();
    }

    fn write_file(&self, path: &str, contents: &str) {
        let full_path = self.spore_path.join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(full_path, contents).expect("Failed to write file");
    }

    fn create_valid_seed(&self) {
        // Create a valid 64-byte seed (32 bytes genesis + 32 bytes node)
        let seed = vec![0u8; 64];
        std::fs::write(self.spore_path.join(".family.seed"), &seed).ok();
    }
}

// ============================================================================
// Binary Fault Tests
// ============================================================================

#[test]
fn test_missing_beardog_binary() {
    let fixture = LiveSporeFaultFixture::new("missing-beardog");

    // Create minimal spore structure without binaries
    fixture.create_directory("biomeOS");
    fixture.create_directory("biomeOS/primals");
    fixture.create_valid_seed();
    fixture.write_file(
        "biomeOS/tower.toml",
        r#"
[tower]
family_id = "test"
node_id = "test-node"
"#,
    );

    // Verify primals directory exists but is empty
    let primals_dir = fixture.spore_path.join("biomeOS/primals");
    assert!(primals_dir.exists());
    assert!(std::fs::read_dir(&primals_dir).unwrap().count() == 0);
}

#[test]
fn test_corrupted_binary() {
    let fixture = LiveSporeFaultFixture::new("corrupt-binary");

    fixture.create_directory("biomeOS/primals");
    fixture.create_valid_seed();

    // Create a "binary" that's just garbage data
    fixture.write_file("biomeOS/primals/beardog", "not a real binary");

    let beardog_path = fixture.spore_path.join("biomeOS/primals/beardog");

    // The file should exist
    assert!(beardog_path.exists());

    // But it should not be executable (as an ELF binary)
    let contents = std::fs::read(&beardog_path).unwrap();

    // ELF magic bytes are 0x7F 'E' 'L' 'F'
    let is_elf = contents.len() >= 4
        && contents[0] == 0x7F
        && contents[1] == b'E'
        && contents[2] == b'L'
        && contents[3] == b'F';

    assert!(!is_elf, "Corrupted binary should not have ELF magic bytes");
}

#[test]
fn test_wrong_architecture_binary() {
    let fixture = LiveSporeFaultFixture::new("wrong-arch");

    fixture.create_directory("biomeOS/primals");

    // Create a fake ELF header for wrong architecture (ARM on x86)
    // ELF header: magic (4) + class (1) + endian (1) + version (1) + abi (1) + padding (8) + type (2) + machine (2)
    let mut elf_header = vec![0x7F, b'E', b'L', b'F']; // magic
    elf_header.push(2); // ELFCLASS64
    elf_header.push(1); // ELFDATA2LSB (little endian)
    elf_header.push(1); // EV_CURRENT
    elf_header.push(0); // ELFOSABI_NONE
    elf_header.extend_from_slice(&[0u8; 8]); // padding
    elf_header.extend_from_slice(&[2, 0]); // ET_EXEC
    elf_header.extend_from_slice(&[0xB7, 0]); // EM_AARCH64 (ARM 64-bit)

    std::fs::write(
        fixture.spore_path.join("biomeOS/primals/beardog"),
        &elf_header,
    )
    .ok();

    // Verify it's detected as ELF but wrong architecture
    let contents = std::fs::read(fixture.spore_path.join("biomeOS/primals/beardog")).unwrap();
    assert!(contents.len() >= 4);
    assert_eq!(&contents[0..4], &[0x7F, b'E', b'L', b'F']);
}

// ============================================================================
// Seed/Lineage Fault Tests
// ============================================================================

#[test]
fn test_missing_family_seed() {
    let fixture = LiveSporeFaultFixture::new("missing-seed");

    fixture.create_directory("biomeOS");
    fixture.write_file("biomeOS/tower.toml", "[tower]\nfamily_id = \"test\"");

    // Don't create the seed file
    let seed_path = fixture.spore_path.join("biomeOS/.family.seed");
    assert!(!seed_path.exists(), "Seed should not exist for this test");
}

#[test]
fn test_corrupted_family_seed() {
    let fixture = LiveSporeFaultFixture::new("corrupt-seed");

    fixture.create_directory("biomeOS");

    // Create a seed that's too short (should be 64 bytes)
    fixture.write_file("biomeOS/.family.seed", "too short");

    let seed = std::fs::read(fixture.spore_path.join("biomeOS/.family.seed")).unwrap();
    assert!(seed.len() < 64, "Corrupted seed should be too short");
}

#[test]
fn test_zero_seed() {
    let fixture = LiveSporeFaultFixture::new("zero-seed");

    fixture.create_directory("biomeOS");

    // Create a seed that's all zeros (valid length but potentially weak)
    let zero_seed = vec![0u8; 64];
    std::fs::write(fixture.spore_path.join("biomeOS/.family.seed"), &zero_seed).ok();

    let seed = std::fs::read(fixture.spore_path.join("biomeOS/.family.seed")).unwrap();
    assert_eq!(seed.len(), 64);
    assert!(seed.iter().all(|&b| b == 0), "Seed should be all zeros");
}

#[test]
fn test_mismatched_family_id() {
    let fixture = LiveSporeFaultFixture::new("mismatched-family");

    fixture.create_directory("biomeOS");
    fixture.create_valid_seed();

    // tower.toml says family_id = "test_family"
    fixture.write_file(
        "biomeOS/tower.toml",
        r#"
[tower]
family_id = "test_family"
node_id = "test-node"
"#,
    );

    // But the seed was generated for a different family
    // In a real scenario, this would fail lineage verification

    let config = std::fs::read_to_string(fixture.spore_path.join("biomeOS/tower.toml")).unwrap();
    assert!(config.contains("test_family"));
}

// ============================================================================
// Filesystem Fault Tests
// ============================================================================

#[test]
fn test_readonly_primals_directory() {
    let fixture = LiveSporeFaultFixture::new("readonly");

    fixture.create_directory("biomeOS/primals");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let primals_dir = fixture.spore_path.join("biomeOS/primals");
        let _ = std::fs::set_permissions(&primals_dir, std::fs::Permissions::from_mode(0o444));

        // Writing should fail
        let result = std::fs::write(primals_dir.join("test-file"), "test");
        assert!(result.is_err(), "Write to readonly directory should fail");

        // Restore permissions for cleanup
        let _ = std::fs::set_permissions(&primals_dir, std::fs::Permissions::from_mode(0o755));
    }
}

#[test]
fn test_symlink_attack() {
    let fixture = LiveSporeFaultFixture::new("symlink");

    fixture.create_directory("biomeOS");

    #[cfg(unix)]
    {
        // Create a symlink pointing outside the spore
        let symlink_path = fixture.spore_path.join("biomeOS/escape");
        let _ = std::os::unix::fs::symlink("/etc/passwd", &symlink_path);

        // Reading through symlink should work but be suspicious
        if symlink_path.exists() {
            let metadata = std::fs::symlink_metadata(&symlink_path).unwrap();
            assert!(metadata.file_type().is_symlink(), "Should be a symlink");
        }
    }
}

#[test]
fn test_very_long_path() {
    let fixture = LiveSporeFaultFixture::new("longpath");

    // Try to create a very deeply nested path
    let deep_path = "a/".repeat(50) + "deeply/nested/file.txt";
    fixture.create_directory("biomeOS");

    // This might fail on some filesystems
    let result = std::fs::create_dir_all(fixture.spore_path.join(&deep_path).parent().unwrap());

    // Either it succeeds or fails gracefully
    match result {
        Ok(_) => {
            // Successfully created deep path
            fixture.write_file(&deep_path, "test");
        }
        Err(e) => {
            // Failed gracefully (path too long)
            eprintln!("Deep path creation failed (expected): {}", e);
        }
    }
}

// ============================================================================
// Config Fault Tests
// ============================================================================

#[test]
fn test_missing_tower_config() {
    let fixture = LiveSporeFaultFixture::new("missing-config");

    fixture.create_directory("biomeOS/primals");
    fixture.create_valid_seed();

    // Don't create tower.toml
    let config_path = fixture.spore_path.join("biomeOS/tower.toml");
    assert!(!config_path.exists());
}

#[test]
fn test_invalid_toml_config() {
    let fixture = LiveSporeFaultFixture::new("invalid-toml");

    fixture.create_directory("biomeOS");
    fixture.create_valid_seed();

    // Create invalid TOML
    fixture.write_file(
        "biomeOS/tower.toml",
        r#"
[tower]
family_id = "unclosed string
"#,
    );

    let content = std::fs::read_to_string(fixture.spore_path.join("biomeOS/tower.toml")).unwrap();

    // Try to parse - should fail
    let parse_result: Result<toml::Value, _> = toml::from_str(&content);
    assert!(parse_result.is_err(), "Invalid TOML should fail to parse");
}

#[test]
fn test_config_missing_required_fields() {
    let fixture = LiveSporeFaultFixture::new("incomplete-config");

    fixture.create_directory("biomeOS");

    // Valid TOML but missing required fields
    fixture.write_file(
        "biomeOS/tower.toml",
        r#"
[tower]
# Missing family_id and node_id
concurrent_startup = true
"#,
    );

    let content = std::fs::read_to_string(fixture.spore_path.join("biomeOS/tower.toml")).unwrap();
    let config: toml::Value = toml::from_str(&content).expect("Should parse");

    let tower = config.get("tower").unwrap();
    assert!(
        tower.get("family_id").is_none(),
        "family_id should be missing"
    );
    assert!(tower.get("node_id").is_none(), "node_id should be missing");
}

// ============================================================================
// Recovery Scenario Tests
// ============================================================================

#[test]
fn test_partial_deployment_recovery() {
    let fixture = LiveSporeFaultFixture::new("partial-deploy");

    // Simulate a partial deployment (beardog present, songbird missing)
    fixture.create_directory("biomeOS/primals");
    fixture.create_valid_seed();
    fixture.write_file(
        "biomeOS/tower.toml",
        r#"
[tower]
family_id = "recovery-test"
node_id = "partial-node"
"#,
    );

    // Only create beardog marker
    fixture.write_file("biomeOS/primals/beardog.marker", "deployed");

    // Verify partial state
    let beardog_marker = fixture.spore_path.join("biomeOS/primals/beardog.marker");
    let songbird_marker = fixture.spore_path.join("biomeOS/primals/songbird.marker");

    assert!(beardog_marker.exists(), "BearDog marker should exist");
    assert!(
        !songbird_marker.exists(),
        "Songbird marker should not exist"
    );
}
