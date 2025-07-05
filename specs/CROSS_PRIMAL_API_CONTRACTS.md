# Cross-Primal API Contracts

**Version:** 1.0.0 | **Status:** Draft | **Date:** January 2025

---

## Overview

This specification defines the standardized API contracts for communication between Primals within biomeOS. These contracts ensure consistent, secure, and reliable integration patterns across the entire ecosystem.

## Authentication & Security Layer

### Universal Authentication Header

All cross-Primal API calls must include BearDog-issued authentication:

```http
Authorization: Bearer <beardog-jwt-token>
X-Biome-ID: <biome-instance-uuid>
X-Request-ID: <unique-request-id>
X-Primal-Source: <calling-primal-id>
X-Primal-Target: <target-primal-id>
Content-Type: application/json
```

### Standard Error Response

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": {
      "field": "specific error details"
    },
    "request_id": "req-uuid",
    "timestamp": "2025-01-15T10:30:00Z",
    "retry_after": "30s"
  }
}
```

## 🐕 BearDog Security Provider APIs

### Authentication Service

#### Token Generation
```http
POST /auth/token
Content-Type: application/json

{
  "primal_id": "primal-toadstool-001",
  "requested_scope": ["volume_access", "service_registration"],
  "duration": "24h"
}
```

Response:
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "scope": ["volume_access", "service_registration"],
  "refresh_token": "refresh_token_here"
}
```

#### Certificate Management
```http
POST /pki/certificates
Content-Type: application/json

{
  "primal_id": "primal-nestgate-001",
  "certificate_type": "service_mtls",
  "common_name": "nestgate.biome.local",
  "san": ["nestgate", "nestgate.biome.local"],
  "duration": "30d"
}
```

Response:
```json
{
  "certificate": "-----BEGIN CERTIFICATE-----\n...",
  "private_key": "-----BEGIN PRIVATE KEY-----\n...",
  "ca_certificate": "-----BEGIN CERTIFICATE-----\n...",
  "expires_at": "2025-02-14T10:30:00Z",
  "serial_number": "1234567890"
}
```

### Secrets Management
```http
GET /secrets/{secret_name}
Authorization: Bearer <token>
X-Primal-ID: primal-nestgate-001

Response:
{
  "secret_name": "storage_encryption_key",
  "secret_value": "encrypted_value_here",
  "version": 3,
  "created_at": "2025-01-15T10:30:00Z",
  "expires_at": "2025-02-15T10:30:00Z"
}
```

## 🎼 Songbird Service Discovery APIs

### Service Registration
```http
POST /services
Authorization: Bearer <beardog-token>
Content-Type: application/json

{
  "service_id": "primal-toadstool-001",
  "primal_type": "toadstool",
  "endpoints": {
    "primary": "https://toadstool:8080",
    "health": "https://toadstool:8080/health"
  },
  "capabilities": ["container_runtime", "gpu_scheduling"],
  "metadata": {
    "zone": "us-west-2a",
    "version": "1.0.0"
  }
}
```

### Service Discovery
```http
GET /discovery
Authorization: Bearer <beardog-token>
Query Parameters:
  - primal_type: toadstool
  - capability: gpu_scheduling
  - health_status: healthy
  - zone: us-west-2a

Response:
{
  "services": [
    {
      "service_id": "primal-toadstool-001",
      "endpoints": {
        "primary": "https://toadstool:8080"
      },
      "capabilities": ["container_runtime", "gpu_scheduling"],
      "health_status": "healthy",
      "load_metrics": {
        "cpu_usage": "45%",
        "memory_usage": "60%"
      }
    }
  ]
}
```

### Health Monitoring
```http
PUT /services/{service_id}/health
Authorization: Bearer <beardog-token>
Content-Type: application/json

{
  "status": "healthy",
  "metrics": {
    "cpu_usage": "45%",
    "memory_usage": "60%",
    "request_rate": "150/min"
  },
  "checks": {
    "database": "healthy",
    "storage": "healthy"
  }
}
```

## 🏰 NestGate Storage APIs

### Volume Provisioning API (for Toadstool)

#### Create Volume
```http
POST /volumes
Authorization: Bearer <beardog-token>
X-Requesting-Primal: primal-toadstool-001
Content-Type: application/json

{
  "volume_name": "jupyter-workspace",
  "size": "100Gi",
  "tier": "hot",
  "filesystem": "zfs",
  "mount_options": {
    "compression": "lz4",
    "snapshots": true,
    "backup": true
  },
  "access_mode": "ReadWriteOnce",
  "service_binding": {
    "service_id": "jupyter-lab-001",
    "mount_path": "/workspace"
  }
}
```

Response:
```json
{
  "volume_id": "vol-uuid-1234",
  "volume_name": "jupyter-workspace", 
  "status": "provisioning",
  "mount_info": {
    "nfs_export": "nestgate:/volumes/vol-uuid-1234",
    "mount_options": "vers=4.1,rsize=1048576,wsize=1048576",
    "access_credentials": {
      "username": "service_account",
      "password": "encrypted_password"
    }
  },
  "created_at": "2025-01-15T10:30:00Z",
  "estimated_ready": "2025-01-15T10:32:00Z"
}
```

#### Volume Status
```http
GET /volumes/{volume_id}
Authorization: Bearer <beardog-token>

Response:
{
  "volume_id": "vol-uuid-1234",
  "status": "ready",
  "usage": {
    "used": "25Gi",
    "available": "75Gi",
    "usage_percent": 25
  },
  "performance_metrics": {
    "iops": 1500,
    "throughput": "150MB/s",
    "latency": "2ms"
  },
  "snapshots": [
    {
      "snapshot_id": "snap-uuid-5678",
      "created_at": "2025-01-15T02:00:00Z",
      "size": "20Gi"
    }
  ]
}
```

### MCP Volume API (for Squirrel)

#### MCP Volume Request
```http
POST /mcp/volumes
Authorization: Bearer <beardog-token>
X-MCP-Agent-ID: research-assistant-001
Content-Type: application/json

{
  "agent_id": "research-assistant-001",
  "volume_request": {
    "name": "agent-workspace",
    "size": "10Gi",
    "tier": "hot",
    "temporary": true,
    "ttl": "24h"
  },
  "access_pattern": "read_write",
  "data_classification": "research_data"
}
```

Response:
```json
{
  "volume_id": "mcp-vol-uuid-9999",
  "mount_info": {
    "protocol": "mcp_native",
    "endpoint": "nestgate://mcp/vol-uuid-9999",
    "access_token": "mcp_access_token"
  },
  "expires_at": "2025-01-16T10:30:00Z"
}
```

## 🍄 Toadstool Runtime APIs

### Service Execution API (from Songbird)

#### Deploy Service
```http
POST /services
Authorization: Bearer <beardog-token>
X-Orchestrator: songbird
Content-Type: application/json

{
  "service_definition": {
    "service_id": "jupyter-lab-001",
    "runtime": "container",
    "image": "jupyter/tensorflow-notebook:latest",
    "resources": {
      "cpu": "4",
      "memory": "16Gi",
      "gpu": 1
    },
    "volumes": [
      {
        "volume_id": "vol-uuid-1234",
        "mount_path": "/workspace",
        "read_only": false
      }
    ],
    "environment": {
      "JUPYTER_TOKEN": "${secrets.jupyter_token}"
    },
    "networking": {
      "ports": [{"container": 8888, "host": 8888}],
      "service_mesh": true
    }
  },
  "deployment_options": {
    "strategy": "rolling_update",
    "health_check": {
      "path": "/health",
      "interval": "30s"
    }
  }
}
```

Response:
```json
{
  "deployment_id": "deploy-uuid-5678",
  "service_id": "jupyter-lab-001",
  "status": "deploying",
  "endpoints": {
    "primary": "https://jupyter-lab-001.biome.local:8888",
    "health": "https://jupyter-lab-001.biome.local:8888/health"
  },
  "estimated_ready": "2025-01-15T10:32:00Z"
}
```

### Agent Execution API (from Squirrel)

#### Execute Agent
```http
POST /agents/execute
Authorization: Bearer <beardog-token>
X-MCP-Platform: squirrel
Content-Type: application/json

{
  "agent_definition": {
    "agent_id": "research-assistant-001",
    "provider": "anthropic",
    "model": "claude-3-sonnet",
    "runtime_config": {
      "memory_limit": "4Gi",
      "cpu_limit": "2",
      "timeout": "300s",
      "sandbox": "strict"
    },
    "capabilities": [
      "code_analysis",
      "data_processing"
    ],
    "volumes": [
      {
        "volume_id": "mcp-vol-uuid-9999",
        "mount_path": "/workspace"
      }
    ]
  },
  "execution_context": {
    "user_id": "user-123",
    "session_id": "session-456",
    "security_context": "research_grade"
  }
}
```

Response:
```json
{
  "execution_id": "exec-uuid-7890",
  "agent_id": "research-assistant-001",
  "status": "starting",
  "runtime_info": {
    "container_id": "container-abc123",
    "process_id": 12345,
    "sandbox_id": "sandbox-def456"
  },
  "communication": {
    "mcp_endpoint": "ws://toadstool:8080/mcp/exec-uuid-7890",
    "stdio_pipes": {
      "stdin": "/tmp/pipes/exec-uuid-7890.stdin",
      "stdout": "/tmp/pipes/exec-uuid-7890.stdout"
    }
  }
}
```

### Resource Management
```http
GET /resources
Authorization: Bearer <beardog-token>

Response:
{
  "compute": {
    "cpu": {
      "total_cores": 32,
      "available_cores": 20,
      "usage_percent": 37.5
    },
    "memory": {
      "total": "256Gi",
      "available": "180Gi", 
      "usage_percent": 29.7
    },
    "gpu": {
      "total_devices": 8,
      "available_devices": 6,
      "types": {
        "nvidia-a100": 4,
        "nvidia-h100": 4
      }
    }
  },
  "active_services": 12,
  "queued_requests": 3
}
```

## 🐿️ Squirrel MCP Platform APIs

### Agent Management API (from Toadstool)

#### Agent Status
```http
GET /agents/{agent_id}/status
Authorization: Bearer <beardog-token>
X-Execution-ID: exec-uuid-7890

Response:
{
  "agent_id": "research-assistant-001",
  "execution_id": "exec-uuid-7890",
  "status": "running",
  "runtime_metrics": {
    "cpu_usage": "25%",
    "memory_usage": "2.1Gi",
    "uptime": "45m30s"
  },
  "mcp_session": {
    "session_id": "mcp-session-123",
    "transport": "websocket",
    "messages_processed": 156,
    "last_activity": "2025-01-15T10:29:30Z"
  },
  "capabilities_status": {
    "code_analysis": "available",
    "data_processing": "available"
  }
}
```

### AI Provider Integration

#### Provider Status
```http
GET /providers
Authorization: Bearer <beardog-token>

Response:
{
  "providers": [
    {
      "name": "anthropic",
      "status": "available",
      "models": [
        {
          "name": "claude-3-sonnet",
          "status": "available",
          "rate_limits": {
            "requests_per_minute": 1000,
            "tokens_per_minute": 100000
          }
        }
      ],
      "health_metrics": {
        "latency": "250ms",
        "success_rate": "99.5%"
      }
    }
  ]
}
```

### Plugin Management
```http
POST /plugins/install
Authorization: Bearer <beardog-token>
Content-Type: application/json

{
  "plugin_name": "data_analyzer",
  "plugin_source": "https://plugins.biome.local/data_analyzer:1.0.0",
  "capabilities": ["pandas_toolkit", "numpy_ops"],
  "sandbox_requirements": {
    "network_access": false,
    "file_access": "/workspace",
    "memory_limit": "1Gi"
  },
  "target_agents": ["research-assistant-001"]
}
```

## Error Handling Patterns

### Retry Logic
```json
{
  "retry_policy": {
    "max_attempts": 3,
    "backoff_strategy": "exponential",
    "base_delay": "1s",
    "max_delay": "30s",
    "retryable_errors": [
      "NETWORK_ERROR",
      "TEMPORARY_UNAVAILABLE", 
      "RATE_LIMITED"
    ]
  }
}
```

### Circuit Breaker
```json
{
  "circuit_breaker": {
    "failure_threshold": 5,
    "recovery_timeout": "60s",
    "half_open_max_calls": 3,
    "state": "closed|open|half_open"
  }
}
```

## Async Operation Patterns

### Long-Running Operations
```http
POST /volumes
Authorization: Bearer <beardog-token>
Content-Type: application/json

{
  "volume_name": "large-dataset",
  "size": "10Ti",
  "tier": "cold"
}

Response:
HTTP/1.1 202 Accepted
Location: /operations/op-uuid-1234
{
  "operation_id": "op-uuid-1234",
  "status": "in_progress",
  "estimated_completion": "2025-01-15T11:00:00Z"
}
```

### Operation Status
```http
GET /operations/{operation_id}
Authorization: Bearer <beardog-token>

Response:
{
  "operation_id": "op-uuid-1234",
  "status": "completed",
  "progress": 100,
  "result": {
    "volume_id": "vol-uuid-5678",
    "mount_info": {...}
  },
  "started_at": "2025-01-15T10:30:00Z",
  "completed_at": "2025-01-15T10:45:00Z"
}
```

## Event Streaming

### WebSocket Events
```javascript
// Connect to event stream
const ws = new WebSocket('wss://songbird:8080/events');

// Event message format
{
  "event_type": "service_health_changed",
  "timestamp": "2025-01-15T10:30:00Z",
  "source": "primal-toadstool-001",
  "data": {
    "service_id": "jupyter-lab-001",
    "old_status": "healthy",
    "new_status": "degraded",
    "reason": "high_cpu_usage"
  }
}
```

## Rate Limiting

### Rate Limit Headers
```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1642248000
X-RateLimit-Window: 3600
```

### Rate Limit Exceeded
```http
HTTP/1.1 429 Too Many Requests
Retry-After: 60
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "API rate limit exceeded",
    "retry_after": "60s"
  }
}
```

## Monitoring & Observability

### Request Tracing
All API calls must include distributed tracing headers:
```http
X-Trace-ID: 550e8400-e29b-41d4-a716-446655440000
X-Span-ID: 6ba7b810-9dad-11d1-80b4-00c04fd430c8
X-Parent-Span-ID: 6ba7b811-9dad-11d1-80b4-00c04fd430c8
```

### Metrics Collection
Standardized metrics for all API endpoints:
```
# Request duration
api_request_duration_seconds{method="POST",endpoint="/volumes",status="200"}

# Request count
api_requests_total{method="POST",endpoint="/volumes",status="200"}

# Error rate
api_errors_total{method="POST",endpoint="/volumes",error_type="validation_error"}
```

## Testing & Validation

### Contract Testing
```python
# Example contract test
def test_volume_creation_contract():
    # Given
    volume_request = {
        "volume_name": "test-volume",
        "size": "10Gi",
        "tier": "hot"
    }
    
    # When
    response = nestgate_client.create_volume(volume_request)
    
    # Then
    assert response.status_code == 202
    assert "volume_id" in response.json()
    assert "mount_info" in response.json()
    
    # Verify async completion
    operation_id = response.json()["operation_id"]
    wait_for_completion(operation_id, timeout=300)
```

### Integration Testing
```bash
# Test cross-Primal workflow
curl -X POST https://toadstool:8080/services \
  -H "Authorization: Bearer ${BEARDOG_TOKEN}" \
  -d @service-definition.json

# Verify service registration in Songbird
curl -X GET "https://songbird:8080/discovery?service_id=jupyter-lab-001" \
  -H "Authorization: Bearer ${BEARDOG_TOKEN}"

# Check volume mounting in NestGate
curl -X GET https://nestgate:8080/volumes/vol-uuid-1234 \
  -H "Authorization: Bearer ${BEARDOG_TOKEN}"
```

This specification provides the foundation for reliable, secure, and consistent communication between all Primals in the biomeOS ecosystem. 