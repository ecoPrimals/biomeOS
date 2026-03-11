// API handlers module

pub mod capability; // Capability discovery and management
pub mod discovery;
pub mod events; // Real-time SSE events
pub mod genome; // GenomeBin build/compose/verify
pub mod genome_dist; // Genome distribution API (wateringHole/genomeBin)
pub mod health; // Health check endpoints
#[allow(dead_code)] // Utility module: pub functions for discovery routes and future REST API
pub mod live_discovery;
pub mod livespores; // LiveSpore USB device discovery
pub mod rendezvous; // Dark Forest rendezvous for Pixel-USB handshake
pub mod topology;
pub mod trust; // Trust handlers using Universal Primal Client
