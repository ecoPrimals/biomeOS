//! Generic integration test framework for eco-primals
//!
//! This module provides a configurable, generic framework for testing
//! primal integrations, replacing hardcoded primal-specific tests.
//!
//! # Features
//!
//! - **Configurable**: Works with any primal types, not hardcoded to specific primals
//! - **Reusable**: Common test builders and utilities
//! - **Comprehensive**: Validation, serialization, and integration tests
//! - **Maintainable**: Modular structure with clear separation of concerns
//!
//! # Usage
//!
//! ```rust
//! use crate::integration_tests::*;
//!
//! // Test any two primals
//! integration_tests::test_basic_primal_integration(
//!     "orchestrator", PrimalType::Songbird,
//!     "storage", PrimalType::NestGate,
//!     BiomeSpecialization::DataCenter
//! );
//!
//! // Test gaming scenario
//! integration_tests::test_gaming_tournament_integration(
//!     "orchestrator", PrimalType::Songbird,
//!     "storage", PrimalType::NestGate
//! );
//!
//! // Test serialization
//! let result = serialization::SerializationTester::test_yaml_serialization_compatibility(
//!     "orchestrator", PrimalType::Songbird,
//!     "storage", PrimalType::NestGate,
//!     BiomeSpecialization::DataCenter
//! );
//! ```

use biomeos_manifest::*;

pub mod common;
pub mod primal_configs;
pub mod integration_tests;
pub mod validation;
pub mod serialization;

// Re-export commonly used types and functions
pub use common::*;
pub use primal_configs::*;
pub use integration_tests::*;
pub use validation::*;
pub use serialization::*;

/// Main integration test suite that replaces hardcoded tests
pub struct IntegrationTestSuite;

impl IntegrationTestSuite {
    /// Run complete integration test suite for any two primals
    pub fn run_complete_integration_test(
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
        specialization: BiomeSpecialization,
    ) -> IntegrationTestResult {
        let mut result = IntegrationTestResult::new();

        // Create test manifest
        let manifest = TestScenarios::two_primal_integration(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
            specialization,
        ).build();

        // Run validation tests
        let validation_result = IntegrationValidator::validate_primal_integration(
            &manifest,
            orchestrator_name,
            storage_name,
            orchestrator_type,
            storage_type,
        );
        result.add_validation_result(validation_result);

        // Run serialization tests
        let yaml_result = SerializationTester::test_yaml_serialization_compatibility(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
            specialization,
        );
        result.add_serialization_result(yaml_result);

        let json_result = SerializationTester::test_json_serialization_compatibility(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
            specialization,
        );
        result.add_serialization_result(json_result);

        // Run integration tests
        test_basic_primal_integration(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
            specialization,
        );
        result.add_info("Basic primal integration test passed");

        test_fault_tolerance_integration(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
        );
        result.add_info("Fault tolerance integration test passed");

        result
    }

    /// Run gaming-specific integration tests
    pub fn run_gaming_integration_test(
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
    ) -> IntegrationTestResult {
        let mut result = IntegrationTestResult::new();

        // Create gaming manifest
        let manifest = TestScenarios::gaming_tournament(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
        ).build();

        // Run gaming-specific validation
        let validation_result = IntegrationValidator::validate_gaming_configuration(
            &manifest,
            orchestrator_name,
        );
        result.add_validation_result(validation_result);

        // Run gaming serialization tests
        let gaming_serialization = SerializationTester::test_gaming_serialization(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
        );
        result.add_serialization_result(gaming_serialization);

        // Run gaming integration tests
        test_gaming_tournament_integration(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
        );
        result.add_info("Gaming tournament integration test passed");

        test_advanced_feature_integration(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
        );
        result.add_info("Advanced feature integration test passed");

        result
    }

    /// Run performance optimization tests
    pub fn run_performance_integration_test(
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
    ) -> IntegrationTestResult {
        let mut result = IntegrationTestResult::new();

        // Run performance integration tests
        test_performance_optimization_integration(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
        );
        result.add_info("Performance optimization integration test passed");

        test_cross_region_integration(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
        );
        result.add_info("Cross-region integration test passed");

        result
    }

    /// Run multi-primal integration tests
    pub fn run_multi_primal_integration_test(
        primals: Vec<(String, PrimalType, u32)>,
        dependencies: Vec<(String, Vec<String>)>,
        specialization: BiomeSpecialization,
    ) -> IntegrationTestResult {
        let mut result = IntegrationTestResult::new();

        // Run multi-primal integration test
        test_multi_primal_integration(primals.clone(), dependencies.clone(), specialization);
        result.add_info("Multi-primal integration test passed");

        // Run large manifest serialization test
        let serialization_result = SerializationTester::test_large_manifest_serialization(
            primals,
            dependencies,
        );
        result.add_serialization_result(serialization_result);

        result
    }

    /// Run cross-format compatibility tests
    pub fn run_cross_format_compatibility_test(
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
    ) -> IntegrationTestResult {
        let mut result = IntegrationTestResult::new();

        // Run cross-format compatibility test
        let compatibility_result = SerializationTester::test_cross_format_compatibility(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
        );
        result.add_serialization_result(compatibility_result);

        result
    }
}

/// Result container for complete integration test runs
#[derive(Debug, Clone)]
pub struct IntegrationTestResult {
    pub validation_results: Vec<ValidationResult>,
    pub serialization_results: Vec<SerializationTestResult>,
    pub info: Vec<String>,
    pub errors: Vec<String>,
}

impl IntegrationTestResult {
    /// Create new empty result
    pub fn new() -> Self {
        Self {
            validation_results: Vec::new(),
            serialization_results: Vec::new(),
            info: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add validation result
    pub fn add_validation_result(&mut self, result: ValidationResult) {
        self.validation_results.push(result);
    }

    /// Add serialization result
    pub fn add_serialization_result(&mut self, result: SerializationTestResult) {
        self.serialization_results.push(result);
    }

    /// Add info message
    pub fn add_info(&mut self, message: &str) {
        self.info.push(message.to_string());
    }

    /// Add error message
    pub fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty() || 
        self.validation_results.iter().any(|r| r.has_errors()) ||
        self.serialization_results.iter().any(|r| r.has_errors())
    }

    /// Get summary of all results
    pub fn summary(&self) -> String {
        let total_validation_errors: usize = self.validation_results.iter()
            .map(|r| r.errors.len())
            .sum();
        let total_validation_warnings: usize = self.validation_results.iter()
            .map(|r| r.warnings.len())
            .sum();
        let total_serialization_errors: usize = self.serialization_results.iter()
            .map(|r| r.errors.len())
            .sum();
        let total_serialization_warnings: usize = self.serialization_results.iter()
            .map(|r| r.warnings.len())
            .sum();

        format!(
            "Integration Test Summary:\n\
            - Validation: {} errors, {} warnings\n\
            - Serialization: {} errors, {} warnings\n\
            - Info messages: {}\n\
            - Direct errors: {}",
            total_validation_errors,
            total_validation_warnings,
            total_serialization_errors,
            total_serialization_warnings,
            self.info.len(),
            self.errors.len()
        )
    }
}

/// Convenience functions for common test scenarios
pub mod convenience {
    use super::*;

    /// Test Songbird + NestGate integration (the original hardcoded scenario)
    pub fn test_songbird_nestgate_integration() -> IntegrationTestResult {
        IntegrationTestSuite::run_complete_integration_test(
            "songbird",
            PrimalType::Songbird,
            "nestgate",
            PrimalType::NestGate,
            BiomeSpecialization::DataCenter,
        )
    }

    /// Test Songbird + NestGate gaming integration
    pub fn test_songbird_nestgate_gaming_integration() -> IntegrationTestResult {
        IntegrationTestSuite::run_gaming_integration_test(
            "songbird",
            PrimalType::Songbird,
            "nestgate",
            PrimalType::NestGate,
        )
    }

    /// Test any orchestrator + storage integration
    pub fn test_orchestrator_storage_integration(
        orchestrator_name: &str,
        orchestrator_type: PrimalType,
        storage_name: &str,
        storage_type: PrimalType,
        specialization: BiomeSpecialization,
    ) -> IntegrationTestResult {
        IntegrationTestSuite::run_complete_integration_test(
            orchestrator_name,
            orchestrator_type,
            storage_name,
            storage_type,
            specialization,
        )
    }

    /// Test cross-primal ecosystem (multiple primals)
    pub fn test_cross_primal_ecosystem() -> IntegrationTestResult {
        let primals = vec![
            ("songbird".to_string(), PrimalType::Songbird, 1),
            ("nestgate".to_string(), PrimalType::NestGate, 2),
            ("squirrel".to_string(), PrimalType::Squirrel, 3),
            ("beardog".to_string(), PrimalType::BearDog, 4),
        ];

        let dependencies = vec![
            ("nestgate".to_string(), vec!["songbird".to_string()]),
            ("squirrel".to_string(), vec!["songbird".to_string()]),
            ("beardog".to_string(), vec!["songbird".to_string(), "nestgate".to_string()]),
        ];

        IntegrationTestSuite::run_multi_primal_integration_test(
            primals,
            dependencies,
            BiomeSpecialization::DataCenter,
        )
    }
}

/// Test configuration helpers
pub mod config {
    use super::*;

    /// Create a test configuration for specific primal types
    pub fn create_test_config(
        orchestrator_type: PrimalType,
        storage_type: PrimalType,
        specialization: BiomeSpecialization,
    ) -> TestConfig {
        TestConfig {
            orchestrator_name: format!("{:?}", orchestrator_type).to_lowercase(),
            orchestrator_type,
            storage_name: format!("{:?}", storage_type).to_lowercase(),
            storage_type,
            specialization,
        }
    }

    /// Configuration for running integration tests
    #[derive(Debug, Clone)]
    pub struct TestConfig {
        pub orchestrator_name: String,
        pub orchestrator_type: PrimalType,
        pub storage_name: String,
        pub storage_type: PrimalType,
        pub specialization: BiomeSpecialization,
    }

    impl TestConfig {
        /// Run complete integration test with this configuration
        pub fn run_complete_test(&self) -> IntegrationTestResult {
            IntegrationTestSuite::run_complete_integration_test(
                &self.orchestrator_name,
                self.orchestrator_type,
                &self.storage_name,
                self.storage_type,
                self.specialization,
            )
        }

        /// Run gaming integration test with this configuration
        pub fn run_gaming_test(&self) -> IntegrationTestResult {
            IntegrationTestSuite::run_gaming_integration_test(
                &self.orchestrator_name,
                self.orchestrator_type,
                &self.storage_name,
                self.storage_type,
            )
        }

        /// Run performance integration test with this configuration
        pub fn run_performance_test(&self) -> IntegrationTestResult {
            IntegrationTestSuite::run_performance_integration_test(
                &self.orchestrator_name,
                self.orchestrator_type,
                &self.storage_name,
                self.storage_type,
            )
        }
    }
} 