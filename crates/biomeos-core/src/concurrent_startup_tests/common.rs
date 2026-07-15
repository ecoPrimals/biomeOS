// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::capabilities::Capability;
use crate::discovery_modern::HealthStatus;
use crate::primal_orchestrator::ManagedPrimal;
use biomeos_types::PrimalId;
use biomeos_types::error::BiomeResult;
use biomeos_types::identifiers::Endpoint;
use std::future::Future;
use std::pin::Pin;

pub(crate) struct MockPrimal {
    pub id: PrimalId,
    pub provides: Vec<Capability>,
    pub requires: Vec<Capability>,
}

impl MockPrimal {
    pub fn new(name: &str, provides: Vec<Capability>, requires: Vec<Capability>) -> Self {
        Self {
            id: PrimalId::new(name).expect("valid primal name"),
            provides,
            requires,
        }
    }
}

impl ManagedPrimal for MockPrimal {
    fn id(&self) -> &PrimalId {
        &self.id
    }
    fn provides(&self) -> &[Capability] {
        &self.provides
    }
    fn requires(&self) -> &[Capability] {
        &self.requires
    }
    fn endpoint(&self) -> Pin<Box<dyn Future<Output = Option<Endpoint>> + Send + '_>> {
        Box::pin(async move { None })
    }
    fn start(&self) -> Pin<Box<dyn Future<Output = BiomeResult<()>> + Send + '_>> {
        Box::pin(async move { Ok(()) })
    }
    fn stop(&self) -> Pin<Box<dyn Future<Output = BiomeResult<()>> + Send + '_>> {
        Box::pin(async move { Ok(()) })
    }
    fn health_check(&self) -> Pin<Box<dyn Future<Output = BiomeResult<HealthStatus>> + Send + '_>> {
        Box::pin(async move { Ok(HealthStatus::Healthy) })
    }
}

pub(crate) fn pid(name: &str) -> PrimalId {
    PrimalId::new(name).expect("valid primal id")
}
