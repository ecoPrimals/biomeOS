// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::test_metadata;
use super::*;
use crate::service::status::*;
use std::collections::HashMap;

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
        assert_eq!(format!("{phase:?}"), format!("{:?}", deserialized));
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
        assert_eq!(format!("{s:?}"), format!("{:?}", deserialized));
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
