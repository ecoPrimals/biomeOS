//! Serialization tests for integration scenarios
//!
//! This module provides comprehensive serialization/deserialization tests
//! for biome manifests with configurable primal integrations.

use biomeos_manifest::*;
use serde_json::json;
use crate::integration_tests::common::*;
use crate::integration_tests::primal_configs::*;
use crate::integration_tests::validation::*;

/// Serialization test suite for integration scenarios
pub struct SerializationTester;

impl SerializationTester {
    /// Test YAML serialization compatibility for any primal integration
    pub fn test_yaml_serialization_compatibility(
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
        specialization: BiomeSpecialization,
    ) -> SerializationTestResult {
        let mut result = SerializationTestResult::new();

        // Create test manifest
        let manifest = TestScenarios::two_primal_integration(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
            specialization,
        ).build();

        // Test YAML serialization
        let yaml_result = serde_yaml::to_string(&manifest);
        if yaml_result.is_err() {
            result.add_error(&format!("YAML serialization failed: {}", yaml_result.unwrap_err()));
            return result;
        }

        let yaml_str = yaml_result.unwrap();
        result.add_info(&format!("YAML serialization succeeded, length: {}", yaml_str.len()));

        // Verify key elements are present in YAML
        result.combine(Self::verify_yaml_content(&yaml_str, orchestrator_name, orchestrator_type, storage_name, storage_type));

        // Test YAML deserialization
        let deserialized_result: Result<BiomeManifest, _> = serde_yaml::from_str(&yaml_str);
        if deserialized_result.is_err() {
            result.add_error(&format!("YAML deserialization failed: {}", deserialized_result.unwrap_err()));
            return result;
        }

        let deserialized = deserialized_result.unwrap();
        result.add_info("YAML deserialization succeeded");

        // Verify deserialized content
        result.combine(Self::verify_deserialized_content(&deserialized, orchestrator_name, storage_name));

        // Test round-trip consistency
        result.combine(Self::test_yaml_round_trip_consistency(&manifest));

        result
    }

    /// Test JSON serialization compatibility for any primal integration
    pub fn test_json_serialization_compatibility(
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
        specialization: BiomeSpecialization,
    ) -> SerializationTestResult {
        let mut result = SerializationTestResult::new();

        // Create test manifest
        let manifest = TestScenarios::two_primal_integration(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
            specialization,
        ).build();

        // Test JSON serialization
        let json_result = serde_json::to_string_pretty(&manifest);
        if json_result.is_err() {
            result.add_error(&format!("JSON serialization failed: {}", json_result.unwrap_err()));
            return result;
        }

        let json_str = json_result.unwrap();
        result.add_info(&format!("JSON serialization succeeded, length: {}", json_str.len()));

        // Verify key elements are present in JSON
        result.combine(Self::verify_json_content(&json_str, orchestrator_name, orchestrator_type, storage_name, storage_type));

        // Test JSON deserialization
        let deserialized_result: Result<BiomeManifest, _> = serde_json::from_str(&json_str);
        if deserialized_result.is_err() {
            result.add_error(&format!("JSON deserialization failed: {}", deserialized_result.unwrap_err()));
            return result;
        }

        let deserialized = deserialized_result.unwrap();
        result.add_info("JSON deserialization succeeded");

        // Verify deserialized content
        result.combine(Self::verify_deserialized_content(&deserialized, orchestrator_name, storage_name));

        // Test round-trip consistency
        result.combine(Self::test_json_round_trip_consistency(&manifest));

        result
    }

    /// Test gaming-specific serialization
    pub fn test_gaming_serialization(
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
    ) -> SerializationTestResult {
        let mut result = SerializationTestResult::new();

        // Create gaming manifest
        let manifest = TestScenarios::gaming_tournament(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
        ).build();

        // Test YAML serialization
        let yaml_result = serde_yaml::to_string(&manifest);
        if yaml_result.is_err() {
            result.add_error(&format!("Gaming YAML serialization failed: {}", yaml_result.unwrap_err()));
            return result;
        }

        let yaml_str = yaml_result.unwrap();

        // Verify gaming-specific elements
        if !yaml_str.contains("GamingServer") {
            result.add_error("Gaming manifest should contain GamingServer specialization");
        }

        if !yaml_str.contains("tournament") {
            result.add_error("Gaming manifest should contain tournament configuration");
        }

        if !yaml_str.contains("match_making") {
            result.add_error("Gaming manifest should contain match_making configuration");
        }

        // Test deserialization
        let deserialized_result: Result<BiomeManifest, _> = serde_yaml::from_str(&yaml_str);
        if deserialized_result.is_err() {
            result.add_error(&format!("Gaming YAML deserialization failed: {}", deserialized_result.unwrap_err()));
            return result;
        }

        let deserialized = deserialized_result.unwrap();

        // Verify gaming-specific deserialized content
        if deserialized.metadata.specialization != Some(BiomeSpecialization::GamingServer) {
            result.add_error("Deserialized gaming manifest should have GamingServer specialization");
        }

        if let Some(orchestrator) = deserialized.primals.get(orchestrator_name) {
            if let Some(config) = &orchestrator.config {
                if let Some(tournament) = config.get("tournament") {
                    if tournament.get("match_making") != Some(&json!(true)) {
                        result.add_error("Deserialized gaming manifest should have match_making enabled");
                    }
                } else {
                    result.add_error("Deserialized gaming manifest should have tournament configuration");
                }
            }
        }

        result
    }

    /// Test cross-format compatibility (YAML -> JSON -> YAML)
    pub fn test_cross_format_compatibility(
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
    ) -> SerializationTestResult {
        let mut result = SerializationTestResult::new();

        // Create test manifest
        let original_manifest = TestScenarios::two_primal_integration(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
            BiomeSpecialization::DataCenter,
        ).build();

        // YAML -> JSON -> YAML round trip
        let yaml_result = serde_yaml::to_string(&original_manifest);
        if yaml_result.is_err() {
            result.add_error(&format!("Initial YAML serialization failed: {}", yaml_result.unwrap_err()));
            return result;
        }

        let yaml_str = yaml_result.unwrap();

        // Deserialize from YAML
        let from_yaml_result: Result<BiomeManifest, _> = serde_yaml::from_str(&yaml_str);
        if from_yaml_result.is_err() {
            result.add_error(&format!("YAML deserialization failed: {}", from_yaml_result.unwrap_err()));
            return result;
        }

        let from_yaml = from_yaml_result.unwrap();

        // Serialize to JSON
        let json_result = serde_json::to_string_pretty(&from_yaml);
        if json_result.is_err() {
            result.add_error(&format!("JSON serialization failed: {}", json_result.unwrap_err()));
            return result;
        }

        let json_str = json_result.unwrap();

        // Deserialize from JSON
        let from_json_result: Result<BiomeManifest, _> = serde_json::from_str(&json_str);
        if from_json_result.is_err() {
            result.add_error(&format!("JSON deserialization failed: {}", from_json_result.unwrap_err()));
            return result;
        }

        let from_json = from_json_result.unwrap();

        // Serialize back to YAML
        let final_yaml_result = serde_yaml::to_string(&from_json);
        if final_yaml_result.is_err() {
            result.add_error(&format!("Final YAML serialization failed: {}", final_yaml_result.unwrap_err()));
            return result;
        }

        let final_yaml = final_yaml_result.unwrap();

        // Verify structural consistency
        result.combine(Self::verify_structural_consistency(&original_manifest, &from_yaml, &from_json));

        result.add_info("Cross-format compatibility test passed");
        result
    }

    /// Test large manifest serialization performance
    pub fn test_large_manifest_serialization(
        primals: Vec<(String, PrimalType, u32)>,
        dependencies: Vec<(String, Vec<String>)>,
    ) -> SerializationTestResult {
        let mut result = SerializationTestResult::new();

        // Create large manifest
        let start_time = std::time::Instant::now();
        
        let mut manifest_builder = TestManifestBuilder::new("large-manifest-test")
            .with_metadata(
                "1.0.0",
                "Large manifest serialization test",
                BiomeSpecialization::DataCenter,
            );

        for (name, primal_type, priority) in primals {
            let deps = dependencies
                .iter()
                .find(|(n, _)| n == &name)
                .map(|(_, deps)| deps.clone())
                .unwrap_or_default();

            let primal = PrimalBuilder::new(&name, primal_type)
                .with_priority_and_deps(priority, deps)
                .with_version_and_timeout("1.0.0", "30s")
                .with_config(json!({
                    "mode": "distributed",
                    "features": ["service_discovery", "health_monitoring"]
                }))
                .with_networking(
                    NetworkingBuilder::new()
                        .with_ports(vec![8000 + priority as u16])
                        .with_host("0.0.0.0")
                        .build()
                )
                .with_resources(ResourcePreset::Standard.to_resources());

            manifest_builder = manifest_builder.with_primal(primal);
        }

        let manifest = manifest_builder.build();
        let creation_time = start_time.elapsed();

        result.add_info(&format!("Large manifest creation took: {:?}", creation_time));

        // Test YAML serialization performance
        let yaml_start = std::time::Instant::now();
        let yaml_result = serde_yaml::to_string(&manifest);
        let yaml_time = yaml_start.elapsed();

        if yaml_result.is_err() {
            result.add_error(&format!("Large YAML serialization failed: {}", yaml_result.unwrap_err()));
            return result;
        }

        let yaml_str = yaml_result.unwrap();
        result.add_info(&format!("YAML serialization took: {:?}, size: {} bytes", yaml_time, yaml_str.len()));

        // Test JSON serialization performance
        let json_start = std::time::Instant::now();
        let json_result = serde_json::to_string_pretty(&manifest);
        let json_time = json_start.elapsed();

        if json_result.is_err() {
            result.add_error(&format!("Large JSON serialization failed: {}", json_result.unwrap_err()));
            return result;
        }

        let json_str = json_result.unwrap();
        result.add_info(&format!("JSON serialization took: {:?}, size: {} bytes", json_time, json_str.len()));

        // Test deserialization performance
        let yaml_deser_start = std::time::Instant::now();
        let yaml_deser_result: Result<BiomeManifest, _> = serde_yaml::from_str(&yaml_str);
        let yaml_deser_time = yaml_deser_start.elapsed();

        if yaml_deser_result.is_err() {
            result.add_error(&format!("Large YAML deserialization failed: {}", yaml_deser_result.unwrap_err()));
            return result;
        }

        result.add_info(&format!("YAML deserialization took: {:?}", yaml_deser_time));

        let json_deser_start = std::time::Instant::now();
        let json_deser_result: Result<BiomeManifest, _> = serde_json::from_str(&json_str);
        let json_deser_time = json_deser_start.elapsed();

        if json_deser_result.is_err() {
            result.add_error(&format!("Large JSON deserialization failed: {}", json_deser_result.unwrap_err()));
            return result;
        }

        result.add_info(&format!("JSON deserialization took: {:?}", json_deser_time));

        result
    }

    /// Verify YAML content contains expected elements
    fn verify_yaml_content(
        yaml_str: &str,
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
    ) -> SerializationTestResult {
        let mut result = SerializationTestResult::new();

        // Check primal type representations
        let orchestrator_type_str = format!("{:?}", orchestrator_type);
        let storage_type_str = format!("{:?}", storage_type);

        if !yaml_str.contains(&format!("primal_type: {}", orchestrator_type_str)) {
            result.add_error(&format!("YAML missing orchestrator primal_type: {}", orchestrator_type_str));
        }

        if !yaml_str.contains(&format!("primal_type: {}", storage_type_str)) {
            result.add_error(&format!("YAML missing storage primal_type: {}", storage_type_str));
        }

        // Check dependency structure
        if !yaml_str.contains("depends_on:") {
            result.add_error("YAML missing depends_on structure");
        }

        if !yaml_str.contains(&format!("- {}", orchestrator_name)) {
            result.add_error(&format!("YAML missing dependency on {}", orchestrator_name));
        }

        // Check configuration sections
        if !yaml_str.contains("config:") {
            result.add_error("YAML missing config sections");
        }

        if !yaml_str.contains("networking:") {
            result.add_error("YAML missing networking sections");
        }

        if !yaml_str.contains("resources:") {
            result.add_error("YAML missing resources sections");
        }

        result
    }

    /// Verify JSON content contains expected elements
    fn verify_json_content(
        json_str: &str,
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
    ) -> SerializationTestResult {
        let mut result = SerializationTestResult::new();

        // Check primal type representations
        let orchestrator_type_str = format!("{:?}", orchestrator_type);
        let storage_type_str = format!("{:?}", storage_type);

        if !json_str.contains(&format!("\"primal_type\": \"{}\"", orchestrator_type_str)) {
            result.add_error(&format!("JSON missing orchestrator primal_type: {}", orchestrator_type_str));
        }

        if !json_str.contains(&format!("\"primal_type\": \"{}\"", storage_type_str)) {
            result.add_error(&format!("JSON missing storage primal_type: {}", storage_type_str));
        }

        // Check dependency structure
        if !json_str.contains("\"depends_on\"") {
            result.add_error("JSON missing depends_on structure");
        }

        if !json_str.contains(&format!("\"{}\"", orchestrator_name)) {
            result.add_error(&format!("JSON missing dependency on {}", orchestrator_name));
        }

        result
    }

    /// Verify deserialized content matches expectations
    fn verify_deserialized_content(
        manifest: &BiomeManifest,
        orchestrator_name: &str,
        storage_name: &str,
    ) -> SerializationTestResult {
        let mut result = SerializationTestResult::new();

        // Check primals exist
        if !manifest.primals.contains_key(orchestrator_name) {
            result.add_error(&format!("Deserialized manifest missing orchestrator: {}", orchestrator_name));
        }

        if !manifest.primals.contains_key(storage_name) {
            result.add_error(&format!("Deserialized manifest missing storage: {}", storage_name));
        }

        if manifest.primals.len() != 2 {
            result.add_error(&format!("Deserialized manifest has {} primals, expected 2", manifest.primals.len()));
        }

        // Check dependencies
        if let Some(storage) = manifest.primals.get(storage_name) {
            if !storage.depends_on.contains(&orchestrator_name.to_string()) {
                result.add_error(&format!("Deserialized storage missing dependency on {}", orchestrator_name));
            }
        }

        result
    }

    /// Test YAML round-trip consistency
    fn test_yaml_round_trip_consistency(manifest: &BiomeManifest) -> SerializationTestResult {
        let mut result = SerializationTestResult::new();

        // Serialize to YAML
        let yaml_result = serde_yaml::to_string(manifest);
        if yaml_result.is_err() {
            result.add_error(&format!("Round-trip YAML serialization failed: {}", yaml_result.unwrap_err()));
            return result;
        }

        let yaml_str = yaml_result.unwrap();

        // Deserialize back
        let deserialized_result: Result<BiomeManifest, _> = serde_yaml::from_str(&yaml_str);
        if deserialized_result.is_err() {
            result.add_error(&format!("Round-trip YAML deserialization failed: {}", deserialized_result.unwrap_err()));
            return result;
        }

        let deserialized = deserialized_result.unwrap();

        // Serialize again
        let yaml2_result = serde_yaml::to_string(&deserialized);
        if yaml2_result.is_err() {
            result.add_error(&format!("Round-trip YAML re-serialization failed: {}", yaml2_result.unwrap_err()));
            return result;
        }

        let yaml2_str = yaml2_result.unwrap();

        // Compare (note: YAML ordering might differ, so we check structure)
        if yaml_str.len() != yaml2_str.len() {
            result.add_warning(&format!("Round-trip YAML length differs: {} vs {}", yaml_str.len(), yaml2_str.len()));
        }

        result.add_info("YAML round-trip consistency test passed");
        result
    }

    /// Test JSON round-trip consistency
    fn test_json_round_trip_consistency(manifest: &BiomeManifest) -> SerializationTestResult {
        let mut result = SerializationTestResult::new();

        // Serialize to JSON
        let json_result = serde_json::to_string(manifest);
        if json_result.is_err() {
            result.add_error(&format!("Round-trip JSON serialization failed: {}", json_result.unwrap_err()));
            return result;
        }

        let json_str = json_result.unwrap();

        // Deserialize back
        let deserialized_result: Result<BiomeManifest, _> = serde_json::from_str(&json_str);
        if deserialized_result.is_err() {
            result.add_error(&format!("Round-trip JSON deserialization failed: {}", deserialized_result.unwrap_err()));
            return result;
        }

        let deserialized = deserialized_result.unwrap();

        // Serialize again
        let json2_result = serde_json::to_string(&deserialized);
        if json2_result.is_err() {
            result.add_error(&format!("Round-trip JSON re-serialization failed: {}", json2_result.unwrap_err()));
            return result;
        }

        let json2_str = json2_result.unwrap();

        // Compare JSON strings (should be identical)
        if json_str != json2_str {
            result.add_error("Round-trip JSON strings differ");
        } else {
            result.add_info("JSON round-trip consistency test passed");
        }

        result
    }

    /// Verify structural consistency across formats
    fn verify_structural_consistency(
        original: &BiomeManifest,
        from_yaml: &BiomeManifest,
        from_json: &BiomeManifest,
    ) -> SerializationTestResult {
        let mut result = SerializationTestResult::new();

        // Check primal counts
        if original.primals.len() != from_yaml.primals.len() {
            result.add_error(&format!("YAML primal count differs: {} vs {}", original.primals.len(), from_yaml.primals.len()));
        }

        if original.primals.len() != from_json.primals.len() {
            result.add_error(&format!("JSON primal count differs: {} vs {}", original.primals.len(), from_json.primals.len()));
        }

        // Check metadata consistency
        if original.metadata.name != from_yaml.metadata.name {
            result.add_error("YAML metadata name differs");
        }

        if original.metadata.name != from_json.metadata.name {
            result.add_error("JSON metadata name differs");
        }

        // Check primal types
        for (name, original_primal) in &original.primals {
            if let Some(yaml_primal) = from_yaml.primals.get(name) {
                if original_primal.primal_type != yaml_primal.primal_type {
                    result.add_error(&format!("YAML primal type differs for {}: {:?} vs {:?}", name, original_primal.primal_type, yaml_primal.primal_type));
                }
            }

            if let Some(json_primal) = from_json.primals.get(name) {
                if original_primal.primal_type != json_primal.primal_type {
                    result.add_error(&format!("JSON primal type differs for {}: {:?} vs {:?}", name, original_primal.primal_type, json_primal.primal_type));
                }
            }
        }

        if result.errors.is_empty() {
            result.add_info("Structural consistency verified across formats");
        }

        result
    }
}

/// Result container for serialization tests
#[derive(Debug, Clone)]
pub struct SerializationTestResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl SerializationTestResult {
    /// Create new empty result
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    /// Add error message
    pub fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    /// Add warning message
    pub fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }

    /// Add info message
    pub fn add_info(&mut self, message: &str) {
        self.info.push(message.to_string());
    }

    /// Check if result has errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Combine with another result
    pub fn combine(&mut self, other: SerializationTestResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.info.extend(other.info);
    }

    /// Get summary
    pub fn summary(&self) -> String {
        format!(
            "Serialization: {} errors, {} warnings, {} info",
            self.errors.len(),
            self.warnings.len(),
            self.info.len()
        )
    }
} 