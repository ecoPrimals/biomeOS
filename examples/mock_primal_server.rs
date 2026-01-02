// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Mock Primal Server - For Dynamic API Discovery Demo
//
// This is a simple mock server that implements /api/schema
// to demonstrate biomeOS's dynamic API discovery.

use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize)]
struct CreateBucketRequest {
    name: String,
    #[serde(default)]
    compression: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Bucket {
    id: String,
    name: String,
    compression: String,
    created_at: String,
}

/// GET /api/schema - Returns OpenAPI v3 specification
async fn api_schema() -> Json<Value> {
    Json(json!({
        "schema_type": "openapi",
        "schema_version": "3.1.0",
        "schema": {
            "openapi": "3.1.0",
            "info": {
                "title": "Mock Primal API",
                "version": "1.0.0",
                "description": "A demonstration primal for dynamic API discovery"
            },
            "paths": {
                "/api/v1/buckets": {
                    "get": {
                        "operationId": "listBuckets",
                        "summary": "List all buckets",
                        "responses": {
                            "200": {
                                "description": "List of buckets",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "type": "array",
                                            "items": {
                                                "$ref": "#/components/schemas/Bucket"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "post": {
                        "operationId": "createBucket",
                        "summary": "Create a new bucket",
                        "requestBody": {
                            "required": true,
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/CreateBucketRequest"
                                    }
                                }
                            }
                        },
                        "responses": {
                            "200": {
                                "description": "Bucket created",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "$ref": "#/components/schemas/Bucket"
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "/api/v1/buckets/{bucket_id}": {
                    "get": {
                        "operationId": "getBucket",
                        "summary": "Get bucket details",
                        "parameters": [
                            {
                                "name": "bucket_id",
                                "in": "path",
                                "required": true,
                                "schema": {
                                    "type": "string"
                                }
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "Bucket details",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "$ref": "#/components/schemas/Bucket"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "components": {
                "schemas": {
                    "CreateBucketRequest": {
                        "type": "object",
                        "required": ["name"],
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Bucket name"
                            },
                            "compression": {
                                "type": "string",
                                "description": "Compression algorithm (optional)",
                                "enum": ["lz4", "zstd", "gzip", "none"]
                            }
                        }
                    },
                    "Bucket": {
                        "type": "object",
                        "required": ["id", "name", "compression", "created_at"],
                        "properties": {
                            "id": {
                                "type": "string"
                            },
                            "name": {
                                "type": "string"
                            },
                            "compression": {
                                "type": "string"
                            },
                            "created_at": {
                                "type": "string",
                                "format": "date-time"
                            }
                        }
                    }
                }
            }
        },
        "capabilities": ["storage", "compression"]
    }))
}

/// GET /api/v1/buckets - List buckets
async fn list_buckets() -> Json<Vec<Bucket>> {
    Json(vec![
        Bucket {
            id: "bucket-001".to_string(),
            name: "example-bucket".to_string(),
            compression: "lz4".to_string(),
            created_at: "2026-01-02T00:00:00Z".to_string(),
        }
    ])
}

/// POST /api/v1/buckets - Create bucket
async fn create_bucket(
    Json(payload): Json<CreateBucketRequest>,
) -> (StatusCode, Json<Bucket>) {
    let bucket = Bucket {
        id: format!("bucket-{:x}", rand::random::<u32>()),
        name: payload.name,
        compression: payload.compression.unwrap_or_else(|| "lz4".to_string()),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    (StatusCode::OK, Json(bucket))
}

/// GET /api/v1/buckets/:id - Get bucket
async fn get_bucket(Path(bucket_id): Path<String>) -> Json<Bucket> {
    Json(Bucket {
        id: bucket_id.clone(),
        name: format!("bucket-{}", bucket_id),
        compression: "lz4".to_string(),
        created_at: "2026-01-02T00:00:00Z".to_string(),
    })
}

/// GET /health - Health check
async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[tokio::main]
async fn main() {
    // Build router
    let app = Router::new()
        .route("/api/schema", get(api_schema))
        .route("/api/v1/buckets", get(list_buckets).post(create_bucket))
        .route("/api/v1/buckets/:bucket_id", get(get_bucket))
        .route("/health", get(health));

    // Bind to address
    let addr = SocketAddr::from(([127, 0, 0, 1], 9876));
    
    println!("🚀 Mock Primal Server starting...");
    println!("   Listening on: http://{}", addr);
    println!("   Schema: http://{}/api/schema", addr);
    println!("   Health: http://{}/health", addr);
    println!();
    println!("Press Ctrl+C to stop");
    
    // Run server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

