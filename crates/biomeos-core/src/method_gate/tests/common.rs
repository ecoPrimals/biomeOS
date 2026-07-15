// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

pub(super) fn make_ionic_token(payload: &serde_json::Value) -> String {
    use base64::Engine;
    let header = serde_json::json!({"alg":"EdDSA","typ":"ionic","ver":1});
    let h = base64::engine::general_purpose::STANDARD.encode(header.to_string().as_bytes());
    let p = base64::engine::general_purpose::STANDARD.encode(payload.to_string().as_bytes());
    let s = base64::engine::general_purpose::STANDARD.encode(b"fake-sig");
    format!("{h}.{p}.{s}")
}
