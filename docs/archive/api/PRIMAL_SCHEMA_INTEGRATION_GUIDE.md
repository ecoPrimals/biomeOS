# Primal API Schema Integration Guide

**Date**: January 2, 2026  
**Audience**: Primal Development Teams  
**Purpose**: Enable dynamic biomeOS integration via `/api/schema`  

---

## 🎯 Overview

biomeOS can now **dynamically discover and adapt to any primal's API** by fetching an OpenAPI specification from a standard endpoint. This eliminates the need for hardcoded client wrappers in biomeOS!

### What This Means for You

**Before**: biomeOS team needs to write and maintain a custom client for your primal  
**After**: You expose `/api/schema`, biomeOS adapts automatically

**Benefits**:
- ✅ No coordination needed for API changes
- ✅ Version updates work automatically
- ✅ New endpoints discovered instantly
- ✅ Industry-standard OpenAPI format
- ✅ Can generate docs, mocks, clients from same spec

---

## 📋 Requirements

### Minimal Implementation

Add **one HTTP endpoint** to your primal:

```
GET /api/schema
```

**Response**:
```json
{
  "schema_type": "openapi",
  "schema_version": "3.1.0",
  "schema": {
    "openapi": "3.1.0",
    "info": {
      "title": "YourPrimal API",
      "version": "1.0.0",
      "description": "API for YourPrimal"
    },
    "paths": {
      "/api/v1/your-operation": {
        "post": {
          "operationId": "yourOperation",
          "summary": "Does something useful",
          "requestBody": {
            "required": true,
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "required": ["param1"],
                  "properties": {
                    "param1": {"type": "string"}
                  }
                }
              }
            }
          },
          "responses": {
            "200": {
              "description": "Success",
              "content": {
                "application/json": {
                  "schema": {
                    "type": "object",
                    "properties": {
                      "result": {"type": "string"}
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  },
  "capabilities": ["your-capability"]
}
```

---

## 🛠️ Implementation Examples

### Example 1: Rust (Axum)

```rust
use axum::{Router, Json, routing::get};
use serde_json::json;

async fn api_schema() -> Json<serde_json::Value> {
    Json(json!({
        "schema_type": "openapi",
        "schema_version": "3.1.0",
        "schema": {
            "openapi": "3.1.0",
            "info": {
                "title": "MyPrimal API",
                "version": "1.0.0"
            },
            "paths": {
                // Your OpenAPI paths here
            }
        },
        "capabilities": ["compute", "storage"]
    }))
}

fn create_router() -> Router {
    Router::new()
        .route("/api/schema", get(api_schema))
        // ... your other routes
}
```

### Example 2: Generate from Existing Code

If you already have a Rust API with types, use `utoipa` to auto-generate:

```rust
use utoipa::{OpenApi, ToSchema};

#[derive(ToSchema, Serialize, Deserialize)]
struct CreateBucketRequest {
    name: String,
    compression: Option<String>,
}

#[derive(OpenApi)]
#[openapi(
    paths(create_bucket, list_buckets),
    components(schemas(CreateBucketRequest))
)]
struct ApiDoc;

async fn api_schema() -> Json<serde_json::Value> {
    let openapi_spec = ApiDoc::openapi();
    Json(json!({
        "schema_type": "openapi",
        "schema_version": "3.1.0",
        "schema": serde_json::to_value(&openapi_spec).unwrap(),
        "capabilities": ["storage"]
    }))
}
```

### Example 3: Static File

You can also serve a pre-generated OpenAPI spec:

```rust
async fn api_schema() -> Json<serde_json::Value> {
    let spec = include_str!("../openapi.json");
    let openapi: serde_json::Value = serde_json::from_str(spec).unwrap();
    
    Json(json!({
        "schema_type": "openapi",
        "schema_version": "3.1.0",
        "schema": openapi,
        "capabilities": ["your-capabilities"]
    }))
}
```

---

## 📝 Schema Response Format

### Wrapper Object

```typescript
{
  "schema_type": "openapi" | "json_schema" | "graphql" | "custom",
  "schema_version": string,  // e.g., "3.1.0"
  "schema": object,          // The actual schema (format depends on type)
  "capabilities": string[]   // Optional: Your primal's capabilities
}
```

### OpenAPI Schema (Recommended)

The `schema` field should contain a **complete OpenAPI v3.x specification**:

```json
{
  "openapi": "3.1.0",
  "info": {
    "title": "Your API",
    "version": "1.0.0",
    "description": "Optional description"
  },
  "paths": {
    "/path/to/operation": {
      "method": {
        "operationId": "uniqueOperationId",  // REQUIRED for dynamic calls
        "summary": "What this does",
        "requestBody": {...},
        "responses": {...}
      }
    }
  },
  "components": {
    "schemas": {
      // Reusable schema definitions
    }
  }
}
```

**Key Requirements**:
- ✅ Every operation MUST have an `operationId`
- ✅ Use standard HTTP methods (GET, POST, PUT, DELETE, PATCH)
- ✅ Define request/response schemas with `application/json`
- ✅ Use components/schemas for reusable types

---

## 🧪 Testing Your Implementation

### 1. Verify Endpoint Works

```bash
curl http://localhost:YOUR_PORT/api/schema | jq .
```

Should return valid JSON with the wrapper structure.

### 2. Validate OpenAPI Spec

```bash
# Extract just the schema
curl http://localhost:YOUR_PORT/api/schema | jq .schema > openapi.json

# Validate with openapi-generator or similar tool
openapi-generator validate -i openapi.json
```

### 3. Test with biomeOS

```rust
use biomeos_core::clients::universal::UniversalPrimalClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // biomeOS will fetch /api/schema automatically
    let client = UniversalPrimalClient::from_endpoint(
        "http://localhost:YOUR_PORT"
    ).await?;
    
    // List discovered operations
    println!("Operations: {:?}", client.list_operations());
    
    // Call any operation
    let result = client.call_operation(
        "yourOperationId",
        serde_json::json!({"param": "value"})
    ).await?;
    
    println!("Result: {}", result);
    Ok(())
}
```

---

## 📊 Current Primal Status

| Primal | OpenAPI Support | Status | Notes |
|--------|----------------|--------|-------|
| **Songbird** | ✅ Yes | Ready to implement | Mentioned in handoff response |
| **BearDog** | ✅ Yes | Ready to implement | Mentioned in handoff response |
| **NestGate** | ❓ Unknown | Not yet asked | Has REST API for ZFS |
| **ToadStool** | ❓ Unknown | Not yet asked | Large compute API |
| **Squirrel** | ❓ Unknown | Not yet asked | AI/ML API |

---

## 🎁 Tools & Resources

### OpenAPI Tools

1. **Specification**: https://swagger.io/specification/
2. **Editor**: https://editor.swagger.io/ (online validator)
3. **Rust Crates**:
   - `utoipa` - Auto-generate from Rust code
   - `openapiv3` - Parse/generate OpenAPI specs
   - `paperclip` - Alternative auto-generation

### Example Specs

See successful API implementations:
- Songbird: `SONGBIRD_INTEGRATION_CLARIFIED.md`
- BearDog: `BEARDOG_HTTP_API_CONFIRMED.md`

### biomeOS Integration

- Architecture: `docs/api/DYNAMIC_API_SCHEMA_DISCOVERY.md`
- Universal Client: `crates/biomeos-core/src/clients/universal.rs`
- OpenAPI Adapter: `crates/biomeos-core/src/clients/openapi_adapter.rs`

---

## 🚀 Migration Strategy

### Phase 1: Add Schema Endpoint (This Week)
- Implement `GET /api/schema`
- Serve minimal OpenAPI spec
- Test with `curl` and validators

### Phase 2: Complete Spec (Next Week)
- Add all operations with `operationId`
- Define request/response schemas
- Add descriptions and examples

### Phase 3: Test with biomeOS (Next Week)
- Verify `UniversalPrimalClient` can discover
- Test dynamic operation calls
- Provide feedback to biomeOS team

### Phase 4: Deprecate Hardcoded Client (Future)
- Once schema is stable and tested
- biomeOS switches from static to dynamic
- Celebrate zero-coupling! 🎊

---

## 💡 Best Practices

### 1. Version Your API
```json
{
  "info": {
    "version": "1.2.3"  // Semantic versioning
  }
}
```

### 2. Use Operation IDs Consistently
```json
{
  "paths": {
    "/buckets": {
      "post": {
        "operationId": "createBucket",  // camelCase, descriptive
        "summary": "Create a new storage bucket"
      }
    }
  }
}
```

### 3. Define Reusable Schemas
```json
{
  "components": {
    "schemas": {
      "Bucket": {
        "type": "object",
        "required": ["id", "name"],
        "properties": {
          "id": {"type": "string"},
          "name": {"type": "string"}
        }
      }
    }
  },
  "paths": {
    "/buckets": {
      "post": {
        "responses": {
          "200": {
            "content": {
              "application/json": {
                "schema": {"$ref": "#/components/schemas/Bucket"}
              }
            }
          }
        }
      }
    }
  }
}
```

### 4. Include Examples
```json
{
  "requestBody": {
    "content": {
      "application/json": {
        "schema": {...},
        "example": {
          "name": "my-bucket",
          "compression": "lz4"
        }
      }
    }
  }
}
```

### 5. Document Error Responses
```json
{
  "responses": {
    "200": {
      "description": "Success"
    },
    "400": {
      "description": "Invalid request"
    },
    "404": {
      "description": "Resource not found"
    },
    "500": {
      "description": "Internal error"
    }
  }
}
```

---

## 🤝 Support

### Questions?

- Check `docs/api/DYNAMIC_API_SCHEMA_DISCOVERY.md` for architecture
- See `EXTENDED_SESSION_COMPLETE.md` for implementation details
- Review OpenAPI specification: https://swagger.io/specification/

### Collaboration

We're happy to help with:
- Reviewing your OpenAPI spec
- Testing integration with biomeOS
- Providing example implementations
- Troubleshooting issues

**This is a collaborative evolution towards zero-coupling!** 🚀

---

## ✨ Benefits Recap

**For Primal Teams**:
- ✅ API changes don't break biomeOS
- ✅ No coordination needed for updates
- ✅ Auto-generated docs from schema
- ✅ Standard tooling ecosystem

**For biomeOS**:
- ✅ No hardcoded client wrappers
- ✅ Works with any primal instantly
- ✅ Automatic API version handling
- ✅ True agnostic orchestration

**For Ecosystem**:
- ✅ Sovereign primal development
- ✅ Loose coupling, high cohesion
- ✅ Easy integration for new primals
- ✅ Future-proof architecture

---

**Document Status**: ✅ Complete Integration Guide  
**Last Updated**: January 2, 2026  
**Next**: Primal teams implement `/api/schema` endpoint  

🎊 **Let's build the zero-coupling future together!** 🎊

