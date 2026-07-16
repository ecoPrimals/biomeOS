// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test")]

//! Unit tests for storage manifest types.

use super::*;
use std::collections::HashMap;

mod config;
mod secret;
mod volume;
