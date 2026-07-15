// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

pub(super) fn roundtrip_json<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) {
    let json = serde_json::to_string(value).unwrap();
    let parsed: T = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&parsed).unwrap();
    let v1: serde_json::Value = serde_json::from_str(&json).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();
    assert_eq!(v1, v2, "roundtrip produced semantically different JSON");
}
