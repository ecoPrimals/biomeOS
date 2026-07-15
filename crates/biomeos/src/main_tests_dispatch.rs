// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use super::super::*;

#[tokio::test]
async fn test_dispatch_mode_version() {
    let cli = Cli::parse_from(["biomeos", "version"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch version should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_doctor() {
    let cli = Cli::parse_from(["biomeos", "doctor"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch doctor should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_cli() {
    let cli = Cli::parse_from(["biomeos", "cli"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch cli should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_model_cache_list() {
    let cli = Cli::parse_from(["biomeos", "model-cache", "list"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch model-cache list should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_plasmodium_status() {
    let cli = Cli::parse_from(["biomeos", "plasmodium", "status"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch plasmodium status should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_deploy_fails_nonexistent_graph() {
    let cli = Cli::parse_from([
        "biomeos",
        "deploy",
        "/nonexistent/path/to/graph-xyz-12345.json",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dispatch_mode_verify_lineage_fails_nonexistent_path() {
    let cli = Cli::parse_from([
        "biomeos",
        "verify-lineage",
        "/nonexistent/path/to/spore-xyz-12345",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dispatch_mode_version_detailed() {
    let cli = Cli::parse_from(["biomeos", "version", "--detailed"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch version --detailed should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_doctor_json() {
    let cli = Cli::parse_from(["biomeos", "doctor", "-f", "json"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch doctor json should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_doctor_subsystem() {
    let cli = Cli::parse_from(["biomeos", "doctor", "--subsystem", "config"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch doctor subsystem should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_genome_list() {
    let _guard = CWD_TEST_LOCK.lock().await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let plasmidbindir = tmp.path().join("plasmidBin");
    std::fs::create_dir_all(&plasmidbindir).expect("create plasmidBin dir");
    let cli = Cli::parse_from(["biomeos", "genome", "list", plasmidbindir.to_str().unwrap()]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch genome list empty dir should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_deploy_validate_only_nonexistent() {
    let cli = Cli::parse_from([
        "biomeos",
        "deploy",
        "/nonexistent/graph-test-validate-12345.toml",
        "--validate-only",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(result.is_err(), "validate-only on nonexistent should fail");
}

#[tokio::test]
async fn test_dispatch_mode_deploy_dry_run_nonexistent() {
    let cli = Cli::parse_from([
        "biomeos",
        "deploy",
        "/nonexistent/graph-test-dryrun-12345.toml",
        "--dry-run",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(result.is_err(), "dry-run on nonexistent should fail");
}

#[tokio::test]
async fn test_dispatch_mode_model_cache_status() {
    let cli = Cli::parse_from(["biomeos", "model-cache", "status"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch model-cache status should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_model_cache_import_hf() {
    let cli = Cli::parse_from(["biomeos", "model-cache", "import-hf"]);
    let _result = dispatch_mode(cli).await;
}

#[tokio::test]
async fn test_dispatch_mode_model_cache_resolve() {
    let cli = Cli::parse_from([
        "biomeos",
        "model-cache",
        "resolve",
        "nonexistent/model-12345",
    ]);
    let _result = dispatch_mode(cli).await;
}

#[tokio::test]
async fn test_dispatch_mode_plasmodium_gates() {
    let cli = Cli::parse_from(["biomeos", "plasmodium", "gates"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch plasmodium gates should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_plasmodium_models() {
    let cli = Cli::parse_from(["biomeos", "plasmodium", "models"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch plasmodium models should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_verify_lineage_detailed_fails() {
    let cli = Cli::parse_from([
        "biomeos",
        "verify-lineage",
        "/nonexistent/detailed-test-12345",
        "--detailed",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dispatch_mode_continuous_nonexistent() {
    let cli = Cli::parse_from([
        "biomeos",
        "continuous",
        "/nonexistent/continuous-test-12345.toml",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(
        result.is_err(),
        "continuous on nonexistent graph should fail"
    );
}

#[tokio::test]
async fn test_dispatch_mode_rootpulse_status() {
    let cli = Cli::parse_from(["biomeos", "rootpulse", "status"]);
    let _result = dispatch_mode(cli).await;
}

#[tokio::test]
async fn test_dispatch_mode_enroll_fails_without_seed() {
    let cli = Cli::parse_from([
        "biomeos",
        "enroll",
        "--family-id",
        "test-family-12345",
        "--node-id",
        "test-node-12345",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(
        result.is_err(),
        "enroll without valid family seed file should fail"
    );
}
