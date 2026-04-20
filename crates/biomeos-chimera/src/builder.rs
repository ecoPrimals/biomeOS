// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Chimera Builder
//!
//! Compiles chimera definitions into deployable units.

use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::{debug, info};

use crate::definition::ChimeraDefinition;
use crate::error::{ChimeraError, ChimeraResult};

/// Builder for compiling chimera definitions
#[derive(Debug)]
pub struct ChimeraBuilder {
    /// The chimera definition to build
    definition: Arc<ChimeraDefinition>,

    /// Binary output directory
    output_dir: PathBuf,

    /// Primal binaries directory
    primals_dir: PathBuf,

    /// Build in release mode
    release: bool,
}

/// Result of a chimera build
#[derive(Debug)]
pub struct BuildResult {
    /// Chimera ID
    pub chimera_id: String,

    /// Output binary path
    pub binary_path: PathBuf,

    /// Build duration
    pub duration: std::time::Duration,

    /// Warnings generated during build
    pub warnings: Vec<String>,
}

impl ChimeraBuilder {
    /// Create a new builder for a chimera definition
    #[must_use]
    pub fn new(definition: Arc<ChimeraDefinition>) -> Self {
        Self {
            definition,
            output_dir: PathBuf::from("bin/chimeras"),
            primals_dir: PathBuf::from("bin/primals"),
            release: true,
        }
    }

    /// Set the output directory
    #[must_use]
    pub fn output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Set the primals directory
    #[must_use]
    pub fn primals_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.primals_dir = dir.into();
        self
    }

    /// Set release mode
    #[must_use]
    pub const fn release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }

    /// Check that all required primals are available
    ///
    /// # Errors
    ///
    /// Returns an error if any required primal is not found in the primals directory
    pub fn check_primals(&self) -> ChimeraResult<Vec<PathBuf>> {
        let mut primal_paths = Vec::new();

        for primal in self.definition.required_primals() {
            // First try exact match
            let exact_path = self.primals_dir.join(primal);

            if exact_path.exists() {
                primal_paths.push(exact_path);
                continue;
            }

            // Then try finding any binary that starts with the primal name
            let found = std::fs::read_dir(&self.primals_dir)
                .ok()
                .and_then(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .find(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            name == primal || name.starts_with(&format!("{primal}-"))
                        })
                        .map(|e| e.path())
                });

            match found {
                Some(path) => primal_paths.push(path),
                None => {
                    return Err(ChimeraError::PrimalNotAvailable {
                        chimera: self.definition.chimera.id.clone(),
                        primal: primal.to_string(),
                    });
                }
            }
        }

        Ok(primal_paths)
    }

    /// Build the chimera
    ///
    /// # Errors
    ///
    /// Returns an error if the build fails due to:
    /// - Missing primals
    /// - Script generation failure
    /// - Build process errors
    pub fn build(&self) -> ChimeraResult<BuildResult> {
        let start = std::time::Instant::now();
        let mut warnings = Vec::new();

        info!("Building chimera: {}", self.definition.chimera.id);

        // 1. Check primals are available
        let primal_paths = self.check_primals()?;
        debug!("Found {} required primals", primal_paths.len());

        // 2. Create output directory
        std::fs::create_dir_all(&self.output_dir)?;

        // 3. Generate chimera orchestrator code
        let generated_code = self.generate_orchestrator();
        debug!(
            "Generated {} bytes of orchestrator code",
            generated_code.len()
        );

        // 4. For now, create a shell script launcher
        // In the future, this will be a compiled Rust binary
        let output_path = self.output_dir.join(&self.definition.chimera.id);
        self.write_launcher(&output_path, &primal_paths)?;

        // 5. Also write the generated code for inspection
        let code_path = self
            .output_dir
            .join(format!("{}.rs", self.definition.chimera.id));
        std::fs::write(&code_path, &generated_code)?;
        warnings.push(format!(
            "Generated code written to {} (not yet compiled)",
            code_path.display()
        ));

        let duration = start.elapsed();
        info!(
            "Built chimera '{}' in {:?}",
            self.definition.chimera.id, duration
        );

        Ok(BuildResult {
            chimera_id: self.definition.chimera.id.clone(),
            binary_path: output_path,
            duration,
            warnings,
        })
    }

    /// Generate the chimera orchestrator Rust code
    #[expect(clippy::expect_used, reason = "write to String cannot fail")]
    fn generate_orchestrator(&self) -> String {
        let def = &self.definition;

        let mut code = String::new();

        // Header
        writeln!(
            code,
            "//! Auto-generated chimera orchestrator: {}\n//! \n//! {}\n//!\n//! Components: {}\n\nuse std::process::Command;\nuse std::sync::Arc;\nuse tokio::sync::RwLock;\n",
            def.chimera.id,
            def.chimera.description.lines().next().unwrap_or(""),
            def.required_primals().join(", ")
        ).expect("write to String cannot fail");

        // Component state struct
        writeln!(
            code,
            "\n/// State for chimera: {}\npub struct ChimeraState {{",
            def.chimera.id
        )
        .expect("write to String cannot fail");

        for (name, component) in &def.components {
            if let Some(array) = &component.array {
                writeln!(
                    code,
                    "    /// {name} instances (min: {}, max: {})",
                    array.min, array.max
                )
                .expect("write to String cannot fail");
                writeln!(code, "    pub {name}_instances: Vec<ComponentInstance>,")
                    .expect("write to String cannot fail");
            } else {
                writeln!(code, "    /// {name} component").expect("write to String cannot fail");
                writeln!(code, "    pub {name}: ComponentInstance,")
                    .expect("write to String cannot fail");
            }
        }

        code.push_str("}\n\n");

        // Component instance struct
        code.push_str(
            "
/// A running component instance
pub struct ComponentInstance {
    /// Process handle
    pub process: Option<std::process::Child>,
    /// Component name
    pub name: String,
    /// Binary path
    pub binary_path: std::path::PathBuf,
}

impl ComponentInstance {
    pub fn new(name: impl Into<String>, binary_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            process: None,
            name: name.into(),
            binary_path: binary_path.into(),
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        let child = Command::new(&self.binary_path)
            .spawn()?;
        self.process = Some(child);
        Ok(())
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(ref mut child) = self.process {
            child.kill()?;
            child.wait()?;
        }
        self.process = None;
        Ok(())
    }
}
",
        );

        self.generate_api_endpoints(&mut code);

        code
    }

    /// Generate `pub mod api { ... }` with capability-forwarding endpoint stubs.
    #[expect(clippy::expect_used, reason = "write to String cannot fail")]
    fn generate_api_endpoints(&self, code: &mut String) {
        let endpoints = &self.definition.fusion.api.endpoints;
        if endpoints.is_empty() {
            return;
        }

        code.push_str("\n/// Unified API endpoints\n");
        code.push_str("pub mod api {\n");

        for endpoint in endpoints {
            writeln!(code, "    /// {}", endpoint.description)
                .expect("write to String cannot fail");
            write!(code, "    pub async fn {}(", endpoint.name)
                .expect("write to String cannot fail");

            let params: Vec<String> = endpoint
                .params
                .iter()
                .map(|p| format!("{p}: &str"))
                .collect();
            code.push_str(&params.join(", "));

            let return_type = if endpoint.returns.is_empty() {
                "()"
            } else {
                &endpoint.returns
            };
            writeln!(code, ") -> anyhow::Result<{return_type}> {{")
                .expect("write to String cannot fail");
            if let Some(cap) = &endpoint.capability {
                writeln!(
                    code,
                    "        crate::ipc::capability_call(\"{cap}\", &[{}]).await",
                    endpoint
                        .params
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .join(", "),
                )
                .expect("write to String cannot fail");
            } else {
                code.push_str("        anyhow::bail!(\"Fusion logic not implemented — set 'capability' in chimera definition to enable IPC forwarding\")\n");
            }
            code.push_str("    }\n\n");
        }

        code.push_str("}\n");
    }

    /// Write a shell launcher script
    #[expect(clippy::expect_used, reason = "write to String cannot fail")]
    fn write_launcher(&self, output_path: &Path, primal_paths: &[PathBuf]) -> ChimeraResult<()> {
        let def = &self.definition;

        let mut script = String::new();
        script.push_str("#!/usr/bin/env bash\n");
        writeln!(script, "# Chimera launcher: {}", def.chimera.id)
            .expect("write to String cannot fail");
        writeln!(script, "# {}\n", def.chimera.name).expect("write to String cannot fail");

        script.push_str("set -euo pipefail\n\n");

        writeln!(script, "echo \"🧬 Starting chimera: {}\"\n", def.chimera.id)
            .expect("write to String cannot fail");

        // Start each primal
        for (i, primal_path) in primal_paths.iter().enumerate() {
            let primal_name = primal_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            writeln!(script, "echo \"  Starting component: {primal_name}\"")
                .expect("write to String cannot fail");
            writeln!(script, "\"{}\" &", primal_path.display())
                .expect("write to String cannot fail");
            writeln!(script, "PID_{i}=$!\n").expect("write to String cannot fail");
        }

        // Cleanup on exit
        script.push_str("cleanup() {\n");
        script.push_str("    echo \"Stopping chimera...\"\n");
        for i in 0..primal_paths.len() {
            writeln!(script, "    kill $PID_{i} 2>/dev/null || true")
                .expect("write to String cannot fail");
        }
        script.push_str("}\n\n");

        script.push_str("trap cleanup EXIT\n\n");
        script.push_str("echo \"✅ Chimera running. Press Ctrl+C to stop.\"\n");
        script.push_str("wait\n");

        std::fs::write(output_path, &script)?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(output_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(output_path, perms)?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::definition::ChimeraDefinition;

    fn test_definition() -> Arc<ChimeraDefinition> {
        let yaml = r#"
chimera:
  id: test-chimera
  name: Test Chimera
  version: "1.0.0"
  description: A test chimera for unit tests

components:
  beardog:
    source: primals/beardog
    version: ">=1.0.0"
    modules:
      - name: btsp
        description: BTSP tunnel
        features: [tunnel, encryption]

fusion:
  bindings: {}
  api:
    endpoints:
      - name: connect
        description: Connect to peer
        params: [peer_did]
        returns: SecureChannel
"#;
        Arc::new(ChimeraDefinition::from_yaml(yaml).unwrap())
    }

    #[test]
    fn test_generate_orchestrator() {
        let def = test_definition();
        let builder = ChimeraBuilder::new(def);

        let code = builder.generate_orchestrator();

        assert!(code.contains("test-chimera"));
        assert!(code.contains("pub struct ChimeraState"));
        assert!(code.contains("pub async fn connect"));
    }

    #[test]
    fn test_builder_new_defaults() {
        let def = test_definition();
        let builder = ChimeraBuilder::new(def);
        // Default output_dir and primals_dir
        let _ = builder;
    }

    #[test]
    fn test_builder_output_dir() {
        let def = test_definition();
        let builder = ChimeraBuilder::new(def).output_dir("/custom/out");
        let _ = builder;
    }

    #[test]
    fn test_builder_primals_dir() {
        let def = test_definition();
        let builder = ChimeraBuilder::new(def).primals_dir("/custom/primals");
        let _ = builder;
    }

    #[test]
    fn test_builder_release() {
        let def = test_definition();
        let builder = ChimeraBuilder::new(def).release(false);
        let _ = builder;
    }

    #[test]
    fn test_check_primals_missing() {
        let def = test_definition();
        let tmp = tempfile::tempdir().unwrap();
        let builder = ChimeraBuilder::new(def).primals_dir(tmp.path());
        let result = builder.check_primals();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("beardog") || err.to_string().contains("Primal"));
    }

    #[test]
    fn test_check_primals_exact_match() {
        let def = test_definition();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("beardog"), b"#!/bin/sh").unwrap();
        let builder = ChimeraBuilder::new(def).primals_dir(tmp.path());
        let result = builder.check_primals();
        assert!(result.is_ok());
        let paths = result.unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with("beardog"));
    }

    #[test]
    fn test_check_primals_prefix_match() {
        let def = test_definition();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("beardog-v1.2.3"), b"#!/bin/sh").unwrap();
        let builder = ChimeraBuilder::new(def).primals_dir(tmp.path());
        let result = builder.check_primals();
        assert!(result.is_ok());
        let paths = result.unwrap();
        assert_eq!(paths.len(), 1);
        assert!(
            paths[0]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("beardog")
        );
    }

    #[test]
    fn test_build_success() {
        let def = test_definition();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("primals")).unwrap();
        std::fs::write(tmp.path().join("primals").join("beardog"), b"#!/bin/sh").unwrap();

        let builder = ChimeraBuilder::new(def)
            .output_dir(tmp.path().join("out"))
            .primals_dir(tmp.path().join("primals"));

        let result = builder.build();
        assert!(result.is_ok());
        let build_result = result.unwrap();
        assert_eq!(build_result.chimera_id, "test-chimera");
        assert!(build_result.binary_path.exists());
        assert!(!build_result.warnings.is_empty());
    }

    #[test]
    fn test_build_result_fields() {
        let def = test_definition();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("primals")).unwrap();
        std::fs::write(tmp.path().join("primals").join("beardog"), b"#!/bin/sh").unwrap();

        let builder = ChimeraBuilder::new(def)
            .output_dir(tmp.path().join("out"))
            .primals_dir(tmp.path().join("primals"));

        let build_result = builder.build().unwrap();
        assert!(build_result.duration.as_secs() < 60);
        assert!(
            build_result
                .warnings
                .iter()
                .any(|w| w.contains("Generated code"))
        );
    }
}
