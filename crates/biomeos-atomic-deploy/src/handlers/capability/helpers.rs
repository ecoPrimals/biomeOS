// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

pub(super) fn elapsed_ms_since(start: std::time::Instant) -> u64 {
    let e = start.elapsed();
    e.as_secs() * 1000 + u64::from(e.subsec_millis())
}
