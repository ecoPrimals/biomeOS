//! YAML validation functionality

use super::types::*;

impl YamlEditorView {
    /// Validate current YAML content
    pub fn validate_yaml(&mut self) {
        self.validation_errors.clear();
        self.validation_warnings.clear();

        // Basic YAML parsing
        match serde_yaml::from_str::<serde_yaml::Value>(&self.current_yaml) {
            Ok(value) => {
                self.validate_yaml_structure(&value);
            }
            Err(e) => {
                self.validation_errors
                    .push(format!("YAML parsing error: {}", e));
            }
        }
    }

    /// Validate YAML structure for biome.yaml
    fn validate_yaml_structure(&mut self, value: &serde_yaml::Value) {
        if let Some(map) = value.as_mapping() {
            // Check for required top-level fields
            if !map.contains_key("apiVersion") {
                self.validation_errors
                    .push("Missing required field: apiVersion".to_string());
            }

            if !map.contains_key("kind") {
                self.validation_errors
                    .push("Missing required field: kind".to_string());
            }

            if !map.contains_key("metadata") {
                self.validation_errors
                    .push("Missing required field: metadata".to_string());
            }

            // Validate metadata section
            if let Some(metadata) = map.get("metadata") {
                self.validate_metadata_section(metadata);
            }

            // Validate primals section
            if let Some(primals) = map.get("primals") {
                self.validate_primals_section(primals);
            } else {
                self.validation_warnings.push(
                    "No 'primals' section found - at least one primal is recommended".to_string(),
                );
            }

            // Validate other sections
            self.validate_resources_section(map.get("resources"));
            self.validate_security_section(map.get("security"));
            self.validate_networking_section(map.get("networking"));
        }
    }

    /// Validate metadata section
    fn validate_metadata_section(&mut self, metadata: &serde_yaml::Value) {
        if let Some(metadata_map) = metadata.as_mapping() {
            if !metadata_map.contains_key("name") {
                self.validation_errors
                    .push("Metadata missing required field: name".to_string());
            }

            if !metadata_map.contains_key("version") {
                self.validation_warnings
                    .push("Metadata missing version field".to_string());
            }
        }
    }

    /// Validate primals section
    fn validate_primals_section(&mut self, primals: &serde_yaml::Value) {
        if let Some(primals_map) = primals.as_mapping() {
            for (primal_name, primal_config) in primals_map {
                if let Some(name_str) = primal_name.as_str() {
                    self.validate_primal_config(name_str, primal_config);
                }
            }
        }
    }

    /// Validate individual primal configuration
    fn validate_primal_config(&mut self, primal_name: &str, config: &serde_yaml::Value) {
        if let Some(config_map) = config.as_mapping() {
            // Check for required primal fields
            if !config_map.contains_key("enabled") {
                self.validation_warnings.push(format!(
                    "Primal '{}' should have 'enabled' field",
                    primal_name
                ));
            }

            // Validate known primal types
            match primal_name {
                "beardog" | "songbird" | "nestgate" | "toadstool" | "squirrel" => {
                    // These are known primals - validate their specific configs
                    if primal_name == "beardog" && !config_map.contains_key("priority") {
                        self.validation_warnings
                            .push("BearDog should have highest priority (1)".to_string());
                    }
                }
                _ => {
                    // Unknown primal - that's okay in biomeOS
                    self.validation_warnings.push(format!(
                        "Unknown primal '{}' - this is fine if it's a custom primal",
                        primal_name
                    ));
                }
            }
        }
    }

    /// Validate resources section
    fn validate_resources_section(&mut self, resources: Option<&serde_yaml::Value>) {
        if let Some(resources_map) = resources.and_then(|r| r.as_mapping()) {
            // Check for reasonable resource limits
            if let Some(compute) = resources_map.get("compute") {
                if let Some(compute_map) = compute.as_mapping() {
                    if let Some(nodes) = compute_map.get("nodes") {
                        if let Some(nodes_seq) = nodes.as_sequence() {
                            for node in nodes_seq {
                                if let Some(node_map) = node.as_mapping() {
                                    if let Some(memory) = node_map.get("memory") {
                                        if let Some(memory_str) = memory.as_str() {
                                            if memory_str.contains("Ti") {
                                                self.validation_warnings.push("Large memory allocation detected - ensure your system has sufficient RAM".to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Validate security section
    fn validate_security_section(&mut self, security: Option<&serde_yaml::Value>) {
        if security.is_none() {
            self.validation_warnings.push(
                "No security section found - consider adding security configuration".to_string(),
            );
        }
    }

    /// Validate networking section
    fn validate_networking_section(&mut self, networking: Option<&serde_yaml::Value>) {
        if networking.is_none() {
            self.validation_warnings.push(
                "No networking section found - using default networking configuration".to_string(),
            );
        }
    }

    /// Get validation status
    pub fn has_validation_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// Get validation errors
    pub fn get_validation_errors(&self) -> &[String] {
        &self.validation_errors
    }

    /// Get validation warnings
    pub fn get_validation_warnings(&self) -> &[String] {
        &self.validation_warnings
    }
}
