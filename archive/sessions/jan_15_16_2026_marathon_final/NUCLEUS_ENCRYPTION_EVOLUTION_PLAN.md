# 🔒 NUCLEUS Encryption Evolution - Zero-Trust by Default

**Date**: January 15, 2026  
**Status**: 🎯 Next Evolution Priority  
**Goal**: Encryption by default with full traceability and zero-knowledge proofs

---

## 📊 **Current State Assessment**

### ✅ **What We Have (Infrastructure)**

1. **BearDog Running** (PID 712206)
   - Socket: `/tmp/beardog-nat0-default.sock`
   - JSON-RPC API: `encryption.encrypt`, `encryption.decrypt`
   - Algorithm: AES-256-GCM
   - FIDO2/HSM-backed keys
   - Genetic lineage (seed derivation)

2. **BearDog Client API** (`biomeos-core`)
   - `CryptoService::encrypt(data, key_ref)` → `EncryptedData`
   - `CryptoService::decrypt(encrypted, key_ref)` → plaintext
   - `KeyService::generate(key_type)` → key_ref
   - `BtspService::establish_tunnel()` → encrypted P2P

3. **All Primals Aware of BearDog**
   - Songbird: `SONGBIRD_SECURITY_PROVIDER=/tmp/beardog-nat0-default.sock`
   - Toadstool: Connected to Songbird (has BearDog access)
   - NestGate: `NESTGATE_SECURITY_PROVIDER=/tmp/beardog-nat0-default.sock`

4. **Family Genetics**
   - Family ID: `nat0`
   - Seed derivation for child identities
   - Lineage verification

### ❌ **What We're Missing (The Gap)**

1. **Unix Socket Data is PLAINTEXT**
   - Current: JSON-RPC over Unix sockets (fast but unencrypted)
   - Needed: Encrypted JSON-RPC (BTSP over Unix sockets)

2. **No Automatic Data Enclaving**
   - Current: Toadstool computes on plaintext
   - Needed: Ephemeral BearDog genetics per compute task

3. **No Storage Encryption**
   - Current: NestGate stores data as-is
   - Needed: Encrypt-at-rest by default

4. **No Zero-Knowledge Traceability**
   - Current: No audit logs
   - Needed: Full lineage tracking without exposing data

---

## 🎯 **Your Vision: Encryption by Default**

### **Core Principles**

1. **ALL data encrypted at rest** (NestGate + BearDog)
2. **ALL data encrypted in transit** (Songbird BTSP + BearDog)
3. **ALL compute on encrypted data** (Toadstool enclaves + BearDog)
4. **Ephemeral genetics per interaction** (BearDog seed derivation)
5. **Zero-knowledge traceability** (logs show lineage, not data)
6. **HSM-backed keys** (FIDO2 hardware security)

### **Example Workflow**

```
User Request: "Process datasets A & B"
  ↓
1. Neural API receives request
2. Creates ephemeral task ID: task_abc123
3. Asks BearDog: "derive_child_seed(family_nat0, task_abc123)"
   → BearDog returns: ephemeral_key_xyz
4. Neural API coordinates:
   a. NestGate.get("dataset_a") → returns encrypted blob
   b. NestGate.get("dataset_b") → returns encrypted blob
   c. Toadstool.compute_enclave(
        task_id: "task_abc123",
        ephemeral_key: "ephemeral_key_xyz",
        encrypted_inputs: [dataset_a_blob, dataset_b_blob],
        operation: "merge"
      )
5. Inside Toadstool enclave:
   a. BearDog.decrypt(dataset_a_blob, ephemeral_key_xyz) → plaintext A
   b. BearDog.decrypt(dataset_b_blob, ephemeral_key_xyz) → plaintext B
   c. Execute operation: result = merge(A, B)
   d. BearDog.encrypt(result, ephemeral_key_xyz) → encrypted result
   e. Wipe plaintext A, B, result from memory
   f. BearDog.destroy_key(ephemeral_key_xyz)
6. Toadstool returns: encrypted_result_blob
7. NestGate.store("result_c", encrypted_result_blob)
8. BearDog.log_lineage({
     task: "task_abc123",
     parent: "family_nat0",
     inputs: ["dataset_a_hash", "dataset_b_hash"],
     output: "result_c_hash",
     timestamp: "2026-01-15T13:53:56Z"
   })
9. Respond to user: "Result stored as result_c"
```

**Zero-Knowledge**: User sees "result_c exists", not the data itself.  
**Traceability**: Audit log shows lineage, hashes, not plaintext.  
**Ephemeral**: task_abc123 key destroyed after use.

---

## 🏗️ **Implementation Plan**

### **Phase 1: Encrypted Storage (NestGate + BearDog)** - 2 weeks

**Goal**: All data stored in NestGate is encrypted at rest.

**Changes**:

1. **NestGate Storage Layer**
   ```rust
   // crates/nestgate/src/storage/encrypted_backend.rs
   
   pub struct EncryptedStorageBackend {
       beardog: BearDogClient,
       backend: Box<dyn StorageBackend>, // Filesystem, S3, etc.
   }
   
   impl EncryptedStorageBackend {
       async fn store(&self, key: &str, data: &[u8]) -> Result<()> {
           // Generate encryption key for this dataset
           let key_ref = self.beardog.generate_key("dataset_key").await?;
           
           // Encrypt data
           let encrypted = self.beardog.encrypt(data, &key_ref).await?;
           
           // Store encrypted blob + metadata
           let metadata = EncryptionMetadata {
               key_ref,
               algorithm: "AES-256-GCM",
               nonce: encrypted.nonce,
               tag: encrypted.tag,
           };
           
           self.backend.store(key, &encrypted.ciphertext).await?;
           self.backend.store(&format!("{}.meta", key), &serde_json::to_vec(&metadata)?).await?;
           
           Ok(())
       }
       
       async fn retrieve(&self, key: &str) -> Result<Vec<u8>> {
           // Get encrypted blob + metadata
           let ciphertext = self.backend.retrieve(key).await?;
           let metadata: EncryptionMetadata = 
               serde_json::from_slice(&self.backend.retrieve(&format!("{}.meta", key)).await?)?;
           
           // Decrypt data
           let encrypted = EncryptedData {
               ciphertext,
               nonce: metadata.nonce,
               tag: metadata.tag,
               algorithm: metadata.algorithm,
           };
           
           let plaintext = self.beardog.decrypt(&encrypted, &metadata.key_ref).await?;
           
           Ok(plaintext)
       }
   }
   ```

2. **NestGate API**
   - Update `storage.store` to use `EncryptedStorageBackend`
   - Update `storage.retrieve` to decrypt automatically
   - All existing APIs work the same (encryption is transparent)

3. **Testing**
   - Store plaintext → verify encrypted on disk
   - Retrieve → verify decrypted correctly
   - Key rotation
   - BearDog unavailable fallback

**Deliverable**: NestGate stores all data encrypted, transparent to users.

---

### **Phase 2: Encrypted Transport (BTSP over Unix Sockets)** - 2 weeks

**Goal**: All primal-to-primal communication is encrypted.

**Changes**:

1. **BTSP Socket Wrapper**
   ```rust
   // crates/biomeos-core/src/clients/transport/btsp_unix.rs
   
   pub struct BtspUnixTransport {
       socket_path: PathBuf,
       beardog: BearDogClient,
       tunnel_id: String,
   }
   
   impl BtspUnixTransport {
       async fn connect(socket_path: &Path, beardog: &BearDogClient) -> Result<Self> {
           // Establish BTSP tunnel over Unix socket
           let tunnel = beardog.establish_tunnel(
               "local_unix_tunnel",
               socket_path.to_str().unwrap()
           ).await?;
           
           Ok(Self {
               socket_path: socket_path.to_path_buf(),
               beardog: beardog.clone(),
               tunnel_id: tunnel.tunnel_id,
           })
       }
       
       async fn send(&self, data: &[u8]) -> Result<()> {
           // Encrypt data using BTSP tunnel
           let encrypted = self.beardog.tunnel_send(&self.tunnel_id, data).await?;
           
           // Send over Unix socket
           let mut stream = UnixStream::connect(&self.socket_path).await?;
           stream.write_all(&encrypted).await?;
           
           Ok(())
       }
       
       async fn receive(&self) -> Result<Vec<u8>> {
           // Receive from Unix socket
           let mut stream = UnixStream::connect(&self.socket_path).await?;
           let mut encrypted = Vec::new();
           stream.read_to_end(&mut encrypted).await?;
           
           // Decrypt using BTSP tunnel
           let plaintext = self.beardog.tunnel_receive(&self.tunnel_id, &encrypted).await?;
           
           Ok(plaintext)
       }
   }
   ```

2. **Update PrimalTransport**
   ```rust
   // crates/biomeos-core/src/primal_client/transport.rs
   
   pub enum PrimalTransport {
       UnixSocket(BtspUnixTransport),  // NEW: Encrypted Unix socket
       Http(HttpTransport),             // Fallback
   }
   ```

3. **Auto-detect encryption support**
   - If BearDog available → use BTSP
   - If no BearDog → fallback to plaintext Unix socket
   - Log warning if encryption unavailable

**Deliverable**: All JSON-RPC over Unix sockets is encrypted via BTSP.

---

### **Phase 3: Ephemeral Genetics & Enclaves (Toadstool)** - 3 weeks

**Goal**: Every compute task gets its own ephemeral BearDog genetics.

**Changes**:

1. **Ephemeral Key Management**
   ```rust
   // crates/toadstool/src/enclave/ephemeral_genetics.rs
   
   pub struct EphemeralGenetics {
       beardog: BearDogClient,
       family_id: String,
       active_tasks: Arc<Mutex<HashMap<String, String>>>, // task_id → key_ref
   }
   
   impl EphemeralGenetics {
       async fn spawn_genetics(&self, task_id: &str) -> Result<String> {
           // Derive child seed from family lineage
           let child_seed = self.beardog.derive_child_seed(
               &self.family_id,
               task_id
           ).await?;
           
           // Generate ephemeral key from child seed
           let key_ref = self.beardog.generate_key_from_seed(
               &child_seed,
               "ephemeral_compute_key"
           ).await?;
           
           // Track active task
           self.active_tasks.lock().await.insert(task_id.to_string(), key_ref.clone());
           
           info!("🧬 Spawned ephemeral genetics for task: {}", task_id);
           
           Ok(key_ref)
       }
       
       async fn destroy_genetics(&self, task_id: &str) -> Result<()> {
           // Get key ref
           let key_ref = self.active_tasks.lock().await.remove(task_id)
               .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;
           
           // Destroy key (wipe from memory)
           self.beardog.destroy_key(&key_ref).await?;
           
           info!("🧬 Destroyed ephemeral genetics for task: {}", task_id);
           
           Ok(())
       }
   }
   ```

2. **Compute Enclave**
   ```rust
   // crates/toadstool/src/enclave/secure_compute.rs
   
   pub struct SecureComputeEnclave {
       genetics: EphemeralGenetics,
       beardog: BearDogClient,
   }
   
   impl SecureComputeEnclave {
       async fn execute_task(&self, task: ComputeTask) -> Result<EncryptedResult> {
           let task_id = task.id.clone();
           
           // 1. Spawn ephemeral genetics
           let key_ref = self.genetics.spawn_genetics(&task_id).await?;
           
           // 2. Decrypt inputs using ephemeral key
           let mut decrypted_inputs = Vec::new();
           for encrypted_input in task.encrypted_inputs {
               let plaintext = self.beardog.decrypt(&encrypted_input, &key_ref).await?;
               decrypted_inputs.push(plaintext);
           }
           
           // 3. Execute computation
           let result = self.execute_operation(&task.operation, &decrypted_inputs)?;
           
           // 4. Encrypt result with ephemeral key
           let encrypted_result = self.beardog.encrypt(&result, &key_ref).await?;
           
           // 5. Wipe plaintext from memory (overwrite with zeros)
           for input in decrypted_inputs.iter_mut() {
               input.zeroize();
           }
           result.zeroize();
           
           // 6. Destroy ephemeral genetics
           self.genetics.destroy_genetics(&task_id).await?;
           
           // 7. Log lineage (zero-knowledge)
           self.beardog.log_lineage(LineageEvent {
               task_id: task_id.clone(),
               parent: self.genetics.family_id.clone(),
               inputs: task.encrypted_inputs.iter().map(|i| hash(i)).collect(),
               output: hash(&encrypted_result.ciphertext),
               timestamp: Utc::now(),
           }).await?;
           
           Ok(encrypted_result)
       }
   }
   ```

3. **Toadstool API Updates**
   - New endpoint: `compute.execute_encrypted(task)`
   - Accepts encrypted inputs
   - Returns encrypted outputs
   - All compute happens in isolated enclave

**Deliverable**: Toadstool executes on encrypted data with ephemeral keys.

---

### **Phase 4: Zero-Knowledge Traceability (BearDog Audit)** - 1 week

**Goal**: Full audit trail without exposing plaintext data.

**Changes**:

1. **Lineage Tracking**
   ```rust
   // crates/beardog/src/lineage/tracker.rs
   
   pub struct LineageTracker {
       storage: LineageStorage, // Append-only log
   }
   
   impl LineageTracker {
       async fn log_event(&self, event: LineageEvent) -> Result<()> {
           // Hash all data references
           let event_hash = LineageEventHash {
               task_id: event.task_id,
               parent: event.parent,
               input_hashes: event.inputs.iter().map(|i| hash(i)).collect(),
               output_hash: hash(&event.output),
               timestamp: event.timestamp,
               signature: self.sign(&event)?, // Cryptographic proof
           };
           
           // Store in append-only log
           self.storage.append(event_hash).await?;
           
           Ok(())
       }
       
       async fn verify_lineage(&self, output_hash: &str) -> Result<LineageChain> {
           // Traverse lineage back to genesis
           let mut chain = Vec::new();
           let mut current = output_hash.to_string();
           
           while let Some(event) = self.storage.find_by_output(&current).await? {
               chain.push(event.clone());
               if event.parent == "genesis" {
                   break;
               }
               current = event.parent;
           }
           
           Ok(LineageChain { events: chain })
       }
   }
   ```

2. **BearDog API**
   - `lineage.log(event)` - Log lineage event
   - `lineage.verify(output_hash)` - Verify lineage chain
   - `lineage.query(task_id)` - Query lineage by task

3. **Neural API Integration**
   - Log every graph node execution
   - Track primal interactions
   - Provide lineage query endpoint

**Deliverable**: Full zero-knowledge audit trail for all NUCLEUS operations.

---

## 📈 **Success Metrics**

| Metric | Target | How to Verify |
|--------|--------|---------------|
| **Storage Encryption** | 100% | All NestGate data encrypted on disk |
| **Transport Encryption** | 100% | All Unix sockets use BTSP |
| **Compute Enclaving** | 100% | All Toadstool tasks use ephemeral keys |
| **Key Lifetime** | <1 minute | Ephemeral keys destroyed after task |
| **Audit Coverage** | 100% | All operations logged in lineage |
| **Performance Impact** | <20% | Encryption overhead minimal |
| **Zero Plaintext Logs** | 100% | No sensitive data in logs |

---

## 🧪 **Testing Strategy**

### **Unit Tests**
- Encrypt/decrypt round-trip
- Ephemeral key lifecycle
- Lineage tracking
- BTSP tunnel establishment

### **Integration Tests**
- Full workflow: NestGate → Toadstool → NestGate
- Multi-task concurrency
- Key destruction verification
- Audit trail completeness

### **Security Tests**
- Memory dump after task (verify wiped)
- Socket sniffing (verify encrypted)
- Unauthorized access attempts
- Key exhaustion attacks

### **Performance Tests**
- Throughput: operations/second
- Latency: encryption overhead
- Memory: enclave footprint
- Concurrency: parallel tasks

---

## 🔮 **Future Enhancements**

### **Phase 5: Hardware Security (HSM/TPM)** - 2 weeks
- Integrate FIDO2 hardware tokens
- TPM-backed key storage
- Hardware-enforced enclaves

### **Phase 6: Multi-Party Computation (MPC)** - 4 weeks
- Split secrets across multiple nodes
- Threshold cryptography
- Secure multi-party computation

### **Phase 7: Zero-Knowledge Proofs (ZKP)** - 6 weeks
- zk-SNARKs for computation verification
- Prove correctness without revealing data
- Privacy-preserving analytics

---

## 💡 **Key Design Decisions**

### **1. Encryption is Transparent**
- Applications don't need to change
- biomeOS handles encryption/decryption automatically
- Opt-out is harder than opt-in (security by default)

### **2. BearDog is the Single Source of Truth**
- All keys managed by BearDog
- All lineage tracked by BearDog
- No key escrow, no backdoors

### **3. Ephemeral by Default**
- Keys live only as long as needed
- Memory wiped after use
- Defense-in-depth: stolen key is worthless

### **4. Zero-Knowledge Everywhere**
- Logs show hashes, not data
- Lineage proves correctness without exposure
- GDPR/HIPAA/SOC2 compliance built-in

---

## 📚 **Related Documents**

- `docs/architecture/BIOMEOS_ENCRYPTION_ARCHITECTURE.md` - Encryption design
- `specs/ENCRYPTION_STRATEGY_SPEC.md` - Strategy specification
- `specs/GENETIC_LINEAGE_ARCHITECTURE_SPEC.md` - Lineage architecture
- `docs/primal-integrations/BEARDOG_MIGRATION_GUIDE.md` - Migration guide
- `BIOMEOS_ATOMICS_ARCHITECTURE.md` - Atomic compositions

---

## 🎯 **Timeline**

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1 | 2 weeks | NestGate encrypted storage |
| Phase 2 | 2 weeks | BTSP over Unix sockets |
| Phase 3 | 3 weeks | Ephemeral genetics & enclaves |
| Phase 4 | 1 week | Zero-knowledge traceability |
| **Total** | **8 weeks** | **Full encryption by default** |

---

## 🚀 **Getting Started**

### **Immediate Next Steps**

1. **Week 1: NestGate Encrypted Storage**
   - Create `EncryptedStorageBackend` wrapper
   - Update NestGate to use it
   - Write comprehensive tests

2. **Week 2: Integration Testing**
   - Store data via NestGate
   - Verify encrypted on disk
   - Verify decrypted on retrieval

3. **Week 3: BTSP Transport Layer**
   - Implement `BtspUnixTransport`
   - Update `PrimalTransport` enum
   - Test encrypted JSON-RPC

4. **Week 4: Toadstool Enclave Prototype**
   - Build `EphemeralGenetics` service
   - Create `SecureComputeEnclave`
   - Test single encrypted task

---

**Version**: 1.0.0  
**Date**: January 15, 2026  
**Status**: 🎯 Ready for Implementation  
**Priority**: **CRITICAL - NEXT EVOLUTION**

🔒 **Encryption is not optional. It's foundational.** 🧬

