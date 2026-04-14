// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! DNS and IPAM manifest types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network DNS specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDnsSpec {
    /// Nameservers
    pub nameservers: Vec<String>,

    /// Search domains
    pub search: Vec<String>,

    /// Options
    pub options: Vec<DnsOptionSpec>,
}

/// DNS option specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsOptionSpec {
    /// Name
    pub name: String,

    /// Value
    pub value: Option<String>,
}

/// IPAM (IP Address Management) specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpamSpec {
    /// IPAM driver
    pub driver: String,

    /// Configuration
    pub config: Vec<IpamConfigSpec>,

    /// Options
    pub options: HashMap<String, String>,
}

/// IPAM configuration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpamConfigSpec {
    /// Subnet
    pub subnet: String,

    /// IP range
    pub ip_range: Option<String>,

    /// Gateway
    pub gateway: Option<String>,

    /// Auxiliary addresses
    pub aux_addresses: HashMap<String, String>,
}
