# 01 - Sovereign Storage with NestGate

**Duration**: 3 minutes  
**Prerequisites**: NestGate running on port 9020

---

## Overview

This demo shows NestGate's **sovereign storage** capabilities: ZFS-backed, lineage-gated, privacy-preserving data storage.

**What it demonstrates**:
- JWT-authenticated storage access
- Lineage-based authorization
- Data sovereignty (your data, your rules)
- Zero-knowledge architecture
- ZFS snapshot integration

---

## The Power of Sovereign Storage

### Traditional Cloud Storage (AWS S3, Google Cloud)
```
Your Data
   ↓
Vendor Controls:
  • Access policies
  • Retention periods
  • Geographic location
  • Encryption keys
  • Audit logs
  • Who can see what

Result: Vendor sovereignty, not user sovereignty
```

### NestGate (Sovereign Storage)
```
Your Data
   ↓
You Control:
  ✅ Access policies (via lineage)
  ✅ Retention periods
  ✅ Geographic location
  ✅ Encryption keys
  ✅ Audit logs
  ✅ Who can see what

Result: True user sovereignty
```

---

## Run the Demo

```bash
cd showcase/01-nestgate/01-sovereign-storage
./demo.sh
```

---

## What You'll See

### Phase 1: Authentication
```
🔐 Authenticating with NestGate...

Using JWT secret from environment
Generating access token...

✅ Authenticated successfully!
Token: eyJhbGciOiJIUzI1NiIs...
Expires: 1 hour
```

### Phase 2: Store Data
```
📦 Storing sovereign data...

Data: {
  "type": "personal_document",
  "content": "Medical records",
  "owner": "genesis-device-abc123",
  "lineage": "verified"
}

✅ Data stored successfully!
ID: doc_123abc
Location: /sovereign/data/doc_123abc
ZFS Snapshot: nestgate@2025-12-28_15:30:00
```

### Phase 3: Retrieve Data
```
📥 Retrieving data with lineage verification...

Request: GET /api/retrieve/doc_123abc
Headers: Authorization: Bearer eyJ...

✅ Data retrieved successfully!
Content: {...}
Lineage: Verified (genesis-device-abc123)
```

### Phase 4: Access Control
```
🚫 Testing unauthorized access...

Request: GET /api/retrieve/doc_123abc
Headers: Authorization: Bearer invalid_token

❌ Access denied!
Reason: Invalid lineage proof
Status: 403 Forbidden

✅ Sovereignty enforced!
```

---

## Architecture

```
┌─────────────────────────────────────────┐
│           NestGate API                  │
│   (JWT + Lineage Authentication)        │
└─────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│      Authorization Layer                │
│  • JWT validation                       │
│  • Lineage verification                 │
│  • Access policy enforcement            │
└─────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│          ZFS Storage Pool               │
│  • Snapshots (point-in-time)            │
│  • Compression (transparent)            │
│  • Deduplication (automatic)            │
│  • Checksums (data integrity)           │
└─────────────────────────────────────────┘
```

---

## NestGate Features

### 1. JWT Authentication
```bash
# Generate token
curl -X POST http://localhost:9020/api/auth \
  -H "Content-Type: application/json" \
  -d '{"secret":"'"$NESTGATE_JWT_SECRET"'"}'

# Returns
{
  "token": "eyJhbGc...",
  "expires_in": 3600
}
```

### 2. Lineage-Gated Access
```rust
// Only devices in your lineage can access your data
struct AccessPolicy {
    owner_lineage: String,      // Your genesis device
    allowed_lineages: Vec<String>, // Trusted devices
    public: bool,               // False by default
}

// Example:
// Owner: genesis-abc123
// Allowed: [laptop-xyz789, phone-def456]
// Public: false
// Result: Only you and your trusted devices can access
```

### 3. ZFS Snapshots
```bash
# Automatic snapshots every hour
nestgate snapshot list

Output:
  nestgate@2025-12-28_14:00:00  (1 hour ago)
  nestgate@2025-12-28_15:00:00  (just now)

# Rollback to previous version
nestgate snapshot rollback 2025-12-28_14:00:00

# Your data is protected from:
# - Ransomware
# - Accidental deletion
# - Corrupted writes
```

### 4. Zero-Knowledge Architecture
```
NestGate NEVER sees:
  ❌ Your plaintext data (encrypted by BearDog first)
  ❌ Your encryption keys (managed by BearDog)
  ❌ Your access patterns (no analytics)

NestGate ONLY stores:
  ✅ Encrypted blobs
  ✅ Metadata (owner lineage, timestamps)
  ✅ Access logs (for your audit)
```

---

## API Reference

### Store Data
```bash
POST /api/store
Headers:
  Authorization: Bearer <jwt_token>
  Content-Type: application/json
Body:
  {
    "data": "<base64_encoded_data>",
    "metadata": {
      "owner_lineage": "genesis-abc123",
      "type": "document",
      "tags": ["personal", "medical"]
    }
  }

Response:
  {
    "id": "doc_123abc",
    "stored_at": "2025-12-28T15:30:00Z",
    "snapshot": "nestgate@2025-12-28_15:30:00"
  }
```

### Retrieve Data
```bash
GET /api/retrieve/:id
Headers:
  Authorization: Bearer <jwt_token>

Response:
  {
    "id": "doc_123abc",
    "data": "<base64_encoded_data>",
    "metadata": {...},
    "retrieved_at": "2025-12-28T15:35:00Z"
  }
```

### List Data
```bash
GET /api/list
Headers:
  Authorization: Bearer <jwt_token>
Query:
  ?owner_lineage=genesis-abc123
  &type=document
  &limit=10

Response:
  {
    "items": [
      {
        "id": "doc_123abc",
        "metadata": {...},
        "created_at": "2025-12-28T15:30:00Z"
      }
    ],
    "total": 1
  }
```

### Delete Data
```bash
DELETE /api/delete/:id
Headers:
  Authorization: Bearer <jwt_token>

Response:
  {
    "id": "doc_123abc",
    "deleted_at": "2025-12-28T15:40:00Z",
    "snapshot_preserved": "nestgate@2025-12-28_15:30:00"
  }
```

---

## Use Cases

### Use Case 1: Personal Medical Records
```
Store medical records with NestGate:
  • Encrypted by BearDog first
  • Stored in NestGate with lineage gate
  • Only your devices can access
  • Snapshots protect from ransomware
  • Share with doctor via lineage grant

Result: Your health data, your control
```

### Use Case 2: Research Data
```
Store research datasets:
  • Large files (GBs or TBs)
  • ZFS deduplication saves space
  • Snapshots for reproducibility
  • Lineage-gated collaboration
  • No cloud vendor fees

Result: Sovereign research infrastructure
```

### Use Case 3: Family Photos
```
Store family photos:
  • Encrypted at rest
  • Accessible by family lineage
  • Snapshots preserve history
  • No subscription fees
  • No vendor lock-in

Result: Sovereign family archive
```

---

## Security Model

### Threat Model

**Protected Against**:
- ✅ Unauthorized access (JWT + lineage)
- ✅ Data corruption (ZFS checksums)
- ✅ Ransomware (ZFS snapshots)
- ✅ Hardware failure (ZFS redundancy)
- ✅ Vendor surveillance (self-hosted)

**NOT Protected Against** (requires BearDog):
- ❌ Physical disk theft (use BearDog encryption first!)
- ❌ Memory dumps (encrypt before sending to NestGate)
- ❌ Side-channel attacks (encrypt locally)

### Defense in Depth

```
Layer 1: BearDog Encryption
  ↓ (Encrypted data only)
Layer 2: NestGate JWT Auth
  ↓ (Authenticated requests only)
Layer 3: Lineage Verification
  ↓ (Authorized lineage only)
Layer 4: ZFS Integrity
  ↓ (Checksummed storage)
Layer 5: ZFS Snapshots
  ↓ (Point-in-time recovery)

Result: Military-grade data sovereignty
```

---

## Comparison

| Feature | AWS S3 | Google Cloud | NestGate |
|---------|--------|--------------|----------|
| **Sovereignty** | Vendor | Vendor | **You** |
| **Encryption Keys** | Vendor | Vendor | **You** |
| **Access Control** | IAM | IAM | **Lineage** |
| **Data Location** | Vendor regions | Vendor regions | **Your hardware** |
| **Audit Logs** | Vendor controls | Vendor controls | **You control** |
| **Snapshots** | Versioning ($) | Versioning ($) | **ZFS (free)** |
| **Deduplication** | No | No | **Yes (ZFS)** |
| **Cost** | Per GB/month | Per GB/month | **Zero fees** |
| **Vendor Lock-in** | High | High | **None** |
| **Privacy** | Vendor can access | Vendor can access | **Zero-knowledge** |

---

## Success Criteria

✅ **Authentication**: JWT tokens work correctly  
✅ **Storage**: Data stored and retrieved successfully  
✅ **Lineage**: Access control enforced  
✅ **Snapshots**: Point-in-time recovery available  
✅ **Zero-Knowledge**: NestGate never sees plaintext  

---

## Integration with BearDog

### Complete Workflow
```bash
# Step 1: Encrypt with BearDog
echo "Sensitive data" | beardog encrypt --key my-key > encrypted.bin

# Step 2: Store in NestGate
curl -X POST http://localhost:9020/api/store \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"data\":\"$(base64 encrypted.bin)\"}"

# Step 3: Retrieve from NestGate
curl http://localhost:9020/api/retrieve/doc_123abc \
  -H "Authorization: Bearer $TOKEN" \
  | jq -r '.data' | base64 -d > encrypted_retrieved.bin

# Step 4: Decrypt with BearDog
beardog decrypt --key my-key < encrypted_retrieved.bin

# Result: End-to-end encrypted sovereign storage!
```

---

## Next Steps

After this demo:
- **02-zfs-snapshots**: Deep dive into snapshot management
- **03-lineage-collaboration**: Share data via lineage
- **04-federation-replication**: Replicate across towers
- **05-benchscale-validation**: Validate with benchScale

---

**Philosophy**: *"Your data. Your hardware. Your rules. That's sovereignty."*

