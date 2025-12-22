//! Deployment and Completion Rendering
//!
//! This module contains UI rendering functions for the deployment and
//! completion phases of the BYOB (Build Your Own Biome) process.

use super::super::data::{get_mock_deployments, get_mock_services};
use super::super::types::*;
use egui::{Color32, RichText, Ui};

/// Render deployment interface
pub fn render_deployment(ui: &mut Ui, deployment_data: &mut DeploymentData) -> bool {
    ui.heading("🚀 Deploy Your Biome");
    ui.separator();

    let mut completed = false;

    // Mock deployment data
    if deployment_data.deployments.is_empty() {
        deployment_data.deployments = get_mock_deployments();
        deployment_data.services = get_mock_services();
        deployment_data.active_deployments = deployment_data.deployments.len();
        deployment_data.total_resource_usage = ResourceUsage {
            cpu_percent: 45.0,
            memory_percent: 55.0,
            storage_percent: 30.0,
            network_mbps: 85.0,
            cpu_cores: 6.0,
            memory_gb: 12.0,
            storage_gb: 75.0,
        };
    }

    // Deployment status
    ui.heading("Deployment Status");
    ui.horizontal(|ui| {
        ui.label("Active Deployments:");
        ui.label(RichText::new(deployment_data.active_deployments.to_string()).strong());
    });

    // Resource usage
    ui.heading("Resource Usage");
    ui.horizontal(|ui| {
        ui.label("CPU:");
        ui.add(
            egui::ProgressBar::new(
                (deployment_data.total_resource_usage.cpu_percent / 100.0) as f32,
            )
            .text(format!(
                "{:.1}%",
                deployment_data.total_resource_usage.cpu_percent
            )),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Memory:");
        ui.add(
            egui::ProgressBar::new(
                (deployment_data.total_resource_usage.memory_percent / 100.0) as f32,
            )
            .text(format!(
                "{:.1}%",
                deployment_data.total_resource_usage.memory_percent
            )),
        );
    });

    ui.separator();

    // Deployments list
    ui.heading("Active Deployments");
    for deployment in &deployment_data.deployments {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&deployment.name).strong());
                ui.label(format!("{:?}", deployment.status));

                // Show primals used
                if !deployment.primals.is_empty() {
                    ui.label("Primals:");
                    for primal in &deployment.primals {
                        ui.label(RichText::new(primal).color(Color32::BLUE));
                    }
                }

                // Show capabilities
                if !deployment.capabilities.is_empty() {
                    ui.label("Capabilities:");
                    for cap in &deployment.capabilities {
                        ui.label(RichText::new(cap.display_name()).color(Color32::GREEN));
                    }
                }
            });
        });
    }

    ui.separator();

    // Services list
    ui.heading("Running Services");
    for service in &deployment_data.services {
        ui.horizontal(|ui| {
            ui.label(&service.name);
            ui.label(format!("{:?}", service.status));
            if let Some(port) = service.port {
                ui.label(format!("Port: {}", port));
            }
            ui.label(format!("Primal: {}", service.primal_name));

            // Show service capabilities
            if !service.capabilities.is_empty() {
                ui.label("Capabilities:");
                for cap in &service.capabilities {
                    ui.label(RichText::new(cap.display_name()).color(Color32::GREEN));
                }
            }
        });
    }

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Back").clicked() {
            // Handle back navigation
        }

        if ui.button("Stop Deployment").clicked() {
            // Handle stop
        }

        if ui.button("Complete").clicked() {
            completed = true;
        }
    });

    completed
}

/// Render completion interface
pub fn render_completion(ui: &mut Ui) {
    ui.heading("🎉 Biome Deployment Complete!");
    ui.separator();

    ui.label("Your biome has been successfully deployed using capability-based primal selection.");
    ui.label("The system automatically matched your requirements with available primals.");

    ui.separator();

    ui.heading("What's Next?");
    ui.label("• Monitor your deployment through the dashboard");
    ui.label("• Scale services based on demand");
    ui.label("• Add more capabilities as needed");
    ui.label("• Explore other niche templates");

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Create Another Biome").clicked() {
            // Handle restart
        }

        if ui.button("View Dashboard").clicked() {
            // Handle dashboard view
        }
    });
} 