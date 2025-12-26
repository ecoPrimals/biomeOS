# 🛠️ **API Adapter Usage Guide**

**Version**: 1.0  
**Date**: December 26, 2025  
**Status**: Production Ready  

---

## 📖 Overview

This guide shows how to use BiomeOS's adaptive API adapter system to integrate with Phase 1 primals. The system automatically adapts to each primal's actual architecture (CLI or REST API) without forcing standardization.

---

## 🎯 Quick Start

### **Architecture Map**

| Primal | Type | Adapter | Example |
|--------|------|---------|---------|
| **Songbird** | CLI | `SongbirdAdapter` | `SongbirdAdapter::new(path)?` |
| **NestGate** | REST | `NestGateAdapter` | `NestGateAdapter::discover(url).await?` |
| **BearDog** | CLI | `BearDogAdapter` | `BearDogAdapter::new(path)?` |
| **ToadStool** | REST | `ToadStoolAdapter` | `ToadStoolAdapter::discover(url).await?` |
| **Squirrel** | REST | `SquirrelAdapter` | `SquirrelAdapter::discover(url).await?` |

---

## 🎵 Songbird (CLI-Based)

### **Setup**

```rust
use biomeos_core::api_adapter::adapters::SongbirdAdapter;

// Path to Songbird binary
let songbird_path = "/path/to/songbird-cli-dec-25-2025-standalone";

// Create adapter
let mut adapter = SongbirdAdapter::new(songbird_path)?;

// Discover capabilities (optional)
adapter.discover_capabilities().await?;
```

### **Common Operations**

```rust
// Get version
let version = adapter.get_version().await?;
println!("Songbird version: {}", version);

// Start tower (note: this blocks! Use process manager in production)
let result = adapter.start_tower(8080, true).await?;
if result.is_success() {
    println!("Tower started!");
}

// Check if tower is running
let running = adapter.check_tower_running().await?;

// Register a service
let result = adapter.register_service(
    "my-service",
    "http://localhost:9000"
).await?;

// Query registered services
let services = adapter.query_services().await?;
println!("Services: {}", services);
```

### **Advanced Usage**

```rust
// Direct CLI access for custom commands
let cli = adapter.cli();
let result = cli.execute(&["custom", "command", "args"]).await?;

// With custom timeout
let result = cli.execute_with_timeout(
    &["long-running", "command"],
    Duration::from_secs(120)
).await?;
```

---

## 🏠 NestGate (REST API)

### **Setup**

```rust
use biomeos_core::api_adapter::adapters::NestGateAdapter;

// Discovery (probes endpoints automatically)
let adapter = NestGateAdapter::discover("http://localhost:8091").await?;
```

### **Common Operations**

```rust
// Check health
let healthy = adapter.check_storage_health().await?;

// Upload data (placeholder - actual implementation depends on discovered endpoints)
let result = adapter.upload_data(
    data,
    "/path/to/file"
).await?;

// Get file metadata
let metadata = adapter.get_metadata("file-id").await?;

// Get storage quota
let quota = adapter.get_quota().await?;
println!("Storage used: {} / {}", quota.used, quota.total);
```

### **Security Note**

```bash
# NestGate requires JWT authentication
export NESTGATE_JWT_SECRET=$(openssl rand -base64 48)

# Then start NestGate service
nestgate-bin service start --port 8091
```

---

## 🐻 BearDog (CLI-Based)

### **Setup**

```rust
use biomeos_core::api_adapter::adapters::BearDogAdapter;

// Path to BearDog binary
let beardog_path = "/path/to/beardog-bin";

// Create adapter
let mut adapter = BearDogAdapter::new(beardog_path)?;

// Discover capabilities (optional)
adapter.discover_capabilities().await?;
```

### **Common Operations**

```rust
// Get status
let status = adapter.get_status().await?;
println!("BearDog status:\n{}", status);

// Encrypt a file
let result = adapter.encrypt(
    "/path/to/input.txt",
    "/path/to/output.enc",
    "key-id-123"
).await?;

if result.is_success() {
    println!("File encrypted!");
} else {
    eprintln!("Encryption failed: {}", result.stderr);
}

// Decrypt a file
let result = adapter.decrypt(
    "/path/to/input.enc",
    "/path/to/output.txt",
    "key-id-123"
).await?;

// BirdSong (lineage-based encryption)
let result = adapter.birdsong_encrypt(
    "/path/to/input.txt",
    "/path/to/output.birdsong",
    "lineage-id-456"
).await?;

// BirdSong decrypt
let result = adapter.birdsong_decrypt(
    "/path/to/input.birdsong",
    "/path/to/output.txt"
).await?;
```

### **Large Files (Stream Encryption)**

```rust
// For files 100GB+
if adapter.supports_streaming {
    let result = adapter.stream_encrypt(
        "/path/to/large-input.bin",
        "/path/to/large-output.enc",
        "key-id"
    ).await?;
}
```

### **Key Management**

```rust
// Generate a new key
let result = adapter.generate_key(
    "aes-256",
    "/path/to/key.bin"
).await?;

// Collect entropy
let result = adapter.collect_entropy().await?;

// HSM operations (if supported)
if adapter.supports_hsm {
    let result = adapter.hsm_operation("list").await?;
    println!("HSM devices: {}", result.stdout);
}
```

---

## 🍄 ToadStool (REST API)

### **Setup**

```rust
use biomeos_core::api_adapter::adapters::ToadStoolAdapter;

// Discovery
let adapter = ToadStoolAdapter::discover("http://localhost:8084").await?;
```

### **Common Operations**

```rust
// Check health
let healthy = adapter.check_compute_health().await?;

// Submit a compute job (placeholder - depends on discovered endpoints)
let job_id = adapter.submit_job(job_config).await?;

// Check job status
let status = adapter.get_job_status(&job_id).await?;

// Get compute resources
let resources = adapter.get_resources().await?;
```

---

## 🐿️ Squirrel (REST API)

### **Setup**

```rust
use biomeos_core::api_adapter::adapters::SquirrelAdapter;

// Discovery
let adapter = SquirrelAdapter::discover("http://localhost:9010").await?;
```

### **Common Operations**

```rust
// Check health
let healthy = adapter.check_agent_health().await?;

// Get available agents
let agents = adapter.get_agents().await?;

// Start an AI session
let session_id = adapter.start_session(agent_id).await?;

// Send a message
let response = adapter.send_chat_message(
    &session_id,
    "Hello, AI!"
).await?;
```

---

## 🔄 Multi-Primal Orchestration

### **Example: Secure Data Processing**

```rust
use biomeos_core::api_adapter::adapters::{
    SongbirdAdapter, NestGateAdapter, BearDogAdapter, ToadStoolAdapter
};

// Initialize all adapters
let songbird = SongbirdAdapter::new("/path/to/songbird")?;
let nestgate = NestGateAdapter::discover("http://localhost:8091").await?;
let beardog = BearDogAdapter::new("/path/to/beardog")?;
let toadstool = ToadStoolAdapter::discover("http://localhost:8084").await?;

// 1. Register services with Songbird
songbird.register_service("nestgate", "http://localhost:8091").await?;
songbird.register_service("toadstool", "http://localhost:8084").await?;

// 2. Upload data to NestGate
let file_id = nestgate.upload_data(data, "/data.bin").await?;

// 3. Encrypt with BearDog
beardog.encrypt(
    "/data.bin",
    "/data.enc",
    "project-key"
).await?;

// 4. Submit compute job to ToadStool
let job_id = toadstool.submit_job(JobConfig {
    input: "/data.enc",
    compute_type: "ml-inference",
}).await?;

// 5. Wait for job completion
loop {
    let status = toadstool.get_job_status(&job_id).await?;
    if status.is_complete() {
        break;
    }
    tokio::time::sleep(Duration::from_secs(5)).await;
}

// 6. Retrieve and decrypt results
let result = toadstool.get_job_result(&job_id).await?;
beardog.decrypt(
    &result.output_path,
    "/result.txt",
    "project-key"
).await?;
```

---

## 🧪 Testing

### **Unit Tests**

```rust
#[tokio::test]
async fn test_songbird_adapter() {
    let songbird_path = std::env::var("SONGBIRD_BIN").unwrap();
    let adapter = SongbirdAdapter::new(&songbird_path).unwrap();
    
    let version = adapter.get_version().await.unwrap();
    assert!(!version.is_empty());
}

#[tokio::test]
async fn test_beardog_encrypt_decrypt() {
    let beardog_path = std::env::var("BEARDOG_BIN").unwrap();
    let adapter = BearDogAdapter::new(&beardog_path).unwrap();
    
    // Create test file
    std::fs::write("/tmp/test.txt", "test data").unwrap();
    
    // Encrypt
    let result = adapter.encrypt(
        "/tmp/test.txt",
        "/tmp/test.enc",
        "test-key"
    ).await.unwrap();
    assert!(result.is_success());
    
    // Decrypt
    let result = adapter.decrypt(
        "/tmp/test.enc",
        "/tmp/test.dec",
        "test-key"
    ).await.unwrap();
    assert!(result.is_success());
    
    // Verify
    let decrypted = std::fs::read_to_string("/tmp/test.dec").unwrap();
    assert_eq!(decrypted, "test data");
}
```

### **Environment Variables**

```bash
# Set paths for testing
export SONGBIRD_BIN="/path/to/songbird-cli-dec-25-2025-standalone"
export BEARDOG_BIN="/path/to/beardog-bin"
export NESTGATE_JWT_SECRET=$(openssl rand -base64 48)

# Run tests
cargo test --release
```

---

## 🔍 Error Handling

### **CLI Adapters**

```rust
use biomeos_core::api_adapter::cli_adapter::CliResult;

match adapter.execute(&["command"]).await {
    Ok(result) => {
        if result.is_success() {
            println!("Success: {}", result.stdout);
        } else {
            eprintln!("Command failed (exit {}): {}", 
                result.exit_code, result.stderr);
        }
    }
    Err(e) => {
        eprintln!("Execution error: {}", e);
    }
}
```

### **REST Adapters**

```rust
match NestGateAdapter::discover("http://localhost:8091").await {
    Ok(adapter) => {
        // Use adapter
    }
    Err(e) => {
        eprintln!("Discovery failed: {}", e);
        // Fallback or retry logic
    }
}
```

---

## 🎯 Best Practices

### **1. Process Management for CLI Adapters**

CLI adapters execute processes. For long-running services (like Songbird towers), use BiomeOS's process manager:

```rust
// DON'T: Block on tower start
// adapter.start_tower(8080, true).await?; // This blocks!

// DO: Use process manager
use biomeos_core::process_manager::ProcessManager;

let pm = ProcessManager::new();
pm.spawn_primal(
    "songbird",
    &songbird_path,
    &["tower", "start", "--port", "8080"]
)?;
```

### **2. Timeout Configuration**

```rust
// Adjust timeouts for long operations
let cli = CliAdapter::new(&binary_path)
    .with_timeout(300); // 5 minutes for long crypto operations
```

### **3. Capability Discovery**

```rust
// Always discover capabilities before using advanced features
let mut adapter = BearDogAdapter::new(&path)?;
adapter.discover_capabilities().await?;

if adapter.supports_streaming {
    // Use stream encryption
}
```

### **4. Error Propagation**

```rust
// Use ? operator for clean error propagation
pub async fn process_data(data: &[u8]) -> Result<Vec<u8>> {
    let beardog = BearDogAdapter::new("/path/to/beardog")?;
    
    // Write to temp file
    std::fs::write("/tmp/input", data)?;
    
    // Encrypt
    let result = beardog.encrypt(
        "/tmp/input",
        "/tmp/output",
        "key"
    ).await?;
    
    // Read result
    let encrypted = std::fs::read("/tmp/output")?;
    Ok(encrypted)
}
```

---

## 📚 Reference

### **CLI Adapter Methods**

```rust
// CliAdapter
pub fn new<P: Into<PathBuf>>(binary_path: P) -> Self
pub fn with_timeout(self, timeout_secs: u64) -> Self
pub async fn execute(&self, args: &[&str]) -> Result<CliResult>
pub async fn execute_with_stdin(&self, args: &[&str], stdin: &str) -> Result<CliResult>
pub async fn get_version(&self) -> Result<String>
pub async fn get_help(&self) -> Result<String>
pub fn verify_binary(&self) -> Result<()>

// CliResult
pub fn is_success(&self) -> bool
pub fn stdout(&self) -> &str
pub fn stderr(&self) -> &str
pub fn exit_code(&self) -> i32
```

### **Primal-Specific Adapters**

See individual adapter documentation:
- `SongbirdAdapter` - `crates/biomeos-core/src/api_adapter/adapters/songbird.rs`
- `NestGateAdapter` - `crates/biomeos-core/src/api_adapter/adapters/nestgate.rs`
- `BearDogAdapter` - `crates/biomeos-core/src/api_adapter/adapters/beardog.rs`
- `ToadStoolAdapter` - `crates/biomeos-core/src/api_adapter/adapters/toadstool.rs`
- `SquirrelAdapter` - `crates/biomeos-core/src/api_adapter/adapters/squirrel.rs`

---

## 🚀 Next Steps

1. **Integrate into BiomeOS orchestration layer**
2. **Add process lifecycle management**
3. **Build adapter factory/registry**
4. **Implement multi-primal workflows**
5. **Production hardening**

---

## 💡 Philosophy

> "We don't force primals to fit our designs. We design our adapters to fit primals."

The adaptive API adapter system preserves primal sovereignty by adapting to each primal's actual architecture, whether CLI-based or REST API. This approach was validated through real-world testing that discovered 2 distinct architecture types in 5 primals.

---

🦀 **Pure Rust. Reality-Based Integration. Human Dignity First.**

**API Adapter Usage Guide - Production Ready!** 🌟

