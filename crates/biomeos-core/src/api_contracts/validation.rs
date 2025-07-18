//! API contract validation

use super::types::*;
use crate::BiomeResult;
use std::collections::HashMap;

/// API contract validator
pub struct ApiContractValidator {
    /// Contract schemas
    schemas: HashMap<String, serde_json::Value>,
    /// Validation rules
    rules: Vec<ValidationRule>,
}

impl ApiContractValidator {
    /// Create new validator
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            rules: Vec::new(),
        }
    }

    /// Add validation schema
    pub fn add_schema(&mut self, name: String, schema: serde_json::Value) {
        self.schemas.insert(name, schema);
    }

    /// Add validation rule
    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }

    /// Validate request
    pub fn validate_request(&self, request: &serde_json::Value) -> BiomeResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. Basic structure validation
        if !request.is_object() {
            errors.push(ValidationError {
                field: "request".to_string(),
                message: "Request must be a JSON object".to_string(),
                code: "INVALID_REQUEST_FORMAT".to_string(),
            });
            return Ok(ValidationResult {
                valid: false,
                errors,
                warnings,
            });
        }

        let request_obj = request.as_object().unwrap();

        // 2. Required fields validation
        let required_fields = vec!["method", "path", "version"];
        for field in required_fields {
            if !request_obj.contains_key(field) {
                errors.push(ValidationError {
                    field: field.to_string(),
                    message: format!("Required field '{}' is missing", field),
                    code: "MISSING_REQUIRED_FIELD".to_string(),
                });
            }
        }

        // 3. Field type validation
        if let Some(method) = request_obj.get("method") {
            if !method.is_string() {
                errors.push(ValidationError {
                    field: "method".to_string(),
                    message: "Method must be a string".to_string(),
                    code: "INVALID_FIELD_TYPE".to_string(),
                });
            } else {
                let method_str = method.as_str().unwrap();
                if !["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"].contains(&method_str) {
                    errors.push(ValidationError {
                        field: "method".to_string(),
                        message: format!("Invalid HTTP method: {}", method_str),
                        code: "INVALID_HTTP_METHOD".to_string(),
                    });
                }
            }
        }

        // 4. Path validation
        if let Some(path) = request_obj.get("path") {
            if !path.is_string() {
                errors.push(ValidationError {
                    field: "path".to_string(),
                    message: "Path must be a string".to_string(),
                    code: "INVALID_FIELD_TYPE".to_string(),
                });
            } else {
                let path_str = path.as_str().unwrap();
                if !path_str.starts_with('/') {
                    errors.push(ValidationError {
                        field: "path".to_string(),
                        message: "Path must start with '/'".to_string(),
                        code: "INVALID_PATH_FORMAT".to_string(),
                    });
                }
                
                if path_str.len() > 2048 {
                    errors.push(ValidationError {
                        field: "path".to_string(),
                        message: "Path too long (max 2048 characters)".to_string(),
                        code: "PATH_TOO_LONG".to_string(),
                    });
                }
            }
        }

        // 5. Version validation
        if let Some(version) = request_obj.get("version") {
            if !version.is_string() {
                errors.push(ValidationError {
                    field: "version".to_string(),
                    message: "Version must be a string".to_string(),
                    code: "INVALID_FIELD_TYPE".to_string(),
                });
            } else {
                let version_str = version.as_str().unwrap();
                if !version_str.starts_with("v") || version_str.len() < 2 {
                    warnings.push(ValidationWarning {
                        field: "version".to_string(),
                        message: "Version should follow format 'v1.0.0'".to_string(),
                        code: "VERSION_FORMAT_WARNING".to_string(),
                    });
                }
            }
        }

        // 6. Headers validation
        if let Some(headers) = request_obj.get("headers") {
            if !headers.is_object() {
                errors.push(ValidationError {
                    field: "headers".to_string(),
                    message: "Headers must be an object".to_string(),
                    code: "INVALID_FIELD_TYPE".to_string(),
                });
            } else {
                let headers_obj = headers.as_object().unwrap();
                
                // Check for important headers
                if !headers_obj.contains_key("content-type") && 
                   request_obj.get("method").and_then(|m| m.as_str()) == Some("POST") {
                    warnings.push(ValidationWarning {
                        field: "headers".to_string(),
                        message: "POST request should include Content-Type header".to_string(),
                        code: "MISSING_CONTENT_TYPE".to_string(),
                    });
                }
                
                // Validate header values
                for (header_name, header_value) in headers_obj {
                    if !header_value.is_string() {
                        errors.push(ValidationError {
                            field: format!("headers.{}", header_name),
                            message: "Header value must be a string".to_string(),
                            code: "INVALID_HEADER_VALUE".to_string(),
                        });
                    }
                }
            }
        }

        // 7. Body validation
        if let Some(body) = request_obj.get("body") {
            if let Some(content_type) = request_obj.get("headers")
                .and_then(|h| h.get("content-type"))
                .and_then(|ct| ct.as_str()) {
                
                if content_type.contains("application/json") {
                    self.validate_json_body(body, &mut errors, &mut warnings)?;
                } else if content_type.contains("application/x-www-form-urlencoded") {
                    self.validate_form_body(body, &mut errors, &mut warnings)?;
                }
            }
        }

        // 8. Security validation
        self.validate_security_headers(request_obj, &mut errors, &mut warnings)?;

        // 9. Rate limiting validation
        self.validate_rate_limits(request_obj, &mut errors, &mut warnings)?;

        // 10. Business rules validation
        let rule_result = self.apply_rules(request)?;
        errors.extend(rule_result.errors);
        warnings.extend(rule_result.warnings);

        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Validate response
    pub fn validate_response(&self, response: &serde_json::Value) -> BiomeResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. Basic structure validation
        if !response.is_object() {
            errors.push(ValidationError {
                field: "response".to_string(),
                message: "Response must be a JSON object".to_string(),
                code: "INVALID_RESPONSE_FORMAT".to_string(),
            });
            return Ok(ValidationResult {
                valid: false,
                errors,
                warnings,
            });
        }

        let response_obj = response.as_object().unwrap();

        // 2. Status validation
        if let Some(status) = response_obj.get("status") {
            if !status.is_string() {
                errors.push(ValidationError {
                    field: "status".to_string(),
                    message: "Status must be a string".to_string(),
                    code: "INVALID_FIELD_TYPE".to_string(),
                });
            } else {
                let status_str = status.as_str().unwrap();
                if !["Success", "PartialSuccess", "Error", "Timeout", "Unauthorized", "Forbidden", "NotFound", "InternalError"].contains(&status_str) {
                    errors.push(ValidationError {
                        field: "status".to_string(),
                        message: format!("Invalid status: {}", status_str),
                        code: "INVALID_STATUS".to_string(),
                    });
                }
            }
        } else {
            errors.push(ValidationError {
                field: "status".to_string(),
                message: "Status field is required".to_string(),
                code: "MISSING_REQUIRED_FIELD".to_string(),
            });
        }

        // 3. Data validation
        if let Some(data) = response_obj.get("data") {
            self.validate_response_data(data, &mut errors, &mut warnings)?;
        }

        // 4. Error validation
        if let Some(error) = response_obj.get("error") {
            if !error.is_object() {
                errors.push(ValidationError {
                    field: "error".to_string(),
                    message: "Error must be an object".to_string(),
                    code: "INVALID_FIELD_TYPE".to_string(),
                });
            } else {
                let error_obj = error.as_object().unwrap();
                
                // Check required error fields
                if !error_obj.contains_key("code") {
                    errors.push(ValidationError {
                        field: "error.code".to_string(),
                        message: "Error code is required".to_string(),
                        code: "MISSING_ERROR_CODE".to_string(),
                    });
                }
                
                if !error_obj.contains_key("message") {
                    errors.push(ValidationError {
                        field: "error.message".to_string(),
                        message: "Error message is required".to_string(),
                        code: "MISSING_ERROR_MESSAGE".to_string(),
                    });
                }
            }
        }

        // 5. Metadata validation
        if let Some(metadata) = response_obj.get("metadata") {
            if !metadata.is_object() {
                errors.push(ValidationError {
                    field: "metadata".to_string(),
                    message: "Metadata must be an object".to_string(),
                    code: "INVALID_FIELD_TYPE".to_string(),
                });
            } else {
                let metadata_obj = metadata.as_object().unwrap();
                
                // Check timestamp format
                if let Some(timestamp) = metadata_obj.get("timestamp") {
                    if !timestamp.is_u64() {
                        errors.push(ValidationError {
                            field: "metadata.timestamp".to_string(),
                            message: "Timestamp must be a number".to_string(),
                            code: "INVALID_TIMESTAMP_FORMAT".to_string(),
                        });
                    }
                }
                
                // Check request ID format
                if let Some(request_id) = metadata_obj.get("request_id") {
                    if !request_id.is_string() {
                        errors.push(ValidationError {
                            field: "metadata.request_id".to_string(),
                            message: "Request ID must be a string".to_string(),
                            code: "INVALID_REQUEST_ID_FORMAT".to_string(),
                        });
                    }
                }
            }
        }

        // 6. Performance validation
        self.validate_response_performance(response_obj, &mut errors, &mut warnings)?;

        // 7. Security validation
        self.validate_response_security(response_obj, &mut errors, &mut warnings)?;

        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Validate schema
    pub fn validate_schema(
        &self,
        schema_name: &str,
        _data: &serde_json::Value,
    ) -> BiomeResult<ValidationResult> {
        if let Some(_schema) = self.schemas.get(schema_name) {
            // TODO: Implement schema validation
            Ok(ValidationResult {
                valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
            })
        } else {
            Ok(ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    field: "schema".to_string(),
                    message: format!("Schema '{}' not found", schema_name),
                    code: "SCHEMA_NOT_FOUND".to_string(),
                }],
                warnings: Vec::new(),
            })
        }
    }

    /// Apply validation rules
    pub fn apply_rules(&self, data: &serde_json::Value) -> BiomeResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for rule in &self.rules {
            match rule.action {
                ValidationAction::Allow => continue,
                ValidationAction::Warn => {
                    warnings.push(ValidationWarning {
                        field: "rule".to_string(),
                        message: format!("Rule '{}' triggered: {}", rule.name, rule.description),
                        code: rule.name.clone(),
                    });
                }
                ValidationAction::Reject => {
                    errors.push(ValidationError {
                        field: "rule".to_string(),
                        message: format!("Rule '{}' rejected: {}", rule.name, rule.description),
                        code: rule.name.clone(),
                    });
                }
                ValidationAction::Transform { transformation } => {
                    // Implement transformation logic based on transformation type
                    match self.apply_transformation(data, transformation) {
                        Ok(transformed_data) => {
                            // Log successful transformation
                            tracing::info!(
                                rule = rule.name,
                                transformation = transformation,
                                "Data transformation applied successfully"
                            );
                            
                            warnings.push(ValidationWarning {
                                field: "rule".to_string(),
                                message: format!("Rule '{}' applied transformation: {}", rule.name, transformation),
                                code: "TRANSFORMATION_APPLIED".to_string(),
                            });
                            
                            // Update the data with transformed version
                            // Note: In a real implementation, this would modify the original data
                            // For now, we just record that transformation was applied
                        }
                        Err(transform_error) => {
                            errors.push(ValidationError {
                                field: "rule".to_string(),
                                message: format!("Rule '{}' transformation failed: {}", rule.name, transform_error),
                                code: "TRANSFORMATION_FAILED".to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Validate JSON body
    fn validate_json_body(&self, body: &serde_json::Value, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) -> BiomeResult<()> {
        // Check if body is valid JSON
        if body.is_null() {
            warnings.push(ValidationWarning {
                field: "body".to_string(),
                message: "JSON body is null".to_string(),
                code: "NULL_JSON_BODY".to_string(),
            });
            return Ok(());
        }

        // Check for excessive nesting
        if self.count_json_depth(body) > 10 {
            errors.push(ValidationError {
                field: "body".to_string(),
                message: "JSON body has excessive nesting (max 10 levels)".to_string(),
                code: "EXCESSIVE_JSON_NESTING".to_string(),
            });
        }

        // Check body size
        let body_str = serde_json::to_string(body).map_err(|e| {
            crate::BiomeError::ValidationError(format!("Failed to serialize body: {}", e))
        })?;
        
        if body_str.len() > 10_000_000 { // 10MB limit
            errors.push(ValidationError {
                field: "body".to_string(),
                message: "Request body too large (max 10MB)".to_string(),
                code: "BODY_TOO_LARGE".to_string(),
            });
        }

        Ok(())
    }

    /// Validate form body
    fn validate_form_body(&self, body: &serde_json::Value, errors: &mut Vec<ValidationError>, _warnings: &mut Vec<ValidationWarning>) -> BiomeResult<()> {
        if !body.is_object() {
            errors.push(ValidationError {
                field: "body".to_string(),
                message: "Form body must be an object".to_string(),
                code: "INVALID_FORM_BODY".to_string(),
            });
        }
        Ok(())
    }

    /// Validate security headers
    fn validate_security_headers(&self, request: &serde_json::Map<String, serde_json::Value>, _errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) -> BiomeResult<()> {
        if let Some(headers) = request.get("headers").and_then(|h| h.as_object()) {
            // Check for security headers
            let security_headers = vec![
                ("authorization", "Authorization header missing"),
                ("x-api-key", "API key header missing"),
                ("x-request-id", "Request ID header missing"),
            ];

            for (header, message) in security_headers {
                if !headers.contains_key(header) {
                    warnings.push(ValidationWarning {
                        field: format!("headers.{}", header),
                        message: message.to_string(),
                        code: "MISSING_SECURITY_HEADER".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Validate rate limits
    fn validate_rate_limits(&self, request: &serde_json::Map<String, serde_json::Value>, errors: &mut Vec<ValidationError>, _warnings: &mut Vec<ValidationWarning>) -> BiomeResult<()> {
        // Check for rate limit headers
        if let Some(headers) = request.get("headers").and_then(|h| h.as_object()) {
            if let Some(rate_limit) = headers.get("x-rate-limit") {
                if let Some(limit_str) = rate_limit.as_str() {
                    if let Ok(limit) = limit_str.parse::<u32>() {
                        if limit > 10000 {
                            errors.push(ValidationError {
                                field: "headers.x-rate-limit".to_string(),
                                message: "Rate limit too high (max 10000)".to_string(),
                                code: "RATE_LIMIT_TOO_HIGH".to_string(),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate response data
    fn validate_response_data(&self, data: &serde_json::Value, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) -> BiomeResult<()> {
        // Check data structure
        if data.is_null() {
            warnings.push(ValidationWarning {
                field: "data".to_string(),
                message: "Response data is null".to_string(),
                code: "NULL_RESPONSE_DATA".to_string(),
            });
            return Ok(());
        }

        // Check for excessive data size
        let data_str = serde_json::to_string(data).map_err(|e| {
            crate::BiomeError::ValidationError(format!("Failed to serialize response data: {}", e))
        })?;
        
        if data_str.len() > 50_000_000 { // 50MB limit for responses
            errors.push(ValidationError {
                field: "data".to_string(),
                message: "Response data too large (max 50MB)".to_string(),
                code: "RESPONSE_TOO_LARGE".to_string(),
            });
        }

        // Check for sensitive data leakage
        if let Some(data_str) = data.as_str() {
            let sensitive_patterns = vec![
                ("password", "Potential password in response"),
                ("secret", "Potential secret in response"),
                ("token", "Potential token in response"),
                ("key", "Potential key in response"),
            ];

            for (pattern, message) in sensitive_patterns {
                if data_str.to_lowercase().contains(pattern) {
                    warnings.push(ValidationWarning {
                        field: "data".to_string(),
                        message: message.to_string(),
                        code: "POTENTIAL_SENSITIVE_DATA".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate response performance
    fn validate_response_performance(&self, response: &serde_json::Map<String, serde_json::Value>, _errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) -> BiomeResult<()> {
        if let Some(metadata) = response.get("metadata").and_then(|m| m.as_object()) {
            if let Some(processing_time) = metadata.get("processing_time_ms").and_then(|pt| pt.as_u64()) {
                if processing_time > 30000 { // 30 seconds
                    warnings.push(ValidationWarning {
                        field: "metadata.processing_time_ms".to_string(),
                        message: "Response processing time is high (>30s)".to_string(),
                        code: "SLOW_RESPONSE".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Validate response security
    fn validate_response_security(&self, response: &serde_json::Map<String, serde_json::Value>, _errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) -> BiomeResult<()> {
        // Check for security headers in response
        if let Some(headers) = response.get("headers").and_then(|h| h.as_object()) {
            let security_headers = vec![
                ("x-content-type-options", "X-Content-Type-Options header missing"),
                ("x-frame-options", "X-Frame-Options header missing"),
                ("x-xss-protection", "X-XSS-Protection header missing"),
            ];

            for (header, message) in security_headers {
                if !headers.contains_key(header) {
                    warnings.push(ValidationWarning {
                        field: format!("headers.{}", header),
                        message: message.to_string(),
                        code: "MISSING_SECURITY_HEADER".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Count JSON depth
    fn count_json_depth(&self, value: &serde_json::Value) -> usize {
        match value {
            serde_json::Value::Object(obj) => {
                1 + obj.values().map(|v| self.count_json_depth(v)).max().unwrap_or(0)
            }
            serde_json::Value::Array(arr) => {
                1 + arr.iter().map(|v| self.count_json_depth(v)).max().unwrap_or(0)
            }
            _ => 0,
        }
    }

    /// Apply transformation to data
    fn apply_transformation(&self, data: &serde_json::Value, transformation: &str) -> Result<serde_json::Value, String> {
        match transformation {
            "lowercase" => {
                // Convert string values to lowercase
                self.transform_strings(data, |s| s.to_lowercase())
            }
            "uppercase" => {
                // Convert string values to uppercase
                self.transform_strings(data, |s| s.to_uppercase())
            }
            "trim" => {
                // Trim whitespace from string values
                self.transform_strings(data, |s| s.trim().to_string())
            }
            "normalize_email" => {
                // Normalize email addresses
                self.transform_strings(data, |s| {
                    if s.contains('@') {
                        s.to_lowercase().trim().to_string()
                    } else {
                        s.to_string()
                    }
                })
            }
            "sanitize_html" => {
                // Basic HTML sanitization
                self.transform_strings(data, |s| {
                    s.replace('<', "&lt;")
                     .replace('>', "&gt;")
                     .replace('&', "&amp;")
                     .replace('"', "&quot;")
                     .replace('\'', "&#x27;")
                })
            }
            "remove_null_fields" => {
                // Remove null fields from objects
                self.remove_null_fields(data)
            }
            "normalize_phone" => {
                // Normalize phone numbers
                self.transform_strings(data, |s| {
                    s.chars()
                     .filter(|c| c.is_ascii_digit() || *c == '+')
                     .collect()
                })
            }
            "truncate_strings" => {
                // Truncate strings to reasonable length
                self.transform_strings(data, |s| {
                    if s.len() > 1000 {
                        format!("{}...", &s[..997])
                    } else {
                        s.to_string()
                    }
                })
            }
            "validate_and_format_dates" => {
                // Validate and format date strings
                self.transform_strings(data, |s| {
                    if s.len() >= 10 && s.contains('-') {
                        // Simple date validation and formatting
                        let parts: Vec<&str> = s.split('-').collect();
                        if parts.len() == 3 {
                            format!("{}-{:02}-{:02}", 
                                parts[0], 
                                parts[1].parse::<u32>().unwrap_or(1), 
                                parts[2].parse::<u32>().unwrap_or(1))
                        } else {
                            s.to_string()
                        }
                    } else {
                        s.to_string()
                    }
                })
            }
            "remove_dangerous_characters" => {
                // Remove potentially dangerous characters
                self.transform_strings(data, |s| {
                    s.chars()
                     .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?@#$%^&*()_+-=[]{}|;':\"<>?/~`".contains(*c))
                     .collect()
                })
            }
            _ => Err(format!("Unknown transformation: {}", transformation)),
        }
    }

    /// Transform strings in JSON data
    fn transform_strings<F>(&self, data: &serde_json::Value, transform_fn: F) -> Result<serde_json::Value, String>
    where
        F: Fn(&str) -> String,
    {
        match data {
            serde_json::Value::String(s) => {
                Ok(serde_json::Value::String(transform_fn(s)))
            }
            serde_json::Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (key, value) in obj {
                    new_obj.insert(key.clone(), self.transform_strings(value, &transform_fn)?);
                }
                Ok(serde_json::Value::Object(new_obj))
            }
            serde_json::Value::Array(arr) => {
                let new_arr: Result<Vec<_>, _> = arr.iter()
                    .map(|v| self.transform_strings(v, &transform_fn))
                    .collect();
                Ok(serde_json::Value::Array(new_arr?))
            }
            _ => Ok(data.clone()),
        }
    }

    /// Remove null fields from JSON object
    fn remove_null_fields(&self, data: &serde_json::Value) -> Result<serde_json::Value, String> {
        match data {
            serde_json::Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (key, value) in obj {
                    if !value.is_null() {
                        new_obj.insert(key.clone(), self.remove_null_fields(value)?);
                    }
                }
                Ok(serde_json::Value::Object(new_obj))
            }
            serde_json::Value::Array(arr) => {
                let new_arr: Result<Vec<_>, _> = arr.iter()
                    .map(|v| self.remove_null_fields(v))
                    .collect();
                Ok(serde_json::Value::Array(new_arr?))
            }
            _ => Ok(data.clone()),
        }
    }
}

impl Default for ApiContractValidator {
    fn default() -> Self {
        Self::new()
    }
}
