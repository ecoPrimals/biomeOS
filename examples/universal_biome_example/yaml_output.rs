//! YAML Output Example
//!
//! This module demonstrates how the universal biome manifest would look in YAML format.

use tracing::info;

/// Show example YAML output for universal biome manifest
pub fn example_yaml_output() -> Result<(), Box<dyn std::error::Error>> {
    // Example of how the universal manifest would look in YAML
    let yaml_example = r#"
apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: web-application
  description: A scalable web application
  version: 1.0.0
  tags:
    - web
    - application
    - scalable
  labels:
    environment: production
    tier: frontend

requirements:
  # No hard-coded primal names - just capabilities!
  required:
    - capability: compute.container_orchestration
      min_version: 1.0.0
      constraints:
        - type: performance
          value: {min_replicas: 2}
    - capability: networking.load_balancing
      min_version: 1.0.0
      fallback: networking.basic_routing
    - capability: storage.persistent_volumes
      min_version: 1.0.0
      
  optional:
    - capability: storage.caching
      min_version: 1.0.0
    - capability: monitoring.metrics_collection
      min_version: 1.0.0

deployment:
  strategy: reliability
  # These are preferences, not requirements
  preferred_primals:
    - primal_type: kubernetes_operator
      weight: 0.8
    - primal_type: container_orchestrator
      weight: 0.6
      
services:
  - name: web-frontend
    description: React frontend application
    required_capabilities:
      - capability: compute.container_orchestration
        min_version: 1.0.0
    config:
      source: myapp/frontend:v1.0.0
      runtime:
        type: container
        version: latest
      environment:
        NODE_ENV: production
        API_URL: http://api:3000
"#;

    info!("Example YAML structure:");
    info!("{}", yaml_example);

    Ok(())
} 