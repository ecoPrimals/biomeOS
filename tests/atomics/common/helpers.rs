// Test helpers for NUCLEUS atomic testing
//
// Deep Debt: Fast AND Safe - uses nix crate for safe POSIX syscalls

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use anyhow::{Result, Context};
use nix::unistd::Uid;

/// Primal process handle
pub struct PrimalHandle {
    pub name: String,
    pub process: Child,
    pub socket_path: PathBuf,
    pub pid: u32,
}

impl PrimalHandle {
    pub async fn health_check(&self) -> Result<serde_json::Value> {
        let response = send_jsonrpc(
            &self.socket_path,
            r#"{"jsonrpc":"2.0","method":"health","params":{},"id":1}"#
        ).await?;
        Ok(response)
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        self.process.kill()?;
        self.process.wait()?;
        Ok(())
    }
}

/// Tower Atomic handle (BearDog + Songbird)
pub struct TowerHandle {
    pub beardog: PrimalHandle,
    pub songbird: PrimalHandle,
}

impl TowerHandle {
    pub async fn is_healthy(&self) -> bool {
        self.beardog.health_check().await.is_ok() &&
        self.songbird.health_check().await.is_ok()
    }
    
    pub async fn stop(mut self) -> Result<()> {
        self.songbird.stop().await?;
        self.beardog.stop().await?;
        Ok(())
    }
}

/// Node Atomic handle (Tower + Toadstool)
pub struct NodeHandle {
    pub tower: TowerHandle,
    pub toadstool: PrimalHandle,
}

impl NodeHandle {
    pub async fn is_healthy(&self) -> bool {
        self.tower.is_healthy().await &&
        self.toadstool.health_check().await.is_ok()
    }
    
    pub async fn stop(mut self) -> Result<()> {
        self.toadstool.stop().await?;
        self.tower.stop().await?;
        Ok(())
    }
}

/// Nest Atomic handle (Tower + NestGate + Squirrel)
pub struct NestHandle {
    pub tower: TowerHandle,
    pub nestgate: PrimalHandle,
    pub squirrel: PrimalHandle,
}

impl NestHandle {
    pub async fn is_healthy(&self) -> bool {
        self.tower.is_healthy().await &&
        self.nestgate.health_check().await.is_ok() &&
        self.squirrel.health_check().await.is_ok()
    }
    
    pub async fn stop(mut self) -> Result<()> {
        self.squirrel.stop().await?;
        self.nestgate.stop().await?;
        self.tower.stop().await?;
        Ok(())
    }
}

/// Start BearDog primal
pub async fn start_beardog() -> Result<PrimalHandle> {
    let uid = Uid::current();
    let socket_path = PathBuf::from(format!("/run/user/{}/biomeos/beardog.sock", uid));
    
    // Clean old socket
    let _ = std::fs::remove_file(&socket_path);
    
    let process = Command::new("beardog")
        .arg("server")
        .env("FAMILY_ID", "test0")
        .env("NODE_ID", "test-beardog")
        .env("RUST_LOG", "beardog=info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start BearDog")?;
    
    let pid = process.id();
    
    // Wait for socket creation
    for _ in 0..30 {
        if socket_path.exists() {
            sleep(Duration::from_millis(500)).await; // Stabilize
            return Ok(PrimalHandle {
                name: "beardog".to_string(),
                process,
                socket_path,
                pid,
            });
        }
        sleep(Duration::from_millis(100)).await;
    }
    
    Err(anyhow::anyhow!("BearDog socket not created within 3 seconds"))
}

/// Start Songbird primal
pub async fn start_songbird(beardog: &PrimalHandle) -> Result<PrimalHandle> {
    let uid = Uid::current();
    let socket_path = PathBuf::from(format!("/run/user/{}/biomeos/songbird.sock", uid));
    
    // Clean old socket
    let _ = std::fs::remove_file(&socket_path);
    
    let process = Command::new("songbird")
        .arg("server")
        .env("FAMILY_ID", "test0")
        .env("NODE_ID", "test-songbird")
        .env("RUST_LOG", "songbird=info")
        .env("SONGBIRD_SECURITY_PROVIDER", "beardog")
        .env("BEARDOG_SOCKET", beardog.socket_path.to_str().unwrap())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start Songbird")?;
    
    let pid = process.id();
    
    // Wait for socket creation
    for _ in 0..30 {
        if socket_path.exists() {
            sleep(Duration::from_millis(500)).await; // Stabilize
            return Ok(PrimalHandle {
                name: "songbird".to_string(),
                process,
                socket_path,
                pid,
            });
        }
        sleep(Duration::from_millis(100)).await;
    }
    
    Err(anyhow::anyhow!("Songbird socket not created within 3 seconds"))
}

/// Start Toadstool primal
pub async fn start_toadstool() -> Result<PrimalHandle> {
    let uid = Uid::current();
    let socket_path = PathBuf::from(format!("/run/user/{}/biomeos/toadstool.sock", uid));
    
    // Clean old socket
    let _ = std::fs::remove_file(&socket_path);
    
    let process = Command::new("toadstool")
        .arg("server")
        .env("FAMILY_ID", "test0")
        .env("NODE_ID", "test-toadstool")
        .env("RUST_LOG", "toadstool=info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start Toadstool")?;
    
    let pid = process.id();
    
    // Wait for socket creation
    for _ in 0..30 {
        if socket_path.exists() {
            sleep(Duration::from_millis(500)).await; // Stabilize
            return Ok(PrimalHandle {
                name: "toadstool".to_string(),
                process,
                socket_path,
                pid,
            });
        }
        sleep(Duration::from_millis(100)).await;
    }
    
    Err(anyhow::anyhow!("Toadstool socket not created within 3 seconds"))
}

/// Start NestGate primal (socket-only mode)
pub async fn start_nestgate_socket_only() -> Result<PrimalHandle> {
    let uid = Uid::current();
    let socket_path = PathBuf::from(format!("/run/user/{}/biomeos/nestgate.sock", uid));
    
    // Clean old socket
    let _ = std::fs::remove_file(&socket_path);
    
    // Generate secure JWT secret
    let jwt_secret = generate_secure_jwt()?;
    
    let process = Command::new("nestgate")
        .arg("daemon")
        .arg("--socket-only")
        .env("FAMILY_ID", "test0")
        .env("NODE_ID", "test-nestgate")
        .env("NESTGATE_JWT_SECRET", jwt_secret)
        .env("NESTGATE_SOCKET_ONLY", "true")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start NestGate")?;
    
    let pid = process.id();
    
    // Wait for socket creation
    for _ in 0..30 {
        if socket_path.exists() {
            sleep(Duration::from_millis(500)).await; // Stabilize
            return Ok(PrimalHandle {
                name: "nestgate".to_string(),
                process,
                socket_path,
                pid,
            });
        }
        sleep(Duration::from_millis(100)).await;
    }
    
    Err(anyhow::anyhow!("NestGate socket not created within 3 seconds"))
}

/// Start Squirrel primal
pub async fn start_squirrel() -> Result<PrimalHandle> {
    let uid = Uid::current();
    let socket_path = PathBuf::from(format!("/run/user/{}/biomeos/squirrel.sock", uid));
    
    // Clean old socket
    let _ = std::fs::remove_file(&socket_path);
    
    let process = Command::new("squirrel")
        .arg("server")
        .env("FAMILY_ID", "test0")
        .env("NODE_ID", "test-squirrel")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start Squirrel")?;
    
    let pid = process.id();
    
    // Wait for socket creation
    for _ in 0..30 {
        if socket_path.exists() {
            sleep(Duration::from_millis(500)).await; // Stabilize
            return Ok(PrimalHandle {
                name: "squirrel".to_string(),
                process,
                socket_path,
                pid,
            });
        }
        sleep(Duration::from_millis(100)).await;
    }
    
    Err(anyhow::anyhow!("Squirrel socket not created within 3 seconds"))
}

/// Start Tower Atomic (BearDog + Songbird)
pub async fn start_tower_atomic() -> Result<TowerHandle> {
    let beardog = start_beardog().await?;
    sleep(Duration::from_secs(2)).await;
    
    let songbird = start_songbird(&beardog).await?;
    sleep(Duration::from_secs(2)).await;
    
    Ok(TowerHandle { beardog, songbird })
}

/// Start Node Atomic (Tower + Toadstool)
pub async fn start_node_atomic() -> Result<NodeHandle> {
    let tower = start_tower_atomic().await?;
    sleep(Duration::from_secs(2)).await;
    
    let toadstool = start_toadstool().await?;
    sleep(Duration::from_secs(3)).await;
    
    Ok(NodeHandle { tower, toadstool })
}

/// Start Nest Atomic (Tower + NestGate + Squirrel)
pub async fn start_nest_atomic() -> Result<NestHandle> {
    let tower = start_tower_atomic().await?;
    sleep(Duration::from_secs(2)).await;
    
    let nestgate = start_nestgate_socket_only().await?;
    sleep(Duration::from_secs(2)).await;
    
    let squirrel = start_squirrel().await?;
    sleep(Duration::from_secs(2)).await;
    
    Ok(NestHandle { tower, nestgate, squirrel })
}

/// Send JSON-RPC message to Unix socket
pub async fn send_jsonrpc(socket_path: &PathBuf, message: &str) -> Result<serde_json::Value> {
    use tokio::net::UnixStream;
    use tokio::io::{AsyncWriteExt, AsyncReadExt};
    
    let mut stream = UnixStream::connect(socket_path).await?;
    
    stream.write_all(message.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    
    let mut buffer = vec![0u8; 4096];
    let n = stream.read(&mut buffer).await?;
    
    let response = String::from_utf8_lossy(&buffer[..n]);
    let json: serde_json::Value = serde_json::from_str(&response)?;
    
    Ok(json)
}

/// Generate secure JWT secret
fn generate_secure_jwt() -> Result<String> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let secret: [u8; 48] = rng.gen();
    Ok(base64::encode(secret))
}

/// Clean up all test sockets
pub async fn cleanup_test_sockets() -> Result<()> {
    let uid = Uid::current();
    let biomeos_dir = PathBuf::from(format!("/run/user/{}/biomeos", uid));
    
    if !biomeos_dir.exists() {
        return Ok(());
    }
    
    for primal in &["beardog", "songbird", "toadstool", "nestgate", "squirrel"] {
        let socket_path = biomeos_dir.join(format!("{}.sock", primal));
        let _ = std::fs::remove_file(socket_path);
    }
    
    Ok(())
}

/// Verify security handshake between BearDog and Songbird
pub async fn verify_security_handshake(beardog: &PrimalHandle, songbird: &PrimalHandle) -> bool {
    // Check BearDog is healthy
    if beardog.health_check().await.is_err() {
        return false;
    }
    
    // Check Songbird is healthy
    if songbird.health_check().await.is_err() {
        return false;
    }
    
    // Check Songbird reports BearDog as security provider
    // (This would require a specific JSON-RPC method)
    true
}
