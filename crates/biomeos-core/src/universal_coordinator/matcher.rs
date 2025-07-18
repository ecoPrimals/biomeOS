//! Requirement matcher for matching requirements to primals
//!
//! This module contains the requirement matcher implementation and matching
//! algorithms for finding the best primals for given requirements.

use std::collections::HashMap;
use async_trait::async_trait;

use crate::universal_primal::{CapabilityRequirement, DiscoveredPrimal};
use crate::BiomeResult;
use super::types::{
    RequirementMatcher, MatchingAlgorithm, MatchingConfig, MatchResult, MatchDetails,
};

impl RequirementMatcher {
    /// Create new requirement matcher
    pub fn new() -> Self {
        Self {
            algorithms: vec![Box::new(SimpleMatchingAlgorithm::default())],
            config: MatchingConfig::default(),
        }
    }

    /// Create matcher with specific configuration
    pub fn with_config(config: MatchingConfig) -> Self {
        Self {
            algorithms: vec![Box::new(SimpleMatchingAlgorithm::default())],
            config,
        }
    }

    /// Add a matching algorithm
    pub fn add_algorithm(&mut self, algorithm: Box<dyn MatchingAlgorithm>) {
        self.algorithms.push(algorithm);
    }

    /// Set configuration
    pub fn set_config(&mut self, config: MatchingConfig) {
        self.config = config;
    }

    /// Get configuration
    pub fn get_config(&self) -> &MatchingConfig {
        &self.config
    }

    /// Match requirements to primals
    pub async fn match_requirements(
        &self,
        requirements: &[CapabilityRequirement],
        primals: &[DiscoveredPrimal],
    ) -> BiomeResult<Vec<MatchResult>> {
        let mut all_results = Vec::new();

        // Run all matching algorithms
        for algorithm in &self.algorithms {
            let results = algorithm
                .match_requirements(requirements, primals, &self.config)
                .await?;
            all_results.extend(results);
        }

        // Sort by score (descending)
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Filter by minimum score
        let filtered_results: Vec<_> = all_results
            .into_iter()
            .filter(|result| result.score >= self.config.min_score)
            .collect();

        Ok(filtered_results)
    }

    /// Find best match for requirements
    pub async fn find_best_match(
        &self,
        requirements: &[CapabilityRequirement],
        primals: &[DiscoveredPrimal],
    ) -> BiomeResult<Option<MatchResult>> {
        let results = self.match_requirements(requirements, primals).await?;
        Ok(results.into_iter().next())
    }

    /// Get available algorithms
    pub fn get_algorithms(&self) -> Vec<&str> {
        self.algorithms.iter().map(|alg| alg.name()).collect()
    }
}

impl Default for RequirementMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple matching algorithm implementation
pub struct SimpleMatchingAlgorithm {
    name: String,
}

impl Default for SimpleMatchingAlgorithm {
    fn default() -> Self {
        Self {
            name: "simple".to_string(),
        }
    }
}

impl SimpleMatchingAlgorithm {
    /// Create new simple matching algorithm
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate capability match score
    fn calculate_capability_score(
        &self,
        requirements: &[CapabilityRequirement],
        primal: &DiscoveredPrimal,
    ) -> f64 {
        if requirements.is_empty() {
            return 1.0;
        }

        let mut total_score = 0.0;
        let mut _matched_count = 0;

        for requirement in requirements {
            for capability in &primal.capabilities {
                if capability.name == requirement.capability {
                    total_score += 1.0;
                    _matched_count += 1;
                    break;
                }
            }
        }

        if requirements.len() > 0 {
            total_score / requirements.len() as f64
        } else {
            0.0
        }
    }

    /// Calculate version match score
    fn calculate_version_score(
        &self,
        requirements: &[CapabilityRequirement],
        primal: &DiscoveredPrimal,
    ) -> f64 {
        // Simple version matching - can be enhanced
        if requirements.is_empty() {
            return 1.0;
        }

        let mut total_score = 0.0;
        let mut _matched_count = 0;

        for requirement in requirements {
            for capability in &primal.capabilities {
                if capability.name == requirement.capability {
                    // For now, just check if versions match exactly
                    if &requirement.min_version == &capability.version {
                        total_score += 1.0;
                    } else {
                        total_score += 0.5; // Partial match
                    }
                    _matched_count += 1;
                    break;
                }
            }
        }

        if _matched_count > 0 {
            total_score / _matched_count as f64
        } else {
            0.0
        }
    }

    /// Calculate performance match score
    fn calculate_performance_score(&self, primal: &DiscoveredPrimal) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Factor 1: Capability performance specifications
        for capability in &primal.capabilities {
            let perf_spec = &capability.performance;
            
            // Latency scoring
            if let Some((min_latency, max_latency)) = perf_spec.latency_ms {
                // Lower latency = higher score
                let avg_latency = (min_latency + max_latency) as f64 / 2.0;
                let latency_score = if avg_latency <= 100.0 {
                    1.0
                } else if avg_latency <= 500.0 {
                    0.8 - (avg_latency - 100.0) / 400.0 * 0.3
                } else if avg_latency <= 1000.0 {
                    0.5 - (avg_latency - 500.0) / 500.0 * 0.3
                } else {
                    0.2
                };
                score += latency_score;
                factors += 1;
            }

            // Throughput scoring
            if let Some(throughput) = &perf_spec.throughput {
                let throughput_score = match throughput.as_str() {
                    "high" => 1.0,
                    "medium" => 0.7,
                    "low" => 0.4,
                    _ => {
                        // Try to parse numeric throughput
                        if let Ok(tps) = throughput.parse::<f64>() {
                            // Scale based on requests per second
                            if tps >= 1000.0 {
                                1.0
                            } else if tps >= 100.0 {
                                0.7 + (tps - 100.0) / 900.0 * 0.3
                            } else if tps >= 10.0 {
                                0.4 + (tps - 10.0) / 90.0 * 0.3
                            } else {
                                0.2
                            }
                        } else {
                            0.5 // Default for unknown throughput
                        }
                    }
                };
                score += throughput_score;
                factors += 1;
            }

            // Scaling capability scoring
            let scaling_score = if perf_spec.scaling.auto_scaling {
                let max_instances = perf_spec.scaling.max_instances;
                if max_instances >= 100 {
                    1.0
                } else if max_instances >= 10 {
                    0.7 + (max_instances as f64 - 10.0) / 90.0 * 0.3
                } else if max_instances >= 2 {
                    0.5 + (max_instances as f64 - 2.0) / 8.0 * 0.2
                } else {
                    0.3
                }
            } else {
                0.4 // Manual scaling penalty
            };
            score += scaling_score;
            factors += 1;
        }

        // Factor 2: Historical performance (simulated based on discovery time)
        let discovery_age = chrono::Utc::now()
            .signed_duration_since(primal.discovered_at)
            .num_hours();
        
        let stability_score = if discovery_age < 1 {
            0.6 // New primal, unknown stability
        } else if discovery_age < 24 {
            0.7 // Recent, building trust
        } else if discovery_age < 168 { // 1 week
            0.8 // Established, good track record
        } else {
            0.9 // Long-running, very stable
        };
        score += stability_score;
        factors += 1;

        // Factor 3: Metadata-based performance hints
        let metadata_score = if let Some(performance_class) = primal.metadata.tags.get("performance_class") {
            match performance_class.as_str() {
                "high" => 1.0,
                "medium" => 0.7,
                "low" => 0.4,
                _ => 0.6,
            }
        } else {
            0.6 // Default for unknown performance class
        };
        score += metadata_score;
        factors += 1;

        if factors > 0 {
            score / factors as f64
        } else {
            0.5 // Default fallback
        }
    }

    /// Calculate resource match score
    fn calculate_resource_score(&self, primal: &DiscoveredPrimal) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Factor 1: Resource availability from capabilities
        for capability in &primal.capabilities {
            let resources = &capability.performance.resource_requirements;
            
            // CPU scoring
            if let Some(cpu) = &resources.cpu {
                let cpu_score = match cpu.as_str() {
                    "high" => 1.0,
                    "medium" => 0.7,
                    "low" => 0.4,
                    _ => {
                        // Try to parse numeric CPU requirements
                        if let Ok(cpu_cores) = cpu.parse::<f64>() {
                            if cpu_cores >= 8.0 {
                                1.0
                            } else if cpu_cores >= 4.0 {
                                0.7 + (cpu_cores - 4.0) / 4.0 * 0.3
                            } else if cpu_cores >= 2.0 {
                                0.5 + (cpu_cores - 2.0) / 2.0 * 0.2
                            } else {
                                0.3
                            }
                        } else {
                            0.6 // Default for unknown CPU
                        }
                    }
                };
                score += cpu_score;
                factors += 1;
            }

            // Memory scoring
            if let Some(memory) = &resources.memory {
                let memory_score = match memory.as_str() {
                    "high" => 1.0,
                    "medium" => 0.7,
                    "low" => 0.4,
                    _ => {
                        // Try to parse memory requirements (e.g., "8GB", "2048MB")
                        if memory.ends_with("GB") {
                            if let Ok(gb) = memory.strip_suffix("GB").unwrap().parse::<f64>() {
                                if gb >= 16.0 {
                                    1.0
                                } else if gb >= 8.0 {
                                    0.8 + (gb - 8.0) / 8.0 * 0.2
                                } else if gb >= 4.0 {
                                    0.6 + (gb - 4.0) / 4.0 * 0.2
                                } else {
                                    0.4
                                }
                            } else {
                                0.5
                            }
                        } else if memory.ends_with("MB") {
                            if let Ok(mb) = memory.strip_suffix("MB").unwrap().parse::<f64>() {
                                let gb = mb / 1024.0;
                                if gb >= 16.0 {
                                    1.0
                                } else if gb >= 8.0 {
                                    0.8 + (gb - 8.0) / 8.0 * 0.2
                                } else if gb >= 4.0 {
                                    0.6 + (gb - 4.0) / 4.0 * 0.2
                                } else {
                                    0.4
                                }
                            } else {
                                0.5
                            }
                        } else {
                            0.6 // Default for unknown memory format
                        }
                    }
                };
                score += memory_score;
                factors += 1;
            }

            // Storage scoring
            if let Some(storage) = &resources.storage {
                let storage_score = match storage.as_str() {
                    "high" => 1.0,
                    "medium" => 0.7,
                    "low" => 0.4,
                    _ => {
                        // Try to parse storage requirements
                        if storage.ends_with("TB") {
                            if let Ok(tb) = storage.strip_suffix("TB").unwrap().parse::<f64>() {
                                if tb >= 10.0 {
                                    1.0
                                } else if tb >= 1.0 {
                                    0.7 + (tb - 1.0) / 9.0 * 0.3
                                } else {
                                    0.5
                                }
                            } else {
                                0.5
                            }
                        } else if storage.ends_with("GB") {
                            if let Ok(gb) = storage.strip_suffix("GB").unwrap().parse::<f64>() {
                                let tb = gb / 1024.0;
                                if tb >= 10.0 {
                                    1.0
                                } else if tb >= 1.0 {
                                    0.7 + (tb - 1.0) / 9.0 * 0.3
                                } else {
                                    0.5
                                }
                            } else {
                                0.5
                            }
                        } else {
                            0.6 // Default for unknown storage format
                        }
                    }
                };
                score += storage_score;
                factors += 1;
            }

            // GPU scoring
            if let Some(gpu) = &resources.gpu {
                let gpu_score = match gpu.as_str() {
                    "high" => 1.0,
                    "medium" => 0.8,
                    "low" => 0.5,
                    "none" => 0.0,
                    _ => {
                        // Try to parse GPU count
                        if let Ok(gpu_count) = gpu.parse::<u32>() {
                            if gpu_count >= 4 {
                                1.0
                            } else if gpu_count >= 2 {
                                0.8 + (gpu_count as f64 - 2.0) / 2.0 * 0.2
                            } else if gpu_count >= 1 {
                                0.6
                            } else {
                                0.0
                            }
                        } else {
                            0.7 // Default for unknown GPU
                        }
                    }
                };
                score += gpu_score;
                factors += 1;
            }
        }

        // Factor 2: Metadata-based resource hints
        if let Some(resource_class) = primal.metadata.tags.get("resource_class") {
            let resource_class_score = match resource_class.as_str() {
                "high" => 1.0,
                "medium" => 0.7,
                "low" => 0.4,
                _ => 0.6,
            };
            score += resource_class_score;
            factors += 1;
        }

        if factors > 0 {
            score / factors as f64
        } else {
            0.5 // Default fallback
        }
    }

    /// Calculate availability match score
    fn calculate_availability_score(&self, primal: &DiscoveredPrimal) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Factor 1: Discovery freshness (health indicator)
        let discovery_age = chrono::Utc::now()
            .signed_duration_since(primal.discovered_at)
            .num_minutes();
        
        let freshness_score = if discovery_age < 5 {
            1.0 // Very fresh discovery
        } else if discovery_age < 30 {
            0.9 - (discovery_age as f64 - 5.0) / 25.0 * 0.1
        } else if discovery_age < 60 {
            0.8 - (discovery_age as f64 - 30.0) / 30.0 * 0.2
        } else if discovery_age < 300 { // 5 minutes
            0.6 - (discovery_age as f64 - 60.0) / 240.0 * 0.2
        } else {
            0.4 // Potentially stale
        };
        score += freshness_score;
        factors += 1;

        // Factor 2: Endpoint availability
        let endpoint_score = if primal.endpoints.is_empty() {
            0.2 // No endpoints, likely unavailable
        } else {
            let healthy_endpoints = primal.endpoints.len();
            if healthy_endpoints >= 3 {
                1.0 // Multiple endpoints, high availability
            } else if healthy_endpoints >= 2 {
                0.8 // Good redundancy
            } else {
                0.6 // Single endpoint
            }
        };
        score += endpoint_score;
        factors += 1;

        // Factor 3: Scaling availability
        let scaling_availability = primal.capabilities.iter().map(|cap| {
            let scaling = &cap.performance.scaling;
            if scaling.auto_scaling {
                if scaling.max_instances >= 10 {
                    1.0 // High scaling capacity
                } else if scaling.max_instances >= 3 {
                    0.8 // Medium scaling
                } else {
                    0.6 // Limited scaling
                }
            } else {
                0.5 // No auto-scaling
            }
        }).fold(0.0, |acc, x| acc + x) / primal.capabilities.len() as f64;
        
        score += scaling_availability;
        factors += 1;

        // Factor 4: Metadata-based availability hints
        if let Some(availability_class) = primal.metadata.tags.get("availability_class") {
            let availability_class_score = match availability_class.as_str() {
                "high" => 1.0,
                "medium" => 0.7,
                "low" => 0.4,
                _ => 0.6,
            };
            score += availability_class_score;
            factors += 1;
        }

        // Factor 5: Uptime indicator (based on discovery source)
        let uptime_score = match primal.discovery_source.as_str() {
            "direct" => 1.0,      // Direct discovery = high confidence
            "registry" => 0.9,    // Registry = good confidence
            "gossip" => 0.7,      // Gossip = medium confidence
            "cached" => 0.5,      // Cached = lower confidence
            _ => 0.6,             // Unknown source
        };
        score += uptime_score;
        factors += 1;

        if factors > 0 {
            score / factors as f64
        } else {
            0.5 // Default fallback
        }
    }
}

#[async_trait]
impl MatchingAlgorithm for SimpleMatchingAlgorithm {
    fn name(&self) -> &str {
        &self.name
    }

    async fn match_requirements(
        &self,
        requirements: &[CapabilityRequirement],
        primals: &[DiscoveredPrimal],
        config: &MatchingConfig,
    ) -> BiomeResult<Vec<MatchResult>> {
        let mut results = Vec::new();

        for primal in primals {
            let capability_score = self.calculate_capability_score(requirements, primal);
            let version_score = self.calculate_version_score(requirements, primal);
            let performance_score = self.calculate_performance_score(primal);
            let resource_score = self.calculate_resource_score(primal);
            let availability_score = self.calculate_availability_score(primal);

            // Calculate weighted total score
            let total_score = capability_score * config.weights.capability_match
                + version_score * config.weights.version_match
                + performance_score * config.weights.performance_match
                + resource_score * config.weights.resource_match
                + availability_score * config.weights.availability_match;

            let mut capability_matches = HashMap::new();
            for requirement in requirements {
                for capability in &primal.capabilities {
                    if capability.name == requirement.capability {
                        capability_matches.insert(capability.name.clone(), 1.0);
                        break;
                    }
                }
            }

            let details = MatchDetails {
                capability_matches,
                version_match: version_score,
                performance_match: performance_score,
                resource_match: resource_score,
                availability_match: availability_score,
            };

            results.push(MatchResult {
                primal: primal.clone(),
                score: total_score,
                details,
            });
        }

        Ok(results)
    }
} 