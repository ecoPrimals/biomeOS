// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(
    clippy::float_cmp,
    reason = "test comparisons against exact known constants"
)]

use super::*;
use redb::Database;
use std::sync::Arc;

#[test]
fn new_weight_has_neutral_defaults() {
    let w = ProviderWeight::new("beardog", "crypto");
    assert_eq!(w.provider.as_ref(), "beardog");
    assert_eq!(w.affinity, 0.5);
    assert!(!w.circuit_open);
    assert!(w.is_available());
}

#[test]
fn record_success_updates_ewma() {
    let mut w = ProviderWeight::new("beardog", "crypto");
    w.record_success(10);
    assert!(w.ewma_latency_ms < 50.0); // moved toward 10
    assert_eq!(w.success_count, 1);
    assert_eq!(w.consecutive_failures, 0);
}

#[test]
fn record_failure_increments_consecutive() {
    let mut w = ProviderWeight::new("beardog", "crypto");
    for _ in 0..4 {
        w.record_failure();
    }
    assert!(!w.circuit_open);
    w.record_failure(); // 5th = threshold
    assert!(w.circuit_open);
}

#[test]
fn circuit_breaker_blocks_routing() {
    let mut w = ProviderWeight::new("beardog", "crypto");
    for _ in 0..5 {
        w.record_failure();
    }
    assert_eq!(w.score(), 0.0);
}

#[test]
fn success_resets_circuit_breaker() {
    let mut w = ProviderWeight::new("beardog", "crypto");
    for _ in 0..5 {
        w.record_failure();
    }
    assert!(w.circuit_open);
    w.record_success(10);
    assert!(!w.circuit_open);
    assert!(w.score() > 0.0);
}

#[test]
fn lower_latency_scores_higher() {
    let mut fast = ProviderWeight::new("fast", "crypto");
    let mut slow = ProviderWeight::new("slow", "crypto");
    for _ in 0..10 {
        fast.record_success(5);
        slow.record_success(500);
    }
    assert!(fast.score() > slow.score());
}

#[test]
fn higher_affinity_scores_higher() {
    let mut preferred = ProviderWeight::new("preferred", "crypto");
    let mut fallback = ProviderWeight::new("fallback", "crypto");
    preferred.affinity = 0.9;
    fallback.affinity = 0.1;
    for _ in 0..10 {
        preferred.record_success(50);
        fallback.record_success(50);
    }
    assert!(preferred.score() > fallback.score());
}

#[test]
fn table_select_best_prefers_fast_provider() {
    let mut table = RoutingWeightTable::new();
    for _ in 0..10 {
        table.record_outcome("crypto", "fast_beardog", true, 5);
        table.record_outcome("crypto", "slow_beardog", true, 500);
    }
    let candidates = vec![Arc::from("fast_beardog"), Arc::from("slow_beardog")];
    let best = table.select_best("crypto", &candidates);
    assert_eq!(best.map(|b| b.as_ref()), Some("fast_beardog"));
}

#[test]
fn table_select_best_skips_broken_circuit() {
    let mut table = RoutingWeightTable::new();
    for _ in 0..10 {
        table.record_outcome("crypto", "healthy", true, 50);
    }
    for _ in 0..5 {
        table.record_outcome("crypto", "broken", false, 0);
    }
    let candidates = vec![Arc::from("healthy"), Arc::from("broken")];
    let best = table.select_best("crypto", &candidates);
    assert_eq!(best.map(|b| b.as_ref()), Some("healthy"));
}

#[test]
fn table_exploration_bonus_for_unknown_providers() {
    let table = RoutingWeightTable::new();
    let candidates = vec![Arc::from("unknown_provider")];
    let best = table.select_best("crypto", &candidates);
    assert!(best.is_some(), "unknown providers get exploration bonus");
}

#[test]
fn table_set_affinity() {
    let mut table = RoutingWeightTable::new();
    table.set_affinity("crypto", "beardog", 0.9);
    let w = table.get("crypto", "beardog").unwrap();
    assert_eq!(w.affinity, 0.9);
}

#[test]
fn table_set_cost_hint() {
    let mut table = RoutingWeightTable::new();
    table.set_cost_hint("compute", "toadstool", 100.0);
    let w = table.get("compute", "toadstool").unwrap();
    assert_eq!(w.cost_hint, Some(100.0));
}

#[test]
fn table_summary_correct() {
    let mut table = RoutingWeightTable::new();
    table.record_outcome("crypto", "beardog", true, 10);
    table.record_outcome("storage", "nestgate", true, 20);
    table.record_outcome("crypto", "beardog", true, 15);
    let summary = table.summary();
    assert_eq!(summary.entries, 2);
    assert_eq!(summary.total_dispatches, 3);
    assert_eq!(summary.unique_providers, 2);
    assert_eq!(summary.unique_capabilities, 2);
}

#[test]
fn table_snapshot_includes_all() {
    let mut table = RoutingWeightTable::new();
    table.record_outcome("crypto", "beardog", true, 10);
    table.record_outcome("storage", "nestgate", true, 20);
    let snap = table.snapshot();
    assert_eq!(snap.len(), 2);
}

#[test]
fn persistent_table_survives_reload() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("routing_weights.redb");

    {
        let mut table = RoutingWeightTable::open(&path);
        assert!(table.is_persistent());
        table.record_outcome("crypto", "beardog", true, 10);
        table.record_outcome("crypto", "beardog", true, 15);
        table.set_affinity("crypto", "beardog", 0.8);
        table.set_cost_hint("crypto", "beardog", 42.0);
    }

    {
        let table = RoutingWeightTable::open(&path);
        let w = table.get("crypto", "beardog").unwrap();
        assert_eq!(w.success_count, 2);
        assert_eq!(w.affinity, 0.8);
        assert_eq!(w.cost_hint, Some(42.0));
    }
}

#[test]
fn persistent_table_flush_writes_all() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("routing_weights_flush.redb");

    {
        let mut table = RoutingWeightTable::new();
        table.record_outcome("storage", "nestgate", true, 20);
        table.record_outcome("compute", "toadstool", false, 0);

        let db = Arc::new(Database::create(&path).unwrap());
        table = table.with_db(db);
        assert!(table.is_persistent());
    }

    {
        let table = RoutingWeightTable::open(&path);
        assert_eq!(table.len(), 2);
        assert!(table.get("storage", "nestgate").is_some());
        assert!(table.get("compute", "toadstool").is_some());
    }
}

#[test]
fn in_memory_table_not_persistent() {
    let table = RoutingWeightTable::new();
    assert!(!table.is_persistent());
}

#[test]
fn utilization_tracker_records_calls() {
    let mut tracker = CapabilityUtilizationTracker::new();
    tracker.record("crypto.hash");
    tracker.record("crypto.hash");
    tracker.record("storage.store");

    assert_eq!(tracker.call_count("crypto.hash"), 2);
    assert_eq!(tracker.call_count("storage.store"), 1);
    assert_eq!(tracker.call_count("nonexistent"), 0);
    assert_eq!(tracker.tracked_methods(), 2);
}

#[test]
fn utilization_hot_methods() {
    let mut tracker = CapabilityUtilizationTracker::new();
    for _ in 0..100 {
        tracker.record("crypto.hash");
    }
    for _ in 0..50 {
        tracker.record("storage.store");
    }
    tracker.record("dag.append");

    let hot = tracker.hot_methods(2);
    assert_eq!(hot.len(), 2);
    assert_eq!(hot[0].method, "crypto.hash");
    assert_eq!(hot[0].call_count, 100);
    assert_eq!(hot[1].method, "storage.store");
}

#[test]
fn utilization_cold_methods() {
    let mut tracker = CapabilityUtilizationTracker::new();
    for _ in 0..100 {
        tracker.record("crypto.hash");
    }
    tracker.record("dag.append");

    let cold = tracker.cold_methods(5);
    assert_eq!(cold.len(), 1);
    assert_eq!(cold[0].method, "dag.append");
}

#[test]
fn utilization_summary() {
    let mut tracker = CapabilityUtilizationTracker::new();
    tracker.record("crypto.hash");
    tracker.record("crypto.hash");
    tracker.record("storage.store");

    let summary = tracker.summary();
    assert_eq!(summary.tracked_methods, 2);
    assert_eq!(summary.total_calls, 3);
    assert_eq!(summary.max_calls_single_method, 2);
    assert_eq!(summary.min_calls_single_method, 1);
}

#[test]
fn utilization_to_json() {
    let mut tracker = CapabilityUtilizationTracker::new();
    tracker.record("crypto.hash");
    let json = tracker.to_json();
    assert_eq!(json["tracked_methods"], 1);
    assert_eq!(json["total_calls"], 1);
    assert!(json["hot_methods"].is_array());
}
