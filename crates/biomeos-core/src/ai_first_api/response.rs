// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! AI-First response builder helpers

use uuid::Uuid;

use super::types::{AIFirstResponse, AIResponseMetadata};

impl<T> AIFirstResponse<T> {
    /// Create a successful AI-first response
    pub fn success(
        request_id: Uuid,
        data: T,
        processing_time_ms: u64,
        confidence_score: f64,
    ) -> Self {
        Self {
            success: true,
            data,
            error: None,
            request_id,
            processing_time_ms,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score,
            suggested_actions: Vec::new(),
        }
    }

    /// Create a failed AI-first response
    pub fn error(
        request_id: Uuid,
        error: biomeos_types::BiomeError,
        processing_time_ms: u64,
        default_data: T,
    ) -> Self {
        Self {
            success: false,
            data: default_data,
            error: Some(error),
            request_id,
            processing_time_ms,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score: 0.0,
            suggested_actions: Vec::new(),
        }
    }
}
