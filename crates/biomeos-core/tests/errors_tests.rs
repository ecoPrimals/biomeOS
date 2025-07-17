//! Unit tests for the biomeos-core errors module

use biomeos_core::errors::BiomeError;
use serde_json;
use std::io;

#[test]
fn test_biome_error_variants() {
    let config_error = BiomeError::ConfigError {
        message: "Invalid configuration".to_string(),
    };

    let network_error = BiomeError::NetworkError {
        message: "Connection failed".to_string(),
    };

    let primal_error = BiomeError::PrimalError {
        message: "Primal initialization failed".to_string(),
    };

    let primal_not_found = BiomeError::PrimalNotFound("songbird".to_string());

    let security_error = BiomeError::SecurityError {
        message: "Authentication failed".to_string(),
    };

    let storage_error = BiomeError::StorageError {
        message: "Disk full".to_string(),
    };

    let io_error = BiomeError::IoError {
        message: "File not found".to_string(),
    };

    let serialization_error = BiomeError::SerializationError {
        message: "JSON parsing failed".to_string(),
    };

    let generic_error = BiomeError::Generic("Something went wrong".to_string());

    // Test error messages
    assert_eq!(
        config_error.to_string(),
        "Configuration error: Invalid configuration"
    );
    assert_eq!(
        network_error.to_string(),
        "Network error: Connection failed"
    );
    assert_eq!(
        primal_error.to_string(),
        "Primal error: Primal initialization failed"
    );
    assert_eq!(primal_not_found.to_string(), "Primal not found: songbird");
    assert_eq!(
        security_error.to_string(),
        "Security error: Authentication failed"
    );
    assert_eq!(storage_error.to_string(), "Storage error: Disk full");
    assert_eq!(io_error.to_string(), "IO error: File not found");
    assert_eq!(
        serialization_error.to_string(),
        "Serialization error: JSON parsing failed"
    );
    assert_eq!(generic_error.to_string(), "Error: Something went wrong");
}

#[test]
fn test_biome_error_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let biome_error = BiomeError::from(io_error);

    match biome_error {
        BiomeError::IoError { message } => {
            assert!(message.contains("File not found"));
        }
        _ => panic!("Expected IoError variant"),
    }
}

#[test]
fn test_biome_error_from_serde_json_error() {
    let json_error = serde_json::from_str::<serde_json::Value>("{invalid json}");
    assert!(json_error.is_err());

    let biome_error = BiomeError::from(json_error.unwrap_err());

    match biome_error {
        BiomeError::SerializationError { message } => {
            assert!(!message.is_empty());
        }
        _ => panic!("Expected SerializationError variant"),
    }
}

#[test]
fn test_biome_error_from_serde_yaml_error() {
    let yaml_error = serde_yaml::from_str::<serde_yaml::Value>("invalid: yaml: content:");
    assert!(yaml_error.is_err());

    let biome_error = BiomeError::from(yaml_error.unwrap_err());

    match biome_error {
        BiomeError::SerializationError { message } => {
            assert!(!message.is_empty());
        }
        _ => panic!("Expected SerializationError variant"),
    }
}

#[test]
fn test_biome_error_clone() {
    let original = BiomeError::ConfigError {
        message: "Test error".to_string(),
    };

    let cloned = original.clone();

    assert_eq!(original.to_string(), cloned.to_string());

    match (original, cloned) {
        (BiomeError::ConfigError { message: msg1 }, BiomeError::ConfigError { message: msg2 }) => {
            assert_eq!(msg1, msg2);
        }
        _ => panic!("Clone should preserve error variant"),
    }
}

#[test]
fn test_biome_error_debug() {
    let error = BiomeError::NetworkError {
        message: "Debug test".to_string(),
    };

    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("NetworkError"));
    assert!(debug_str.contains("Debug test"));
}

#[test]
fn test_biome_error_serialization() {
    let error = BiomeError::PrimalError {
        message: "Serialization test".to_string(),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&error);
    assert!(json.is_ok());

    // Test deserialization
    let deserialized: Result<BiomeError, _> = serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok());

    let deserialized_error = deserialized.unwrap();
    assert_eq!(error.to_string(), deserialized_error.to_string());
}

#[test]
fn test_biome_error_chain() {
    // Test creating a chain of errors
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
    let biome_error = BiomeError::from(io_error);

    let wrapped_error = BiomeError::StorageError {
        message: format!("Storage operation failed: {}", biome_error),
    };

    let final_error = BiomeError::Generic(format!("Operation failed: {}", wrapped_error));

    let error_string = final_error.to_string();
    assert!(error_string.contains("Operation failed"));
    assert!(error_string.contains("Storage operation failed"));
    assert!(error_string.contains("Permission denied"));
}

#[test]
fn test_biome_error_display_consistency() {
    let errors = vec![
        BiomeError::ConfigError {
            message: "Config".to_string(),
        },
        BiomeError::NetworkError {
            message: "Network".to_string(),
        },
        BiomeError::PrimalError {
            message: "Primal".to_string(),
        },
        BiomeError::PrimalNotFound("test".to_string()),
        BiomeError::SecurityError {
            message: "Security".to_string(),
        },
        BiomeError::StorageError {
            message: "Storage".to_string(),
        },
        BiomeError::IoError {
            message: "IO".to_string(),
        },
        BiomeError::SerializationError {
            message: "Serialization".to_string(),
        },
        BiomeError::Generic("Generic".to_string()),
    ];

    for error in errors {
        let display_str = error.to_string();
        let debug_str = format!("{:?}", error);

        // All errors should have non-empty display strings
        assert!(!display_str.is_empty());
        assert!(!debug_str.is_empty());

        // Debug should contain more information than display
        assert!(debug_str.len() >= display_str.len());
    }
}

#[test]
fn test_biome_error_result_pattern() {
    fn might_fail(should_fail: bool) -> Result<String, BiomeError> {
        if should_fail {
            Err(BiomeError::Generic("Operation failed".to_string()))
        } else {
            Ok("Success".to_string())
        }
    }

    // Test success case
    let result = might_fail(false);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");

    // Test failure case
    let result = might_fail(true);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.to_string(), "Error: Operation failed");
}

#[test]
fn test_biome_error_pattern_matching() {
    let errors = vec![
        BiomeError::ConfigError {
            message: "config".to_string(),
        },
        BiomeError::NetworkError {
            message: "network".to_string(),
        },
        BiomeError::PrimalNotFound("test".to_string()),
    ];

    for error in errors {
        let error_type = match error {
            BiomeError::ConfigError { .. } => "config",
            BiomeError::NetworkError { .. } => "network",
            BiomeError::PrimalNotFound(_) => "primal_not_found",
            BiomeError::PrimalError { .. } => "primal",
            BiomeError::SecurityError { .. } => "security",
            BiomeError::StorageError { .. } => "storage",
            BiomeError::IoError { .. } => "io",
            BiomeError::SerializationError { .. } => "serialization",
            BiomeError::Generic(_) => "generic",
        };

        assert!(!error_type.is_empty());
    }
}

#[test]
fn test_biome_error_source_chain() {
    use std::error::Error;

    let io_error = io::Error::new(io::ErrorKind::TimedOut, "Operation timed out");
    let biome_error = BiomeError::from(io_error);

    // Test that we can access the error source
    let source = biome_error.source();
    assert!(source.is_none()); // BiomeError doesn't implement source() to preserve inner errors

    // Test that error implements standard Error trait
    let error: &dyn Error = &biome_error;
    assert!(!error.to_string().is_empty());
}

#[test]
fn test_biome_error_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let error = Arc::new(BiomeError::ConfigError {
        message: "Concurrent test".to_string(),
    });

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let error_clone = Arc::clone(&error);
            thread::spawn(move || {
                let error_str = error_clone.to_string();
                assert!(error_str.contains("Concurrent test"));
                format!("Thread {} processed: {}", i, error_str)
            })
        })
        .collect();

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.contains("Thread"));
        assert!(result.contains("Concurrent test"));
    }
}
