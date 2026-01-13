//! Caching for schemas and format hints

use std::collections::HashMap;
use std::time::Instant;

use crate::primal_client::handle::{FormatHint, PrimalHandle, PrimalId};
use crate::primal_client::schema::ApiSchema;

/// Client cache
#[derive(Debug)]
pub struct ClientCache {
    /// Cached schemas with timestamps
    schemas: HashMap<PrimalId, (ApiSchema, Instant)>,

    /// Cached format hints with timestamps
    formats: HashMap<PrimalId, (FormatHint, Instant)>,

    /// Cached discovered primals with timestamps
    primals: HashMap<String, (Vec<PrimalHandle>, Instant)>,
}

impl ClientCache {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            formats: HashMap::new(),
            primals: HashMap::new(),
        }
    }

    /// Get cached schema if not expired
    pub fn get_schema(&self, primal_id: &PrimalId, ttl: std::time::Duration) -> Option<&ApiSchema> {
        self.schemas.get(primal_id).and_then(|(schema, timestamp)| {
            if timestamp.elapsed() < ttl {
                Some(schema)
            } else {
                None
            }
        })
    }

    /// Cache schema
    pub fn set_schema(&mut self, primal_id: PrimalId, schema: ApiSchema) {
        self.schemas.insert(primal_id, (schema, Instant::now()));
    }

    /// Get cached format hint if not expired
    pub fn get_format(&self, primal_id: &PrimalId, ttl: std::time::Duration) -> Option<FormatHint> {
        self.formats.get(primal_id).and_then(|(hint, timestamp)| {
            if timestamp.elapsed() < ttl {
                Some(*hint)
            } else {
                None
            }
        })
    }

    /// Cache format hint
    pub fn set_format(&mut self, primal_id: PrimalId, hint: FormatHint) {
        self.formats.insert(primal_id, (hint, Instant::now()));
    }

    /// Get cached primals if not expired
    pub fn get_primals(
        &self,
        capability: &str,
        ttl: std::time::Duration,
    ) -> Option<&Vec<PrimalHandle>> {
        self.primals
            .get(capability)
            .and_then(|(primals, timestamp)| {
                if timestamp.elapsed() < ttl {
                    Some(primals)
                } else {
                    None
                }
            })
    }

    /// Cache discovered primals
    pub fn set_primals(&mut self, capability: String, primals: Vec<PrimalHandle>) {
        self.primals.insert(capability, (primals, Instant::now()));
    }

    /// Clear expired entries
    pub fn clear_expired(&mut self, ttl: std::time::Duration) {
        self.schemas
            .retain(|_, (_, timestamp)| timestamp.elapsed() < ttl);
        self.formats
            .retain(|_, (_, timestamp)| timestamp.elapsed() < ttl);
        self.primals
            .retain(|_, (_, timestamp)| timestamp.elapsed() < ttl);
    }
}

impl Default for ClientCache {
    fn default() -> Self {
        Self::new()
    }
}
