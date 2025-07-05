//! YAML Editor View
//! 
//! Advanced YAML editor for biome.yaml files with syntax highlighting, validation,
//! template loading, and live preview capabilities.

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

use crate::state::AppState;
use crate::api::BiomeOSApi;
use crate::views::{BaseView, View};

/// YAML Editor view with comprehensive editing capabilities
pub struct YamlEditorView {
    pub base: BaseView,
    pub current_yaml: String,
    pub original_yaml: String,
    pub file_path: Option<String>,
    pub is_modified: bool,
    pub selected_template: Option<String>,
    pub available_templates: Vec<YamlTemplate>,
    pub validation_errors: Vec<String>,
    pub validation_warnings: Vec<String>,
    pub show_preview: bool,
    pub show_validation_panel: bool,
    pub show_template_browser: bool,
    pub cursor_position: usize,
    pub search_query: String,
    pub replace_query: String,
    pub auto_save: bool,
    pub syntax_highlighting: bool,
    pub line_numbers: bool,
    pub word_wrap: bool,
    pub editor_font_size: f32,
    pub yaml_sections: HashMap<String, YamlSection>,
    pub collapsed_sections: HashMap<String, bool>,
    pub editor_mode: EditorMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorMode {
    Raw,
    Structured,
    Preview,
}

#[derive(Debug, Clone)]
pub struct YamlTemplate {
    pub name: String,
    pub description: String,
    pub file_path: String,
    pub category: String,
    pub content: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct YamlSection {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub section_type: YamlSectionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum YamlSectionType {
    Metadata,
    Primals,
    Services,
    Resources,
    Security,
    Networking,
    Agents,
    Extensions,
}

impl YamlEditorView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        let mut view = Self {
            base: BaseView::new(state, api),
            current_yaml: String::new(),
            original_yaml: String::new(),
            file_path: None,
            is_modified: false,
            selected_template: None,
            available_templates: Vec::new(),
            validation_errors: Vec::new(),
            validation_warnings: Vec::new(),
            show_preview: false,
            show_validation_panel: true,
            show_template_browser: false,
            cursor_position: 0,
            search_query: String::new(),
            replace_query: String::new(),
            auto_save: false,
            syntax_highlighting: true,
            line_numbers: true,
            word_wrap: true,
            editor_font_size: 14.0,
            yaml_sections: HashMap::new(),
            collapsed_sections: HashMap::new(),
            editor_mode: EditorMode::Raw,
        };
        
        // Load available templates
        view.load_templates();
        
        // Start with basic template
        view.load_template("basic-development");
        
        view
    }

    /// Load available YAML templates
    fn load_templates(&mut self) {
        self.available_templates = vec![
            YamlTemplate {
                name: "basic-development".to_string(),
                description: "Basic development environment with all Primals".to_string(),
                file_path: "specs/examples/basic-development.biome.yaml".to_string(),
                category: "Development".to_string(),
                content: self.get_basic_template(),
                features: vec!["Low security".to_string(), "Minimal resources".to_string(), "All primals".to_string()],
            },
            YamlTemplate {
                name: "ai-research".to_string(),
                description: "AI research environment with GPU compute".to_string(),
                file_path: "specs/examples/ai-research.biome.yaml".to_string(),
                category: "Research".to_string(),
                content: self.get_ai_research_template(),
                features: vec!["GPU support".to_string(), "Large storage".to_string(), "ML workflows".to_string()],
            },
            YamlTemplate {
                name: "secure-enterprise".to_string(),
                description: "Enterprise-grade security and compliance".to_string(),
                file_path: "specs/examples/secure-enterprise.biome.yaml".to_string(),
                category: "Enterprise".to_string(),
                content: self.get_enterprise_template(),
                features: vec!["Maximum security".to_string(), "Compliance".to_string(), "Audit trails".to_string()],
            },
            YamlTemplate {
                name: "custom-template".to_string(),
                description: "Custom biome configuration".to_string(),
                file_path: "templates/biome.yaml".to_string(),
                category: "Custom".to_string(),
                content: self.get_custom_template(),
                features: vec!["Customizable".to_string(), "All features".to_string(), "Agnostic".to_string()],
            },
        ];
    }

    /// Get basic development template content
    fn get_basic_template(&self) -> String {
        r#"# Basic Development Biome
# A simple biomeOS setup for development work with all Primals enabled

apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: dev-biome
  version: "1.0.0"
  description: "Basic development environment with all Primals"
  specialization: development
  owner: "dev-team"
  tags:
    - development
    - basic
    - learning

# Primal Configuration - All enabled with minimal resources
primals:
  # 🐕 BearDog - Security (Minimal for dev)
  beardog:
    enabled: true
    priority: 1
    startup_timeout: 30s
    config:
      security_level: low
      compliance: []
      hsm_integration: false
      dev_mode: true
      
  # 🎼 Songbird - Service Mesh
  songbird:
    enabled: true
    priority: 2
    startup_timeout: 30s
    depends_on: [beardog]
    config:
      discovery_backend: memory
      load_balancing: round_robin
      federation_enabled: false
      
  # 🏰 NestGate - Storage (Basic ZFS)
  nestgate:
    enabled: true
    priority: 3
    startup_timeout: 45s
    depends_on: [beardog, songbird]
    config:
      zfs_pool: "devpool"
      tiered_storage: false
      protocols: [nfs]
      
  # 🍄 Toadstool - Runtime
  toadstool:
    enabled: true
    priority: 4
    startup_timeout: 30s
    depends_on: [beardog, songbird, nestgate]
    config:
      runtimes: [container, native]
      resource_limits:
        cpu: "0-7"
        memory: "16Gi"
        
  # 🐿️ Squirrel - MCP Platform
  squirrel:
    enabled: true
    priority: 5
    startup_timeout: 30s
    depends_on: [beardog, songbird, toadstool]
    config:
      ai_providers: [openai]
      plugin_sandboxing: relaxed
      mcp_transports: [stdio]

# Basic resource configuration
resources:
  compute:
    nodes:
      - name: dev-node
        cpu_cores: 8
        memory: "16Gi"
        storage:
          local: "100Gi"
          
# Development-friendly networking
networking:
  discovery:
    provider: songbird
    backend: memory
    
# Basic security for development
security:
  authentication:
    provider: beardog
    methods: [token]
    token_lifetime: 24h
    
  service_mesh:
    mtls_enabled: false
    
# Development services
services:
  dev-environment:
    primal: toadstool
    runtime: container
    image: "ubuntu:22.04"
    ports:
      - "8080:8080"
    volumes:
      - "dev-workspace:/workspace"
    environment:
      - "DEV_MODE=true"
"#.to_string()
    }

    /// Get AI research template content
    fn get_ai_research_template(&self) -> String {
        r#"# AI Research Biome
# High-performance biomeOS setup for AI/ML research with GPU compute

apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: ai-research-biome
  version: "1.0.0"
  description: "AI research environment with GPU compute and ML workflows"
  specialization: research
  owner: "ai-research-team"
  tags:
    - ai-research
    - gpu-compute
    - machine-learning
    - high-performance

# Primal Configuration - Optimized for AI workloads
primals:
  # 🐕 BearDog - Enhanced Security for Research Data
  beardog:
    enabled: true
    priority: 1
    startup_timeout: 30s
    config:
      security_level: high
      compliance: [gdpr, hipaa]
      hsm_integration: true
      data_classification: true
      
  # 🎼 Songbird - High-Performance Service Mesh
  songbird:
    enabled: true
    priority: 2
    startup_timeout: 45s
    depends_on: [beardog]
    config:
      discovery_backend: consul
      load_balancing: health_based
      federation_enabled: true
      gpu_aware_routing: true
      
  # 🏰 NestGate - Large-Scale Tiered Storage
  nestgate:
    enabled: true
    priority: 3
    startup_timeout: 60s
    depends_on: [beardog, songbird]
    config:
      zfs_pool: "researchpool"
      tiered_storage: true
      protocols: [nfs, smb, s3]
      deduplication: true
      compression: zstd
      
  # 🍄 Toadstool - Multi-Runtime with GPU Support
  toadstool:
    enabled: true
    priority: 4
    startup_timeout: 45s
    depends_on: [beardog, songbird, nestgate]
    config:
      runtimes: [container, wasm, native, gpu]
      resource_limits:
        cpu: "0-31"
        memory: "256Gi"
        gpu: "0-7"
      gpu_scheduling: true
      
  # 🐿️ Squirrel - Advanced AI Agent Platform
  squirrel:
    enabled: true
    priority: 5
    startup_timeout: 60s
    depends_on: [beardog, songbird, toadstool]
    config:
      ai_providers: [openai, anthropic, local]
      plugin_sandboxing: strict
      mcp_transports: [stdio, websocket]
      model_management: true

# AI-optimized resources
resources:
  compute:
    nodes:
      - name: gpu-node-1
        cpu_cores: 32
        memory: "256Gi"
        gpu:
          - type: nvidia-a100
            count: 4
            memory: "80Gi"
        storage:
          local: "4Ti"
          nvme: true
          
# AI agent configuration
agents:
  research-assistant:
    provider: anthropic
    model: claude-3-sonnet
    runtime: squirrel
    capabilities:
      - code_analysis
      - data_processing
      - research_assistance
    resources:
      memory: "4Gi"
      cpu: 2
      timeout: 300s
      
  data-scientist:
    provider: local
    model: "llama-3-70b"
    runtime: squirrel
    capabilities:
      - statistical_analysis
      - visualization
      - model_training
    resources:
      memory: "16Gi"
      cpu: 8
      gpu: 1
"#.to_string()
    }

    /// Get enterprise template content
    fn get_enterprise_template(&self) -> String {
        r#"# Secure Enterprise Biome
# Enterprise-grade biomeOS setup with enhanced security and compliance

apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: enterprise-biome
  version: "1.0.0"
  description: "Secure enterprise environment with compliance and audit"
  specialization: enterprise
  owner: "enterprise-security-team"
  tags:
    - enterprise
    - security
    - compliance
    - audit
    - production

# Primal Configuration - Maximum Security
primals:
  # 🐕 BearDog - Maximum Security Configuration
  beardog:
    enabled: true
    priority: 1
    startup_timeout: 60s
    config:
      security_level: maximum
      compliance: [sox, gdpr, hipaa, pci_dss, iso27001]
      hsm_integration: true
      fips_140_2: true
      multi_party_approval: true
      threat_detection: true
      
  # 🎼 Songbird - Secure Service Mesh
  songbird:
    enabled: true
    priority: 2
    startup_timeout: 60s
    depends_on: [beardog]
    config:
      discovery_backend: consul
      load_balancing: health_based
      federation_enabled: true
      mtls_required: true
      zero_trust: true
      
  # 🏰 NestGate - Encrypted Storage
  nestgate:
    enabled: true
    priority: 3
    startup_timeout: 90s
    depends_on: [beardog, songbird]
    config:
      zfs_pool: "enterprise-pool"
      tiered_storage: true
      protocols: [nfs, smb, s3]
      encryption_at_rest: aes256
      encryption_in_transit: true
      
  # 🍄 Toadstool - Secure Runtime
  toadstool:
    enabled: true
    priority: 4
    startup_timeout: 60s
    depends_on: [beardog, songbird, nestgate]
    config:
      runtimes: [container, native]
      resource_limits:
        cpu: "0-15"
        memory: "128Gi"
      security_profiles: strict
      container_scanning: true
      
  # 🐿️ Squirrel - Controlled AI Platform
  squirrel:
    enabled: true
    priority: 5
    startup_timeout: 60s
    depends_on: [beardog, songbird, toadstool]
    config:
      ai_providers: [enterprise_openai]
      plugin_sandboxing: maximum
      mcp_transports: [stdio]
      audit_logging: true

# Enterprise security configuration
security:
  authentication:
    provider: beardog
    methods: [jwt, mutual_tls, biometric]
    token_lifetime: 8h
    multi_factor: required
    
  service_mesh:
    mtls_enabled: true
    cipher_suites: [ECDHE-ECDSA-AES256-GCM-SHA384]
    cert_rotation: 7d
    
  compliance:
    standards: [sox, gdpr, hipaa, pci_dss]
    audit_retention: 7y
    log_encryption: true
    
  access_control:
    rbac: true
    policies: [zero_trust, least_privilege]
    
# Enterprise monitoring
observability:
  metrics:
    provider: prometheus
    retention: 365d
    encryption: true
    
  logging:
    provider: elasticsearch
    retention: 2y
    structured: true
    encryption: true
    
  alerting:
    provider: alertmanager
    severity_levels: [info, warning, critical, emergency]
    
# Compliance agents
agents:
  compliance-monitor:
    provider: enterprise_openai
    model: gpt-4
    runtime: squirrel
    capabilities:
      - policy_compliance
      - risk_assessment
      - audit_assistance
    resources:
      memory: "8Gi"
      cpu: 4
      timeout: 600s
    sandbox:
      type: maximum_security
      network_access: restricted
      data_access: audit_only
"#.to_string()
    }

    /// Get custom template content
    fn get_custom_template(&self) -> String {
        r#"# Custom biome.yaml - Completely Agnostic biomeOS Manifest
# This can define ANY Primal types - current, future, or community-created

metadata:
  version: "1.0.0"
  name: "custom-biome" 
  description: "Custom biome configuration"
  tags: ["custom", "flexible", "agnostic"]
  author: "Custom User"

# Primals section - completely open to any type
primals:
  
  # Current Primals - these exist today
  compute:
    primal_type: "toadstool"
    version: ">=1.0.0"
    name: "primary-compute"
    required: true
    config:
      container_runtime: "podman"
      vm_engine: "qemu"
      resource_pools:
        cpu: 8
        memory: "16GB"
        storage: "500GB"
    networking:
      host: "0.0.0.0" 
      port: 8080
    expose: ["compute.vm", "compute.container"]

  orchestration:
    primal_type: "songbird"
    version: "^2.0.0"
    name: "service-mesh"
    required: true
    config:
      service_discovery: true
      load_balancing: "round_robin"
      health_checks: true
      plugins:
        - "metrics"
        - "tracing"
    networking:
      port: 8081
    dependencies: ["compute"]
    expose: ["orchestration.service", "orchestration.mesh"]

  storage:
    primal_type: "nestgate"
    version: ">=3.0.0" 
    name: "sovereign-nas"
    required: true
    config:
      zfs_pools:
        - name: "rpool"
          devices: ["/dev/sda", "/dev/sdb"]
          raid_level: "mirror"
      protocols: ["nfs", "smb", "s3"]
      encryption: "beardog"
    networking:
      port: 8082
    expose: ["storage.zfs", "storage.protocols"]

  ai_platform:
    primal_type: "squirrel"
    version: ">=1.5.0"
    name: "mcp-agents"
    required: false
    config:
      mcp_protocol: "latest"
      plugin_system: true
      ai_models:
        - "claude-3-sonnet"
        - "gpt-4"
      sandboxing: "secure"
    networking:
      port: 8083
    dependencies: ["compute", "orchestration"]
    expose: ["ai.mcp", "ai.agents", "ai.plugins"]

  security:
    primal_type: "beardog"
    version: ">=2.0.0"
    name: "crypto-security"
    required: true
    config:
      encryption_algorithms: ["ChaCha20-Poly1305", "AES-256-GCM"]
      key_management: "sovereign"
      authentication: "biometric+token"
    networking:
      port: 8084
    expose: ["security.encryption", "security.auth"]

# Global networking configuration
networking:
  mode: "bridge"
  discovery:
    method: "dns"
    config:
      domain: "biome.local"

# Global security configuration  
security:
  auth:
    method: "beardog_unified"
    config:
      biometric_required: true
      token_expiry: "24h"
  tls:
    enabled: true
    certificates:
      auto_generate: true
      ca: "biome_ca"

# Global resource limits
resources:
  cpu:
    max_cores: 16
  memory:
    max_mb: 32768
  storage:
    max_mb: 1048576  # 1TB
  network:
    max_bandwidth_mbps: 1000

# Extensions for future features
extensions:
  monitoring:
    enabled: true
    metrics_endpoint: "/metrics"
    log_level: "info"
  
  backup:
    enabled: true
    schedule: "0 2 * * *"  # Daily at 2 AM
    retention: "30d"
  
  updates:
    auto_update: false
    check_interval: "24h"
"#.to_string()
    }

    /// Load a specific template
    fn load_template(&mut self, template_name: &str) {
        if let Some(template) = self.available_templates.iter().find(|t| t.name == template_name) {
            self.current_yaml = template.content.clone();
            self.original_yaml = template.content.clone();
            self.selected_template = Some(template_name.to_string());
            self.is_modified = false;
            self.validate_yaml();
            self.parse_yaml_sections();
        }
    }

    /// Parse YAML into sections for structured editing
    fn parse_yaml_sections(&mut self) {
        self.yaml_sections.clear();
        
        let lines: Vec<&str> = self.current_yaml.lines().collect();
        let mut current_section: Option<String> = None;
        let mut section_start = 0;
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Detect top-level sections
            if !trimmed.starts_with(' ') && !trimmed.starts_with('#') && !trimmed.is_empty() && trimmed.contains(':') {
                // Save previous section
                if let Some(section_name) = current_section.clone() {
                    let section_content = lines[section_start..i].join("\n");
                    let section_type = self.determine_section_type(&section_name);
                    self.yaml_sections.insert(section_name.clone(), YamlSection {
                        name: section_name,
                        start_line: section_start,
                        end_line: i,
                        content: section_content,
                        section_type,
                    });
                }
                
                // Start new section
                let section_name = trimmed.split(':').next().unwrap_or("unknown").to_string();
                current_section = Some(section_name);
                section_start = i;
            }
        }
        
        // Save last section
        if let Some(section_name) = current_section {
            let section_content = lines[section_start..].join("\n");
            let section_type = self.determine_section_type(&section_name);
            self.yaml_sections.insert(section_name.clone(), YamlSection {
                name: section_name,
                start_line: section_start,
                end_line: lines.len(),
                content: section_content,
                section_type,
            });
        }
    }

    /// Determine the type of a YAML section
    fn determine_section_type(&self, section_name: &str) -> YamlSectionType {
        match section_name.to_lowercase().as_str() {
            "metadata" | "apiversion" | "kind" => YamlSectionType::Metadata,
            "primals" => YamlSectionType::Primals,
            "services" => YamlSectionType::Services,
            "resources" => YamlSectionType::Resources,
            "security" => YamlSectionType::Security,
            "networking" => YamlSectionType::Networking,
            "agents" => YamlSectionType::Agents,
            _ => YamlSectionType::Extensions,
        }
    }

    /// Validate YAML syntax and biome.yaml structure
    fn validate_yaml(&mut self) {
        self.validation_errors.clear();
        self.validation_warnings.clear();
        
        // Basic YAML syntax validation
        match serde_yaml::from_str::<serde_yaml::Value>(&self.current_yaml) {
            Ok(value) => {
                // Validate biome.yaml structure
                self.validate_biome_structure(&value);
            }
            Err(e) => {
                self.validation_errors.push(format!("YAML syntax error: {}", e));
            }
        }
    }

    /// Validate biome.yaml specific structure
    fn validate_biome_structure(&mut self, value: &serde_yaml::Value) {
        if let Some(map) = value.as_mapping() {
            // Check for required fields
            if !map.contains_key("metadata") && !map.contains_key("apiVersion") {
                self.validation_errors.push("Missing required 'metadata' or 'apiVersion' field".to_string());
            }
            
            // Validate primals section
            if let Some(primals) = map.get("primals") {
                if let Some(primals_map) = primals.as_mapping() {
                    for (primal_name, primal_config) in primals_map {
                        if let Some(primal_name_str) = primal_name.as_str() {
                            self.validate_primal_config(primal_name_str, primal_config);
                        }
                    }
                }
            } else {
                self.validation_warnings.push("No 'primals' section found - at least one primal is recommended".to_string());
            }
            
            // Validate other sections
            self.validate_resources_section(map.get("resources"));
            self.validate_security_section(map.get("security"));
            self.validate_networking_section(map.get("networking"));
        }
    }

    /// Validate individual primal configuration
    fn validate_primal_config(&mut self, primal_name: &str, config: &serde_yaml::Value) {
        if let Some(config_map) = config.as_mapping() {
            // Check for required primal fields
            if !config_map.contains_key("enabled") {
                self.validation_warnings.push(format!("Primal '{}' should have 'enabled' field", primal_name));
            }
            
            // Validate known primal types
            match primal_name {
                "beardog" | "songbird" | "nestgate" | "toadstool" | "squirrel" => {
                    // These are known primals - validate their specific configs
                    if primal_name == "beardog" && !config_map.contains_key("priority") {
                        self.validation_warnings.push("BearDog should have highest priority (1)".to_string());
                    }
                }
                _ => {
                    // Unknown primal - that's okay in biomeOS
                    self.validation_warnings.push(format!("Unknown primal '{}' - this is fine if it's a custom primal", primal_name));
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
            self.validation_warnings.push("No security section found - consider adding security configuration".to_string());
        }
    }

    /// Validate networking section
    fn validate_networking_section(&mut self, networking: Option<&serde_yaml::Value>) {
        if networking.is_none() {
            self.validation_warnings.push("No networking section found - using default networking configuration".to_string());
        }
    }

    /// Save current YAML to file
    fn save_yaml(&mut self) {
        if let Some(file_path) = &self.file_path {
            // In a real implementation, this would save to the file system
            // For now, we'll just mark as saved
            self.original_yaml = self.current_yaml.clone();
            self.is_modified = false;
            
            // Show success message
            // Could add a notification system here
        }
    }

    /// Create new YAML file
    fn new_yaml(&mut self) {
        self.current_yaml = String::new();
        self.original_yaml = String::new();
        self.file_path = None;
        self.is_modified = false;
        self.selected_template = None;
        self.validation_errors.clear();
        self.validation_warnings.clear();
    }

    /// Render the main editor interface
    fn render_editor(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Editor controls
            if ui.button("📁 New").clicked() {
                self.new_yaml();
            }
            
            if ui.button("💾 Save").clicked() {
                self.save_yaml();
            }
            
            ui.separator();
            
            if ui.button("📋 Templates").clicked() {
                self.show_template_browser = !self.show_template_browser;
            }
            
            if ui.button("🔍 Validate").clicked() {
                self.validate_yaml();
            }
            
            ui.separator();
            
            ui.checkbox(&mut self.show_preview, "Preview");
            ui.checkbox(&mut self.syntax_highlighting, "Syntax Highlighting");
            ui.checkbox(&mut self.line_numbers, "Line Numbers");
        });
        
        ui.add_space(5.0);
        
        // Modified indicator
        if self.is_modified {
            ui.colored_label(egui::Color32::YELLOW, "● Modified");
        }
        
        ui.add_space(10.0);
        
        // Main editor area
        ui.horizontal(|ui| {
            // Left panel - Editor
            ui.vertical(|ui| {
                ui.heading("📝 YAML Editor");
                
                // Editor
                let editor_response = ui.add(
                    egui::TextEdit::multiline(&mut self.current_yaml)
                        .desired_width(f32::INFINITY)
                        .desired_rows(25)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                );
                
                if editor_response.changed() {
                    self.is_modified = true;
                    self.validate_yaml();
                    self.parse_yaml_sections();
                }
            });
            
            // Right panel - Validation
            if self.show_validation_panel {
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.heading("✅ Validation");
                    
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            // Validation errors
                            if !self.validation_errors.is_empty() {
                                ui.colored_label(egui::Color32::RED, "❌ Errors:");
                                for error in &self.validation_errors {
                                    ui.colored_label(egui::Color32::RED, format!("  • {}", error));
                                }
                            }
                            
                            // Validation warnings
                            if !self.validation_warnings.is_empty() {
                                ui.colored_label(egui::Color32::YELLOW, "⚠️ Warnings:");
                                for warning in &self.validation_warnings {
                                    ui.colored_label(egui::Color32::YELLOW, format!("  • {}", warning));
                                }
                            }
                            
                            if self.validation_errors.is_empty() && self.validation_warnings.is_empty() {
                                ui.colored_label(egui::Color32::GREEN, "✅ YAML is valid!");
                            }
                        });
                });
            }
        });
    }

    /// Render template browser
    fn render_template_browser(&mut self, ui: &mut egui::Ui) {
        if !self.show_template_browser {
            return;
        }
        
        ui.collapsing("📋 Template Browser", |ui| {
            ui.label("Choose a template to start with:");
            
            // Clone the templates to avoid borrowing issues
            let templates = self.available_templates.clone();
            
            for template in &templates {
                ui.horizontal(|ui| {
                    if ui.button(&template.name).clicked() {
                        self.load_template(&template.name);
                        self.show_template_browser = false;
                    }
                    
                    ui.label(&template.description);
                    ui.colored_label(egui::Color32::LIGHT_BLUE, &template.category);
                });
            }
        });
    }

    /// Render structured section editor
    fn render_structured_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏗️ Structured Editor");
        
        egui::ScrollArea::vertical()
            .show(ui, |ui| {
                for (section_name, section) in &self.yaml_sections {
                    let is_collapsed = *self.collapsed_sections.get(section_name).unwrap_or(&false);
                    
                    ui.horizontal(|ui| {
                        let collapse_response = ui.button(if is_collapsed { "▶" } else { "▼" });
                        if collapse_response.clicked() {
                            self.collapsed_sections.insert(section_name.clone(), !is_collapsed);
                        }
                        
                        ui.label(egui::RichText::new(section_name).heading());
                        
                        // Section type badge
                        let color = match section.section_type {
                            YamlSectionType::Metadata => egui::Color32::LIGHT_BLUE,
                            YamlSectionType::Primals => egui::Color32::GREEN,
                            YamlSectionType::Services => egui::Color32::YELLOW,
                            YamlSectionType::Resources => egui::Color32::LIGHT_RED,
                            YamlSectionType::Security => egui::Color32::RED,
                            YamlSectionType::Networking => egui::Color32::BLUE,
                            YamlSectionType::Agents => egui::Color32::LIGHT_GREEN,
                            YamlSectionType::Extensions => egui::Color32::GRAY,
                        };
                        
                        ui.colored_label(color, format!("{:?}", section.section_type));
                    });
                    
                    if !is_collapsed {
                        ui.indent(section_name, |ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new(&section.content)
                                    .monospace()
                                    .color(egui::Color32::LIGHT_GRAY)
                            ));
                        });
                    }
                }
            });
    }
}

impl View for YamlEditorView {
    fn render(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("📝 biome.yaml Editor");
        ui.separator();
        
        // Template browser
        self.render_template_browser(ui);
        
        ui.add_space(10.0);
        
        // Editor mode tabs
        ui.horizontal(|ui| {
            if ui.selectable_label(self.editor_mode == EditorMode::Raw, "✏️ Raw Editor").clicked() {
                self.editor_mode = EditorMode::Raw;
            }
            if ui.selectable_label(self.editor_mode == EditorMode::Structured, "🏗️ Structured").clicked() {
                self.editor_mode = EditorMode::Structured;
            }
            if ui.selectable_label(self.editor_mode == EditorMode::Preview, "👁️ Preview").clicked() {
                self.editor_mode = EditorMode::Preview;
            }
        });
        
        ui.add_space(10.0);
        
        // Main content based on mode
        match self.editor_mode {
            EditorMode::Raw => self.render_editor(ui),
            EditorMode::Structured => self.render_structured_editor(ui),
            EditorMode::Preview => {
                ui.heading("👁️ Preview");
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(&self.current_yaml)
                                .monospace()
                                .color(egui::Color32::LIGHT_GRAY)
                        ));
                    });
            }
        }
    }
} 
