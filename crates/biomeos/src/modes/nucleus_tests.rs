// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use biomeos_test_utils::TestEnvGuard;
use std::path::PathBuf;

#[test]
fn test_nucleus_mode_from_str_valid() {
    assert!(matches!(
        "tower".parse::<NucleusMode>().expect("tower should parse"),
        NucleusMode::Tower
    ));
    assert!(matches!(
        "Tower"
            .parse::<NucleusMode>()
            .expect("Tower should parse (case insensitive)"),
        NucleusMode::Tower
    ));
    assert!(matches!(
        "node".parse::<NucleusMode>().expect("node should parse"),
        NucleusMode::Node
    ));
    assert!(matches!(
        "nest".parse::<NucleusMode>().expect("nest should parse"),
        NucleusMode::Nest
    ));
    assert!(matches!(
        "full".parse::<NucleusMode>().expect("full should parse"),
        NucleusMode::Full
    ));
    assert!(matches!(
        "nucleus"
            .parse::<NucleusMode>()
            .expect("nucleus should parse"),
        NucleusMode::Full
    ));
}

#[test]
fn test_nucleus_mode_from_str_invalid() {
    let err = "invalid".parse::<NucleusMode>().unwrap_err();
    assert!(err.to_string().contains("Unknown nucleus mode"));
    assert!(err.to_string().contains("invalid"));
    assert!(err.to_string().contains("tower|node|nest|full"));

    let err2 = "".parse::<NucleusMode>().unwrap_err();
    assert!(err2.to_string().contains("Unknown nucleus mode"));
}

#[test]
fn test_nucleus_mode_primals() {
    assert_eq!(
        NucleusMode::Tower.primals(),
        vec!["beardog", "songbird"],
        "Tower mode primals"
    );
    assert_eq!(
        NucleusMode::Node.primals(),
        vec!["beardog", "songbird", "toadstool"],
        "Node mode primals"
    );
    assert_eq!(
        NucleusMode::Nest.primals(),
        vec!["beardog", "songbird", "nestgate", "squirrel"],
        "Nest mode primals"
    );
    assert_eq!(
        NucleusMode::Full.primals(),
        vec!["beardog", "songbird", "nestgate", "toadstool", "squirrel"],
        "Full mode primals"
    );
}

#[test]
fn test_base64_encode_empty() {
    assert_eq!(base64_encode(&[]), "");
}

#[test]
fn test_base64_encode_single_byte() {
    let result = base64_encode(&[0x4d]);
    assert_eq!(result.len(), 4);
    assert!(result.ends_with("=="));
}

#[test]
fn test_base64_encode_three_bytes() {
    let result = base64_encode(b"Man");
    assert_eq!(result, "TWFu");
}

#[test]
fn test_base64_encode_roundtrip_alphabet() {
    let data = b"Hello, World!";
    let encoded = base64_encode(data);
    assert!(!encoded.is_empty());
    assert!(encoded.len() <= data.len().div_ceil(3) * 4 + 4);
    for c in encoded.chars() {
        assert!(
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=',
            "Invalid base64 char: {c:?}"
        );
    }
}

#[test]
fn test_socket_path_for_capability_uses_taxonomy() {
    let socket_dir = std::path::Path::new("/tmp/sock");
    let family_id = "fam1";
    let path = super::socket_path_for_capability(socket_dir, family_id, "security");
    assert!(path.to_string_lossy().contains("beardog"));
    assert!(path.to_string_lossy().contains("fam1"));
    assert!(path.to_string_lossy().ends_with(".sock"));
    let path2 = super::socket_path_for_capability(socket_dir, family_id, "discovery");
    assert!(path2.to_string_lossy().contains("songbird"));
}

#[test]
fn test_socket_path_for_capability_fallbacks() {
    let socket_dir = std::path::Path::new("/tmp/sock");
    let family_id = "fam1";
    let enc = super::socket_path_for_capability(socket_dir, family_id, "encryption");
    assert!(enc.to_string_lossy().contains("beardog"));
    let reg = super::socket_path_for_capability(socket_dir, family_id, "registry");
    assert!(reg.to_string_lossy().contains("songbird"));
    let unknown = super::socket_path_for_capability(socket_dir, family_id, "unknown-cap");
    assert!(unknown.to_string_lossy().contains("unknown"));
}

#[test]
fn test_build_primal_command_beardog() {
    let cmd = build_primal_command(
        "beardog",
        std::path::Path::new("/usr/bin/beardog"),
        std::path::Path::new("/tmp/sockets"),
        "fam123",
        "node1",
    );
    assert_eq!(cmd.get_program(), std::path::Path::new("/usr/bin/beardog"));
    let args: Vec<_> = cmd.get_args().collect();
    assert!(args.iter().any(|a| a.to_str() == Some("server")));
    assert!(args.iter().any(|a| a.to_str() == Some("--socket")));
}

#[test]
fn test_build_primal_command_songbird() {
    let cmd = build_primal_command(
        "songbird",
        std::path::Path::new("/usr/bin/songbird"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let envs: Vec<_> = cmd.get_envs().collect();
    assert!(
        envs.iter()
            .any(|(k, _)| k == &std::ffi::OsStr::new("SONGBIRD_SECURITY_PROVIDER"))
    );
    assert!(
        envs.iter()
            .any(|(k, _)| k == &std::ffi::OsStr::new("FAMILY_ID"))
    );
}

#[test]
fn test_build_primal_command_nestgate() {
    let cmd = build_primal_command(
        "nestgate",
        std::path::Path::new("/usr/bin/nestgate"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let args: Vec<_> = cmd.get_args().collect();
    assert!(args.iter().any(|a| a.to_str() == Some("daemon")));
    assert!(args.iter().any(|a| a.to_str() == Some("--family-id")));
}

#[test]
fn test_build_primal_command_toadstool() {
    let cmd = build_primal_command(
        "toadstool",
        std::path::Path::new("/usr/bin/toadstool"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let envs: Vec<_> = cmd.get_envs().collect();
    assert!(
        envs.iter()
            .any(|(k, _)| k == &std::ffi::OsStr::new("TOADSTOOL_FAMILY_ID"))
    );
}

#[test]
fn test_build_primal_command_squirrel() {
    let cmd = build_primal_command(
        "squirrel",
        std::path::Path::new("/usr/bin/squirrel"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let envs: Vec<_> = cmd.get_envs().collect();
    assert!(
        envs.iter()
            .any(|(k, _)| k == &std::ffi::OsStr::new("BIOMEOS_DISCOVERY_SOCKET"))
    );
}

#[test]
fn test_build_primal_command_unknown_primal() {
    let cmd = build_primal_command(
        "unknown_primal",
        std::path::Path::new("/usr/bin/unknown"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let args: Vec<_> = cmd.get_args().collect();
    assert!(args.iter().any(|a| a.to_str() == Some("server")));
}

#[test]
fn test_format_nucleus_summary() {
    let children = vec![
        ("beardog".to_string(), 1234),
        ("songbird".to_string(), 1235),
    ];
    let lines = format_nucleus_summary(
        &children,
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
        NucleusMode::Tower,
        "bootstrap",
    );
    assert!(!lines.is_empty());
    assert!(lines.iter().any(|l| l.contains("NUCLEUS started")));
    assert!(lines.iter().any(|l| l.contains("Family:")));
    assert!(lines.iter().any(|l| l.contains("Node:")));
    assert!(lines.iter().any(|l| l.contains("beardog")));
    assert!(lines.iter().any(|l| l.contains("1234")));
}

#[test]
fn test_format_nucleus_summary_empty_children() {
    let lines = format_nucleus_summary(
        &[],
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
        NucleusMode::Node,
        "coordinated",
    );
    assert!(lines.iter().any(|l| l.contains("coordinated")));
    assert!(lines.iter().any(|l| l.contains("Health check")));
}

#[test]
fn test_ecosystem_state_debug() {
    let _ = format!("{:?}", EcosystemState::Bootstrap);
    let _ = format!(
        "{:?}",
        EcosystemState::Coordinated {
            active_primals: vec!["beardog".to_string()],
        }
    );
}

#[test]
fn test_startup_config_debug() {
    let config = StartupConfig {
        mode: NucleusMode::Tower,
        node_id: "n1".to_string(),
        family_id: "f1".to_string(),
        socket_dir: PathBuf::from("/tmp"),
    };
    let _ = format!("{config:?}");
}

#[test]
fn test_resolve_startup_config_invalid_mode() {
    let result = resolve_startup_config("invalid", "node1", None);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unknown nucleus mode")
    );
}

#[test]
fn test_resolve_startup_config_valid() {
    let config =
        resolve_startup_config_with("tower", "node1", Some("fam1"), Some("/tmp/test-nucleus"))
            .unwrap();
    assert!(matches!(config.mode, NucleusMode::Tower));
    assert_eq!(config.node_id, "node1");
    assert_eq!(config.family_id, "fam1");
    assert_eq!(
        config.socket_dir,
        std::path::PathBuf::from("/tmp/test-nucleus")
    );
}

#[test]
fn test_resolve_socket_dir_env_override() {
    let result =
        resolve_socket_dir_with(Some("/tmp/biomeos-test-socket-dir")).expect("should succeed");
    assert_eq!(
        result,
        std::path::PathBuf::from("/tmp/biomeos-test-socket-dir")
    );
}

#[test]
fn test_discover_binaries_empty_primals() {
    let map = discover_binaries(&[]).expect("empty primals should succeed");
    assert!(map.is_empty());
}

#[test]
fn test_nucleus_mode_from_str_case_insensitive() {
    assert!(matches!(
        "NODE".parse::<NucleusMode>().expect("parse"),
        NucleusMode::Node
    ));
    assert!(matches!(
        "FULL".parse::<NucleusMode>().expect("parse"),
        NucleusMode::Full
    ));
}

#[test]
fn test_base64_encode_two_bytes() {
    let result = base64_encode(&[0x4d, 0x61]);
    assert_eq!(result.len(), 4);
    assert!(result.ends_with("="));
}

#[test]
fn test_base64_encode_four_bytes() {
    let result = base64_encode(&[0x4d, 0x61, 0x6e, 0x21]);
    assert_eq!(result, "TWFuIQ==");
}

#[test]
fn test_resolve_socket_dir_default() {
    let result = resolve_socket_dir_with(None);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(!path.as_os_str().is_empty());
}

#[tokio::test]
async fn test_run_fails_on_invalid_mode() {
    let result = run("invalid_mode_xyz".to_string(), "node1".to_string(), None).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Unknown nucleus mode"),
        "Expected parse error: {err}"
    );
}

#[test]
#[cfg(unix)]
fn test_discover_binaries_finds_in_path() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let unique_name = "biomeos_test_binary_xyz";
    let binary_path = temp_dir.path().join(unique_name);
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0").expect("write test binary");
    let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&binary_path, perms).unwrap();

    let path_dirs = vec![temp_dir.path()];
    let map =
        discover_binaries_with(&[unique_name], None, &path_dirs).expect("discover should succeed");

    assert!(
        map.contains_key(unique_name),
        "{unique_name} should be found in PATH, got: {map:?}"
    );
}

#[tokio::test]
async fn test_wait_for_socket_times_out_on_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("nonexistent.sock");
    let result = wait_for_socket(&path, Duration::from_millis(50), Duration::ZERO).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("did not appear"));
}

#[tokio::test]
async fn test_wait_for_socket_succeeds_when_file_exists() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("test.sock");
    std::fs::write(&path, "").expect("create socket file");
    let result = wait_for_socket(&path, Duration::from_secs(1), Duration::ZERO).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_detect_ecosystem_bootstrap_when_dir_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let nonexistent = temp.path().join("nonexistent_subdir");
    let state = detect_ecosystem(&nonexistent, "test-family").await;
    assert!(
        matches!(state, EcosystemState::Bootstrap),
        "Expected Bootstrap when dir does not exist, got: {state:?}"
    );
}

#[tokio::test]
async fn test_detect_ecosystem_bootstrap_when_dir_empty() {
    let temp = tempfile::tempdir().expect("temp dir");
    let state = detect_ecosystem(temp.path(), "test-family").await;
    assert!(
        matches!(state, EcosystemState::Bootstrap),
        "Expected Bootstrap when dir is empty, got: {state:?}"
    );
}

#[tokio::test]
async fn test_detect_ecosystem_bootstrap_when_stale_sockets_only() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("beardog-test-family.sock");
    std::fs::write(&socket_path, "").expect("create stale socket");
    let state = detect_ecosystem(temp.path(), "test-family").await;
    assert!(
        matches!(state, EcosystemState::Bootstrap),
        "Expected Bootstrap when sockets exist but don't respond, got: {state:?}"
    );
}

#[test]
fn test_base64_encode_standard_vectors() {
    assert_eq!(base64_encode(b""), "");
    assert_eq!(base64_encode(b"f"), "Zg==");
    assert_eq!(base64_encode(b"fo"), "Zm8=");
    assert_eq!(base64_encode(b"foo"), "Zm9v");
    assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
    assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
    assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
}

#[test]
fn test_build_primal_command_nestgate_has_jwt_secret() {
    let cmd = build_primal_command(
        "nestgate",
        std::path::Path::new("/usr/bin/nestgate"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let envs: Vec<_> = cmd.get_envs().collect();
    let jwt = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("NESTGATE_JWT_SECRET"));
    assert!(jwt.is_some(), "NESTGATE_JWT_SECRET should be set");
    let (_, v) = jwt.unwrap();
    assert!(v.is_some(), "JWT secret value should be present");
    let secret = v.unwrap().to_string_lossy();
    assert!(!secret.is_empty(), "JWT secret should be non-empty");
}

#[test]
fn test_build_primal_command_squirrel_with_ai_providers() {
    let config = PrimalCommandConfig {
        name: "squirrel",
        binary: std::path::Path::new("/usr/bin/squirrel"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node1",
        anthropic_api_key: Some("test-key"),
        openai_api_key: None,
        ai_http_providers: None,
    };
    let cmd = build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai_providers = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(
        ai_providers.is_some(),
        "AI_HTTP_PROVIDERS should be set when API key present"
    );
}

#[test]
fn test_build_primal_command_squirrel_with_openai_key() {
    let config = PrimalCommandConfig {
        name: "squirrel",
        binary: std::path::Path::new("/usr/bin/squirrel"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node1",
        anthropic_api_key: None,
        openai_api_key: Some("sk-test"),
        ai_http_providers: None,
    };
    let cmd = build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai_providers = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(ai_providers.is_some());
}

#[test]
fn test_discover_binaries_finds_in_plasmidbin() {
    let temp = tempfile::tempdir().expect("temp dir");
    let plasmid_bin = temp.path().join("primals");
    std::fs::create_dir_all(&plasmid_bin).expect("create primals dir");
    let unique_name = "biomeos_test_primal_xyz";
    let binary_path = plasmid_bin.join(unique_name);
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0").expect("write test binary");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }

    let plasmid_bin_dir = plasmid_bin.parent().unwrap();
    let map = discover_binaries_with(&[unique_name], Some(plasmid_bin_dir), &[])
        .expect("discover should succeed");

    assert!(
        map.contains_key(unique_name),
        "{unique_name} should be found in plasmidBin, got: {map:?}"
    );
}

#[test]
fn test_discover_binaries_finds_in_plasmidbin_subdir() {
    let temp = tempfile::tempdir().expect("temp dir");
    let plasmid_bin = temp.path().join("primals");
    std::fs::create_dir_all(&plasmid_bin).expect("create primals dir");
    let unique_name = "biomeos_test_subdir_xyz";
    let primal_dir = plasmid_bin.join(unique_name);
    std::fs::create_dir_all(&primal_dir).expect("create primal subdir");
    let binary_path = primal_dir.join(unique_name);
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0").expect("write test binary");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }

    let plasmid_bin_dir = plasmid_bin.parent().unwrap();
    let map = discover_binaries_with(&[unique_name], Some(plasmid_bin_dir), &[])
        .expect("discover should succeed");

    assert!(
        map.contains_key(unique_name),
        "{unique_name} should be found in plasmidBin subdir, got: {map:?}"
    );
}

#[test]
fn test_discover_binaries_warns_on_missing() {
    let map =
        discover_binaries(&["nonexistent_primal_xyz_12345"]).expect("discover should succeed");
    assert!(map.is_empty());
}

#[test]
fn test_format_nucleus_summary_health_check_line() {
    let lines = format_nucleus_summary(
        &[("beardog".to_string(), 1)],
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
        NucleusMode::Tower,
        "bootstrap",
    );
    assert!(lines.iter().any(|l| l.contains("nc -U")));
    assert!(lines.iter().any(|l| l.contains("health")));
}

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
fn test_base64_encode_single_byte_padding() {
    let r = super::base64_encode(&[0x41]);
    assert_eq!(r.len(), 4);
    assert!(r.ends_with("=="));
}

#[test]
fn test_base64_encode_two_byte_padding() {
    let r = super::base64_encode(&[0x41, 0x42]);
    assert_eq!(r.len(), 4);
    assert!(r.ends_with("="));
}

#[test]
fn test_resolve_startup_config_uses_biomeos_socket_dir_env() {
    let _guard = TestEnvGuard::set("BIOMEOS_SOCKET_DIR", "/tmp/nucleus-env-test");
    let config = resolve_startup_config("tower", "node1", Some("fam1")).expect("should succeed");
    assert_eq!(config.socket_dir, PathBuf::from("/tmp/nucleus-env-test"));
}

#[test]
fn test_build_primal_command_squirrel_ai_default_model_env() {
    let _guard = TestEnvGuard::set("AI_DEFAULT_MODEL", "custom-model-v1");
    let cmd = build_primal_command(
        "squirrel",
        std::path::Path::new("/usr/bin/squirrel"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
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
    let _guard = TestEnvGuard::set("ANTHROPIC_API_KEY", "sk-ant-test");
    let cmd = build_primal_command(
        "squirrel",
        std::path::Path::new("/usr/bin/squirrel"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
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
    let _guard = TestEnvGuard::set("OPENAI_API_KEY", "sk-openai-test");
    let cmd = build_primal_command(
        "squirrel",
        std::path::Path::new("/usr/bin/squirrel"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai_providers = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(ai_providers.is_some());
}

#[test]
fn test_discover_binaries_empty_path_env() {
    let _guard = TestEnvGuard::set("PATH", "");
    let map = discover_binaries(&["beardog"]).expect("discover should not panic");
    // May or may not find beardog depending on relative paths (plasmidBin, target/release)
    // Just verify it doesn't panic with empty PATH
    let _ = map;
}
