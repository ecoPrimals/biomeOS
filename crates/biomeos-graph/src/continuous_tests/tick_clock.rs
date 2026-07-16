// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::time::Duration;

use crate::continuous::*;
use crate::graph::TickConfig;

#[test]
fn test_tick_clock_basic() {
    let clock = TickClock::new(60.0);
    assert_eq!(clock.tick_count(), 0);
    assert!((clock.target_hz() - 60.0).abs() < 0.01);

    let tick_dur = clock.tick_duration();
    assert!((tick_dur.as_secs_f64() - 1.0 / 60.0).abs() < 0.0001);
}

#[test]
fn test_tick_clock_advance_zero_elapsed() {
    let mut clock = TickClock::new(60.0);
    let ticks = clock.advance();
    assert_eq!(ticks, 0);
}

#[tokio::test(start_paused = true)]
async fn test_tick_clock_advance_after_sleep() {
    let mut clock = TickClock::new(10.0); // 10 Hz = 100ms per tick
    tokio::time::advance(Duration::from_millis(10)).await;
    let _ticks = clock.advance();
    assert_eq!(clock.tick_count(), 0, "10ms is not enough for a 100ms tick");

    tokio::time::advance(Duration::from_millis(250)).await;
    let ticks = clock.advance();
    assert!(
        ticks >= 1,
        "250ms should produce at least one 100ms tick, got {ticks}"
    );
    assert!(clock.tick_count() >= 1);
}

#[tokio::test(start_paused = true)]
async fn test_tick_clock_max_accumulator_clamp() {
    let config = TickConfig {
        target_hz: 10.0,
        max_accumulator_ms: 200.0,
        budget_warning_ms: 4.0,
    };
    let mut clock = TickClock::from_config(&config);
    tokio::time::advance(Duration::from_millis(500)).await;
    let ticks = clock.advance();
    assert!(
        ticks <= 2,
        "Should clamp to max_accumulator worth of ticks, got {ticks}"
    );
}

/// When real time far exceeds `max_accumulator`, `skipped > 1.0` and the clock logs a clamp warning.
#[tokio::test(start_paused = true)]
async fn test_tick_clock_clamp_logs_when_skipping_multiple_ticks() {
    let config = TickConfig {
        target_hz: 10.0,
        max_accumulator_ms: 200.0,
        budget_warning_ms: 4.0,
    };
    let mut clock = TickClock::from_config(&config);
    tokio::time::advance(Duration::from_millis(800)).await;
    let ticks = clock.advance();
    assert!(
        (1..=2).contains(&ticks),
        "clamped accumulator should yield 1–2 ticks at 10 Hz, got {ticks}"
    );
}

#[tokio::test(start_paused = true)]
async fn test_tick_clock_reset_accumulator() {
    let mut clock = TickClock::new(60.0);
    tokio::time::advance(Duration::from_millis(50)).await;
    clock.reset_accumulator();
    let ticks = clock.advance();
    assert_eq!(ticks, 0);
}

#[tokio::test(start_paused = true)]
async fn test_tick_clock_yields_multiple_ticks_one_advance() {
    let config = TickConfig {
        target_hz: 10.0,
        max_accumulator_ms: 500.0,
        budget_warning_ms: 4.0,
    };
    let mut clock = TickClock::from_config(&config);
    tokio::time::advance(Duration::from_millis(350)).await;
    let ticks = clock.advance();
    assert!(
        ticks >= 2,
        "350ms at 10 Hz (100ms/tick) should yield multiple ticks, got {ticks}"
    );
    assert!(clock.tick_count() >= 2);
}

#[tokio::test(start_paused = true)]
async fn test_tick_clock_target_hz_matches_tick_duration() {
    let c = TickClock::new(50.0);
    assert!((c.target_hz() - 50.0).abs() < 0.001);
    let dur = c.tick_duration();
    assert!((dur.as_secs_f64() - 0.02).abs() < 1e-6);
}
