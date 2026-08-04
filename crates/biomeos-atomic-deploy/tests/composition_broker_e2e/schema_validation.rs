// SPDX-License-Identifier: AGPL-3.0-or-later

//! Signal schema validation for nest-tier signals in signal_tools.toml.

use super::graphs_dir;

#[test]
fn nest_signals_in_schema() {
    use biomeos_atomic_deploy::handlers::signal::load_signal_schema;

    let dir = graphs_dir();
    let schema = load_signal_schema(&dir).expect("load signal_tools.toml");
    let tools = schema["tools"].as_array().expect("tools array");

    let nest_tools: Vec<&serde_json::Value> = tools
        .iter()
        .filter(|t| t["tier"].as_str() == Some("nest"))
        .collect();

    assert_eq!(
        nest_tools.len(),
        12,
        "signal_tools.toml should define all 12 Nest signals (9 core + 3 data federation)"
    );

    let nest_names: Vec<&str> = nest_tools
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    assert!(nest_names.contains(&"nest.ingest_spore"));
    assert!(nest_names.contains(&"nest.ingest_dataset"));
    assert!(nest_names.contains(&"nest.store"));
    assert!(nest_names.contains(&"nest.commit"));
    assert!(nest_names.contains(&"nest.retrieve"));
    assert!(nest_names.contains(&"nest.sync"));
    assert!(nest_names.contains(&"nest.emit_spore"));
    assert!(nest_names.contains(&"nest.verify"));
    assert!(nest_names.contains(&"nest.federate"));
    assert!(nest_names.contains(&"nest.declare_dataset"));
    assert!(nest_names.contains(&"nest.acquire_file"));
    assert!(nest_names.contains(&"nest.complete_dataset"));
}

#[test]
fn ingest_signals_are_sequential() {
    use biomeos_atomic_deploy::handlers::signal::load_signal_schema;

    let dir = graphs_dir();
    let schema = load_signal_schema(&dir).expect("load signal_tools.toml");
    let tools = schema["tools"].as_array().expect("tools array");

    for name in ["nest.ingest_spore", "nest.ingest_dataset"] {
        let tool = tools
            .iter()
            .find(|t| t["name"].as_str() == Some(name))
            .unwrap_or_else(|| panic!("missing {name} in schema"));
        assert_eq!(
            tool["coordination"].as_str(),
            Some("sequential"),
            "{name} must be sequential — pipeline ordering"
        );
    }
}
