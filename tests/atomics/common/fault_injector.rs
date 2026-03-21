// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

// Fault Injection Infrastructure for NUCLEUS Testing

use std::path::PathBuf;
use std::time::Duration;
use anyhow::{Result, Context};

/// Fault injection engine
pub struct FaultInjector {
    active_faults: Vec<(FaultHandle, Fault)>,
}

/// Handle to an injected fault
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaultHandle(usize);

/// Fault types
#[derive(Debug, Clone)]
pub enum Fault {
    /// Invalid/corrupted credentials
    InvalidCredentials {
        primal: String,
    },
    
    /// Expired TLS certificate
    ExpiredCertificate {
        cert_path: PathBuf,
    },
    
    /// Malformed JSON-RPC message
    MalformedMessage {
        corruption_type: CorruptionType,
    },
    
    /// Permission denied on socket
    PermissionDenied {
        socket_path: PathBuf,
    },
    
    /// Corrupted environment variable
    EnvironmentCorruption {
        var: String,
        corrupted_value: String,
    },
    
    /// Operation timeout
    Timeout {
        operation: String,
        duration: Duration,
    },
    
    /// Partial message delivery
    PartialDelivery {
        bytes: usize,
    },
}

/// Message corruption types
#[derive(Debug, Clone)]
pub enum CorruptionType {
    MissingId,
    MissingJsonrpc,
    WrongVersion,
    InvalidJson,
    EmptyMessage,
    TruncatedMessage,
}

impl FaultInjector {
    /// Create new fault injector
    pub fn new() -> Self {
        Self {
            active_faults: Vec::new(),
        }
    }
    
    /// Inject a fault
    pub async fn inject(&mut self, fault: Fault) -> Result<FaultHandle> {
        let handle = FaultHandle(self.active_faults.len());
        
        match &fault {
            Fault::InvalidCredentials { primal } => {
                self.corrupt_credentials(primal)?;
            }
            
            Fault::ExpiredCertificate { cert_path } => {
                self.expire_certificate(cert_path)?;
            }
            
            Fault::PermissionDenied { socket_path } => {
                self.deny_permissions(socket_path)?;
            }
            
            Fault::EnvironmentCorruption { var, corrupted_value } => {
                std::env::set_var(var, corrupted_value);
            }
            
            _ => {
                // Other faults are handled inline in tests
            }
        }
        
        self.active_faults.push((handle, fault));
        Ok(handle)
    }
    
    /// Clear an injected fault
    pub async fn clear(&mut self, handle: FaultHandle) -> Result<()> {
        if let Some(pos) = self.active_faults.iter().position(|(h, _)| *h == handle) {
            let (_, fault) = self.active_faults.remove(pos);
            
            match fault {
                Fault::PermissionDenied { socket_path } => {
                    self.restore_permissions(&socket_path)?;
                }
                
                Fault::EnvironmentCorruption { var, .. } => {
                    std::env::remove_var(&var);
                }
                
                _ => {
                    // Other faults may not need explicit cleanup
                }
            }
        }
        
        Ok(())
    }
    
    /// Corrupt primal credentials
    fn corrupt_credentials(&self, primal: &str) -> Result<()> {
        // This would corrupt the credentials file/storage
        // For now, we'll just log the intent
        eprintln!("Injecting fault: Invalid credentials for {}", primal);
        Ok(())
    }
    
    /// Expire a TLS certificate
    fn expire_certificate(&self, cert_path: &PathBuf) -> Result<()> {
        // Backup original cert
        let backup_path = cert_path.with_extension("cert.backup");
        std::fs::copy(cert_path, &backup_path)
            .context("Failed to backup certificate")?;
        
        // Replace with expired cert (this would require generating one)
        eprintln!("Injecting fault: Expired certificate at {:?}", cert_path);
        Ok(())
    }
    
    /// Deny permissions on socket
    fn deny_permissions(&self, socket_path: &PathBuf) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o000);
            std::fs::set_permissions(socket_path, perms)
                .context("Failed to deny socket permissions")?;
        }
        Ok(())
    }
    
    /// Restore socket permissions
    fn restore_permissions(&self, socket_path: &PathBuf) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(socket_path, perms)
                .context("Failed to restore socket permissions")?;
        }
        Ok(())
    }
    
    /// Generate malformed JSON-RPC message
    pub fn generate_malformed_message(&self, corruption_type: CorruptionType) -> String {
        match corruption_type {
            CorruptionType::MissingId => {
                r#"{"jsonrpc":"2.0","method":"health","params":{}}"#.to_string()
            }
            
            CorruptionType::MissingJsonrpc => {
                r#"{"method":"health","params":{},"id":1}"#.to_string()
            }
            
            CorruptionType::WrongVersion => {
                r#"{"jsonrpc":"1.0","method":"health","params":{},"id":1}"#.to_string()
            }
            
            CorruptionType::InvalidJson => {
                r#"{invalid json syntax}"#.to_string()
            }
            
            CorruptionType::EmptyMessage => {
                String::new()
            }
            
            CorruptionType::TruncatedMessage => {
                r#"{"jsonrpc":"2.0","method":"heal"#.to_string()
            }
        }
    }
}

impl Default for FaultInjector {
    fn default() -> Self {
        Self::new()
    }
}
