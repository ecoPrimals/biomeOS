# 🏗️ API Evolution: Agnostic Response Handling

**Date**: January 3, 2026  
**Context**: Identity endpoint wrapper debate  
**Goal**: Modern, idiomatic, agnostic API patterns for ecoPrimals ecosystem

---

## 🎯 The Core Insight

**Current Problem**: We're forcing a choice between wrapped and unwrapped responses.

**Reality**: Both patterns have valid use cases:
- **Wrapped** (`{"success": true, "data": {...}}`): Good for security, error handling, metadata
- **Unwrapped** (`{...}`): Simpler, cleaner, more REST-idiomatic

**Question**: How do we evolve to support both elegantly and agnostically?

---

## 🔍 Why Wrapped Responses Matter

### Security Benefit
```json
// Success - safe to expose data
{
  "success": true,
  "data": {
    "encryption_tag": "beardog:family:iidn:tower_abc",
    "family_id": "iidn"
  }
}

// Failure - no data exposure
{
  "success": false,
  "error": {
    "code": "unauthorized",
    "message": "Insufficient permissions"
  }
  // NO "data" field - secure by default
}
```

### Problem with Unwrapped
```json
// Success - good
{
  "encryption_tag": "beardog:family:iidn:tower_abc",
  "family_id": "iidn"
}

// Failure - what do we return?
{
  "error": "unauthorized"
}
// Client must check for "error" field vs. data fields
// Inconsistent pattern
```

---

## 🏛️ Modern Idiomatic Solution: Use HTTP Status Codes

### The REST Way

**Success Responses** (2xx status codes):
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "encryption_tag": "beardog:family:iidn:tower_abc",
  "family_id": "iidn",
  "identity_attestations": [...]
}
```

**Error Responses** (4xx/5xx status codes):
```http
HTTP/1.1 403 Forbidden
Content-Type: application/json

{
  "error": {
    "code": "unauthorized",
    "message": "Insufficient permissions to access identity",
    "details": {
      "required_capability": "security/identity",
      "peer_capabilities": ["orchestration"]
    }
  }
}
```

**Key Insight**: HTTP status code tells you success/failure. Body format can be consistent.

---

## 🎨 Proposed Idiomatic Pattern

### Pattern 1: HTTP Status Codes (Recommended)

**For Success** (200, 201, etc.):
- Return **unwrapped** data directly
- HTTP status indicates success

**For Errors** (400, 401, 403, 404, 500, etc.):
- Return **error object** with code, message, details
- HTTP status indicates failure

**Client Handling**:
```rust
// Rust example
let response = reqwest::get(url).await?;

if response.status().is_success() {
    // Parse data directly
    let data: IdentityResponse = response.json().await?;
    Ok(data)
} else {
    // Parse error
    let error: ErrorResponse = response.json().await?;
    Err(error.into())
}
```

**Benefits**:
- ✅ Idiomatic REST/HTTP
- ✅ Secure (errors don't expose data)
- ✅ Simple client code
- ✅ Industry standard
- ✅ Works with any HTTP client
- ✅ Compatible with OpenAPI/Swagger

---

### Pattern 2: Content Negotiation (Advanced)

**Request Specifies Format**:
```http
GET /api/v1/trust/identity
Accept: application/json
X-Response-Format: unwrapped
```

**or**

```http
GET /api/v1/trust/identity
Accept: application/json
X-Response-Format: wrapped
```

**Server Responds Accordingly**:
```rust
// In API handler
let format = request
    .headers()
    .get("X-Response-Format")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("unwrapped");

match format {
    "wrapped" => Ok(Json(ApiResponse { success: true, data })),
    _ => Ok(Json(data)),
}
```

**Benefits**:
- ✅ Client chooses format
- ✅ Backward compatible
- ✅ Agnostic (supports both)

**Drawbacks**:
- ❌ More complexity
- ❌ Non-standard header

---

### Pattern 3: API Versioning

**v1 Endpoints** (Wrapped - for backward compatibility):
```
GET /api/v1/trust/identity
→ {"success": true, "data": {...}}
```

**v2 Endpoints** (Unwrapped - modern):
```
GET /api/v2/trust/identity
→ {...}
```

**Benefits**:
- ✅ Clear migration path
- ✅ Both versions coexist
- ✅ No breaking changes

**Client Code**:
```rust
// Clients can choose version
let endpoint = format!("{}/api/{}/trust/identity", base_url, api_version);
```

---

## 🎯 Recommended Evolution for ecoPrimals

### Phase 1: Immediate Fix (This Week)

**Short-term**: Make current integration work

**Option A** - BearDog unwraps `/api/v1/trust/identity`:
```rust
// Quick fix for current integration
Ok(Json(identity_response))
```

**Option B** - Songbird handles wrapped format:
```rust
// Parse wrapper if present
if let Ok(wrapped) = response.json::<ApiResponse<T>>().await {
    wrapped.data
} else {
    response.json::<T>().await?
}
```

**Timeline**: 10 minutes  
**Goal**: Unblock two-tower test

---

### Phase 2: Standardize on HTTP Status Codes (Next Sprint)

**For All Primals**: Use HTTP status codes for success/failure

**Success Responses** (200-299):
```rust
// Return data directly
Ok(Json(data))
```

**Error Responses** (400-599):
```rust
// Return error object with appropriate status
Err(ApiError {
    status: StatusCode::FORBIDDEN,
    error: ErrorDetail {
        code: "unauthorized",
        message: "Insufficient permissions",
        details: serde_json::json!({...})
    }
})
```

**Standard Error Format** (RFC 7807 Problem Details):
```json
{
  "type": "https://ecoprimals.org/errors/unauthorized",
  "title": "Unauthorized Access",
  "status": 403,
  "detail": "Peer lacks required capability: security/identity",
  "instance": "/api/v1/trust/identity",
  "peer_id": "unknown-tower",
  "required_capability": "security/identity"
}
```

**Timeline**: 1-2 weeks  
**Goal**: Industry-standard REST patterns

---

### Phase 3: Agnostic Client Library (Future)

**Universal Primal Client** that handles any format:

```rust
pub struct UniversalPrimalClient {
    http_client: reqwest::Client,
    base_url: String,
}

impl UniversalPrimalClient {
    pub async fn call<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self.http_client
            .get(format!("{}/{}", self.base_url, endpoint))
            .send()
            .await?;

        // Check HTTP status first (idiomatic REST)
        if response.status().is_success() {
            // Try unwrapped format first
            if let Ok(data) = response.json::<T>().await {
                return Ok(data);
            }
            
            // Fall back to wrapped format
            let wrapped: ApiResponse<T> = response.json().await?;
            Ok(wrapped.data)
        } else {
            // Parse error
            let error: ErrorResponse = response.json().await?;
            Err(error.into())
        }
    }
}
```

**Benefits**:
- ✅ Works with any primal (wrapped or unwrapped)
- ✅ Automatic format detection
- ✅ Clean client code

**Timeline**: 1 month  
**Goal**: Zero-coupling, format-agnostic clients

---

## 📋 Implementation Guide

### For BearDog Team (Immediate)

**Step 1**: Use HTTP status codes for errors

**Before**:
```rust
Ok(Json(ApiResponse {
    success: false,
    error: Some("unauthorized")
}))
```

**After**:
```rust
Err((
    StatusCode::FORBIDDEN,
    Json(ErrorResponse {
        error: "unauthorized",
        message: "Insufficient permissions"
    })
))
```

**Step 2**: Return unwrapped data for success

**Before**:
```rust
Ok(Json(ApiResponse {
    success: true,
    data: identity
}))
```

**After**:
```rust
// HTTP 200 indicates success
Ok(Json(identity))
```

---

### For Songbird Team (Immediate)

**Step 1**: Check HTTP status first

**Before**:
```rust
let identity: IdentityResponse = response.json().await?;
```

**After**:
```rust
if response.status().is_success() {
    let identity: IdentityResponse = response.json().await?;
    Ok(identity)
} else {
    let error: ErrorResponse = response.json().await?;
    Err(error.into())
}
```

**Step 2**: Handle both formats (transitional)

```rust
// Try unwrapped first
if let Ok(identity) = response.json::<IdentityResponse>().await {
    return Ok(identity);
}

// Fall back to wrapped
let wrapped: ApiResponse<IdentityResponse> = response.json().await?;
Ok(wrapped.data)
```

---

## 🎯 Long-Term Vision: OpenAPI-Driven

### OpenAPI Schema Defines Format

**Primal advertises its API schema**:
```yaml
openapi: 3.1.0
info:
  title: BearDog Security API
  version: 2.0.0

paths:
  /api/v2/trust/identity:
    get:
      responses:
        '200':
          description: Success
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/IdentityResponse'
        '403':
          description: Forbidden
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
```

**Client reads schema and adapts**:
```rust
let schema = primal.get_openapi_schema().await?;
let client = OpenApiClient::from_schema(schema);

// Client automatically handles format based on schema
let identity: IdentityResponse = client.get("/trust/identity").await?;
```

**Benefits**:
- ✅ Format is self-documenting
- ✅ Clients auto-adapt
- ✅ No hardcoding
- ✅ Industry standard

---

## 📊 Decision Matrix

| Pattern | Complexity | Standards | Backward Compat | Agnostic | Recommended |
|---------|-----------|-----------|-----------------|----------|-------------|
| **HTTP Status Codes** | Low | High | Medium | Medium | ✅ **Phase 2** |
| **Content Negotiation** | Medium | Medium | High | High | 🤔 Maybe |
| **API Versioning** | Medium | High | High | Low | 🤔 Long-term |
| **Universal Client** | High | Medium | High | High | ✅ **Phase 3** |
| **OpenAPI-Driven** | High | High | High | High | ✅ **Ultimate** |

---

## 🚀 Action Plan

### This Week (Immediate)
1. **Quick Fix**: BearDog unwraps identity endpoint OR Songbird handles wrapper
2. **Document**: Update Generic Trust API spec with HTTP status code pattern
3. **Test**: Verify two-tower federation works

### Next Sprint (1-2 weeks)
4. **Standardize**: All primals use HTTP status codes for success/failure
5. **Error Format**: Implement RFC 7807 Problem Details across ecosystem
6. **Testing**: Update all E2E tests for new patterns

### Next Month (Future)
7. **Universal Client**: Build format-agnostic client library
8. **OpenAPI**: All primals expose OpenAPI schemas
9. **Auto-Adaptation**: Clients read schemas and auto-adapt

---

## 📝 Updated Generic Trust API Spec

### Success Response (HTTP 200)
```json
{
  "decision": "auto_accept",
  "confidence": 1.0,
  "reason": "Same genetic family",
  "reason_code": "same_genetic_family",
  "metadata": {...}
}
```

### Error Response (HTTP 4xx/5xx)
```json
{
  "type": "https://ecoprimals.org/errors/unauthorized",
  "title": "Unauthorized",
  "status": 403,
  "detail": "Peer lacks required capability",
  "instance": "/api/v1/trust/evaluate",
  "code": "unauthorized",
  "peer_id": "unknown-tower"
}
```

### Client Pattern
```rust
let response = client.post(url).send().await?;

match response.status() {
    StatusCode::OK => {
        let data = response.json().await?;
        Ok(data)
    }
    status => {
        let error = response.json::<ErrorResponse>().await?;
        Err(ApiError::from_response(status, error))
    }
}
```

---

## 🎯 Key Principles

1. **HTTP Status Codes Are The Source of Truth**
   - 2xx = success (unwrapped data)
   - 4xx/5xx = error (error object)

2. **Consistent Error Format**
   - RFC 7807 Problem Details
   - Machine-readable codes
   - Human-readable messages

3. **Secure by Default**
   - Errors never expose sensitive data
   - Status code prevents accidental data leaks

4. **Agnostic Clients**
   - Read OpenAPI schemas
   - Auto-adapt to formats
   - No hardcoded assumptions

5. **Backward Compatible**
   - Support wrapped format during transition
   - Graceful degradation
   - Version negotiation

---

## 📞 Next Steps

### Immediate (This Session)
- [ ] Share this architecture doc with BearDog & Songbird teams
- [ ] Decide: Quick fix (unwrap) or graceful handling (both formats)
- [ ] Update Generic Trust API spec with HTTP status pattern
- [ ] Unblock two-tower test

### Short-Term (This Week)
- [ ] Document HTTP status code pattern for all primals
- [ ] Create example implementations
- [ ] Update integration tests

### Long-Term (Next Month)
- [ ] Implement Universal Client library
- [ ] Migrate all primals to HTTP status pattern
- [ ] Add OpenAPI schema endpoints

---

**Status**: Architecture defined  
**Immediate**: Quick fix to unblock testing  
**Evolution**: HTTP status codes → Universal Client → OpenAPI-driven  
**Timeline**: Immediate fix (10 min) → Full evolution (1-2 months)

🏗️ **Modern, idiomatic, agnostic API patterns for the ecoPrimals ecosystem!** 🏗️

