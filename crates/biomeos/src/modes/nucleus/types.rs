// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::{Context, Result};
use biomeos_core::family_discovery::get_family_id;
use biomeos_core::socket_discovery::neural_api::resolve_neural_api_socket;
use biomeos_types::{
    SystemPaths,
    primal_names::{
        BARRACUDA, BEARDOG, CORALREEF, LOAMSPINE, NESTGATE, PETALTONGUE, RHIZOCRYPT, SKUNKBAT,
        SONGBIRD, SQUIRREL, SWEETGRASS, TOADSTOOL,
    },
};
use serde::Deserialize;
use std::path::PathBuf;

/// Detected ecosystem state at startup
#[derive(Debug)]
pub(crate) enum EcosystemState {
    /// No ecosystem detected -- we are the genesis orchestrator
    Bootstrap,
    /// Existing ecosystem detected with these active primals
    Coordinated { active_primals: Vec<String> },
}

/// NUCLEUS deployment pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NucleusMode {
    /// BearDog + Songbird + skunkBat (security + mesh + defense)
    Tower,
    /// Tower + ToadStool + barraCuda + coralReef (compute pipeline)
    Node,
    /// Tower + NestGate + rhizoCrypt + loamSpine + sweetGrass + Squirrel (storage + provenance)
    Nest,
    /// Core 5: BearDog + Songbird + NestGate + ToadStool + Squirrel (legacy compat)
    Core,
    /// All 13 primals + Neural API
    Full,
}

impl std::str::FromStr for NucleusMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "tower" => Ok(NucleusMode::Tower),
            "node" => Ok(NucleusMode::Node),
            "nest" => Ok(NucleusMode::Nest),
            "core" => Ok(NucleusMode::Core),
            "full" | "nucleus" => Ok(NucleusMode::Full),
            _ => Err(anyhow::anyhow!(
                "Unknown nucleus mode: '{s}'. Use tower|node|nest|core|full"
            )),
        }
    }
}

impl NucleusMode {
    /// Get the primals needed for this mode (in startup order).
    ///
    /// Startup ordering: security first (bearDog), then mesh (songbird),
    /// then defense (skunkBat), then compute (toadstool, coralreef, barracuda),
    /// then storage/provenance (nestgate, rhizocrypt, loamspine, sweetgrass),
    /// then AI (squirrel), then UI (petaltongue).
    pub(crate) fn primals(self) -> Vec<&'static str> {
        match self {
            NucleusMode::Tower => vec![BEARDOG, SONGBIRD, SKUNKBAT],
            NucleusMode::Node => vec![BEARDOG, SONGBIRD, SKUNKBAT, TOADSTOOL, CORALREEF, BARRACUDA],
            NucleusMode::Nest => vec![
                BEARDOG, SONGBIRD, SKUNKBAT, NESTGATE, RHIZOCRYPT, LOAMSPINE, SWEETGRASS, SQUIRREL,
            ],
            NucleusMode::Core => vec![BEARDOG, SONGBIRD, NESTGATE, TOADSTOOL, SQUIRREL],
            NucleusMode::Full => vec![
                BEARDOG,
                SONGBIRD,
                SKUNKBAT,
                TOADSTOOL,
                CORALREEF,
                BARRACUDA,
                NESTGATE,
                RHIZOCRYPT,
                LOAMSPINE,
                SWEETGRASS,
                SQUIRREL,
                PETALTONGUE,
            ],
        }
    }
}

/// Resolved startup configuration (pure, testable)
#[derive(Debug, Clone)]
pub(crate) struct StartupConfig {
    pub mode: NucleusMode,
    pub node_id: String,
    pub family_id: String,
    pub socket_dir: PathBuf,
}

/// Resolve startup configuration from mode string and optional overrides.
pub(crate) fn resolve_startup_config(
    mode: &str,
    node_id: &str,
    family_id: Option<&str>,
) -> Result<StartupConfig> {
    resolve_startup_config_with(
        mode,
        node_id,
        family_id,
        std::env::var(biomeos_types::env_config::vars::SOCKET_DIR)
            .ok()
            .as_deref(),
    )
}

pub(crate) fn resolve_startup_config_with(
    mode: &str,
    node_id: &str,
    family_id: Option<&str>,
    socket_dir_override: Option<&str>,
) -> Result<StartupConfig> {
    let mode: NucleusMode = mode.parse()?;
    let family_id =
        family_id.map_or_else(biomeos_core::family_discovery::get_family_id, String::from);
    let socket_dir = super::nucleus_procs::resolve_socket_dir_with(socket_dir_override)?;
    Ok(StartupConfig {
        mode,
        node_id: node_id.to_string(),
        family_id,
        socket_dir,
    })
}

/// Configuration for building a primal process command.
#[derive(Debug, Clone)]
pub(crate) struct PrimalCommandConfig<'a> {
    pub name: &'a str,
    pub binary: &'a std::path::Path,
    pub socket_dir: &'a std::path::Path,
    pub family_id: &'a str,
    pub node_id: &'a str,
    pub anthropic_api_key: Option<&'a str>,
    pub openai_api_key: Option<&'a str>,
    pub ai_http_providers: Option<&'a str>,
    /// When set, used instead of reading `AI_DEFAULT_MODEL` from the environment.
    pub ai_default_model: Option<&'a str>,
}

/// Resolve socket path for a capability using taxonomy-based discovery.
///
/// Delegates to `CapabilityTaxonomy::resolve_to_primal` without hardcoded
/// fallbacks — if the taxonomy can't resolve the capability, we return an
/// `unknown-{family_id}.sock` path that simply won't exist on disk,
/// triggering the appropriate "socket not found" error at connect time.
pub(crate) fn socket_path_for_capability(
    socket_dir: &std::path::Path,
    family_id: &str,
    capability: &str,
) -> PathBuf {
    let primal_name =
        biomeos_types::CapabilityTaxonomy::resolve_to_primal(capability).unwrap_or("unknown");
    socket_dir.join(format!("{primal_name}-{family_id}.sock"))
}

/// Build a primal process command (testable, no spawn).
/// Returns std::process::Command for inspection and testing.
/// Socket paths use capability-based resolution via taxonomy.
pub(crate) fn build_primal_command(
    name: &str,
    binary: &std::path::Path,
    socket_dir: &std::path::Path,
    family_id: &str,
    node_id: &str,
) -> std::process::Command {
    let has_ai =
        std::env::var("ANTHROPIC_API_KEY").is_ok() || std::env::var("OPENAI_API_KEY").is_ok();
    let ai_providers = has_ai.then(|| {
        std::env::var(biomeos_types::env_config::vars::AI_HTTP_PROVIDERS)
            .unwrap_or_else(|_| "anthropic,openai".to_string())
    });
    let anthropic = std::env::var("ANTHROPIC_API_KEY").ok();
    let openai = std::env::var("OPENAI_API_KEY").ok();
    let config = PrimalCommandConfig {
        name,
        binary,
        socket_dir,
        family_id,
        node_id,
        anthropic_api_key: anthropic.as_deref(),
        openai_api_key: openai.as_deref(),
        ai_http_providers: ai_providers.as_deref(),
        ai_default_model: None,
    };
    build_primal_command_with(&config)
}

pub(crate) fn build_primal_command_with(config: &PrimalCommandConfig<'_>) -> std::process::Command {
    use biomeos_types::defaults::env_vars::socket_env_key;

    let socket_path = config
        .socket_dir
        .join(format!("{}-{}.sock", config.name, config.family_id));
    let mut cmd = std::process::Command::new(config.binary);

    let self_socket_key = socket_env_key(config.name);
    cmd.env(&self_socket_key, socket_path.as_os_str());

    let profiles = super::nucleus_launch::load_nucleus_profiles();
    let profile = profiles.profiles.get(config.name);
    let defaults = &profiles.default;

    let subcommand = profile
        .and_then(|p| p.subcommand.as_deref())
        .or(defaults.subcommand.as_deref())
        .unwrap_or("server");
    cmd.arg(subcommand);

    let pass_socket = profile
        .and_then(|p| p.pass_socket_flag)
        .or(defaults.pass_socket_flag)
        .unwrap_or(true);
    if pass_socket {
        cmd.arg("--socket").arg(socket_path.as_os_str());
    }

    let pass_family_id = profile
        .and_then(|p| p.pass_family_id_flag)
        .or(defaults.pass_family_id_flag)
        .unwrap_or(false);
    if pass_family_id {
        cmd.arg("--family-id").arg(config.family_id);
    }

    // Capability-resolved socket env vars (e.g. SONGBIRD_SECURITY_PROVIDER → security socket)
    let cap_sockets = profile.map_or(&defaults.capability_sockets, |p| &p.capability_sockets);
    for (env_key, capability) in cap_sockets {
        let resolved = socket_path_for_capability(config.socket_dir, config.family_id, capability);
        cmd.env(env_key, &resolved);
    }

    // Static env vars from profile (with variable substitution)
    // $family_id / $node_id → literal values from config
    // $UPPER_CASE → passthrough from parent process environment
    let env_vars = profile.map_or(&defaults.env_vars, |p| &p.env_vars);
    for (key, value) in env_vars {
        let resolved = if value.starts_with('$')
            && value.len() > 1
            && value[1..]
                .chars()
                .all(|c| c.is_ascii_uppercase() || c == '_')
        {
            let env_name = &value[1..];
            match std::env::var(env_name) {
                Ok(v) => v,
                Err(_) => continue,
            }
        } else {
            value
                .replace("$family_id", config.family_id)
                .replace("$node_id", config.node_id)
        };
        cmd.env(key, &resolved);
    }

    // JWT secret generation
    let gen_jwt = profile
        .and_then(|p| p.generate_jwt_secret)
        .or(defaults.generate_jwt_secret)
        .unwrap_or(false);
    if gen_jwt {
        cmd.env(
            "NESTGATE_JWT_SECRET",
            super::nucleus_procs::generate_jwt_secret(),
        );
    }

    // AI model passthrough
    let pass_ai_model = profile
        .and_then(|p| p.pass_ai_model)
        .or(defaults.pass_ai_model)
        .unwrap_or(false);
    if pass_ai_model {
        if let Some(model) = config.ai_default_model {
            cmd.env(biomeos_types::env_config::vars::AI_DEFAULT_MODEL, model);
        } else if let Ok(model) = std::env::var(biomeos_types::env_config::vars::AI_DEFAULT_MODEL) {
            cmd.env(biomeos_types::env_config::vars::AI_DEFAULT_MODEL, model);
        }
    }

    // AI HTTP providers passthrough
    let pass_ai_providers = profile
        .and_then(|p| p.pass_ai_providers)
        .or(defaults.pass_ai_providers)
        .unwrap_or(false);
    if pass_ai_providers && (config.anthropic_api_key.is_some() || config.openai_api_key.is_some())
    {
        cmd.env(
            "AI_HTTP_PROVIDERS",
            config.ai_http_providers.unwrap_or("anthropic,openai"),
        );
    }

    cmd.env("FAMILY_ID", config.family_id)
        .env(
            biomeos_types::env_config::vars::NODE_ID_LEGACY,
            config.node_id,
        )
        .env(
            biomeos_types::env_config::vars::SOCKET_DIR,
            config.socket_dir.as_os_str(),
        );
    cmd
}

/// Parsed spore deploy manifest (`spore.toml` or `.manifest.toml` with deploy section).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct SporeDeployManifest {
    pub spore: SporeDeploySpec,
}

/// Deploy parameters embedded in a spore manifest.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct SporeDeploySpec {
    pub mode: String,
    pub node_id: String,
    pub graph_id: String,
}

/// Summary of a `lifecycle.status` response (pure, testable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NucleusStatusSummary {
    pub count: usize,
    pub healthy: usize,
    pub primal_names: Vec<String>,
}

/// Resolve the Neural API socket for lifecycle RPC calls.
pub(crate) fn resolve_lifecycle_socket(
    socket: Option<PathBuf>,
    family_id: Option<String>,
) -> (PathBuf, String) {
    let family = family_id.unwrap_or_else(get_family_id);
    let path = socket.unwrap_or_else(|| {
        resolve_neural_api_socket(&family, None, None).unwrap_or_else(|| {
            SystemPaths::new_lazy().primal_socket(&format!("neural-api-{family}"))
        })
    });
    (path, family)
}

/// Parse and validate a spore deploy manifest from TOML text.
pub(crate) fn parse_spore_deploy_manifest(content: &str) -> Result<SporeDeployManifest> {
    let manifest: SporeDeployManifest =
        toml::from_str(content).context("Invalid spore manifest")?;
    if manifest.spore.mode.is_empty() {
        anyhow::bail!("Spore manifest missing mode");
    }
    if manifest.spore.node_id.is_empty() {
        anyhow::bail!("Spore manifest missing node_id");
    }
    if manifest.spore.graph_id.is_empty() {
        anyhow::bail!("Spore manifest missing graph_id");
    }
    Ok(manifest)
}

/// Parse a `lifecycle.status` RPC result into a summary.
pub(crate) fn parse_nucleus_status(result: &serde_json::Value) -> Result<NucleusStatusSummary> {
    let primals = result
        .get("primals")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let primal_names: Vec<String> = primals
        .iter()
        .filter_map(|p| p.get("name").and_then(serde_json::Value::as_str))
        .map(String::from)
        .collect();
    let count = result
        .get("count")
        .and_then(serde_json::Value::as_u64)
        .map_or(primal_names.len(), |n| n as usize);
    let healthy = result
        .get("healthy")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as usize;
    Ok(NucleusStatusSummary {
        count,
        healthy,
        primal_names,
    })
}
