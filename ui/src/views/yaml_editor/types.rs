//! Core types and data structures for YAML editor

use std::collections::HashMap;

/// YAML Editor view with comprehensive editing capabilities
pub struct YamlEditorView {
    pub base: crate::views::BaseView,
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
