//! Validation Module
//!
//! This module provides comprehensive validation utilities for UI forms,
//! configurations, and user input using the unified types from biomeos-types.

use crate::ValidationResult;
use regex::Regex;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

/// Validation rule trait
pub trait ValidationRule: Send + Sync {
    /// Validate a value and return validation result
    fn validate(&self, value: &str) -> ValidationResult;

    /// Get the rule name/identifier
    fn rule_name(&self) -> &str;

    /// Get a human-readable description of the rule
    fn description(&self) -> &str;
}

/// Field validator that combines multiple validation rules
pub struct FieldValidator {
    name: String,
    rules: Vec<Box<dyn ValidationRule>>,
}

impl FieldValidator {
    /// Create a new field validator
    pub fn new(field_name: impl Into<String>) -> Self {
        Self {
            name: field_name.into(),
            rules: Vec::new(),
        }
    }

    /// Mark field as required
    pub fn required(mut self) -> Self {
        self.rules.push(Box::new(rules::RequiredRule));
        self
    }

    /// Add a validation rule
    pub fn rule(mut self, rule: Box<dyn ValidationRule>) -> Self {
        self.rules.push(rule);
        self
    }

    /// Validate the field value
    pub fn validate(&self, value: &str) -> ValidationResult {
        // Check if required field is empty
        if self.rules.iter().any(|r| r.rule_name() == "required") && value.trim().is_empty() {
            return ValidationResult {
                is_valid: false,
                errors: vec![format!("{} is required", self.name)],
                warnings: vec![],
                manifest: None,
            };
        }

        // Skip validation if field is empty and not required
        if !self.rules.iter().any(|r| r.rule_name() == "required") && value.trim().is_empty() {
            return ValidationResult {
                is_valid: true,
                errors: vec![],
                warnings: vec![],
                manifest: None,
            };
        }

        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();

        for rule in &self.rules {
            let result = rule.validate(value);
            all_errors.extend(result.errors);
            all_warnings.extend(result.warnings);
        }

        ValidationResult {
            is_valid: all_errors.is_empty(),
            errors: all_errors,
            warnings: all_warnings,
            manifest: None,
        }
    }
}

/// Form validator that manages multiple field validators
pub struct FormValidator {
    fields: HashMap<String, FieldValidator>,
}

impl FormValidator {
    /// Create a new form validator
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Add a field validator
    pub fn field(mut self, field_name: impl Into<String>, validator: FieldValidator) -> Self {
        self.fields.insert(field_name.into(), validator);
        self
    }

    /// Validate all fields with provided values
    pub fn validate(&self, values: &HashMap<String, String>) -> FormValidationResult {
        let mut field_results = HashMap::new();
        let mut is_valid = true;

        for (field_name, validator) in &self.fields {
            let value = values.get(field_name).map(|s| s.as_str()).unwrap_or("");
            let result = validator.validate(value);

            if !result.is_valid {
                is_valid = false;
            }

            field_results.insert(field_name.clone(), result);
        }

        FormValidationResult {
            is_valid,
            field_results,
        }
    }
}

impl Default for FormValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Form validation result
#[derive(Debug)]
pub struct FormValidationResult {
    pub is_valid: bool,
    pub field_results: HashMap<String, ValidationResult>,
}

impl FormValidationResult {
    /// Get all errors across all fields
    pub fn all_errors(&self) -> Vec<String> {
        self.field_results
            .values()
            .flat_map(|r| r.errors.iter())
            .cloned()
            .collect()
    }

    /// Get all warnings across all fields
    pub fn all_warnings(&self) -> Vec<String> {
        self.field_results
            .values()
            .flat_map(|r| r.warnings.iter())
            .cloned()
            .collect()
    }

    /// Get errors for a specific field
    pub fn field_errors(&self, field_name: &str) -> Vec<&String> {
        self.field_results
            .get(field_name)
            .map(|r| r.errors.iter().collect())
            .unwrap_or_default()
    }
}

/// Predefined validation rules
pub mod rules {
    use super::*;

    /// Required field validation
    pub struct RequiredRule;

    impl ValidationRule for RequiredRule {
        fn validate(&self, value: &str) -> ValidationResult {
            if value.trim().is_empty() {
                ValidationResult {
                    is_valid: false,
                    errors: vec!["This field is required".to_string()],
                    warnings: vec![],
                    manifest: None,
                }
            } else {
                ValidationResult {
                    is_valid: true,
                    errors: vec![],
                    warnings: vec![],
                    manifest: None,
                }
            }
        }

        fn rule_name(&self) -> &str {
            "required"
        }

        fn description(&self) -> &str {
            "Field must not be empty"
        }
    }

    /// Length validation rule
    pub struct LengthRule {
        min_length: Option<usize>,
        max_length: Option<usize>,
    }

    impl LengthRule {
        pub fn min(min_length: usize) -> Self {
            Self {
                min_length: Some(min_length),
                max_length: None,
            }
        }

        pub fn max(max_length: usize) -> Self {
            Self {
                min_length: None,
                max_length: Some(max_length),
            }
        }

        pub fn range(min_length: usize, max_length: usize) -> Self {
            Self {
                min_length: Some(min_length),
                max_length: Some(max_length),
            }
        }
    }

    impl ValidationRule for LengthRule {
        fn validate(&self, value: &str) -> ValidationResult {
            let length = value.len();
            let mut errors = Vec::new();

            if let Some(min) = self.min_length {
                if length < min {
                    errors.push(format!("Must be at least {} characters long", min));
                }
            }

            if let Some(max) = self.max_length {
                if length > max {
                    errors.push(format!("Must be no more than {} characters long", max));
                }
            }

            ValidationResult {
                is_valid: errors.is_empty(),
                errors,
                warnings: vec![],
                manifest: None,
            }
        }

        fn rule_name(&self) -> &str {
            "length"
        }

        fn description(&self) -> &str {
            "Field must meet length requirements"
        }
    }

    /// Regular expression validation rule
    pub struct RegexRule {
        regex: Regex,
        error_message: String,
    }

    impl RegexRule {
        pub fn new(pattern: &str, error_message: impl Into<String>) -> Result<Self, regex::Error> {
            Ok(Self {
                regex: Regex::new(pattern)?,
                error_message: error_message.into(),
            })
        }

        /// Email validation rule
        pub fn email() -> Self {
            Self {
                regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap(),
                error_message: "Must be a valid email address".to_string(),
            }
        }

        /// Domain name validation rule
        pub fn domain() -> Self {
            Self {
                regex: Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap(),
                error_message: "Must be a valid domain name".to_string(),
            }
        }

        /// Alphanumeric validation rule
        pub fn alphanumeric() -> Self {
            Self {
                regex: Regex::new(r"^[a-zA-Z0-9]+$").unwrap(),
                error_message: "Must contain only letters and numbers".to_string(),
            }
        }
    }

    impl ValidationRule for RegexRule {
        fn validate(&self, value: &str) -> ValidationResult {
            if self.regex.is_match(value) {
                ValidationResult {
                    is_valid: true,
                    errors: vec![],
                    warnings: vec![],
                    manifest: None,
                }
            } else {
                ValidationResult {
                    is_valid: false,
                    errors: vec![self.error_message.clone()],
                    warnings: vec![],
                    manifest: None,
                }
            }
        }

        fn rule_name(&self) -> &str {
            "regex"
        }

        fn description(&self) -> &str {
            &self.error_message
        }
    }

    /// IP address validation rule
    pub struct IpAddressRule;

    impl ValidationRule for IpAddressRule {
        fn validate(&self, value: &str) -> ValidationResult {
            match IpAddr::from_str(value) {
                Ok(_) => ValidationResult {
                    is_valid: true,
                    errors: vec![],
                    warnings: vec![],
                    manifest: None,
                },
                Err(_) => ValidationResult {
                    is_valid: false,
                    errors: vec!["Must be a valid IP address".to_string()],
                    warnings: vec![],
                    manifest: None,
                },
            }
        }

        fn rule_name(&self) -> &str {
            "ip_address"
        }

        fn description(&self) -> &str {
            "Must be a valid IP address"
        }
    }

    /// Port number validation rule
    pub struct PortRule;

    impl ValidationRule for PortRule {
        fn validate(&self, value: &str) -> ValidationResult {
            match value.parse::<u16>() {
                Ok(0) => ValidationResult {
                    is_valid: false,
                    errors: vec!["Port 0 is reserved and cannot be used".to_string()],
                    warnings: vec![],
                    manifest: None,
                },
                Ok(port) if port <= 1024 => ValidationResult {
                    is_valid: true,
                    errors: vec![],
                    warnings: vec![format!("Port {} is in the well-known range (1-1024)", port)],
                    manifest: None,
                },
                Ok(_) => ValidationResult {
                    is_valid: true,
                    errors: vec![],
                    warnings: vec![],
                    manifest: None,
                },
                Err(_) => ValidationResult {
                    is_valid: false,
                    errors: vec!["Invalid port number format".to_string()],
                    warnings: vec![],
                    manifest: None,
                },
            }
        }

        fn rule_name(&self) -> &str {
            "port"
        }

        fn description(&self) -> &str {
            "Must be a valid port number (1-65535)"
        }
    }

    /// URL validation rule
    pub struct UrlRule;

    impl ValidationRule for UrlRule {
        fn validate(&self, value: &str) -> ValidationResult {
            match url::Url::parse(value) {
                Ok(url) => {
                    let mut warnings = Vec::new();

                    if url.scheme() == "http" {
                        warnings
                            .push("HTTP URLs are not secure. Consider using HTTPS.".to_string());
                    }

                    ValidationResult {
                        is_valid: true,
                        errors: vec![],
                        warnings,
                        manifest: None,
                    }
                }
                Err(_) => ValidationResult {
                    is_valid: false,
                    errors: vec!["Must be a valid URL".to_string()],
                    warnings: vec![],
                    manifest: None,
                },
            }
        }

        fn rule_name(&self) -> &str {
            "url"
        }

        fn description(&self) -> &str {
            "Must be a valid URL"
        }
    }
}

/// Configuration-specific validators
pub mod config_validators {
    use super::*;

    /// Create a validator for service names
    pub fn service_name_validator() -> FieldValidator {
        FieldValidator::new("service_name")
            .required()
            .rule(Box::new(rules::LengthRule::range(1, 64)))
            .rule(Box::new(rules::RegexRule::new(
                r"^[a-z][a-z0-9-]*[a-z0-9]$",
                "Service name must start with a letter, contain only lowercase letters, numbers, and hyphens, and end with a letter or number"
            ).unwrap()))
    }

    /// Create a validator for biome names
    pub fn biome_name_validator() -> FieldValidator {
        FieldValidator::new("biome_name")
            .required()
            .rule(Box::new(rules::LengthRule::range(1, 64)))
            .rule(Box::new(rules::RegexRule::new(
                r"^[a-z][a-z0-9-]*[a-z0-9]$",
                "Biome name must start with a letter, contain only lowercase letters, numbers, and hyphens, and end with a letter or number"
            ).unwrap()))
    }

    /// Create a validator for endpoints
    pub fn endpoint_validator() -> FieldValidator {
        FieldValidator::new("endpoint")
            .required()
            .rule(Box::new(rules::UrlRule))
    }

    /// Create a validator for environment variables
    pub fn env_var_validator() -> FieldValidator {
        FieldValidator::new("environment_variable")
            .required()
            .rule(Box::new(rules::RegexRule::new(
                r"^[A-Z][A-Z0-9_]*$",
                "Environment variable must start with a capital letter and contain only uppercase letters, numbers, and underscores"
            ).unwrap()))
    }
}
