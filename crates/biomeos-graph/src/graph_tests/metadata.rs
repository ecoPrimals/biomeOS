// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;

use super::super::graph::*;

#[test]
fn test_graph_category_default() {
    let cat = GraphCategory::default();
    assert_eq!(cat, GraphCategory::Utility);
}

#[test]
fn test_graph_category_serde() {
    let cat = GraphCategory::Deployment;
    let json = serde_json::to_string(&cat).unwrap();
    assert_eq!(json, "\"deployment\"");

    let deserialized: GraphCategory = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, GraphCategory::Deployment);
}

#[test]
fn test_all_graph_categories() {
    let categories = vec![
        (GraphCategory::Deployment, "\"deployment\""),
        (GraphCategory::Validation, "\"validation\""),
        (GraphCategory::Testing, "\"testing\""),
        (GraphCategory::Utility, "\"utility\""),
        (GraphCategory::Lifecycle, "\"lifecycle\""),
    ];
    for (cat, expected_json) in categories {
        let json = serde_json::to_string(&cat).unwrap();
        assert_eq!(json, expected_json);
    }
}

#[test]
fn test_graph_metadata_default() {
    let meta = GraphMetadata::default();
    assert!(meta.family_id.is_none());
    assert!(meta.author.is_none());
    assert!(meta.created.is_none());
    assert!(meta.category.is_none());
    assert!(meta.composition.is_none());
    assert!(meta.genetics_tier.is_none());
    assert!(meta.content_hash.is_none());
    assert!(meta.signature.is_none());
    assert!(meta.signed_by.is_none());
    assert!(meta.extra.is_empty());
}

#[test]
fn test_graph_metadata_serde() {
    let meta = GraphMetadata {
        family_id: Some("family-123".to_string()),
        author: Some("biomeOS".to_string()),
        created: Some("2026-01-01".to_string()),
        category: Some(GraphCategory::Deployment),
        composition: None,
        composition_model: None,
        genetics_tier: Some(GeneticsTier::MitoBeacon),
        content_hash: None,
        signature: None,
        signed_by: None,
        extra: HashMap::new(),
    };
    let json = serde_json::to_string(&meta).unwrap();
    assert!(json.contains("family-123"));
    assert!(json.contains("biomeOS"));
    assert!(json.contains("mito_beacon"));
}

#[test]
fn test_genetics_tier_json_roundtrip() {
    for tier in [
        GeneticsTier::None,
        GeneticsTier::Tag,
        GeneticsTier::MitoBeacon,
        GeneticsTier::Nuclear,
    ] {
        let json = serde_json::to_string(&tier).unwrap();
        let back: GeneticsTier = serde_json::from_str(&json).unwrap();
        assert_eq!(back, tier);
    }
}

#[test]
fn test_genetics_tier_ordering() {
    assert!(GeneticsTier::None < GeneticsTier::Tag);
    assert!(GeneticsTier::Tag < GeneticsTier::MitoBeacon);
    assert!(GeneticsTier::MitoBeacon < GeneticsTier::Nuclear);
}

#[test]
fn test_genetics_tier_from_str() {
    assert_eq!("none".parse::<GeneticsTier>().unwrap(), GeneticsTier::None);
    assert_eq!(
        "mito_beacon".parse::<GeneticsTier>().unwrap(),
        GeneticsTier::MitoBeacon
    );
    assert!("bogus".parse::<GeneticsTier>().is_err());
}

#[test]
fn test_graph_metadata_genetics_tier_from_toml() {
    let toml_str = r#"
        [graph]
        id = "g"
        name = "G"
        version = "1.0.0"

        [graph.metadata]
        genetics_tier = "mito_beacon"
    "#;
    let g: DeploymentGraph = toml::from_str(toml_str).unwrap();
    assert_eq!(
        g.definition.metadata.genetics_tier,
        Some(GeneticsTier::MitoBeacon)
    );
}
