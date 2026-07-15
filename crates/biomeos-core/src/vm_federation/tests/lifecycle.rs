// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::Result;

use super::VmFederationManager;

#[tokio::test]
#[ignore = "Requires benchscale VM harness and libvirt"]
async fn test_full_lifecycle() -> Result<()> {
    // Only run if benchscale is available AND libvirt testing is enabled
    if std::env::var("BENCHSCALE_TEST_LIBVIRT").is_err() {
        // Skip test if libvirt testing not enabled
        return Ok(());
    }

    let Ok(manager) = VmFederationManager::new() else {
        return Ok(());
    };

    let name = "test-federation";

    // This would actually create VMs if libvirt is available
    manager.create(name).await?;
    manager.start(name)?;
    manager.test(name)?;
    manager.stop(name)?;
    manager.destroy(name)?;
    Ok(())
}
