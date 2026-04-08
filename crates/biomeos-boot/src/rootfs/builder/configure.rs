// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::{Context, Result};
use std::fmt::Write;
use std::path::Path;
use tracing::info;

use crate::rootfs::dns::parse_resolv_conf;

use super::types::RootFsBuilder;

impl RootFsBuilder {
    pub(crate) fn configure_system(&self, root: &Path) -> Result<()> {
        info!("⚙️  Configuring system...");

        self.configure_dns(root)?;

        let hostname_path = root.join("etc/hostname");
        std::fs::write(&hostname_path, format!("{}\n", self.config.hostname))?;

        info!("  ✓ System configured (hostname: {})", self.config.hostname);
        Ok(())
    }

    pub(crate) fn configure_dns(&self, root: &Path) -> Result<()> {
        let resolv_conf = root.join("etc/resolv.conf");

        let dns_servers = if let Some(ref servers) = self.config.dns_servers {
            servers.clone()
        } else {
            Self::discover_system_dns()?
        };

        if dns_servers.is_empty() {
            info!("  Using system DNS configuration");
            return Ok(());
        }

        let mut content = String::new();
        for server in &dns_servers {
            writeln!(content, "nameserver {server}")
                .map_err(|e| anyhow::anyhow!("Failed to write DNS config: {e}"))?;
        }

        std::fs::write(&resolv_conf, content)?;
        info!("  ✓ DNS configured ({} servers)", dns_servers.len());

        Ok(())
    }

    pub(crate) fn discover_system_dns() -> Result<Vec<String>> {
        let system_resolv = std::fs::read_to_string("/etc/resolv.conf")
            .context("Failed to read /etc/resolv.conf")?;

        Ok(parse_resolv_conf(&system_resolv))
    }
}
