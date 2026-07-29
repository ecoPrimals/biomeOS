// SPDX-License-Identifier: AGPL-3.0-or-later

//! Topology validation for nest.ingest_spore and nest.ingest_dataset signal graphs.

use super::graphs_dir;

mod ingest_spore {
    use super::*;

    fn load_graph() -> toml::Value {
        let path = graphs_dir().join("signals/nest_ingest_spore.toml");
        let content = std::fs::read_to_string(&path).expect("read nest_ingest_spore.toml");
        toml::from_str(&content).expect("parse nest_ingest_spore.toml")
    }

    #[test]
    fn graph_is_sequential_nest_tier() {
        let parsed = load_graph();
        let graph = parsed.get("graph").expect("missing [graph]");
        assert_eq!(graph["signal_tier"].as_str(), Some("nest"));
        assert_eq!(graph["signal_name"].as_str(), Some("ingest_spore"));
        assert_eq!(
            graph["coordination"].as_str(),
            Some("sequential"),
            "ingest_spore must be sequential: validate → store → DAG → ledger → braid → sign"
        );
    }

    #[test]
    fn graph_requires_btsp_security() {
        let parsed = load_graph();
        let meta = &parsed["graph"]["metadata"];
        assert_eq!(
            meta["security_model"].as_str(),
            Some("btsp_enforced"),
            "ingest_spore crosses composition boundaries — BTSP required"
        );
        assert_eq!(meta["secure_by_default"].as_bool(), Some(true));
    }

    #[test]
    fn graph_spans_both_atomics() {
        let parsed = load_graph();
        let meta = &parsed["graph"]["metadata"];
        let fragments = meta["fragments"].as_array().expect("fragments array");
        let fragment_strs: Vec<&str> = fragments.iter().filter_map(|f| f.as_str()).collect();
        assert!(
            fragment_strs.contains(&"tower_atomic"),
            "must include Tower (bearDog is tower-resident)"
        );
        assert!(
            fragment_strs.contains(&"nest_atomic"),
            "must include Nest (nestGate is nest-resident)"
        );
    }

    #[test]
    fn pipeline_has_six_nodes_in_order() {
        let parsed = load_graph();
        let nodes = parsed["graph"]["nodes"].as_array().expect("nodes array");
        assert_eq!(nodes.len(), 6, "ingest_spore: 6-node pipeline");

        let expected_sequence = [
            ("validate_envelope", "nestgate", 1),
            ("store_content", "nestgate", 2),
            ("dag_session", "rhizocrypt", 3),
            ("ledger_entry", "loamspine", 4),
            ("attribution_braid", "sweetgrass", 5),
            ("sign_receipt", "beardog", 6),
        ];

        for (i, (name, binary, order)) in expected_sequence.iter().enumerate() {
            let node = &nodes[i];
            assert_eq!(node["name"].as_str(), Some(*name), "node {i} name");
            assert_eq!(node["binary"].as_str(), Some(*binary), "node {i} binary");
            assert_eq!(node["order"].as_integer(), Some(*order), "node {i} order");
            assert_eq!(
                node["spawn"].as_bool(),
                Some(false),
                "ingest_spore connects to running primals"
            );
        }
    }

    #[test]
    fn dependency_chain_is_linear() {
        let parsed = load_graph();
        let nodes = parsed["graph"]["nodes"].as_array().expect("nodes array");

        assert!(
            nodes[0].get("depends_on").is_none(),
            "validate_envelope has no dependencies"
        );

        let names: Vec<&str> = nodes.iter().map(|n| n["name"].as_str().unwrap()).collect();
        for i in 1..nodes.len() {
            let deps = nodes[i]["depends_on"].as_array().expect("depends_on");
            assert!(
                deps.iter().any(|d| d.as_str() == Some(names[i - 1])),
                "node '{}' must depend on predecessor '{}'",
                names[i],
                names[i - 1]
            );
        }
    }

    #[test]
    fn provenance_trio_represented() {
        let parsed = load_graph();
        let nodes = parsed["graph"]["nodes"].as_array().expect("nodes array");
        let binaries: Vec<&str> = nodes.iter().filter_map(|n| n["binary"].as_str()).collect();

        assert!(
            binaries.contains(&"rhizocrypt"),
            "rhizoCrypt (DAG layer) in pipeline"
        );
        assert!(
            binaries.contains(&"loamspine"),
            "loamSpine (ledger layer) in pipeline"
        );
        assert!(
            binaries.contains(&"sweetgrass"),
            "sweetGrass (attribution layer) in pipeline"
        );
    }

    #[test]
    fn capability_routing_uses_domains() {
        let parsed = load_graph();
        let nodes = parsed["graph"]["nodes"].as_array().expect("nodes array");

        let expected_domains = [
            ("validate_envelope", "storage"),
            ("store_content", "storage"),
            ("dag_session", "dag"),
            ("ledger_entry", "ledger"),
            ("attribution_braid", "attribution"),
            ("sign_receipt", "security"),
        ];

        for (name, domain) in expected_domains {
            let node = nodes
                .iter()
                .find(|n| n["name"].as_str() == Some(name))
                .unwrap_or_else(|| panic!("missing node: {name}"));
            assert_eq!(
                node["by_capability"].as_str(),
                Some(domain),
                "{name} should route by {domain} domain"
            );
        }
    }

    #[test]
    fn required_nodes_mark_critical_path() {
        let parsed = load_graph();
        let nodes = parsed["graph"]["nodes"].as_array().expect("nodes array");

        let required: Vec<&str> = nodes
            .iter()
            .filter(|n| n["required"].as_bool() == Some(true))
            .filter_map(|n| n["name"].as_str())
            .collect();

        assert!(
            required.contains(&"validate_envelope"),
            "envelope validation is critical"
        );
        assert!(
            required.contains(&"store_content"),
            "content storage is critical"
        );
        assert!(
            required.contains(&"sign_receipt"),
            "cryptographic receipt is critical"
        );

        let optional: Vec<&str> = nodes
            .iter()
            .filter(|n| n["required"].as_bool() == Some(false))
            .filter_map(|n| n["name"].as_str())
            .collect();

        assert!(
            optional.contains(&"dag_session"),
            "DAG is optional for degraded operation"
        );
        assert!(
            optional.contains(&"ledger_entry"),
            "ledger is optional for degraded operation"
        );
        assert!(
            optional.contains(&"attribution_braid"),
            "braid is optional for degraded operation"
        );
    }
}

mod ingest_dataset {
    use super::*;

    fn load_graph() -> toml::Value {
        let path = graphs_dir().join("signals/nest_ingest_dataset.toml");
        let content = std::fs::read_to_string(&path).expect("read nest_ingest_dataset.toml");
        toml::from_str(&content).expect("parse nest_ingest_dataset.toml")
    }

    #[test]
    fn graph_is_sequential_nest_tier() {
        let parsed = load_graph();
        let graph = parsed.get("graph").expect("missing [graph]");
        assert_eq!(graph["signal_tier"].as_str(), Some("nest"));
        assert_eq!(graph["signal_name"].as_str(), Some("ingest_dataset"));
        assert_eq!(
            graph["coordination"].as_str(),
            Some("sequential"),
            "dataset ingestion is a pipeline"
        );
    }

    #[test]
    fn graph_requires_btsp_security() {
        let parsed = load_graph();
        let meta = &parsed["graph"]["metadata"];
        assert_eq!(
            meta["security_model"].as_str(),
            Some("btsp_enforced"),
            "dataset ingestion crosses composition boundaries"
        );
        assert_eq!(meta["secure_by_default"].as_bool(), Some(true));
    }

    #[test]
    fn pipeline_has_five_nodes_in_order() {
        let parsed = load_graph();
        let nodes = parsed["graph"]["nodes"].as_array().expect("nodes array");
        assert_eq!(nodes.len(), 5, "ingest_dataset: 5-node pipeline");

        let expected_sequence = [
            ("create_session", "rhizocrypt", 1),
            ("store_content", "nestgate", 2),
            ("record_event", "rhizocrypt", 3),
            ("dehydrate", "rhizocrypt", 4),
            ("record_provenance", "sweetgrass", 5),
        ];

        for (i, (name, binary, order)) in expected_sequence.iter().enumerate() {
            let node = &nodes[i];
            assert_eq!(node["name"].as_str(), Some(*name), "node {i} name");
            assert_eq!(node["binary"].as_str(), Some(*binary), "node {i} binary");
            assert_eq!(node["order"].as_integer(), Some(*order), "node {i} order");
        }
    }

    #[test]
    fn dependency_chain_is_linear() {
        let parsed = load_graph();
        let nodes = parsed["graph"]["nodes"].as_array().expect("nodes array");

        assert!(
            nodes[0].get("depends_on").is_none(),
            "create_session has no dependencies"
        );

        let names: Vec<&str> = nodes.iter().map(|n| n["name"].as_str().unwrap()).collect();
        for i in 1..nodes.len() {
            let deps = nodes[i]["depends_on"].as_array().expect("depends_on");
            assert!(
                deps.iter().any(|d| d.as_str() == Some(names[i - 1])),
                "node '{}' must depend on predecessor '{}'",
                names[i],
                names[i - 1]
            );
        }
    }

    #[test]
    fn all_nodes_required_for_dataset_integrity() {
        let parsed = load_graph();
        let nodes = parsed["graph"]["nodes"].as_array().expect("nodes array");

        for node in nodes {
            let name = node["name"].as_str().unwrap_or("?");
            assert_eq!(
                node["required"].as_bool(),
                Some(true),
                "node '{name}' must be required for dataset integrity guarantees"
            );
        }
    }

    #[test]
    fn provenance_trio_coverage() {
        let parsed = load_graph();
        let nodes = parsed["graph"]["nodes"].as_array().expect("nodes array");
        let binaries: Vec<&str> = nodes.iter().filter_map(|n| n["binary"].as_str()).collect();

        assert!(
            binaries.contains(&"rhizocrypt"),
            "rhizoCrypt provides DAG session + dehydration"
        );
        assert!(
            binaries.contains(&"nestgate"),
            "nestGate provides CAS content storage"
        );
        assert!(
            binaries.contains(&"sweetgrass"),
            "sweetGrass records provenance/attribution"
        );

        let rhizo_count = binaries.iter().filter(|b| **b == "rhizocrypt").count();
        assert_eq!(
            rhizo_count, 3,
            "rhizoCrypt handles 3 phases: session, event, dehydrate"
        );
    }

    #[test]
    fn capability_domains_are_correct() {
        let parsed = load_graph();
        let nodes = parsed["graph"]["nodes"].as_array().expect("nodes array");

        let expected_domains = [
            ("create_session", "dag"),
            ("store_content", "storage"),
            ("record_event", "dag"),
            ("dehydrate", "dag"),
            ("record_provenance", "attribution"),
        ];

        for (name, domain) in expected_domains {
            let node = nodes
                .iter()
                .find(|n| n["name"].as_str() == Some(name))
                .unwrap_or_else(|| panic!("missing node: {name}"));
            assert_eq!(
                node["by_capability"].as_str(),
                Some(domain),
                "{name} should route by {domain}"
            );
        }
    }

    #[test]
    fn no_spawn_connects_to_running_primals() {
        let parsed = load_graph();
        let nodes = parsed["graph"]["nodes"].as_array().expect("nodes array");
        for node in nodes {
            let name = node["name"].as_str().unwrap_or("?");
            assert_eq!(
                node["spawn"].as_bool(),
                Some(false),
                "'{name}' connects to running primals, no spawn"
            );
        }
    }
}
