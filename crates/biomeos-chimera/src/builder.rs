// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
    pub fn release(mut self, release: bool) -> Self {
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

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let child = Command::new(&self.binary_path)
            .spawn()?;
        self.process = Some(child);
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

        // API endpoints
        if !def.fusion.api.endpoints.is_empty() {
            code.push_str("\n/// Unified API endpoints\n");
            code.push_str("pub mod api {\n");

            for endpoint in &def.fusion.api.endpoints {
                writeln!(code, "    /// {}", endpoint.description)
                    .expect("write to String cannot fail");
                write!(code, "    pub async fn {}(", endpoint.name)
                    .expect("write to String cannot fail");

                // Parameters
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
                writeln!(
                    code,
                    ") -> Result<{return_type}, Box<dyn std::error::Error>> {{"
                )
                .expect("write to String cannot fail");
                code.push_str("        Err(\"Fusion logic not implemented\".into())\n");
                code.push_str("    }\n\n");
            }

            code.push_str("}\n");
        }

        code
    }

    /// Write a shell launcher script
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
mod tests {
    use super::*;
    use crate::definition::ChimeraDefinition;

    #[test]
    fn test_generate_orchestrator() {
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

        let def = ChimeraDefinition::from_yaml(yaml).unwrap();
        let builder = ChimeraBuilder::new(Arc::new(def));

        let code = builder.generate_orchestrator();

        assert!(code.contains("test-chimera"));
        assert!(code.contains("pub struct ChimeraState"));
        assert!(code.contains("pub async fn connect"));
    }
}
