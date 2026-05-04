// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Continuation of nucleus mode tests (split from `nucleus_tests.rs`).

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;
use std::path::PathBuf;

#[test]
fn test_socket_path_for_capability_unknown_returns_unknown_sock() {
    let path = super::socket_path_for_capability(
        std::path::Path::new("/tmp"),
        "fam1",
        "arbitrary-unknown-capability",
    );
    assert!(path.to_string_lossy().contains("unknown"));
    assert!(path.to_string_lossy().ends_with(".sock"));
}

#[test]
fn test_build_primal_command_squirrel_without_ai_keys() {
    let config = super::PrimalCommandConfig {
        name: "squirrel",
        binary: std::path::Path::new("/usr/bin/squirrel"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node1",
        anthropic_api_key: None,
        openai_api_key: None,
        ai_http_providers: None,
        ai_default_model: None,
    };
    let cmd = super::build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai_providers = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(
        ai_providers.is_none(),
        "AI_HTTP_PROVIDERS should not be set when no API keys"
    );
}

#[test]
fn test_resolve_startup_config_family_from_env_when_not_provided() {
    let config = resolve_startup_config_with("tower", "node1", None, Some("/tmp/sock"));
    assert!(config.is_ok());
    let config = config.unwrap();
    assert!(!config.family_id.is_empty());
}

#[test]
fn test_format_nucleus_summary_coordinated_mode() {
    let lines = format_nucleus_summary(
        &[("beardog".to_string(), 999)],
        std::path::Path::new("/run/sock"),
        "fam1",
        "node1",
        NucleusMode::Full,
        "coordinated",
    );
    assert!(lines.iter().any(|l| l.contains("coordinated")));
    assert!(lines.iter().any(|l| l.contains("999")));
}

#[test]
fn test_primal_command_config_debug() {
    let config = super::PrimalCommandConfig {
        name: "beardog",
        binary: std::path::Path::new("/bin/beardog"),
        socket_dir: std::path::Path::new("/tmp"),
        family_id: "f",
        node_id: "n",
        anthropic_api_key: None,
        openai_api_key: None,
        ai_http_providers: None,
        ai_default_model: None,
    };
    let _ = format!("{config:?}");
}

#[test]
fn test_build_primal_command_squirrel_with_custom_ai_providers() {
    let config = super::PrimalCommandConfig {
        name: "squirrel",
        binary: std::path::Path::new("/usr/bin/squirrel"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node1",
        anthropic_api_key: Some("key"),
        openai_api_key: None,
        ai_http_providers: Some("custom,anthropic"),
        ai_default_model: None,
    };
    let cmd = super::build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai_providers = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(ai_providers.is_some());
    let (_, v) = ai_providers.unwrap();
    assert_eq!(v.unwrap().to_string_lossy(), "custom,anthropic");
}

#[test]
fn test_generate_jwt_secret_produces_nonempty_string() {
    let secret = super::generate_jwt_secret();
    assert!(!secret.is_empty());
    assert!(secret.len() > 32, "JWT secret should be substantial");
}

#[test]
fn test_resolve_startup_config_uses_biomeos_socket_dir_env() {
    let config = resolve_startup_config_with(
        "tower",
        "node1",
        Some("fam1"),
        Some("/tmp/nucleus-env-test"),
    )
    .expect("should succeed");
    assert_eq!(config.socket_dir, PathBuf::from("/tmp/nucleus-env-test"));
}

#[test]
fn test_build_primal_command_squirrel_ai_default_model_env() {
    let config = super::PrimalCommandConfig {
        name: "squirrel",
        binary: std::path::Path::new("/usr/bin/squirrel"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node1",
        anthropic_api_key: None,
        openai_api_key: None,
        ai_http_providers: None,
        ai_default_model: Some("custom-model-v1"),
    };
    let cmd = super::build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let model = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_DEFAULT_MODEL"));
    assert!(model.is_some(), "AI_DEFAULT_MODEL should be set from env");
    let (_, v) = model.unwrap();
    assert_eq!(v.unwrap().to_string_lossy(), "custom-model-v1");
}

#[test]
fn test_build_primal_command_squirrel_with_anthropic_key_env() {
    let config = super::PrimalCommandConfig {
        name: "squirrel",
        binary: std::path::Path::new("/usr/bin/squirrel"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node1",
        anthropic_api_key: Some("sk-ant-test"),
        openai_api_key: None,
        ai_http_providers: None,
        ai_default_model: None,
    };
    let cmd = super::build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai_providers = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(
        ai_providers.is_some(),
        "AI_HTTP_PROVIDERS should be set when ANTHROPIC_API_KEY present"
    );
}

#[test]
fn test_build_primal_command_squirrel_with_openai_key_env() {
    let config = super::PrimalCommandConfig {
        name: "squirrel",
        binary: std::path::Path::new("/usr/bin/squirrel"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node1",
        anthropic_api_key: None,
        openai_api_key: Some("sk-openai-test"),
        ai_http_providers: None,
        ai_default_model: None,
    };
    let cmd = super::build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai_providers = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(ai_providers.is_some());
}

#[test]
fn test_discover_binaries_empty_path_env() {
    let map =
        discover_binaries_with(&["beardog"], None, &[], None).expect("discover should not panic");
    // May or may not find beardog depending on relative paths (plasmidBin, target/release)
    // Just verify it doesn't panic with empty PATH
    let _ = map;
}

#[test]
fn test_build_primal_command_with_beardog_server_socket() {
    let cmd = build_primal_command(
        "beardog",
        std::path::Path::new("/usr/bin/beardog"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let args: Vec<_> = cmd.get_args().map(|a| a.to_str().unwrap()).collect();
    assert!(
        args.windows(2).any(|w| w == ["server", "--socket"]),
        "beardog should use default server socket args, got {args:?}"
    );
}

#[tokio::test]
async fn test_detect_ecosystem_coordinated_when_socket_responds() {
    use std::sync::Arc;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;
    use tokio::sync::Notify;

    let tmp = tempfile::tempdir().expect("tempdir");
    let family = "coord-test-family";
    let sock_name = format!("beardog-{family}.sock");
    let sock_path = tmp.path().join(&sock_name);

    let ready = Arc::new(Notify::new());
    let ready_c = Arc::clone(&ready);
    let sock_path_c = sock_path.clone();
    let server = tokio::spawn(async move {
        let listener = UnixListener::bind(&sock_path_c).expect("bind");
        ready_c.notify_one();
        let (stream, _) = listener.accept().await.expect("accept");
        let (mut r, mut w) = stream.into_split();
        let mut line = String::new();
        BufReader::new(&mut r)
            .read_line(&mut line)
            .await
            .expect("read");
        let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse rpc");
        let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {"status": "ok"}
        });
        w.write_all(format!("{resp}\n").as_bytes())
            .await
            .expect("write");
    });

    ready.notified().await;
    let state = detect_ecosystem(tmp.path(), family).await;
    server.await.expect("server");

    match state {
        EcosystemState::Coordinated { active_primals } => {
            assert!(
                active_primals.iter().any(|p| p == "beardog"),
                "expected beardog active, got {active_primals:?}"
            );
        }
        EcosystemState::Bootstrap => {
            panic!("expected Coordinated when health RPC succeeds, got Bootstrap");
        }
    }
}

#[test]
fn test_nucleus_mode_debug() {
    let _ = format!("{:?}", NucleusMode::Full);
}

#[test]
fn test_resolve_startup_config_with_explicit_family() {
    let c =
        resolve_startup_config_with("nest", "n1", Some("myfam"), Some("/tmp/sock-nest")).unwrap();
    assert_eq!(c.family_id, "myfam");
    assert!(matches!(c.mode, NucleusMode::Nest));
}

#[test]
fn test_socket_path_for_capability_registry_alias() {
    // "registry" is a taxonomy alias for Discovery → songbird
    let p = super::socket_path_for_capability(std::path::Path::new("/run"), "fam", "registry");
    assert!(p.to_string_lossy().contains("songbird"));
}

#[test]
fn test_format_nucleus_summary_full_mode_label() {
    let lines = format_nucleus_summary(
        &[],
        std::path::Path::new("/x"),
        "f",
        "n",
        NucleusMode::Full,
        "bootstrap",
    );
    assert!(lines.iter().any(|l| l.contains("Full")));
}

#[tokio::test]
async fn test_wait_for_socket_immediate_with_zero_poll() {
    let tmp = tempfile::tempdir().expect("temp");
    let p = tmp.path().join("s.sock");
    std::fs::write(&p, b"").expect("touch");
    let r = wait_for_socket(&p, Duration::from_secs(1), Duration::ZERO).await;
    assert!(r.is_ok());
}

#[test]
fn test_discover_binaries_with_missing_primal_returns_partial_ok() {
    let map = discover_binaries_with(&["missing_primal_abc"], None, &[], None).expect("ok");
    assert!(map.is_empty());
}

#[tokio::test]
async fn test_discover_binaries_with_livespore_usb_arch_primals() {
    let temp = tempfile::tempdir().expect("tempdir");
    let arch = std::env::consts::ARCH;
    let primal_dir = temp.path().join("livespore-usb").join(arch).join("primals");
    std::fs::create_dir_all(&primal_dir).expect("mkdir");
    let name = "biomeos_unique_primal_livespore_usb_xyz";
    let binary_path = primal_dir.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let map = discover_binaries_with(&[name], None, &[], Some(temp.path())).expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} under livespore-usb/{arch}/primals, got {map:?}"
    );
}

#[test]
fn test_nucleus_mode_from_str_nucleus_alias() {
    let m: NucleusMode = "nucleus".parse().expect("parse");
    assert!(matches!(m, NucleusMode::Full));
}

#[test]
fn test_socket_path_for_capability_encryption_alias() {
    let p = super::socket_path_for_capability(std::path::Path::new("/s"), "fam", "encryption");
    assert!(p.to_string_lossy().contains("beardog"));
}

#[test]
fn test_resolve_startup_config_with_explicit_family_override() {
    let c = resolve_startup_config_with("full", "n1", Some("explicit-fam"), Some("/tmp/sock-full"))
        .unwrap();
    assert_eq!(c.family_id, "explicit-fam");
    assert!(matches!(c.mode, NucleusMode::Full));
}

#[test]
fn test_format_nucleus_summary_includes_socket_paths_for_all_children() {
    let lines = format_nucleus_summary(
        &[("beardog".to_string(), 10), ("songbird".to_string(), 11)],
        std::path::Path::new("/run/s"),
        "fam",
        "node",
        NucleusMode::Nest,
        "bootstrap",
    );
    assert!(lines.iter().any(|l| l.contains("beardog")));
    assert!(lines.iter().any(|l| l.contains("songbird")));
}

#[tokio::test]
async fn test_health_check_semantic_fallback_when_plain_health_fails() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;
    use tokio::sync::Notify;

    let tmp = tempfile::tempdir().expect("tempdir");
    let family = "hfam-semantic";
    let sock_name = format!("beardog-{family}.sock");
    let sock_path = tmp.path().join(&sock_name);

    let ready = std::sync::Arc::new(Notify::new());
    let ready_c = std::sync::Arc::clone(&ready);
    let sock_path_c = sock_path.clone();
    let server = tokio::spawn(async move {
        let listener = UnixListener::bind(&sock_path_c).expect("bind");
        ready_c.notify_one();
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut r, mut w) = stream.into_split();
            let mut line = String::new();
            BufReader::new(&mut r)
                .read_line(&mut line)
                .await
                .expect("read");
            let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse rpc");
            let method = req["method"].as_str().expect("method string");
            let id = req.get("id").cloned();
            let resp = if method == "health" {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": "Method not found" }
                })
            } else if method == "health.status" {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "status": "ok" }
                })
            } else {
                panic!("unexpected method {method}");
            };
            w.write_all(format!("{resp}\n").as_bytes())
                .await
                .expect("write");
        }
    });

    ready.notified().await;
    let result = super::health_check(&sock_path).await;
    server.await.expect("server task");
    assert!(
        result.is_ok(),
        "semantic health fallback should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_discover_binaries_with_livespore_usb_primals_flat_layout() {
    let _guard = crate::CWD_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let primal_dir = temp.path().join("livespore-usb").join("primals");
    std::fs::create_dir_all(&primal_dir).expect("mkdir");
    let name = "biomeos_unique_primal_livespore_flat_xyz";
    let binary_path = primal_dir.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    let map = discover_binaries_with(&[name], None, &[], None);
    std::env::set_current_dir(old).unwrap();
    let map = map.expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} under livespore-usb/primals, got {map:?}"
    );
}

#[tokio::test]
async fn test_discover_binaries_with_plasmidbin_optimized_arch() {
    let _guard = crate::CWD_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let arch = std::env::consts::ARCH;
    let primal_dir = temp.path().join("plasmidBin").join("optimized").join(arch);
    std::fs::create_dir_all(&primal_dir).expect("mkdir");
    let name = "biomeos_unique_primal_plasmid_opt_xyz";
    let binary_path = primal_dir.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    let map = discover_binaries_with(&[name], None, &[], None);
    std::env::set_current_dir(old).unwrap();
    let map = map.expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} under plasmidBin/optimized/{arch}, got {map:?}"
    );
}

#[tokio::test]
async fn test_discover_binaries_with_target_release_relative() {
    let _guard = crate::CWD_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let release_dir = temp.path().join("target").join("release");
    std::fs::create_dir_all(&release_dir).expect("mkdir");
    let name = "biomeos_unique_primal_target_release_xyz";
    let binary_path = release_dir.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    let map = discover_binaries_with(&[name], None, &[], None);
    std::env::set_current_dir(old).unwrap();
    let map = map.expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} under target/release, got {map:?}"
    );
}

#[tokio::test]
async fn test_discover_binaries_with_relative_plasmidbin_primals() {
    let _guard = crate::CWD_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let nested = temp.path().join("work").join("a").join("b");
    std::fs::create_dir_all(&nested).expect("mkdir nested");
    // From cwd `work/a/b`, `../../plasmidBin/primals` resolves to `work/plasmidBin/primals`.
    let plasmid_primals = temp.path().join("work").join("plasmidBin").join("primals");
    std::fs::create_dir_all(&plasmid_primals).expect("mkdir plasmidBin/primals");
    let name = "biomeos_unique_primal_rel_plasmid_xyz";
    let binary_path = plasmid_primals.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(&nested).unwrap();
    let map = discover_binaries_with(&[name], None, &[], None);
    std::env::set_current_dir(old).unwrap();
    let map = map.expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} via ../../plasmidBin/primals, got {map:?}"
    );
}

#[tokio::test]
async fn test_discover_binaries_with_plasmidbin_direct_binary() {
    let _guard = crate::CWD_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let plasmid = temp.path().join("plasmidBin");
    std::fs::create_dir_all(&plasmid).expect("mkdir plasmidBin");
    let name = "biomeos_unique_primal_plasmidbin_root_xyz";
    let binary_path = plasmid.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    let map = discover_binaries_with(&[name], None, &[], None);
    std::env::set_current_dir(old).unwrap();
    let map = map.expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} under plasmidBin/, got {map:?}"
    );
}

#[test]
fn test_build_primal_command_squirrel_custom_ai_http_providers_when_both_keys_set() {
    let config = super::PrimalCommandConfig {
        name: "squirrel",
        binary: std::path::Path::new("/usr/bin/squirrel"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node1",
        anthropic_api_key: Some("sk-ant"),
        openai_api_key: Some("sk-openai"),
        ai_http_providers: Some("custom_a,custom_b"),
        ai_default_model: None,
    };
    let cmd = super::build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(ai.is_some());
    assert_eq!(
        ai.unwrap().1.unwrap().to_string_lossy(),
        "custom_a,custom_b"
    );
}

#[test]
fn test_build_primal_command_squirrel_with_ai_default_model() {
    let config = super::PrimalCommandConfig {
        name: "squirrel",
        binary: std::path::Path::new("/usr/bin/squirrel"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node1",
        anthropic_api_key: None,
        openai_api_key: None,
        ai_http_providers: None,
        ai_default_model: Some("claude-3-sonnet"),
    };
    let cmd = super::build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let model = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_DEFAULT_MODEL"));
    assert!(model.is_some(), "AI_DEFAULT_MODEL should be set");
    assert_eq!(
        model.unwrap().1.unwrap().to_string_lossy(),
        "claude-3-sonnet"
    );
}

#[test]
fn test_discover_search_path_with_cwd() {
    let cwd = std::path::Path::new("/home/user/project");
    let rel = PathBuf::from("plasmidBin");
    let result = super::discover_search_path(rel, Some(cwd));
    assert_eq!(result, PathBuf::from("/home/user/project/plasmidBin"));
}

#[test]
fn test_discover_search_path_without_cwd() {
    let rel = PathBuf::from("plasmidBin");
    let result = super::discover_search_path(rel, None);
    assert_eq!(result, PathBuf::from("plasmidBin"));
}

#[test]
fn test_discover_binaries_with_cwd_finds_in_target_release() {
    let temp = tempfile::tempdir().expect("tempdir");
    let release_dir = temp.path().join("target/release");
    std::fs::create_dir_all(&release_dir).expect("create target/release");
    let name = "biomeos_cwd_test_binary";
    let binary = release_dir.join(name);
    std::fs::write(&binary, b"#!/bin/sh\nexit 0").expect("write");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary, perms).unwrap();
    }
    let map =
        super::discover_binaries_with(&[name], None, &[], Some(temp.path())).expect("discover");
    assert!(
        map.contains_key(name),
        "should find {name} in target/release with cwd, got {map:?}"
    );
}

#[test]
fn test_build_primal_command_with_all_env_keys_set() {
    let config = super::PrimalCommandConfig {
        name: "beardog",
        binary: std::path::Path::new("/usr/bin/beardog"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node-42",
        anthropic_api_key: None,
        openai_api_key: None,
        ai_http_providers: None,
        ai_default_model: None,
    };
    let cmd = super::build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let family = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("FAMILY_ID"));
    assert!(family.is_some());
    assert_eq!(family.unwrap().1.unwrap().to_string_lossy(), "fam1");
    let node = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("NODE_ID"));
    assert!(node.is_some());
    assert_eq!(node.unwrap().1.unwrap().to_string_lossy(), "node-42");
    let node_id_env = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("NODE_ID"));
    assert!(node_id_env.is_some());
}

#[test]
fn test_nucleus_mode_clone_and_copy() {
    let mode = NucleusMode::Full;
    let cloned = mode;
    assert!(matches!(cloned, NucleusMode::Full));
}
