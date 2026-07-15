// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_build_primal_command_squirrel_without_ai_keys() {
    let config = PrimalCommandConfig {
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
    let cmd = build_primal_command_with(&config);
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
fn test_primal_command_config_debug() {
    let config = PrimalCommandConfig {
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
    let config = PrimalCommandConfig {
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
    let cmd = build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai_providers = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(ai_providers.is_some());
    let (_, v) = ai_providers.unwrap();
    assert_eq!(v.unwrap().to_string_lossy(), "custom,anthropic");
}

#[test]
fn test_build_primal_command_squirrel_ai_default_model_env() {
    let config = PrimalCommandConfig {
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
    let cmd = build_primal_command_with(&config);
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
    let config = PrimalCommandConfig {
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
    let cmd = build_primal_command_with(&config);
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
    let config = PrimalCommandConfig {
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
    let cmd = build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai_providers = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(ai_providers.is_some());
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

#[test]
fn test_build_primal_command_squirrel_custom_ai_http_providers_when_both_keys_set() {
    let config = PrimalCommandConfig {
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
    let cmd = build_primal_command_with(&config);
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
    let config = PrimalCommandConfig {
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
    let cmd = build_primal_command_with(&config);
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
fn test_build_primal_command_with_all_env_keys_set() {
    let config = PrimalCommandConfig {
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
    let cmd = build_primal_command_with(&config);
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
