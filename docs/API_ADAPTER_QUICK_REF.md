# 🎯 **API Adapter Quick Reference**

**Version**: 1.0 | **Date**: Dec 26, 2025 | **Status**: Production Ready

---

## 🚀 **Quick Start (30 seconds)**

```rust
use biomeos_core::api_adapter::adapters::*;

// CLI-based primals
let songbird = SongbirdAdapter::new("/path/to/songbird")?;
let beardog = BearDogAdapter::new("/path/to/beardog")?;

// REST API primals
let nestgate = NestGateAdapter::discover("http://localhost:8091").await?;
let toadstool = ToadStoolAdapter::discover("http://localhost:8084").await?;
let squirrel = SquirrelAdapter::discover("http://localhost:9010").await?;
```

---

## 📊 **Architecture Map**

| Primal | Type | Port | Adapter Pattern |
|--------|------|------|-----------------|
| **Songbird** 🎵 | CLI | 8080 | `::new(path)?` |
| **NestGate** 🏠 | REST | 8091 | `::discover(url).await?` |
| **BearDog** 🐻 | CLI | N/A | `::new(path)?` |
| **ToadStool** 🍄 | REST | 8084 | `::discover(url).await?` |
| **Squirrel** 🐿️ | REST | 9010 | `::discover(url).await?` |

---

## 🎵 **Songbird (CLI)**

```rust
let adapter = SongbirdAdapter::new("/path/to/songbird")?;

// Version
adapter.get_version().await?;

// Start tower (blocks! Use process manager in production)
adapter.start_tower(8080, true).await?;

// Register service
adapter.register_service("my-service", "http://localhost:9000").await?;

// Query services
adapter.query_services().await?;
```

---

## 🏠 **NestGate (REST)**

```rust
// Requires JWT!
// export NESTGATE_JWT_SECRET=$(openssl rand -base64 48)

let adapter = NestGateAdapter::discover("http://localhost:8091").await?;

// Health
adapter.check_storage_health().await?;

// Upload (placeholder - depends on discovered endpoints)
adapter.upload_data(data, "/path").await?;

// Metadata
adapter.get_metadata("file-id").await?;
```

---

## 🐻 **BearDog (CLI)**

```rust
let adapter = BearDogAdapter::new("/path/to/beardog")?;

// Status
adapter.get_status().await?;

// Encrypt
adapter.encrypt("/in.txt", "/out.enc", "key-id").await?;

// Decrypt
adapter.decrypt("/in.enc", "/out.txt", "key-id").await?;

// BirdSong (lineage encryption)
adapter.birdsong_encrypt("/in.txt", "/out.birdsong", "lineage-id").await?;
adapter.birdsong_decrypt("/in.birdsong", "/out.txt").await?;

// Large files (100GB+)
if adapter.supports_streaming {
    adapter.stream_encrypt("/big.bin", "/big.enc", "key").await?;
}

// Key generation
adapter.generate_key("aes-256", "/key.bin").await?;
```

---

## 🍄 **ToadStool (REST)**

```rust
let adapter = ToadStoolAdapter::discover("http://localhost:8084").await?;

// Health
adapter.check_compute_health().await?;

// Submit job (placeholder)
adapter.submit_job(config).await?;

// Job status
adapter.get_job_status(&job_id).await?;
```

---

## 🐿️ **Squirrel (REST)**

```rust
let adapter = SquirrelAdapter::discover("http://localhost:9010").await?;

// Health
adapter.check_agent_health().await?;

// Agents
adapter.get_agents().await?;

// Session
let session = adapter.start_session(agent_id).await?;

// Chat
adapter.send_chat_message(&session, "Hello!").await?;
```

---

## 🔧 **CLI Adapter (Advanced)**

```rust
use biomeos_core::api_adapter::cli_adapter::CliAdapter;

let cli = CliAdapter::new("/path/to/binary")
    .with_timeout(60); // seconds

// Execute
let result = cli.execute(&["arg1", "arg2"]).await?;

// With stdin
let result = cli.execute_with_stdin(&["cat"], "input data").await?;

// Check result
if result.is_success() {
    println!("Output: {}", result.stdout);
} else {
    eprintln!("Error (exit {}): {}", result.exit_code, result.stderr);
}
```

---

## 🎯 **Common Patterns**

### **Multi-Primal Workflow**

```rust
// 1. Register with Songbird
songbird.register_service("nestgate", "http://localhost:8091").await?;

// 2. Store in NestGate
let file_id = nestgate.upload_data(data, "/file").await?;

// 3. Encrypt with BearDog
beardog.encrypt("/file", "/file.enc", "key").await?;

// 4. Process with ToadStool
let job = toadstool.submit_job(config).await?;
```

### **Error Handling**

```rust
match adapter.execute(&["cmd"]).await {
    Ok(result) if result.is_success() => {
        // Success
    }
    Ok(result) => {
        eprintln!("Failed: {}", result.stderr);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### **Process Management (CLI)**

```rust
// DON'T: Block on long-running command
// adapter.start_tower(...).await?; // Blocks!

// DO: Use process manager
ProcessManager::new().spawn_primal(
    "songbird",
    &path,
    &["tower", "start", "--port", "8080"]
)?;
```

---

## 📚 **Environment Setup**

```bash
# Binary paths for testing
export SONGBIRD_BIN="/path/to/songbird-cli-dec-25-2025-standalone"
export BEARDOG_BIN="/path/to/beardog-bin"

# NestGate JWT (required!)
export NESTGATE_JWT_SECRET=$(openssl rand -base64 48)

# Run tests
cargo test --release
```

---

## 🎓 **Key Concepts**

**Adaptive Discovery**: System learns each primal's API instead of enforcing standards

**CLI vs REST**: 40% CLI-based (Songbird, BearDog), 60% REST (NestGate, ToadStool, Squirrel)

**Sovereignty**: No primal forced to change - adapters match reality

**Philosophy**: *"We adapt to primals, not the other way around"*

---

## 📖 **Full Documentation**

- **Usage Guide**: `docs/API_ADAPTER_USAGE_GUIDE.md`
- **Discovery Reports**: `showcase/api-adapter-test-results/`
- **Source Code**: `crates/biomeos-core/src/api_adapter/`

---

🦀 **Pure Rust. Reality-Based Integration. Human Dignity First.**

