// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::types::{
    DownloadResponse, GenomeInfoResponse, GenomeSummary, VerifyResponse,
};

#[test]
fn test_download_response_serialization() {
    let resp = DownloadResponse {
        url: "/api/v1/genome/x/data".to_string(),
        size: 1024,
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("/api/v1/genome"));
    assert!(json.contains("1024"));
}

#[test]
fn test_genome_info_response_serialization() {
    let resp = GenomeInfoResponse {
        name: "test".to_string(),
        version: "1.0".to_string(),
        architectures: vec!["x86_64".to_string()],
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("test"));
    assert!(json.contains("x86_64"));
}

#[test]
fn test_genome_summary_serialization() {
    let summary = GenomeSummary {
        id: "id-1".to_string(),
        name: "name".to_string(),
        version: "1.0".to_string(),
        architectures: vec!["aarch64".to_string()],
    };
    let json = serde_json::to_string(&summary).expect("serialize");
    assert!(json.contains("id-1"));
    assert!(json.contains("aarch64"));
}

#[test]
fn test_verify_response_serialization() {
    let resp = VerifyResponse {
        valid: true,
        message: "All checksums valid".to_string(),
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("valid"));
    assert!(json.contains("checksums"));
}
