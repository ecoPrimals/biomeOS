// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

mod constructor;
mod get_suggestions;
mod helpers;
mod learn_from_event;
mod send_feedback;
mod send_learning_event;
mod squirrel_availability;
