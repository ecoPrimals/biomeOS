//! Team Selection Rendering
//!
//! This module contains UI rendering functions for the team selection phase
//! of the BYOB (Build Your Own Biome) process.

use super::super::data::{get_mock_teams, get_primal_discovery};
use super::super::types::*;
use biomeos_primal_sdk::PrimalCapability;
use egui::{Color32, RichText, Ui};

/// Render team selection interface
pub fn render_team_selection(ui: &mut Ui, team_data: &mut TeamData) -> bool {
    ui.heading("🏗️ Team Setup");
    ui.separator();

    let mut next_clicked = false;

    ui.horizontal(|ui| {
        ui.label("Team Name:");
        ui.text_edit_singleline(&mut team_data.team_info.name);
    });

    ui.horizontal(|ui| {
        ui.label("Description:");
        ui.text_edit_multiline(&mut team_data.team_info.description);
    });

    ui.horizontal(|ui| {
        ui.label("Team Size:");
        egui::ComboBox::from_label("")
            .selected_text(format!("{:?}", team_data.team_info.size))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut team_data.team_info.size, TeamSize::Solo, "Solo");
                ui.selectable_value(
                    &mut team_data.team_info.size,
                    TeamSize::Small,
                    "Small (2-5)",
                );
                ui.selectable_value(
                    &mut team_data.team_info.size,
                    TeamSize::Medium,
                    "Medium (6-15)",
                );
                ui.selectable_value(
                    &mut team_data.team_info.size,
                    TeamSize::Large,
                    "Large (16+)",
                );
            });
    });

    ui.horizontal(|ui| {
        ui.label("Focus Area:");
        ui.text_edit_singleline(&mut team_data.team_info.focus_area);
    });

    ui.horizontal(|ui| {
        ui.label("Experience Level:");
        egui::ComboBox::from_label("")
            .selected_text(format!("{:?}", team_data.team_info.experience_level))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut team_data.team_info.experience_level,
                    ExperienceLevel::Beginner,
                    "Beginner",
                );
                ui.selectable_value(
                    &mut team_data.team_info.experience_level,
                    ExperienceLevel::Intermediate,
                    "Intermediate",
                );
                ui.selectable_value(
                    &mut team_data.team_info.experience_level,
                    ExperienceLevel::Advanced,
                    "Advanced",
                );
                ui.selectable_value(
                    &mut team_data.team_info.experience_level,
                    ExperienceLevel::Expert,
                    "Expert",
                );
            });
    });

    ui.separator();

    // Capability selection
    ui.heading("Required Capabilities");
    ui.label("Select the capabilities your team needs:");

    let all_capabilities = vec![
        PrimalCapability::compute(),
        PrimalCapability::storage(),
        PrimalCapability::networking(),
        PrimalCapability::security(),
        PrimalCapability::ai(),
        PrimalCapability::orchestration(),
        PrimalCapability::web_development(),
        PrimalCapability::gaming(),
        PrimalCapability::machine_learning(),
        PrimalCapability::analytics(),
    ];

    egui::Grid::new("capabilities_grid")
        .num_columns(3)
        .spacing([10.0, 10.0])
        .show(ui, |ui| {
            for (i, capability) in all_capabilities.iter().enumerate() {
                let mut selected = team_data
                    .team_info
                    .required_capabilities
                    .contains(capability);
                if ui
                    .checkbox(&mut selected, capability.display_name())
                    .changed()
                {
                    if selected {
                        team_data
                            .team_info
                            .required_capabilities
                            .insert(capability.clone());
                    } else {
                        team_data.team_info.required_capabilities.remove(capability);
                    }
                }

                if (i + 1) % 3 == 0 {
                    ui.end_row();
                }
            }
        });

    ui.separator();

    // Show available primals that support selected capabilities
    if !team_data.team_info.required_capabilities.is_empty() {
        ui.heading("Available Primals");
        ui.label("Primals that support your selected capabilities:");

        let discovery = get_primal_discovery();
        let compatible_primals =
            discovery.find_primals_for_capabilities(&team_data.team_info.required_capabilities);

        for primal in compatible_primals {
            ui.horizontal(|ui| {
                ui.label(format!("• {}", primal.display_name));
                ui.label(format!("({})", primal.description));

                // Show matching capabilities
                let matching_caps: Vec<_> = primal
                    .capabilities
                    .intersection(&team_data.team_info.required_capabilities)
                    .collect();

                if !matching_caps.is_empty() {
                    ui.label("Provides:");
                    for cap in matching_caps {
                        ui.label(RichText::new(cap.display_name()).color(Color32::GREEN));
                    }
                }
            });
        }
    }

    ui.separator();

    // Show mock teams for inspiration
    ui.heading("Example Teams");
    ui.label("Get inspiration from these example teams:");

    let mock_teams = get_mock_teams();
    for team in mock_teams {
        ui.horizontal(|ui| {
            if ui.button(&team.name).clicked() {
                team_data.team_info = team.clone();
            }
            ui.label(&team.description);
        });
    }

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Next: Choose Niche").clicked() {
            next_clicked = true;
        }
    });

    next_clicked
}
