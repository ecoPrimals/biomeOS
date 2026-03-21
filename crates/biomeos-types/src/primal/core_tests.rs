// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

// Unit tests for primal core types
use super::*;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_type_creation() {
        let primal = PrimalType::new("storage", "nestgate", "1.0.0");
        assert_eq!(primal.category, "storage");
        assert_eq!(primal.name, "nestgate");
        assert_eq!(primal.version, "1.0.0");
    }

    #[test]
    fn test_primal_type_from_discovered() {
        let primal = PrimalType::from_discovered("compute", "toadstool", "2.1.0");
        assert_eq!(primal.category, "compute");
        assert_eq!(primal.name, "toadstool");
        assert_eq!(primal.version, "2.1.0");
    }

    #[test]
    fn test_primal_type_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("port".to_string(), "9020".to_string());
        metadata.insert("protocol".to_string(), "https".to_string());

        let primal = PrimalType::with_metadata("storage", "nestgate", "1.0.0", metadata);
        assert_eq!(primal.metadata.get("port"), Some(&"9020".to_string()));
        assert_eq!(primal.metadata.get("protocol"), Some(&"https".to_string()));
    }

    #[test]
    fn test_primal_type_category_check() {
        let primal = PrimalType::new("security", "beardog", "0.9.0");
        assert!(primal.is_category("security"));
        assert!(!primal.is_category("storage"));
    }

    #[test]
    fn test_primal_type_name_check() {
        let primal = PrimalType::new("orchestration", "songbird", "3.0.0");
        assert!(primal.is_name("songbird"));
        assert!(!primal.is_name("nestgate"));
    }

    #[test]
    fn test_primal_type_identifier() {
        let primal = PrimalType::new("storage", "nestgate", "1.0.0");
        assert_eq!(primal.identifier(), "storage:nestgate:1.0.0");
    }

    #[test]
    fn test_primal_metadata_creation() {
        let primal_type = PrimalType::new("storage", "test-primal", "1.0.0");

        let metadata = PrimalMetadata {
            id: Uuid::new_v4(),
            primal_type,
            display_name: "Test Primal".to_string(),
            description: "Test primal service".to_string(),
            author: "test@example.com".to_string(),
            license: "MIT".to_string(),
            repository: Some("https://github.com/test/test".to_string()),
            documentation: Some("https://docs.test.example.com".to_string()),
            tags: vec!["testing".to_string(), "primal".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version_history: vec!["1.0.0".to_string()],
            resource_requirements: ResourceRequirements::default(),
            additional: HashMap::new(),
        };

        assert_eq!(metadata.display_name, "Test Primal");
        assert_eq!(metadata.license, "MIT");
        assert!(!metadata.tags.is_empty());
    }

    #[test]
    fn test_resource_requirements() {
        let requirements = ResourceRequirements {
            cpu: Some(2),
            memory: Some(2048),
            disk: Some(10000),
            network: Some(100),
            gpu: None,
            additional: vec![],
        };

        assert_eq!(requirements.cpu, Some(2));
        assert_eq!(requirements.memory, Some(2048));
        assert!(requirements.gpu.is_none());
    }

    #[test]
    fn test_resource_requirements_minimal() {
        let requirements = ResourceRequirements {
            cpu: Some(1),
            memory: Some(512),
            disk: Some(5000),
            network: None,
            gpu: None,
            additional: vec![],
        };

        assert_eq!(requirements.cpu, Some(1));
        assert!(requirements.network.is_none());
    }

    #[test]
    fn test_multiple_primal_types() {
        let primals = [
            PrimalType::new("storage", "nestgate", "1.0.0"),
            PrimalType::new("security", "beardog", "0.9.0"),
            PrimalType::new("orchestration", "songbird", "3.0.0"),
            PrimalType::new("compute", "toadstool", "2.1.0"),
        ];

        assert_eq!(primals.len(), 4);
        assert!(primals.iter().any(|p| p.name == "nestgate"));
        assert!(primals.iter().any(|p| p.name == "beardog"));
        assert!(primals.iter().any(|p| p.category == "storage"));
    }

    #[test]
    fn test_primal_type_equality() {
        let primal1 = PrimalType::new("storage", "nestgate", "1.0.0");
        let primal2 = PrimalType::new("storage", "nestgate", "1.0.0");
        let primal3 = PrimalType::new("storage", "nestgate", "1.0.1");

        assert_eq!(primal1, primal2);
        assert_ne!(primal1, primal3);
    }

    #[test]
    fn test_primal_type_clone() {
        let primal1 = PrimalType::new("storage", "nestgate", "1.0.0");
        let primal2 = primal1.clone();

        assert_eq!(primal1, primal2);
        assert_eq!(primal1.name, primal2.name);
    }

    #[test]
    fn test_empty_metadata() {
        let primal = PrimalType::new("test", "test-primal", "1.0.0");
        assert!(primal.metadata.is_empty());
    }

    #[test]
    fn test_metadata_modification() {
        let mut primal = PrimalType::new("storage", "nestgate", "1.0.0");
        primal
            .metadata
            .insert("key".to_string(), "value".to_string());

        assert_eq!(primal.metadata.get("key"), Some(&"value".to_string()));
        assert_eq!(primal.metadata.len(), 1);
    }
}
