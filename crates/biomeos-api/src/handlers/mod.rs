// API handlers module

#[allow(dead_code)] // Route handlers ready to wire into axum Router
pub mod capability;
pub mod discovery;
pub mod events; // Real-time SSE events
#[allow(dead_code)] // Route handlers ready to wire into axum Router
pub mod genome;
pub mod health; // Health check endpoints
#[allow(dead_code)] // Discovery utilities used by other handlers
pub mod live_discovery;
pub mod livespores; // LiveSpore USB device discovery
pub mod rendezvous; // Dark Forest rendezvous for Pixel-USB handshake
pub mod topology;
pub mod trust; // ✅ Trust handlers using Universal Primal Client
