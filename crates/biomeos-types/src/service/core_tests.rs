// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project
//
// Test module for service/core.rs - included via #[path]

#![allow(clippy::unwrap_used)]

use super::*;
use crate::primal::PrimalCapability;
use crate::primal::PrimalType;
use std::collections::HashMap;

fn test_metadata() -> ServiceMetadata {
    let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let now = Utc::now();
    ServiceMetadata {
        id,
        name: "test-service".to_string(),
        namespace: Some("default".to_string()),
        version: "1.2.3".to_string(),
        description: Some("A test service".to_string()),
        author: Some("test-author".to_string()),
        labels: {
            let mut m = HashMap::new();
            m.insert("app".to_string(), "test".to_string());
            m
        },
        annotations: HashMap::new(),
        tags: vec!["web".to_string(), "api".to_string()],
        created_at: now,
        updated_at: now,
        owner_references: vec![],
    }
}

#[test]
fn test_universal_service_serde_json_roundtrip() {
    let service = UniversalService::default();
    let json = serde_json::to_string(&service).unwrap();
    let deserialized: UniversalService = serde_json::from_str(&json).unwrap();
    assert_eq!(service.metadata.name, deserialized.metadata.name);
    assert!(matches!(
        (service.status.phase, deserialized.status.phase),
        (ServicePhase::Pending, ServicePhase::Pending)
    ));
}

#[test]
fn test_universal_service_serde_yaml_roundtrip() {
    let service = UniversalService::default();
    let yaml = serde_yaml::to_string(&service).unwrap();
    let deserialized: UniversalService = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(service.metadata.name, deserialized.metadata.name);
}

#[test]
fn test_universal_service_default() {
    let service = UniversalService::default();
    assert_eq!(service.metadata.name, "default-service");
    assert_eq!(service.metadata.version, "1.0.0");
    assert!(matches!(service.status.phase, ServicePhase::Pending));
    assert_eq!(service.spec.scaling.min_replicas, 1);
    assert_eq!(service.spec.scaling.max_replicas, 1);
    assert_eq!(service.spec.lifecycle.termination_grace_period, 30);
}

#[test]
fn test_service_metadata_serde_json_roundtrip() {
    let metadata = test_metadata();
    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: ServiceMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(metadata.name, deserialized.name);
    assert_eq!(metadata.namespace, deserialized.namespace);
    assert_eq!(metadata.tags, deserialized.tags);
}

#[test]
fn test_owner_reference_serde_json_roundtrip() {
    let owner = OwnerReference {
        api_version: "v1".to_string(),
        kind: "Service".to_string(),
        name: "parent-service".to_string(),
        uid: Uuid::new_v4(),
        controller: true,
        block_owner_deletion: false,
    };
    let json = serde_json::to_string(&owner).unwrap();
    let deserialized: OwnerReference = serde_json::from_str(&json).unwrap();
    assert_eq!(owner.api_version, deserialized.api_version);
    assert_eq!(owner.kind, deserialized.kind);
}

#[test]
fn test_service_type_enum_variants_serde() {
    let types = vec![
        ServiceType::Primal {
            category: "compute".to_string(),
            specialization: Some("gpu".to_string()),
        },
        ServiceType::Application {
            app_type: "web".to_string(),
            framework: Some("actix".to_string()),
        },
        ServiceType::Infrastructure {
            component: "load-balancer".to_string(),
            provider: Some("nginx".to_string()),
        },
        ServiceType::Database {
            engine: "postgres".to_string(),
            version: "15".to_string(),
        },
        ServiceType::MessageQueue {
            system: "rabbitmq".to_string(),
            queue_type: "topic".to_string(),
        },
        ServiceType::Cache {
            system: "redis".to_string(),
            cache_type: "standalone".to_string(),
        },
        ServiceType::LoadBalancer {
            lb_type: "nginx".to_string(),
            algorithm: "round_robin".to_string(),
        },
        ServiceType::ApiGateway {
            gateway_type: "kong".to_string(),
            features: vec!["rate-limiting".to_string()],
        },
        ServiceType::Monitoring {
            system: "prometheus".to_string(),
            component: "exporter".to_string(),
        },
        ServiceType::Security {
            component: "vault".to_string(),
            domain: "secrets".to_string(),
        },
        ServiceType::Custom {
            type_name: "custom".to_string(),
            attributes: {
                let mut m = HashMap::new();
                m.insert("key".to_string(), "value".to_string());
                m
            },
        },
    ];
    for st in types {
        let json = serde_json::to_string(&st).unwrap();
        let deserialized: ServiceType = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(json, json2);
    }
}

#[test]
fn test_service_phase_enum_serde() {
    let phases = [
        ServicePhase::Pending,
        ServicePhase::Starting,
        ServicePhase::Running,
        ServicePhase::Stopping,
        ServicePhase::Stopped,
        ServicePhase::Failed,
        ServicePhase::Unknown,
    ];
    for phase in phases {
        let json = serde_json::to_string(&phase).unwrap();
        let deserialized: ServicePhase = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", phase), format!("{:?}", deserialized));
    }
}

#[test]
fn test_condition_status_enum_serde() {
    let statuses = [
        ConditionStatus::True,
        ConditionStatus::False,
        ConditionStatus::Unknown,
    ];
    for s in statuses {
        let json = serde_json::to_string(&s).unwrap();
        let deserialized: ConditionStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", s), format!("{:?}", deserialized));
    }
}

#[test]
fn test_replica_status_serde_json_roundtrip() {
    let status = ReplicaStatus {
        desired: 3,
        current: 2,
        ready: 2,
        available: 1,
        unavailable: 1,
    };
    let json = serde_json::to_string(&status).unwrap();
    let deserialized: ReplicaStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(status.desired, deserialized.desired);
    assert_eq!(status.current, deserialized.current);
}

#[test]
fn test_service_endpoint_serde_json_roundtrip() {
    let endpoint = ServiceEndpoint {
        name: "http".to_string(),
        address: "127.0.0.1".to_string(),
        port: 8080,
        protocol: EndpointProtocol::Http,
        ready: true,
        metadata: HashMap::new(),
    };
    let json = serde_json::to_string(&endpoint).unwrap();
    let deserialized: ServiceEndpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(endpoint.name, deserialized.name);
    assert_eq!(endpoint.port, deserialized.port);
}

#[test]
fn test_endpoint_protocol_enum_serde() {
    let protocols = [
        EndpointProtocol::Http,
        EndpointProtocol::Https,
        EndpointProtocol::Tcp,
        EndpointProtocol::Udp,
        EndpointProtocol::Grpc,
        EndpointProtocol::WebSocket,
        EndpointProtocol::Custom("custom-proto".to_string()),
    ];
    for p in protocols {
        let json = serde_json::to_string(&p).unwrap();
        let deserialized: EndpointProtocol = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(json, json2);
    }
}

#[test]
fn test_service_dependency_serde_json_roundtrip() {
    let dep = ServiceDependency {
        name: "db".to_string(),
        dependency_type: DependencyType::Hard,
        condition: DependencyCondition::Ready,
        timeout: Some(30),
        status: DependencyStatus::Satisfied,
    };
    let json = serde_json::to_string(&dep).unwrap();
    let deserialized: ServiceDependency = serde_json::from_str(&json).unwrap();
    assert_eq!(dep.name, deserialized.name);
    assert_eq!(dep.timeout, deserialized.timeout);
}

#[test]
fn test_dependency_type_and_condition_serde() {
    let dep_types = [
        DependencyType::Hard,
        DependencyType::Soft,
        DependencyType::Weak,
    ];
    for dt in dep_types {
        let json = serde_json::to_string(&dt).unwrap();
        let _: DependencyType = serde_json::from_str(&json).unwrap();
    }
    let conditions = [
        DependencyCondition::Started,
        DependencyCondition::Ready,
        DependencyCondition::Healthy,
        DependencyCondition::Custom("custom".to_string()),
    ];
    for c in conditions {
        let json = serde_json::to_string(&c).unwrap();
        let _: DependencyCondition = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_service_scaling_default_and_serde() {
    let scaling = ServiceScaling::default();
    assert!(matches!(scaling.scaling_type, ScalingType::Manual));
    assert_eq!(scaling.min_replicas, 1);
    assert_eq!(scaling.max_replicas, 1);
    let json = serde_json::to_string(&scaling).unwrap();
    let deserialized: ServiceScaling = serde_json::from_str(&json).unwrap();
    assert_eq!(scaling.min_replicas, deserialized.min_replicas);
}

#[test]
fn test_scaling_policy_and_metrics_serde() {
    let policy = ScalingPolicy {
        name: "scale-up".to_string(),
        direction: ScalingDirection::Up,
        amount: ScalingAmount::Fixed(2),
        cooldown: 60,
    };
    let json = serde_json::to_string(&policy).unwrap();
    let deserialized: ScalingPolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(policy.name, deserialized.name);

    let metric = ScalingMetric {
        name: "cpu".to_string(),
        metric_type: ScalingMetricType::CpuUtilization,
        target_value: 80.0,
        current_value: Some(45.0),
    };
    let json = serde_json::to_string(&metric).unwrap();
    let deserialized: ScalingMetric = serde_json::from_str(&json).unwrap();
    assert_eq!(metric.target_value, deserialized.target_value);
}

#[test]
fn test_scaling_type_and_amount_serde() {
    let scaling_types = [
        ScalingType::Manual,
        ScalingType::Hpa,
        ScalingType::Vpa,
        ScalingType::Custom("custom".to_string()),
    ];
    for st in scaling_types {
        let json = serde_json::to_string(&st).unwrap();
        let _: ScalingType = serde_json::from_str(&json).unwrap();
    }
    let amounts = [ScalingAmount::Fixed(5), ScalingAmount::Percent(50)];
    for a in amounts {
        let json = serde_json::to_string(&a).unwrap();
        let _: ScalingAmount = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_service_configuration_default_and_serde() {
    let config = ServiceConfiguration::default();
    assert_eq!(config.sources.len(), 1);
    assert!(matches!(config.sources[0], ConfigSource::Environment));
    assert!(config.environment.is_empty());
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ServiceConfiguration = serde_json::from_str(&json).unwrap();
    assert_eq!(config.sources.len(), deserialized.sources.len());
}

#[test]
fn test_config_source_and_format_serde() {
    let sources = [
        ConfigSource::Environment,
        ConfigSource::Files,
        ConfigSource::External {
            url: "https://config.example.com".to_string(),
            auth: Some("token".to_string()),
        },
        ConfigSource::ConfigMap("my-config".to_string()),
        ConfigSource::Secret("my-secret".to_string()),
    ];
    for s in sources {
        let json = serde_json::to_string(&s).unwrap();
        let _: ConfigSource = serde_json::from_str(&json).unwrap();
    }
    let formats = [
        ConfigFormat::Json,
        ConfigFormat::Yaml,
        ConfigFormat::Toml,
        ConfigFormat::Properties,
        ConfigFormat::Ini,
        ConfigFormat::Custom("custom".to_string()),
    ];
    for f in formats {
        let json = serde_json::to_string(&f).unwrap();
        let _: ConfigFormat = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_service_lifecycle_default_and_serde() {
    let lifecycle = ServiceLifecycle::default();
    assert!(matches!(lifecycle.restart_policy, RestartPolicy::Always));
    assert_eq!(lifecycle.termination_grace_period, 30);
    let json = serde_json::to_string(&lifecycle).unwrap();
    let deserialized: ServiceLifecycle = serde_json::from_str(&json).unwrap();
    assert_eq!(
        lifecycle.termination_grace_period,
        deserialized.termination_grace_period
    );
}

#[test]
fn test_restart_policy_and_lifecycle_failure_action_serde() {
    let policies = [
        RestartPolicy::Always,
        RestartPolicy::OnFailure,
        RestartPolicy::Never,
        RestartPolicy::UnlessStopped,
    ];
    for p in policies {
        let json = serde_json::to_string(&p).unwrap();
        let _: RestartPolicy = serde_json::from_str(&json).unwrap();
    }
    let actions = [
        LifecycleFailureAction::Ignore,
        LifecycleFailureAction::Abort,
        LifecycleFailureAction::Retry,
    ];
    for a in actions {
        let json = serde_json::to_string(&a).unwrap();
        let _: LifecycleFailureAction = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_debug_implementations() {
    let service = UniversalService::default();
    let debug_str = format!("{:?}", service);
    assert!(debug_str.contains("UniversalService"));
    assert!(debug_str.contains("default-service"));

    let phase = ServicePhase::Running;
    assert!(format!("{:?}", phase).contains("Running"));

    let endpoint = EndpointProtocol::Https;
    assert!(format!("{:?}", endpoint).contains("Https"));
}

#[test]
fn test_service_spec_with_primal_type_serde() {
    let mut spec = UniversalService::default().spec;
    spec.primal_type = Some(PrimalType::new("compute", "toadstool", "1.0.0"));
    spec.capabilities = vec![PrimalCapability::new("compute", "execution", "1.0")];
    let json = serde_json::to_string(&spec).unwrap();
    let deserialized: ServiceSpec = serde_json::from_str(&json).unwrap();
    assert!(deserialized.primal_type.is_some());
    assert_eq!(deserialized.capabilities.len(), 1);
}

#[test]
fn test_service_condition_serde_json_roundtrip() {
    let condition = ServiceCondition {
        condition_type: "Ready".to_string(),
        status: ConditionStatus::True,
        last_transition_time: Utc::now(),
        reason: Some("ServiceStarted".to_string()),
        message: Some("All replicas ready".to_string()),
    };
    let json = serde_json::to_string(&condition).unwrap();
    let deserialized: ServiceCondition = serde_json::from_str(&json).unwrap();
    assert_eq!(condition.condition_type, deserialized.condition_type);
    assert!(matches!(
        (condition.status, deserialized.status),
        (ConditionStatus::True, ConditionStatus::True)
    ));
}

#[test]
fn test_config_file_serde_json_roundtrip() {
    let config_file = ConfigFile {
        path: "/etc/config/app.json".to_string(),
        format: ConfigFormat::Json,
        required: true,
        watch: false,
    };
    let json = serde_json::to_string(&config_file).unwrap();
    let deserialized: ConfigFile = serde_json::from_str(&json).unwrap();
    assert_eq!(config_file.path, deserialized.path);
    assert!(matches!(
        (config_file.format, deserialized.format),
        (ConfigFormat::Json, ConfigFormat::Json)
    ));
}

#[test]
fn test_lifecycle_hook_serde_json_roundtrip() {
    let hook = LifecycleHook {
        name: "pre-start".to_string(),
        command: vec!["/bin/init.sh".to_string()],
        timeout: Some(10),
        on_failure: LifecycleFailureAction::Abort,
    };
    let json = serde_json::to_string(&hook).unwrap();
    let deserialized: LifecycleHook = serde_json::from_str(&json).unwrap();
    assert_eq!(hook.name, deserialized.name);
    assert_eq!(hook.command, deserialized.command);
}

#[test]
fn test_scaling_metric_type_custom_serde() {
    let metric_type = ScalingMetricType::Custom {
        source: "prometheus".to_string(),
        query: "rate(http_requests_total[5m])".to_string(),
    };
    let json = serde_json::to_string(&metric_type).unwrap();
    let deserialized: ScalingMetricType = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_dependency_status_enum_serde() {
    let statuses = [
        DependencyStatus::Satisfied,
        DependencyStatus::Pending,
        DependencyStatus::Failed,
        DependencyStatus::Timeout,
    ];
    for s in statuses {
        let json = serde_json::to_string(&s).unwrap();
        let _: DependencyStatus = serde_json::from_str(&json).unwrap();
    }
}
