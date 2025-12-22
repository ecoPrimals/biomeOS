//! Manifest Generation and YAML Editing Rendering
//!
//! This module contains UI rendering functions for the manifest generation and
//! YAML editing phases of the BYOB (Build Your Own Biome) process.

use super::super::data::get_primal_discovery;
use super::super::types::*;
use egui::{Color32, RichText, Ui};
use std::collections::HashMap;

/// Render manifest generation interface
pub fn render_manifest_generation(
    ui: &mut Ui,
    template: &NicheTemplate,
    customizations: &HashMap<String, String>,
    generated_manifest: &mut String,
) -> bool {
    ui.heading("📄 Generate Manifest");
    ui.separator();

    let mut next_clicked = false;

    ui.horizontal(|ui| {
        ui.label("Niche:");
        ui.label(RichText::new(&template.name).strong());
    });

    ui.separator();

    // Generate manifest if not already done
    if generated_manifest.is_empty() {
        *generated_manifest = super::super::templates::generate_manifest(template, customizations);
    }

    ui.heading("Generated Manifest");
    ui.label("Preview of your biome manifest:");

    egui::ScrollArea::vertical()
        .max_height(300.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(generated_manifest)
                    .font(egui::TextStyle::Monospace)
                    .code_editor(),
            );
        });

    ui.separator();

    // Show capability analysis
    ui.heading("Capability Analysis");
    ui.label("Required capabilities and available primals:");

    let discovery = get_primal_discovery();
    for capability in &template.required_capabilities {
        let compatible_primals = discovery
            .get_registry()
            .find_primals_by_capability(capability);
        ui.horizontal(|ui| {
            ui.label(format!("• {}", capability.display_name()));
            if compatible_primals.is_empty() {
                ui.label(RichText::new("⚠️ No compatible primals found").color(Color32::RED));
            } else {
                ui.label(format!("✅ {} primals available", compatible_primals.len()));
                for primal in compatible_primals {
                    ui.label(RichText::new(&primal.display_name).color(Color32::GREEN));
                }
            }
        });
    }

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Back").clicked() {
            // Handle back navigation
        }

        if ui.button("Next: Edit YAML").clicked() {
            next_clicked = true;
        }

        if ui.button("Save Manifest").clicked() {
            // Handle save functionality
        }
    });

    next_clicked
}

/// Render YAML editing interface
pub fn render_yaml_editing(ui: &mut Ui, manifest: &mut String) -> bool {
    ui.heading("📝 Edit YAML Manifest");
    ui.separator();

    let mut next_clicked = false;

    ui.label("Fine-tune your biome manifest:");

    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(manifest)
                    .font(egui::TextStyle::Monospace)
                    .code_editor(),
            );
        });

    ui.separator();

    // Validation status
    ui.heading("Validation Status");
    
    // Basic YAML validation (simplified)
    let is_valid = !manifest.trim().is_empty() && manifest.contains("version:");
    
    if is_valid {
        ui.label(RichText::new("✅ Manifest appears valid").color(Color32::GREEN));
    } else {
        ui.label(RichText::new("⚠️ Manifest may have issues").color(Color32::YELLOW));
        ui.label("Make sure your manifest includes version and basic structure.");
    }

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Back").clicked() {
            // Handle back navigation
        }

        if ui.button("Next: Deploy").clicked() {
            next_clicked = true;
        }

        if ui.button("Save to File").clicked() {
            // Handle file save
        }

        if ui.button("Load from File").clicked() {
            // Handle file load
        }
    });

    next_clicked
} 