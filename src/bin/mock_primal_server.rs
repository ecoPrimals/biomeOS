//! Mock Primal Server
//!
//! A simple HTTP server that implements basic primal API endpoints
//! for testing the Universal UI system.

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::net::TcpListener;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let port = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "8080".to_string());
    
    let primal_name = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "mock-primal".to_string());
    
    info!("🚀 Starting Mock Primal Server: {} on port {}", primal_name, port);
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/capabilities", get(get_capabilities))
        .route("/api/v1/status", get(get_status))
        .route("/api/v1/metrics", get(get_metrics))
        .route("/api/v1/services", get(list_services))
        .route("/api/v1/services/:id", get(get_service))
        .route("/api/v1/services/:id/scale", post(scale_service))
        .route("/api/v1/services/:id/logs", get(get_logs))
        .route("/api/v1/coordinate", post(coordinate_deployment))
        .route("/ping", get(ping))
        .route("/status", get(get_status));
    
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("✅ Mock Primal '{}' listening on {}", primal_name, addr);
    info!("📋 Available endpoints:");
    info!("  • GET  /health");
    info!("  • GET  /api/v1/capabilities");
    info!("  • GET  /api/v1/status");
    info!("  • GET  /api/v1/metrics");
    info!("  • GET  /api/v1/services");
    info!("  • POST /api/v1/coordinate");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0",
        "uptime_seconds": 42
    }))
}

async fn ping() -> Json<Value> {
    Json(json!({
        "pong": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_capabilities() -> Json<Value> {
    Json(json!({
        "capabilities": [
            "compute",
            "orchestration",
            "monitoring",
            "deployment",
            "scaling"
        ],
        "supported_runtimes": [
            "native",
            "wasm",
            "container"
        ],
        "api_version": "v1",
        "primal_type": "mock",
        "features": {
            "real_time_monitoring": true,
            "auto_scaling": true,
            "health_checks": true,
            "log_aggregation": true,
            "metrics_collection": true
        }
    }))
}

async fn get_status() -> Json<Value> {
    Json(json!({
        "primal_name": "mock-primal",
        "status": "running",
        "health": "healthy",
        "services_count": 3,
        "active_deployments": 1,
        "resource_usage": {
            "cpu_percent": 25.5,
            "memory_percent": 45.2,
            "disk_percent": 12.8
        },
        "last_updated": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_metrics() -> Json<Value> {
    Json(json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "metrics": {
            "cpu_usage": 25.5,
            "memory_usage": 45.2,
            "disk_usage": 12.8,
            "network_in": 1024000,
            "network_out": 512000,
            "requests_per_second": 15.3,
            "active_connections": 42
        },
        "services": {
            "web-service": {
                "status": "running",
                "replicas": 3,
                "cpu_usage": 15.2,
                "memory_usage": 256000000,
                "requests_per_minute": 850
            },
            "api-service": {
                "status": "running",
                "replicas": 2,
                "cpu_usage": 8.7,
                "memory_usage": 128000000,
                "requests_per_minute": 420
            },
            "database": {
                "status": "running",
                "replicas": 1,
                "cpu_usage": 1.6,
                "memory_usage": 512000000,
                "connections": 15
            }
        }
    }))
}

async fn list_services() -> Json<Value> {
    Json(json!({
        "services": [
            {
                "id": "web-service",
                "name": "Web Service",
                "status": "running",
                "replicas": 3,
                "desired_replicas": 3,
                "image": "nginx:latest",
                "ports": [80, 443],
                "created_at": "2025-07-08T20:00:00Z",
                "updated_at": chrono::Utc::now().to_rfc3339()
            },
            {
                "id": "api-service",
                "name": "API Service",
                "status": "running",
                "replicas": 2,
                "desired_replicas": 2,
                "image": "api:v1.2.3",
                "ports": [8080],
                "created_at": "2025-07-08T20:05:00Z",
                "updated_at": chrono::Utc::now().to_rfc3339()
            },
            {
                "id": "database",
                "name": "Database",
                "status": "running",
                "replicas": 1,
                "desired_replicas": 1,
                "image": "postgres:14",
                "ports": [5432],
                "created_at": "2025-07-08T20:10:00Z",
                "updated_at": chrono::Utc::now().to_rfc3339()
            }
        ],
        "total": 3
    }))
}

async fn get_service(Path(service_id): Path<String>) -> Json<Value> {
    let service_data = match service_id.as_str() {
        "web-service" => json!({
            "id": "web-service",
            "name": "Web Service",
            "status": "running",
            "replicas": 3,
            "desired_replicas": 3,
            "image": "nginx:latest",
            "ports": [80, 443],
            "environment": {
                "ENV": "production",
                "LOG_LEVEL": "info"
            },
            "resources": {
                "cpu_limit": "500m",
                "memory_limit": "512Mi"
            },
            "health": {
                "healthy_replicas": 3,
                "unhealthy_replicas": 0,
                "last_health_check": chrono::Utc::now().to_rfc3339()
            }
        }),
        "api-service" => json!({
            "id": "api-service",
            "name": "API Service",
            "status": "running",
            "replicas": 2,
            "desired_replicas": 2,
            "image": "api:v1.2.3",
            "ports": [8080],
            "environment": {
                "DATABASE_URL": "postgres://localhost:5432/mydb",
                "API_KEY": "***hidden***"
            },
            "resources": {
                "cpu_limit": "1000m",
                "memory_limit": "1Gi"
            },
            "health": {
                "healthy_replicas": 2,
                "unhealthy_replicas": 0,
                "last_health_check": chrono::Utc::now().to_rfc3339()
            }
        }),
        "database" => json!({
            "id": "database",
            "name": "Database",
            "status": "running",
            "replicas": 1,
            "desired_replicas": 1,
            "image": "postgres:14",
            "ports": [5432],
            "environment": {
                "POSTGRES_DB": "mydb",
                "POSTGRES_USER": "admin"
            },
            "resources": {
                "cpu_limit": "2000m",
                "memory_limit": "4Gi"
            },
            "health": {
                "healthy_replicas": 1,
                "unhealthy_replicas": 0,
                "last_health_check": chrono::Utc::now().to_rfc3339()
            }
        }),
        _ => json!({
            "error": "Service not found",
            "service_id": service_id
        })
    };
    
    Json(service_data)
}

async fn scale_service(Path(service_id): Path<String>, Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    let replicas = payload.get("replicas")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    
    info!("🔄 Scaling service {} to {} replicas", service_id, replicas);
    
    if replicas > 10 {
        warn!("⚠️  Requested replica count {} exceeds maximum (10)", replicas);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    Ok(Json(json!({
        "service_id": service_id,
        "old_replicas": 2,
        "new_replicas": replicas,
        "status": "scaling",
        "message": format!("Scaling {} to {} replicas", service_id, replicas),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

async fn get_logs(Path(service_id): Path<String>, Query(params): Query<HashMap<String, String>>) -> Json<Value> {
    let lines = params.get("lines")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);
    
    let since = params.get("since")
        .map(|s| s.as_str())
        .unwrap_or("1h");
    
    let logs = match service_id.as_str() {
        "web-service" => vec![
            "[2025-07-08T21:00:00Z] INFO: Starting nginx server",
            "[2025-07-08T21:00:01Z] INFO: Listening on port 80",
            "[2025-07-08T21:00:02Z] INFO: SSL certificate loaded",
            "[2025-07-08T21:01:00Z] INFO: Health check passed",
            "[2025-07-08T21:02:00Z] INFO: Processed 1000 requests",
            "[2025-07-08T21:03:00Z] INFO: Memory usage: 45MB",
        ],
        "api-service" => vec![
            "[2025-07-08T21:00:00Z] INFO: API server starting",
            "[2025-07-08T21:00:01Z] INFO: Connected to database",
            "[2025-07-08T21:00:02Z] INFO: Routes registered",
            "[2025-07-08T21:01:00Z] INFO: Health endpoint active",
            "[2025-07-08T21:02:00Z] INFO: Processed 500 API calls",
            "[2025-07-08T21:03:00Z] DEBUG: Cache hit ratio: 85%",
        ],
        "database" => vec![
            "[2025-07-08T21:00:00Z] INFO: PostgreSQL starting",
            "[2025-07-08T21:00:01Z] INFO: Database system ready",
            "[2025-07-08T21:00:02Z] INFO: Accepting connections",
            "[2025-07-08T21:01:00Z] INFO: Checkpoint completed",
            "[2025-07-08T21:02:00Z] INFO: 15 active connections",
            "[2025-07-08T21:03:00Z] INFO: Query performance optimal",
        ],
        _ => vec!["Service not found"]
    };
    
    let limited_logs = logs.into_iter()
        .take(lines)
        .collect::<Vec<_>>();
    
    Json(json!({
        "service_id": service_id,
        "lines_requested": lines,
        "lines_returned": limited_logs.len(),
        "since": since,
        "logs": limited_logs,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn coordinate_deployment(Json(payload): Json<Value>) -> Json<Value> {
    let deployment_name = payload.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unnamed-deployment");
    
    info!("🚀 Coordinating deployment: {}", deployment_name);
    
    Json(json!({
        "deployment_id": format!("deploy-{}", uuid::Uuid::new_v4()),
        "name": deployment_name,
        "status": "coordinating",
        "steps": [
            {
                "step": "validation",
                "status": "completed",
                "duration_ms": 150
            },
            {
                "step": "resource_allocation",
                "status": "in_progress",
                "duration_ms": null
            },
            {
                "step": "service_deployment",
                "status": "pending",
                "duration_ms": null
            },
            {
                "step": "health_verification",
                "status": "pending",
                "duration_ms": null
            }
        ],
        "estimated_completion": "2025-07-08T21:08:00Z",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
} 