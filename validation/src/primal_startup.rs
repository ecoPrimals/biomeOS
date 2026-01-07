//! Primal startup system (capability-based)
//!
//! Start primals based on required capabilities, not hardcoded names.

use crate::capabilities::{Capability, CapabilityProfile, PrimalBinary};
use crate::deployment::DeployedVm;
use anyhow::{Context, Result};

/// Primal startup manager
#[derive(Debug)]
pub struct PrimalStartup {
    pub profile: CapabilityProfile,
}

impl PrimalStartup {
    /// Create a new primal startup manager
    #[must_use]
    pub fn new(profile: CapabilityProfile) -> Self {
        Self { profile }
    }

    /// Discover available primals on a VM
    pub async fn discover_primals(&self, vm: &DeployedVm) -> Result<Vec<DiscoveredPrimal>> {
        println!("  🔍 Discovering primals on {}...", vm.name);

        // List primalBins directory
        let output = vm
            .ssh_exec("ls /opt/biomeos/primalBins/ 2>/dev/null || echo ''")
            .context("Failed to list primalBins")?;

        let mut primals = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("total") {
                continue;
            }

            // For now, we'll use filename to infer capabilities
            // In production, each primal would report via CLI: ./primal capabilities
            let capabilities = self.infer_capabilities(line);

            if !capabilities.is_empty() {
                primals.push(DiscoveredPrimal {
                    name: line.to_string(),
                    path: format!("/opt/biomeos/primalBins/{}", line),
                    capabilities,
                });
            }
        }

        println!("    Found {} primals", primals.len());
        for primal in &primals {
            println!("      • {} ({:?})", primal.name, primal.capabilities);
        }

        Ok(primals)
    }

    /// Infer capabilities from primal name (temporary - will use CLI query)
    fn infer_capabilities(&self, name: &str) -> Vec<Capability> {
        // Future: Replace with actual capability query: `./primal capabilities`
        // Currently using inference based on naming conventions
        match name.to_lowercase().as_str() {
            "songbird" => vec![Capability::P2PCoordination],
            "beardog" => vec![Capability::Identity],
            "nestgate" => vec![Capability::Storage],
            "toadstool" => vec![Capability::Compute],
            "sweetgrass" => vec![Capability::TemporalTracking],
            "rhizocrypt" => vec![Capability::Encryption],
            "loamspine" => vec![Capability::StateManagement],
            "petaltongue" => vec![Capability::Visualization],
            _ if name.contains("p2p") => vec![Capability::P2PCoordination],
            _ if name.contains("storage") => vec![Capability::Storage],
            _ if name.contains("compute") => vec![Capability::Compute],
            _ => Vec::new(),
        }
    }

    /// Match primals to required capabilities
    pub fn match_capabilities(&self, primals: &[DiscoveredPrimal]) -> Result<Vec<PrimalMatch>> {
        println!("  🔗 Matching capabilities...");

        let mut matches = Vec::new();

        // Match required capabilities
        for cap in &self.profile.required_capabilities {
            let primal = primals
                .iter()
                .find(|p| p.provides(cap))
                .ok_or_else(|| {
                    anyhow::anyhow!("No primal found for required capability: {:?}", cap)
                })?;

            matches.push(PrimalMatch {
                capability: cap.clone(),
                primal: primal.clone(),
                required: true,
            });

            println!("    ✅ {:?} → {}", cap, primal.name);
        }

        // Match optional capabilities
        for cap in &self.profile.optional_capabilities {
            if let Some(primal) = primals.iter().find(|p| p.provides(cap)) {
                matches.push(PrimalMatch {
                    capability: cap.clone(),
                    primal: primal.clone(),
                    required: false,
                });

                println!("    ✅ {:?} → {} (optional)", cap, primal.name);
            } else {
                println!("    ⚠️  {:?} - no primal available (optional, skipping)", cap);
            }
        }

        Ok(matches)
    }

    /// Start matched primals on a VM
    pub async fn start_primals(
        &self,
        vm: &DeployedVm,
        matches: &[PrimalMatch],
    ) -> Result<Vec<StartedPrimal>> {
        println!("  🚀 Starting primals on {}...", vm.name);

        let mut started = Vec::new();

        for m in matches {
            println!("    Starting {} (for {:?})...", m.primal.name, m.capability);

            // Start primal in background
            let start_cmd = format!(
                "cd /opt/biomeos/primalBins && nohup ./{} orchestrate > /tmp/{}.log 2>&1 & echo $!",
                m.primal.name, m.primal.name
            );

            let output = vm
                .ssh_exec(&start_cmd)
                .with_context(|| format!("Failed to start {}", m.primal.name))?;

            let pid = output.trim().parse::<u32>().unwrap_or(0);

            // Small delay for startup
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Verify it's running
            let check_cmd = format!("ps -p {} > /dev/null && echo 'running'", pid);
            let status = vm.ssh_exec(&check_cmd).unwrap_or_default();

            if status.contains("running") {
                println!("      ✅ {} started (PID: {})", m.primal.name, pid);
                started.push(StartedPrimal {
                    primal: m.primal.clone(),
                    pid,
                });
            } else {
                let msg = format!("{} failed to start", m.primal.name);
                if m.required {
                    anyhow::bail!(msg);
                } else {
                    println!("      ⚠️  {} (optional)", msg);
                }
            }
        }

        Ok(started)
    }
}

/// A discovered primal on a VM
#[derive(Debug, Clone)]
pub struct DiscoveredPrimal {
    pub name: String,
    pub path: String,
    pub capabilities: Vec<Capability>,
}

impl DiscoveredPrimal {
    /// Check if this primal provides a capability
    #[must_use]
    pub fn provides(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }
}

/// A matched primal for a capability
#[derive(Debug, Clone)]
pub struct PrimalMatch {
    pub capability: Capability,
    pub primal: DiscoveredPrimal,
    pub required: bool,
}

/// A started primal on a VM
#[derive(Debug, Clone)]
pub struct StartedPrimal {
    pub primal: DiscoveredPrimal,
    pub pid: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_capabilities() {
        let profile = CapabilityProfile::minimal_federation();
        let startup = PrimalStartup::new(profile);

        let caps = startup.infer_capabilities("songbird");
        assert_eq!(caps.len(), 1);
        assert!(matches!(caps[0], Capability::P2PCoordination));
    }

    #[test]
    fn test_discovered_primal_provides() {
        let primal = DiscoveredPrimal {
            name: "songbird".to_string(),
            path: "/opt/biomeos/primalBins/songbird".to_string(),
            capabilities: vec![Capability::P2PCoordination],
        };

        assert!(primal.provides(&Capability::P2PCoordination));
        assert!(!primal.provides(&Capability::Identity));
    }
}

