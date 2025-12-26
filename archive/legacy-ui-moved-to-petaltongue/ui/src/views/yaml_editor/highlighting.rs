//! YAML syntax highlighting

use super::types::*;
use eframe::egui;

impl YamlEditorView {
    /// Highlight YAML syntax
    pub fn highlight_yaml_syntax(&self, layout_job: &mut egui::text::LayoutJob, text: &str) {
        layout_job.text = text.to_string();

        let mut current_pos = 0;

        for line in text.lines() {
            let line_start = current_pos;
            let line_end = current_pos + line.len();

            // Highlight comments
            if let Some(comment_pos) = line.find('#') {
                let comment_start = line_start + comment_pos;
                let comment_end = line_end;

                layout_job.sections.push(egui::text::LayoutSection {
                    leading_space: 0.0,
                    byte_range: comment_start..comment_end,
                    format: egui::TextFormat {
                        color: egui::Color32::from_rgb(128, 128, 128),
                        italics: true,
                        ..Default::default()
                    },
                });
            }

            // Highlight keys (text before colon)
            if let Some(colon_pos) = line.find(':') {
                let key_start = line_start + line.len() - line.trim_start().len();
                let key_end = line_start + colon_pos;

                layout_job.sections.push(egui::text::LayoutSection {
                    leading_space: 0.0,
                    byte_range: key_start..key_end,
                    format: egui::TextFormat {
                        color: egui::Color32::from_rgb(100, 150, 255),
                        ..Default::default()
                    },
                });
            }

            // Highlight strings (quoted values)
            self.highlight_strings_in_line(layout_job, line, line_start);

            // Highlight numbers
            self.highlight_numbers_in_line(layout_job, line, line_start);

            // Highlight booleans
            self.highlight_booleans_in_line(layout_job, line, line_start);

            current_pos = line_end + 1; // +1 for newline
        }
    }

    /// Highlight strings in a line
    fn highlight_strings_in_line(
        &self,
        layout_job: &mut egui::text::LayoutJob,
        line: &str,
        line_start: usize,
    ) {
        let mut chars = line.char_indices().peekable();

        while let Some((i, ch)) = chars.next() {
            if ch == '"' {
                // Find closing quote
                let mut string_end = i + 1;
                let mut escaped = false;

                while let Some((j, next_ch)) = chars.next() {
                    if next_ch == '"' && !escaped {
                        string_end = j + 1;
                        break;
                    }
                    escaped = next_ch == '\\' && !escaped;
                }

                layout_job.sections.push(egui::text::LayoutSection {
                    leading_space: 0.0,
                    byte_range: (line_start + i)..(line_start + string_end),
                    format: egui::TextFormat {
                        color: egui::Color32::from_rgb(150, 255, 150),
                        ..Default::default()
                    },
                });
            }
        }
    }

    /// Highlight numbers in a line
    fn highlight_numbers_in_line(
        &self,
        layout_job: &mut egui::text::LayoutJob,
        line: &str,
        line_start: usize,
    ) {
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut current_pos = 0;

        for word in words {
            if let Some(word_start) = line[current_pos..].find(word) {
                let word_pos = current_pos + word_start;

                if word.parse::<f64>().is_ok() {
                    layout_job.sections.push(egui::text::LayoutSection {
                        leading_space: 0.0,
                        byte_range: (line_start + word_pos)..(line_start + word_pos + word.len()),
                        format: egui::TextFormat {
                            color: egui::Color32::from_rgb(255, 200, 100),
                            ..Default::default()
                        },
                    });
                }

                current_pos = word_pos + word.len();
            }
        }
    }

    /// Highlight booleans in a line
    fn highlight_booleans_in_line(
        &self,
        layout_job: &mut egui::text::LayoutJob,
        line: &str,
        line_start: usize,
    ) {
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut current_pos = 0;

        for word in words {
            if let Some(word_start) = line[current_pos..].find(word) {
                let word_pos = current_pos + word_start;

                if matches!(word, "true" | "false" | "yes" | "no" | "on" | "off") {
                    layout_job.sections.push(egui::text::LayoutSection {
                        leading_space: 0.0,
                        byte_range: (line_start + word_pos)..(line_start + word_pos + word.len()),
                        format: egui::TextFormat {
                            color: egui::Color32::from_rgb(255, 150, 100),
                            ..Default::default()
                        },
                    });
                }

                current_pos = word_pos + word.len();
            }
        }
    }

    /// Toggle syntax highlighting
    pub fn toggle_syntax_highlighting(&mut self) {
        self.syntax_highlighting = !self.syntax_highlighting;
    }

    /// Set syntax highlighting theme
    pub fn set_highlighting_theme(&mut self, theme: HighlightingTheme) {
        // Would implement different color schemes
        match theme {
            HighlightingTheme::Default => {
                // Default colors already set
            }
            HighlightingTheme::Dark => {
                // Dark theme colors
            }
            HighlightingTheme::Light => {
                // Light theme colors
            }
        }
    }
}

/// Syntax highlighting themes
#[derive(Debug, Clone, PartialEq)]
pub enum HighlightingTheme {
    Default,
    Dark,
    Light,
}
