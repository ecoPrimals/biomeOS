//! UI Components Module
//!
//! This module provides reusable UI components that integrate with the unified
//! biomeos-types system for consistent data representation across the ecosystem.

use crate::{BiomeManifest, Health, PrimalCapability, ResourceMetrics};
use std::collections::HashMap;

/// Trait for renderable UI components
pub trait UIComponent {
    /// Component identifier
    fn id(&self) -> &str;

    /// Component title for display
    fn title(&self) -> &str;

    /// Whether the component is currently active/visible
    fn is_active(&self) -> bool;

    /// Update component state with new data
    fn update(&mut self, data: ComponentData);
}

/// Data types that can be passed to UI components
#[derive(Debug, Clone)]
pub enum ComponentData {
    /// System health information
    Health(Health),

    /// Resource usage metrics
    Metrics(ResourceMetrics),

    /// Biome configuration manifest
    Manifest(BiomeManifest),

    /// Primal capabilities list
    Capabilities(Vec<PrimalCapability>),

    /// Generic string data
    Text(String),

    /// Generic JSON data
    Json(serde_json::Value),
}

/// Health status display component
#[derive(Debug)]
pub struct HealthStatusComponent {
    id: String,
    title: String,
    current_health: Health,
    history: Vec<Health>,
    is_active: bool,
}

impl HealthStatusComponent {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            current_health: Health::Unknown {
                reason: "No health data available".to_string(),
                last_known: None,
            },
            history: Vec::new(),
            is_active: true,
        }
    }

    pub fn current_health(&self) -> &Health {
        &self.current_health
    }

    pub fn health_history(&self) -> &[Health] {
        &self.history
    }
}

impl UIComponent for HealthStatusComponent {
    fn id(&self) -> &str {
        &self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn update(&mut self, data: ComponentData) {
        if let ComponentData::Health(health) = data {
            self.history.push(self.current_health.clone());
            self.current_health = health;

            // Keep only last 100 health records
            if self.history.len() > 100 {
                self.history.remove(0);
            }
        }
    }
}

/// Resource metrics display component
#[derive(Debug)]
pub struct ResourceMetricsComponent {
    id: String,
    title: String,
    current_metrics: ResourceMetrics,
    is_active: bool,
}

impl ResourceMetricsComponent {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            current_metrics: ResourceMetrics {
                cpu_usage: Some(0.0),
                memory_usage: Some(0.0),
                disk_usage: Some(0.0),
                network_io: Some(crate::NetworkIoMetrics {
                    bytes_in_per_sec: 0.0,
                    bytes_out_per_sec: 0.0,
                    packets_in_per_sec: 0.0,
                    packets_out_per_sec: 0.0,
                }),
            },
            is_active: true,
        }
    }

    pub fn current_metrics(&self) -> &ResourceMetrics {
        &self.current_metrics
    }

    pub fn cpu_usage_percent(&self) -> f64 {
        self.current_metrics.cpu_usage.unwrap_or(0.0) * 100.0
    }

    pub fn memory_usage_percent(&self) -> f64 {
        self.current_metrics.memory_usage.unwrap_or(0.0) * 100.0
    }

    pub fn disk_usage_percent(&self) -> f64 {
        self.current_metrics.disk_usage.unwrap_or(0.0) * 100.0
    }
}

impl UIComponent for ResourceMetricsComponent {
    fn id(&self) -> &str {
        &self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn update(&mut self, data: ComponentData) {
        if let ComponentData::Metrics(metrics) = data {
            self.current_metrics = metrics;
        }
    }
}

/// Manifest editor component
#[derive(Debug)]
pub struct ManifestEditorComponent {
    id: String,
    title: String,
    current_manifest: Option<BiomeManifest>,
    yaml_content: String,
    is_dirty: bool,
    is_active: bool,
}

impl ManifestEditorComponent {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            current_manifest: None,
            yaml_content: String::new(),
            is_dirty: false,
            is_active: true,
        }
    }

    pub fn current_manifest(&self) -> Option<&BiomeManifest> {
        self.current_manifest.as_ref()
    }

    pub fn yaml_content(&self) -> &str {
        &self.yaml_content
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn set_yaml_content(&mut self, content: String) {
        self.yaml_content = content;
        self.is_dirty = true;
    }

    pub fn save(&mut self) -> Result<(), crate::BiomeError> {
        // Parse YAML and validate
        let manifest =
            biomeos_manifest::BiomeManifestProcessor::load_from_yaml(&self.yaml_content)?;
        self.current_manifest = Some(manifest);
        self.is_dirty = false;
        Ok(())
    }
}

impl UIComponent for ManifestEditorComponent {
    fn id(&self) -> &str {
        &self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn update(&mut self, data: ComponentData) {
        match data {
            ComponentData::Manifest(manifest) => {
                self.current_manifest = Some(manifest);
                // Convert back to YAML
                if let Some(manifest_ref) = &self.current_manifest {
                    if let Ok(yaml) =
                        biomeos_manifest::BiomeManifestProcessor::save_to_yaml(manifest_ref)
                    {
                        self.yaml_content = yaml;
                    }
                }
                self.is_dirty = false;
            }
            ComponentData::Text(content) => {
                self.yaml_content = content;
                self.is_dirty = true;
            }
            _ => {}
        }
    }
}

/// Component registry for managing UI components
pub struct ComponentRegistry {
    components: HashMap<String, Box<dyn UIComponent>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn register<T: UIComponent + 'static>(&mut self, component: T) {
        let id = component.id().to_string();
        self.components.insert(id, Box::new(component));
    }

    pub fn get(&self, id: &str) -> Option<&dyn UIComponent> {
        self.components.get(id).map(|c| c.as_ref())
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Box<dyn UIComponent>> {
        self.components.get_mut(id)
    }

    pub fn update_component(&mut self, id: &str, data: ComponentData) {
        if let Some(component) = self.components.get_mut(id) {
            component.update(data);
        }
    }

    pub fn active_components(&self) -> Vec<&dyn UIComponent> {
        self.components
            .values()
            .filter_map(|c| {
                if c.is_active() {
                    Some(c.as_ref())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
