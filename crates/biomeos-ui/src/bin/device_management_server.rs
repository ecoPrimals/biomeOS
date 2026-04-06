// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! biomeOS Device Management Server
//!
//! JSON-RPC server that provides device.management capability for petalTongue integration.
//!
//! This server:
//! - Discovers devices and primals from the running system
//! - Provides JSON-RPC 2.0 API over Unix socket
//! - Advertises device.management capability via Songbird UDP multicast
//! - Serves live data to petalTongue GUI
//!
//! EVOLVED (Jan 27, 2026): Integrated with Songbird for capability advertisement
//!
//! Implementation: [`biomeos_ui::device_management_server`].

use anyhow::Result;
use biomeos_ui::device_management_server;

#[tokio::main]
async fn main() -> Result<()> {
    device_management_server::run().await
}
