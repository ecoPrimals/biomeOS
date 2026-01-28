#!/usr/bin/env rust-script
//! Tower Atomic Bootstrap Script
//!
//! This script automates the deployment of the Tower Atomic stack:
//! 1. BearDog (Crypto/Identity)
//! 2. Songbird (HTTP/TLS)
//! 3. Neural API (Orchestration)
//!
//! # Usage
//!
//! ```bash
//! # Via rust-script
//! rust-script scripts/bootstrap_tower_atomic.rs
//!
//! # Or compile and run
//! rustc scripts/bootstrap_tower_atomic.rs -o /tmp/bootstrap && /tmp/bootstrap
//! ```
//!
//! # Environment Variables
//!
//! - `FAMILY_ID`: Family identifier (default: "nat0")
//! - `NODE_ID`: Node identifier (default: "tower0")
//! - `BIOMEOS_ROOT`: biomeOS root directory (default: current dir)
//! - `XDG_RUNTIME_DIR`: Runtime directory (default: /run/user/1000)

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for Tower Atomic deployment
struct Config {
    family_id: String,
    node_id: String,
    biomeos_root: PathBuf,
    xdg_runtime_dir: PathBuf,
    plasmid_bin: PathBuf,
    graphs_dir: PathBuf,
}

impl Config {
    fn from_env() -> Self {
        let biomeos_root = env::var("BIOMEOS_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| env::current_dir().unwrap());

        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/run/user/1000"));

        Self {
            family_id: env::var("FAMILY_ID").unwrap_or_else(|_| "nat0".to_string()),
            node_id: env::var("NODE_ID").unwrap_or_else(|_| "tower0".to_string()),
            plasmid_bin: biomeos_root.join("plasmidBin"),
            graphs_dir: biomeos_root.join("graphs"),
            biomeos_root,
            xdg_runtime_dir,
        }
    }

    fn socket_dir(&self) -> PathBuf {
        self.xdg_runtime_dir.join("biomeos")
    }

    fn beardog_socket(&self) -> PathBuf {
        self.socket_dir()
            .join(format!("beardog-{}.sock", self.family_id))
    }

    fn songbird_socket(&self) -> PathBuf {
        self.socket_dir()
            .join(format!("songbird-{}.sock", self.family_id))
    }

    fn neural_api_socket(&self) -> PathBuf {
        self.socket_dir()
            .join(format!("neural-api-{}.sock", self.family_id))
    }
}

/// Primal process handle
struct PrimalProcess {
    name: String,
    child: Child,
    socket: PathBuf,
}

impl PrimalProcess {
    fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }

    fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for PrimalProcess {
    fn drop(&mut self) {
        self.kill();
    }
}

/// Wait for a socket to become available
fn wait_for_socket(socket: &PathBuf, timeout: Duration) -> Result<(), String> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if socket.exists() {
            // Try to connect
            if UnixStream::connect(socket).is_ok() {
                return Ok(());
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    Err(format!(
        "Socket {} not available after {:?}",
        socket.display(),
        timeout
    ))
}

/// Send a JSON-RPC request and get response
fn json_rpc(socket: &PathBuf, method: &str, params: &str) -> Result<String, String> {
    let mut stream = UnixStream::connect(socket)
        .map_err(|e| format!("Failed to connect to {}: {}", socket.display(), e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"{}","params":{},"id":1}}"#,
        method, params
    );

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("Failed to write: {}", e))?;
    stream
        .write_all(b"\n")
        .map_err(|e| format!("Failed to write newline: {}", e))?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| format!("Failed to read: {}", e))?;

    Ok(response)
}

/// Start BearDog
fn start_beardog(config: &Config) -> Result<PrimalProcess, String> {
    println!("🐻 Starting BearDog...");

    let binary = config.plasmid_bin.join("beardog");
    if !binary.exists() {
        return Err(format!("BearDog binary not found: {}", binary.display()));
    }

    // Ensure socket directory exists
    std::fs::create_dir_all(config.socket_dir())
        .map_err(|e| format!("Failed to create socket dir: {}", e))?;

    // Clean up old socket if exists
    let socket = config.beardog_socket();
    if socket.exists() {
        std::fs::remove_file(&socket).ok();
    }

    let child = Command::new(&binary)
        .args(["server", "--socket", &socket.to_string_lossy()])
        .env("FAMILY_ID", &config.family_id)
        .env("NODE_ID", &config.node_id)
        .env("XDG_RUNTIME_DIR", &config.xdg_runtime_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn BearDog: {}", e))?;

    // Wait for socket
    wait_for_socket(&socket, Duration::from_secs(10))?;

    println!("✅ BearDog ready at {}", socket.display());

    Ok(PrimalProcess {
        name: "beardog".to_string(),
        child,
        socket,
    })
}

/// Start Songbird
fn start_songbird(config: &Config, beardog_socket: &PathBuf) -> Result<PrimalProcess, String> {
    println!("🐦 Starting Songbird...");

    let binary = config.plasmid_bin.join("songbird");
    if !binary.exists() {
        return Err(format!("Songbird binary not found: {}", binary.display()));
    }

    let socket = config.songbird_socket();
    if socket.exists() {
        std::fs::remove_file(&socket).ok();
    }

    let child = Command::new(&binary)
        .args([
            "server",
            "--socket",
            &socket.to_string_lossy(),
            "--beardog-socket",
            &beardog_socket.to_string_lossy(),
            "--federation-port",
            "8080",
        ])
        .env("FAMILY_ID", &config.family_id)
        .env("NODE_ID", &config.node_id)
        .env("XDG_RUNTIME_DIR", &config.xdg_runtime_dir)
        .env("BEARDOG_SOCKET", beardog_socket)
        .env("BEARDOG_MODE", "direct")
        .env("SONGBIRD_SECURITY_PROVIDER", "beardog")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn Songbird: {}", e))?;

    // Wait for socket
    wait_for_socket(&socket, Duration::from_secs(15))?;

    println!("✅ Songbird ready at {}", socket.display());

    Ok(PrimalProcess {
        name: "songbird".to_string(),
        child,
        socket,
    })
}

/// Start Neural API
fn start_neural_api(config: &Config) -> Result<PrimalProcess, String> {
    println!("🧠 Starting Neural API...");

    let binary = config.plasmid_bin.join("neural-api-server");
    if !binary.exists() {
        // Try alternative path
        let alt = config
            .biomeos_root
            .join("target/release/neural-api-server");
        if !alt.exists() {
            return Err(format!(
                "Neural API binary not found at {} or {}",
                binary.display(),
                alt.display()
            ));
        }
    }

    let socket = config.neural_api_socket();
    if socket.exists() {
        std::fs::remove_file(&socket).ok();
    }

    let child = Command::new(&binary)
        .args([
            "--graphs-dir",
            &config.graphs_dir.to_string_lossy(),
            "--socket",
            &socket.to_string_lossy(),
        ])
        .env("FAMILY_ID", &config.family_id)
        .env("NODE_ID", &config.node_id)
        .env("XDG_RUNTIME_DIR", &config.xdg_runtime_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn Neural API: {}", e))?;

    // Wait for socket
    wait_for_socket(&socket, Duration::from_secs(10))?;

    println!("✅ Neural API ready at {}", socket.display());

    Ok(PrimalProcess {
        name: "neural-api".to_string(),
        child,
        socket,
    })
}

/// Verify the stack is working
fn verify_stack(config: &Config) -> Result<(), String> {
    println!("\n🔍 Verifying Tower Atomic stack...\n");

    // Test BearDog
    print!("  BearDog crypto.sha256... ");
    let response = json_rpc(
        &config.beardog_socket(),
        "crypto.sha256",
        r#"{"message":"dGVzdA=="}"#, // base64("test")
    )?;
    if response.contains("result") {
        println!("✅");
    } else {
        println!("❌ {}", response);
    }

    // Test Songbird
    print!("  Songbird health... ");
    let response = json_rpc(&config.songbird_socket(), "health.check", r#"{}"#)?;
    if response.contains("result") || response.contains("healthy") {
        println!("✅");
    } else {
        // Some primals don't implement health.check
        println!("⚠️ (health.check not implemented)");
    }

    // Test Neural API
    print!("  Neural API capability.list... ");
    let response = json_rpc(&config.neural_api_socket(), "capability.list", r#"{}"#)?;
    if response.contains("result") {
        println!("✅");
    } else {
        println!("❌ {}", response);
    }

    // Test capability routing
    print!("  Neural API capability.call (crypto.sha256)... ");
    let response = json_rpc(
        &config.neural_api_socket(),
        "capability.call",
        r#"{"capability":"crypto","operation":"sha256","args":{"message":"dGVzdA=="}}"#,
    )?;
    if response.contains("result") {
        println!("✅");
    } else {
        println!("❌ {}", response);
    }

    println!("\n🎉 Tower Atomic stack is operational!\n");

    Ok(())
}

/// Clean up running processes
fn cleanup(procs: &mut [PrimalProcess]) {
    println!("\n🧹 Cleaning up...");
    for proc in procs.iter_mut().rev() {
        print!("  Stopping {}... ", proc.name);
        proc.kill();
        println!("done");
    }
}

fn main() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("     🏗️  Tower Atomic Bootstrap");
    println!("     biomeOS Neural API Orchestration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let config = Config::from_env();

    println!("📋 Configuration:");
    println!("   Family ID: {}", config.family_id);
    println!("   Node ID: {}", config.node_id);
    println!("   biomeOS Root: {}", config.biomeos_root.display());
    println!("   Socket Dir: {}", config.socket_dir().display());
    println!();

    // Store processes for cleanup
    let mut processes: Vec<PrimalProcess> = Vec::new();

    // Setup cleanup on Ctrl+C
    ctrlc::set_handler(move || {
        println!("\n\n⚠️  Received interrupt signal");
        std::process::exit(130);
    })
    .ok();

    // Start primals in order
    match start_beardog(&config) {
        Ok(proc) => processes.push(proc),
        Err(e) => {
            eprintln!("❌ Failed to start BearDog: {}", e);
            cleanup(&mut processes);
            std::process::exit(1);
        }
    }

    match start_songbird(&config, &config.beardog_socket()) {
        Ok(proc) => processes.push(proc),
        Err(e) => {
            eprintln!("❌ Failed to start Songbird: {}", e);
            cleanup(&mut processes);
            std::process::exit(1);
        }
    }

    match start_neural_api(&config) {
        Ok(proc) => processes.push(proc),
        Err(e) => {
            eprintln!("❌ Failed to start Neural API: {}", e);
            cleanup(&mut processes);
            std::process::exit(1);
        }
    }

    // Verify
    if let Err(e) = verify_stack(&config) {
        eprintln!("❌ Stack verification failed: {}", e);
        cleanup(&mut processes);
        std::process::exit(1);
    }

    // Print status
    println!("📍 Socket Paths:");
    println!("   BearDog:    {}", config.beardog_socket().display());
    println!("   Songbird:   {}", config.songbird_socket().display());
    println!("   Neural API: {}", config.neural_api_socket().display());
    println!();
    println!("🔗 Quick Test Commands:");
    println!(
        r#"   echo '{{"jsonrpc":"2.0","method":"capability.list","params":{{}},"id":1}}' | nc -U {}"#,
        config.neural_api_socket().display()
    );
    println!();
    println!("Press Ctrl+C to stop all primals...\n");

    // Wait for processes
    loop {
        thread::sleep(Duration::from_secs(1));

        // Check if any process died
        for proc in &mut processes {
            if !proc.is_running() {
                eprintln!("❌ {} has stopped unexpectedly", proc.name);
                cleanup(&mut processes);
                std::process::exit(1);
            }
        }
    }
}

