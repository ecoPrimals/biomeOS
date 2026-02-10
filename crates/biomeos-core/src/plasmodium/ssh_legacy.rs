//! SSH-based remote gate query — **DEPRECATED** legacy transport.
//!
//! Prefer Songbird mesh RPC via `Plasmodium::query_remote_gate()` which uses
//! capability-based discovery. This SSH fallback will be removed when all
//! gates support mesh discovery.

use anyhow::{Context, Result};
use serde_json::Value;
use tracing::info;

use super::types::{BondType, ComputeInfo, GateInfo, GpuInfo, PrimalStatus};

/// Query a remote gate's NUCLEUS status via SSH.
///
/// **DEPRECATED**: This method uses `ssh` shell-out. Prefer Songbird mesh RPC
/// via `query_remote_gate()` which uses capability-based discovery.
///
/// Evolution path: Replace with `capability.call("topology.proprioception")`
/// routed through Songbird mesh relay. When all gates support mesh discovery,
/// this SSH fallback can be removed entirely.
///
/// Executes lightweight system commands over SSH to collect:
/// - GPU info (from /proc/driver/nvidia/)
/// - RAM/CPU info (/proc)
/// - Running primals (pgrep)
/// - System load (/proc/loadavg)
/// - NestGate storage test
#[deprecated(note = "Use Songbird mesh RPC (query_remote_gate) when available")]
pub(crate) async fn query_remote_gate_ssh(ssh_target: &str, node_id: &str) -> Result<GateInfo> {
    info!("Querying remote gate {} via SSH ({})", node_id, ssh_target);

    // Collect all info in one SSH command to minimize round trips
    let script = r#"
echo "===GPU==="
nvidia-smi --query-gpu=name,memory.total --format=csv,noheader,nounits 2>/dev/null || echo "NONE"
echo "===RAM==="
grep MemTotal /proc/meminfo | awk '{print $2}'
echo "===CPU==="
nproc
echo "===LOAD==="
cat /proc/loadavg | awk '{print $1}'
echo "===PRIMALS==="
for p in beardog songbird nestgate toadstool squirrel; do
  if pgrep -x $p > /dev/null 2>&1 || pgrep -f "$p server" > /dev/null 2>&1 || pgrep -f "$p daemon" > /dev/null 2>&1; then
    echo "$p:running"
  else
    echo "$p:stopped"
  fi
done
echo "===NESTGATE==="
echo '{"jsonrpc":"2.0","method":"storage.list","params":{"family_id":"default","prefix":"model-cache:"},"id":1}' | nc -U /run/user/$(id -u)/biomeos/nestgate.sock -w 2 -q 1 2>/dev/null || echo "UNREACHABLE"
echo "===HOSTNAME==="
hostname
echo "===END==="
"#;

    let output = tokio::process::Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=5",
            "-o",
            "BatchMode=yes",
            ssh_target,
            script,
        ])
        .output()
        .await
        .context(format!("SSH to {} failed", ssh_target))?;

    if !output.status.success() {
        anyhow::bail!(
            "SSH to {} exited with {}: {}",
            ssh_target,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse sections
    let mut gpus = Vec::new();
    let mut ram_gb = 0u64;
    let mut cpu_cores = 0usize;
    let mut load = 0.0f64;
    let mut primals = Vec::new();
    let mut models = Vec::new();
    let mut hostname = node_id.to_string();
    let mut section = "";

    for line in stdout.lines() {
        match line.trim() {
            "===GPU===" => {
                section = "gpu";
                continue;
            }
            "===RAM===" => {
                section = "ram";
                continue;
            }
            "===CPU===" => {
                section = "cpu";
                continue;
            }
            "===LOAD===" => {
                section = "load";
                continue;
            }
            "===PRIMALS===" => {
                section = "primals";
                continue;
            }
            "===NESTGATE===" => {
                section = "nestgate";
                continue;
            }
            "===HOSTNAME===" => {
                section = "hostname";
                continue;
            }
            "===END===" => break,
            _ => {}
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match section {
            "gpu" => {
                if line != "NONE" {
                    let parts: Vec<&str> = line.split(", ").collect();
                    if parts.len() >= 2 {
                        gpus.push(GpuInfo {
                            name: parts[0].to_string(),
                            vram_mb: parts[1].parse().unwrap_or(0),
                            gate_id: node_id.to_string(),
                        });
                    }
                }
            }
            "ram" => {
                ram_gb = line.parse::<u64>().unwrap_or(0) / 1_048_576;
            }
            "cpu" => {
                cpu_cores = line.parse().unwrap_or(0);
            }
            "load" => {
                let load_1m: f64 = line.parse().unwrap_or(0.0);
                let cores = if cpu_cores > 0 { cpu_cores as f64 } else { 1.0 };
                load = (load_1m / cores).min(1.0);
            }
            "primals" => {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    primals.push(PrimalStatus {
                        name: parts[0].to_string(),
                        healthy: parts[1] == "running",
                        version: None,
                    });
                }
            }
            "nestgate" => {
                if line != "UNREACHABLE" {
                    // Parse NestGate response for model keys
                    if let Ok(response) = serde_json::from_str::<Value>(line) {
                        if let Some(keys) = response
                            .get("result")
                            .and_then(|r| r.get("keys"))
                            .and_then(|k| k.as_array())
                        {
                            for key in keys {
                                if let Some(k) = key.as_str() {
                                    if let Some(model_id) = k.strip_prefix("model-cache:") {
                                        models.push(model_id.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "hostname" => {
                hostname = line.to_string();
            }
            _ => {}
        }
    }

    let address = ssh_target
        .split('@')
        .next_back()
        .unwrap_or(ssh_target)
        .to_string();

    // Use the node_id from PLASMODIUM_PEERS (e.g. "gate2") rather than remote hostname
    // The remote hostname might be the same as local (e.g. both "pop-os")
    let display_id = if node_id != hostname {
        format!("{node_id} ({hostname})")
    } else {
        node_id.to_string()
    };

    Ok(GateInfo {
        gate_id: display_id,
        address,
        is_local: false,
        primals,
        compute: ComputeInfo {
            gpus,
            ram_gb,
            cpu_cores,
        },
        models,
        load,
        reachable: true,
        bond_type: BondType::Covalent,
    })
}
