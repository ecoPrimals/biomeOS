// Unit tests for SporeVerifier
//
// Tests verification logic, SHA256 checksums, and status detection.

use biomeos_spore::manifest::{BinaryManifest, BinaryInfo};
use biomeos_spore::verification::{SporeVerifier, VerificationStatus};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_calculate_sha256_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.bin");
    
    // Write test data
    let test_data = b"Hello, biomeOS!";
    fs::write(&test_file, test_data).unwrap();
    
    // Calculate SHA256
    let result = SporeVerifier::calculate_sha256(&test_file).await;
    assert!(result.is_ok());
    
    let hash = result.unwrap();
    
    // Verify it matches expected hash
    let mut hasher = Sha256::new();
    hasher.update(test_data);
    let expected = format!("{:x}", hasher.finalize());
    
    assert_eq!(hash, expected);
}

#[tokio::test]
async fn test_calculate_sha256_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let empty_file = temp_dir.path().join("empty.bin");
    
    // Create empty file
    fs::write(&empty_file, b"").unwrap();
    
    let result = SporeVerifier::calculate_sha256(&empty_file).await;
    assert!(result.is_ok());
    
    // Empty file should have predictable hash
    let hash = result.unwrap();
    assert!(!hash.is_empty());
}

#[tokio::test]
async fn test_calculate_sha256_large_file() {
    let temp_dir = TempDir::new().unwrap();
    let large_file = temp_dir.path().join("large.bin");
    
    // Create 1MB file
    let data = vec![0u8; 1024 * 1024];
    fs::write(&large_file, data).unwrap();
    
    let result = SporeVerifier::calculate_sha256(&large_file).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_calculate_sha256_nonexistent_file() {
    let result = SporeVerifier::calculate_sha256("/nonexistent/file.bin").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_binary_fresh() {
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("test-binary");
    
    // Create test binary
    let test_data = b"Test binary content";
    fs::write(&binary_path, test_data).unwrap();
    
    // Calculate actual hash
    let mut hasher = Sha256::new();
    hasher.update(test_data);
    let expected_hash = format!("{:x}", hasher.finalize());
    
    // Create manifest with correct hash
    let manifest = BinaryInfo::new(
        "test-binary".to_string(),
        "1.0.0".to_string(),
        expected_hash.clone(),
        vec![],
    );
    
    // Verify
    let result = SporeVerifier::verify_binary(&binary_path, &manifest).await;
    assert!(result.is_ok());
    
    let report = result.unwrap();
    assert_eq!(report.status, VerificationStatus::Fresh);
    assert_eq!(report.expected_sha256, expected_hash);
    assert_eq!(report.actual_sha256, Some(expected_hash));
}

#[tokio::test]
async fn test_verify_binary_modified() {
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("modified-binary");
    
    // Create binary with one content
    let original_data = b"Original content";
    fs::write(&binary_path, original_data).unwrap();
    
    // Calculate hash of DIFFERENT content
    let mut hasher = Sha256::new();
    hasher.update(b"Different content");
    let wrong_hash = format!("{:x}", hasher.finalize());
    
    // Manifest with wrong hash
    let manifest = BinaryInfo::new(
        "modified-binary".to_string(),
        "1.0.0".to_string(),
        wrong_hash,
        vec![],
    );
    
    // Verify
    let result = SporeVerifier::verify_binary(&binary_path, &manifest).await;
    assert!(result.is_ok());
    
    let report = result.unwrap();
    assert_eq!(report.status, VerificationStatus::Modified);
}

#[tokio::test]
async fn test_verify_binary_missing() {
    let temp_dir = TempDir::new().unwrap();
    let missing_path = temp_dir.path().join("nonexistent");
    
    let manifest = BinaryInfo::new(
        "missing".to_string(),
        "1.0.0".to_string(),
        "any_hash".to_string(),
        vec![],
    );
    
    let result = SporeVerifier::verify_binary(&missing_path, &manifest).await;
    assert!(result.is_ok());
    
    let report = result.unwrap();
    assert_eq!(report.status, VerificationStatus::Missing);
    assert!(report.actual_sha256.is_none());
}

#[test]
fn test_verification_status_equality() {
    assert_eq!(VerificationStatus::Fresh, VerificationStatus::Fresh);
    assert_eq!(VerificationStatus::Stale, VerificationStatus::Stale);
    assert_ne!(VerificationStatus::Fresh, VerificationStatus::Stale);
}

#[tokio::test]
async fn test_sha256_consistency() {
    // Same content should produce same hash
    let temp_dir = TempDir::new().unwrap();
    
    let file1 = temp_dir.path().join("file1.bin");
    let file2 = temp_dir.path().join("file2.bin");
    
    let content = b"Consistent content";
    fs::write(&file1, content).unwrap();
    fs::write(&file2, content).unwrap();
    
    let hash1 = SporeVerifier::calculate_sha256(&file1).await.unwrap();
    let hash2 = SporeVerifier::calculate_sha256(&file2).await.unwrap();
    
    assert_eq!(hash1, hash2);
}

#[tokio::test]
async fn test_sha256_different_content() {
    // Different content should produce different hashes
    let temp_dir = TempDir::new().unwrap();
    
    let file1 = temp_dir.path().join("file1.bin");
    let file2 = temp_dir.path().join("file2.bin");
    
    fs::write(&file1, b"Content A").unwrap();
    fs::write(&file2, b"Content B").unwrap();
    
    let hash1 = SporeVerifier::calculate_sha256(&file1).await.unwrap();
    let hash2 = SporeVerifier::calculate_sha256(&file2).await.unwrap();
    
    assert_ne!(hash1, hash2);
}

#[tokio::test]
async fn test_verification_report_fields() {
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("test");
    
    fs::write(&binary_path, b"test").unwrap();
    
    let actual_hash = SporeVerifier::calculate_sha256(&binary_path).await.unwrap();
    
    let manifest = BinaryInfo::new(
        "test".to_string(),
        "1.0.0".to_string(),
        actual_hash.clone(),
        vec![],
    );
    
    let report = SporeVerifier::verify_binary(&binary_path, &manifest).await.unwrap();
    
    assert_eq!(report.name, "test");
    assert_eq!(report.expected_sha256, actual_hash);
    assert_eq!(report.actual_sha256, Some(actual_hash));
    assert_eq!(report.path, binary_path);
}

#[tokio::test]
async fn test_binary_with_features() {
    let temp_dir = TempDir::new().unwrap();
    let binary_path = temp_dir.path().join("featured-binary");
    
    fs::write(&binary_path, b"binary with features").unwrap();
    let hash = SporeVerifier::calculate_sha256(&binary_path).await.unwrap();
    
    let manifest = BinaryInfo::new(
        "featured-binary".to_string(),
        "2.0.0".to_string(),
        hash.clone(),
        vec!["feature1".to_string(), "feature2".to_string()],
    );
    
    let report = SporeVerifier::verify_binary(&binary_path, &manifest).await.unwrap();
    assert_eq!(report.status, VerificationStatus::Fresh);
}

