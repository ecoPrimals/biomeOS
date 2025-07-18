//! Federation-Wide Optimization System
//!
//! This module implements intelligent resource optimization and load balancing
//! across all primals in the biomeOS federation, enabling efficient resource
//! utilization and optimal performance distribution.
//!
//! # Architecture
//!
//! The federation optimization system is organized into several key modules:
//!
//! - **types**: Core types and structures for optimization
//! - **optimizer**: Main optimization coordinator and logic
//! - **resource_state**: Federation resource state management
//! - **load_balancer**: Load balancing strategies and implementation
//! - **resource_predictor**: Predictive analytics for resource demand
//!
//! # Usage
//!
//! ```rust
//! use biomeos_core::federation_optimization::{
//!     FederationOptimizer, OptimizationConfig, LoadBalancingStrategy
//! };
//!
//! let config = OptimizationConfig::default();
//! let optimizer = FederationOptimizer::new(coordinator, service_registry, config);
//! optimizer.start_optimization().await?;
//! ```

pub mod types;
pub mod optimizer;
pub mod resource_state;
pub mod load_balancer;
pub mod resource_predictor;

// Re-export main types for convenience
pub use types::*;
pub use optimizer::{FederationOptimizer, ResourceAnalysis, OptimizationRecommendation};
pub use resource_state::FederationResourceState;
pub use load_balancer::{FederationLoadBalancer, HealthTracker, LoadBalancingRequest};
pub use resource_predictor::{ResourcePredictor, ResourcePrediction}; 