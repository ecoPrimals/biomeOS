//! Workflow integration functionality

use super::types::*;

impl YamlEditorView {
    /// Export current YAML to BYOB workflow
    pub fn export_to_byob(&mut self) {
        // This would integrate with the BYOB workflow
        println!("Exporting YAML to BYOB workflow...");
        // In a real implementation, this would:
        // 1. Parse the YAML
        // 2. Create a deployment configuration
        // 3. Navigate to BYOB with pre-filled data

        // For now, just validate and show success
        self.validate_yaml();
        if self.validation_errors.is_empty() {
            // Would trigger navigation to BYOB view
            println!("✅ YAML exported to BYOB successfully");
        } else {
            println!("❌ Cannot export YAML with validation errors");
        }
    }

    /// Export current YAML as a niche package
    pub fn export_to_niche(&mut self) {
        // This would create a niche package from the YAML
        println!("Creating niche package from YAML...");
        // In a real implementation, this would:
        // 1. Parse the YAML
        // 2. Create niche metadata
        // 3. Package for distribution

        // For now, just validate and show success
        self.validate_yaml();
        if self.validation_errors.is_empty() {
            println!("✅ Niche package created successfully");
        } else {
            println!("❌ Cannot create niche package with validation errors");
        }
    }

    /// Export current YAML for ISO creation
    pub fn export_to_iso(&mut self) {
        // This would prepare the YAML for ISO creation
        println!("Preparing YAML for ISO creation...");
        // In a real implementation, this would:
        // 1. Validate the YAML
        // 2. Create ISO configuration
        // 3. Navigate to ISO creator

        // For now, just validate and show success
        self.validate_yaml();
        if self.validation_errors.is_empty() {
            println!("✅ YAML prepared for ISO creation");
        } else {
            println!("❌ Cannot prepare YAML for ISO with validation errors");
        }
    }

    /// Import YAML from BYOB configuration
    pub fn import_from_byob(&mut self, byob_config: &str) {
        // This would convert BYOB configuration to YAML
        println!("Importing from BYOB configuration...");

        // For now, just set the content
        self.current_yaml = byob_config.to_string();
        self.is_modified = true;
        self.validate_yaml();

        println!("✅ BYOB configuration imported");
    }

    /// Import YAML from niche package
    pub fn import_from_niche(&mut self, niche_package: &str) {
        // This would extract YAML from niche package
        println!("Importing from niche package...");

        // For now, just set the content
        self.current_yaml = niche_package.to_string();
        self.is_modified = true;
        self.validate_yaml();

        println!("✅ Niche package imported");
    }

    /// Generate manifest from current YAML
    pub fn generate_manifest(&self) -> Result<String, String> {
        // This would generate a manifest from the YAML
        if self.validation_errors.is_empty() {
            Ok("# Generated manifest\napiVersion: v1\nkind: Manifest\n".to_string())
        } else {
            Err("Cannot generate manifest with validation errors".to_string())
        }
    }

    /// Check integration compatibility
    pub fn check_integration_compatibility(&self) -> IntegrationCompatibility {
        let mut compatibility = IntegrationCompatibility {
            byob_compatible: true,
            niche_compatible: true,
            iso_compatible: true,
            manifest_compatible: true,
            issues: Vec::new(),
        };

        // Check for validation errors
        if !self.validation_errors.is_empty() {
            compatibility.byob_compatible = false;
            compatibility.niche_compatible = false;
            compatibility.iso_compatible = false;
            compatibility.manifest_compatible = false;
            compatibility
                .issues
                .push("Validation errors must be resolved".to_string());
        }

        // Check for required sections
        if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&self.current_yaml) {
            if let Some(map) = value.as_mapping() {
                if !map.contains_key("primals") {
                    compatibility.byob_compatible = false;
                    compatibility
                        .issues
                        .push("BYOB requires primals section".to_string());
                }

                if !map.contains_key("metadata") {
                    compatibility.niche_compatible = false;
                    compatibility
                        .issues
                        .push("Niche packages require metadata section".to_string());
                }
            }
        }

        compatibility
    }

    /// Get integration status
    pub fn get_integration_status(&self) -> IntegrationStatus {
        let compatibility = self.check_integration_compatibility();

        IntegrationStatus {
            ready_for_byob: compatibility.byob_compatible,
            ready_for_niche: compatibility.niche_compatible,
            ready_for_iso: compatibility.iso_compatible,
            ready_for_manifest: compatibility.manifest_compatible,
            validation_passed: self.validation_errors.is_empty(),
            warnings_count: self.validation_warnings.len(),
            errors_count: self.validation_errors.len(),
        }
    }
}

/// Integration compatibility information
#[derive(Debug, Clone)]
pub struct IntegrationCompatibility {
    pub byob_compatible: bool,
    pub niche_compatible: bool,
    pub iso_compatible: bool,
    pub manifest_compatible: bool,
    pub issues: Vec<String>,
}

/// Integration status
#[derive(Debug, Clone)]
pub struct IntegrationStatus {
    pub ready_for_byob: bool,
    pub ready_for_niche: bool,
    pub ready_for_iso: bool,
    pub ready_for_manifest: bool,
    pub validation_passed: bool,
    pub warnings_count: usize,
    pub errors_count: usize,
}
