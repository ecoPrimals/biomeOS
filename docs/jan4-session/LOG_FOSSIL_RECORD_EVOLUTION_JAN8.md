# 🦴 Log Management & Fossil Record Evolution

**Date:** January 8, 2026  
**Session:** Phase 2 Deep Debt Evolution  
**Status:** 🔄 **IN PROGRESS**

---

## 🎯 Problem Statement

### Current Issues
1. **Stale Log Pollution:** `/tmp/primals/` contains many old UUID-named logs
2. **Hard to Track Active Logs:** No clear distinction between running and archived
3. **Manual Cleanup Required:** No automated log rotation or archival
4. **No Forensics:** Can't track deployment history or issues over time
5. **Security Gap:** No encrypted log storage for high-security deployments

### Discovery During LAN Federation
While verifying LAN federation, we encountered:
- Multiple old log files from previous deployments
- Difficulty identifying which logs were from current processes
- UUID-based naming made it hard to correlate logs with nodes
- No structured way to preserve historical logs for forensics

---

## 🏗️ Architectural Vision

### FossilRecord System
A structured, Rust-based log management system that:
1. **Archives old logs** automatically (fossil record)
2. **Tracks active logs** clearly (living logs)
3. **Preserves forensics** (deployment history, issues, metrics)
4. **Integrates with spores** (self-tracking USB deployments)
5. **Enables encryption** (BearDog parent-seed-only access)

### Key Principles
- **Idiomatic Rust:** Type-safe, async-aware, composable
- **Self-Tracking:** Spores maintain their own log history
- **Forensic-Ready:** Structured metadata for investigation
- **Encryption-Ready:** Hooks for BearDog encryption
- **Zero-Config:** Automatic management, no manual intervention

---

## 📐 Design Specification

### Directory Structure

#### On Host System
```
/var/biomeos/logs/
├── active/                          # Currently running instances
│   ├── node-alpha/
│   │   ├── tower.log               # Named, not UUID
│   │   ├── beardog.log
│   │   └── songbird.log
│   ├── node-beta/
│   │   ├── tower.log
│   │   ├── beardog.log
│   │   └── songbird.log
│   └── .metadata.toml              # Active instance tracking
│
├── fossil/                          # Archived logs
│   ├── 2026-01-08_10-31-13_node-alpha/
│   │   ├── tower.log
│   │   ├── beardog.log
│   │   ├── songbird.log
│   │   └── .fossil.toml            # Metadata: why archived, when, issues
│   ├── 2026-01-08_09-38-15_node-alpha/
│   │   └── ...
│   └── index.toml                  # Searchable index of all fossils
│
└── .config.toml                     # Log management configuration
```

#### On USB Spore
```
/media/{mount}/biomeOS/
├── .spore.logs/                     # Spore's own log history
│   ├── deployments/
│   │   ├── 2026-01-08_10-31-13.log # Each deployment tracked
│   │   ├── 2026-01-08_15-42-00.log
│   │   └── index.toml              # Deployment history
│   ├── fossil/                      # Archived from this spore
│   │   ├── 2026-01-08_10-31-13/
│   │   │   └── ...
│   │   └── index.toml
│   └── .metadata.toml               # Spore log configuration
│
└── .family.seed                     # For future encryption
```

### Data Structures

#### Rust Types
```rust
// crates/biomeos-spore/src/logs.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for log management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Where to store active logs
    pub active_dir: PathBuf,
    
    /// Where to archive fossil logs
    pub fossil_dir: PathBuf,
    
    /// Maximum age before auto-archival (seconds)
    pub max_active_age_secs: u64,
    
    /// Whether to enable BearDog encryption
    pub enable_encryption: bool,
    
    /// Compression for fossil logs
    pub compress_fossils: bool,
}

/// Metadata for an active log session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveLogSession {
    /// Node ID
    pub node_id: String,
    
    /// When this session started
    pub started_at: DateTime<Utc>,
    
    /// PIDs of running processes
    pub process_pids: Vec<u32>,
    
    /// Log file paths
    pub log_files: Vec<LogFile>,
    
    /// Deployment this session is from
    pub deployment_id: String,
}

/// Individual log file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFile {
    /// Primal name (tower, beardog, songbird)
    pub primal: String,
    
    /// File path
    pub path: PathBuf,
    
    /// Process PID
    pub pid: Option<u32>,
    
    /// Size in bytes
    pub size_bytes: u64,
    
    /// Last modified
    pub last_modified: DateTime<Utc>,
}

/// Fossil record metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FossilRecord {
    /// Original node ID
    pub node_id: String,
    
    /// When session started
    pub session_started: DateTime<Utc>,
    
    /// When session ended (archived)
    pub session_ended: DateTime<Utc>,
    
    /// Why was this archived?
    pub archival_reason: ArchivalReason,
    
    /// Deployment ID
    pub deployment_id: String,
    
    /// Issues detected (errors, warnings)
    pub issues: Vec<LogIssue>,
    
    /// Metrics summary
    pub metrics: Option<LogMetrics>,
    
    /// Encrypted with BearDog?
    pub encrypted: bool,
    
    /// Parent seed fingerprint (for decryption)
    pub parent_seed_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchivalReason {
    /// Normal shutdown
    GracefulShutdown,
    
    /// Process crashed
    Crash { exit_code: i32 },
    
    /// Manual archival by user
    Manual,
    
    /// Automatic archival (age threshold)
    AutomaticRotation,
    
    /// New deployment replacing old
    Redeployment,
    
    /// System reboot
    Reboot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIssue {
    /// Timestamp of issue
    pub timestamp: DateTime<Utc>,
    
    /// Severity (error, warning, info)
    pub severity: IssueSeverity,
    
    /// Primal where issue occurred
    pub primal: String,
    
    /// Issue description
    pub description: String,
    
    /// Log line where it occurred
    pub log_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMetrics {
    /// Total lines logged
    pub total_lines: u64,
    
    /// Lines by severity
    pub errors: u64,
    pub warnings: u64,
    pub info: u64,
    
    /// Total size
    pub total_size_bytes: u64,
    
    /// Session duration
    pub duration_secs: u64,
}

/// Searchable index of all fossil records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FossilIndex {
    /// All fossils
    pub fossils: Vec<FossilIndexEntry>,
    
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FossilIndexEntry {
    /// Node ID
    pub node_id: String,
    
    /// Session timestamp
    pub session_started: DateTime<Utc>,
    
    /// Archival reason
    pub archival_reason: ArchivalReason,
    
    /// Path to fossil directory
    pub fossil_path: PathBuf,
    
    /// Number of issues
    pub issue_count: usize,
    
    /// Encrypted?
    pub encrypted: bool,
}
```

---

## 🔧 Implementation Plan

### Phase 1: Core Log Management (Immediate)
1. ✅ **Create `crates/biomeos-spore/src/logs.rs`**
   - Define data structures
   - Implement log archival logic
   - Add compression support

2. ✅ **Integrate with Tower**
   - Tower tracks active logs by node ID
   - Auto-archive on shutdown
   - Detect crashes and preserve logs

3. ✅ **CLI Commands**
   - `biomeos logs active` - Show running instances
   - `biomeos logs fossil` - Browse archived logs
   - `biomeos logs archive <node-id>` - Manual archival
   - `biomeos logs clean --older-than <days>` - Cleanup old fossils

4. ✅ **Migration Script**
   - Clean up `/tmp/primals/*.log` into fossil structure
   - Identify active vs stale logs
   - Preserve for forensics

### Phase 2: Spore Integration (Near-Term)
1. **Spore Log Tracking**
   - Each spore maintains `.spore.logs/`
   - Deployment history tracked
   - Issues logged locally on USB

2. **Self-Healing Logs**
   - Spore detects if logs are missing
   - Auto-creates log directories
   - Preserves across redeployments

3. **Forensic Analysis**
   - `biomeos spore analyze-logs <mount>`
   - Show deployment history
   - Identify recurring issues

### Phase 3: Encryption & Security (Future)
1. **BearDog Integration**
   - Encrypt fossils with parent seed
   - Only parent lineage can decrypt
   - Secure audit trail for high-security deployments

2. **Encrypted Spore Logs**
   - `.spore.logs/` encrypted on USB
   - Requires `.family.seed` to read
   - Tamper-evident logging

3. **Distributed Forensics**
   - Nodes can share encrypted logs with family members
   - Cross-node issue correlation
   - Encrypted log federation

---

## 🎯 Expected Benefits

### Immediate (Phase 1)
- ✅ **Clear active logs:** Easy to identify running instances
- ✅ **Automated archival:** No manual cleanup needed
- ✅ **Forensic preservation:** Historical logs for debugging
- ✅ **Issue detection:** Auto-scan for errors/warnings

### Near-Term (Phase 2)
- ✅ **Spore self-tracking:** Each USB has its own history
- ✅ **Deployment auditing:** Track all uses of a spore
- ✅ **Portable forensics:** Logs travel with the spore

### Long-Term (Phase 3)
- ✅ **Encrypted audit trail:** Secure logging for compliance
- ✅ **Parent-seed-only access:** Family-based log encryption
- ✅ **Distributed forensics:** Cross-node issue correlation

---

## 📋 Example Usage

### After Implementation

#### Check Active Logs
```bash
$ biomeos logs active

🌱 Active Log Sessions
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Node: node-alpha
  Started: 2026-01-08 10:31:13 (2h 15m ago)
  PIDs: 1760032 (tower), 1760062 (beardog), 1760063 (songbird)
  Logs:
    • tower.log    (142 KB, active)
    • beardog.log  (221 KB, active)
    • songbird.log (3.0 MB, active)
  Issues: 0 errors, 2 warnings

Node: node-beta
  Started: 2026-01-08 10:31:18 (2h 15m ago)
  PIDs: 1760211 (tower), 1760247 (beardog), 1760248 (songbird)
  Logs:
    • tower.log    (145 KB, active)
    • beardog.log  (219 KB, active)
    • songbird.log (3.0 MB, active)
  Issues: 0 errors, 1 warning

Total: 2 active sessions
```

#### Browse Fossil Logs
```bash
$ biomeos logs fossil --node node-alpha --limit 5

🦴 Fossil Record for node-alpha
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[1] 2026-01-08 09:38:15 → 10:26:00 (48m)
    Reason: Redeployment
    Issues: 3 warnings (BearDog socket timeouts)
    Size: 15.2 MB (compressed)
    
[2] 2026-01-08 09:11:00 → 09:30:00 (19m)
    Reason: Manual shutdown
    Issues: 0
    Size: 8.4 MB (compressed)

[3] 2026-01-07 20:47:00 → 21:54:00 (1h 7m)
    Reason: Crash (exit code 1)
    Issues: 12 errors (BearDog HSM failure)
    Size: 42.1 MB (compressed)

Use 'biomeos logs fossil --show <num>' to view details
```

#### Analyze Spore Logs
```bash
$ biomeos spore analyze-logs /media/BEA6-BBCE

🌱 Spore Log Analysis: node-epsilon
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Deployment History:
  • 2026-01-08 15:42:00 - LAN deployment (computer: pop-os)
  • 2026-01-08 10:31:00 - Local test (computer: localhost)
  • 2026-01-08 09:15:00 - Initial creation

Total Deployments: 3
Total Runtime: 4h 32m
Total Issues: 2 warnings, 0 errors

Recent Issues:
  [WARN] 2026-01-08 15:43:00 - BTSP tunnel fallback to HTTPS

Self-Tracking Status: ✅ Operational
Encryption: 🔐 Ready (not yet enabled)
```

---

## 🔐 Future: Encrypted Forensics

### BearDog Integration (Phase 3)

#### Encrypt Fossil Logs
```bash
$ biomeos logs archive node-alpha --encrypt

🔐 Archiving node-alpha with encryption...

✅ Logs archived to fossil record
✅ Encrypted with BearDog (parent seed: nat0)
✅ Only family members can decrypt

Fossil: /var/biomeos/logs/fossil/2026-01-08_10-31-13_node-alpha.enc
Metadata: .fossil.toml (unencrypted)
```

#### Decrypt for Analysis
```bash
$ biomeos logs fossil --show 1 --decrypt

🔐 Decrypting fossil record...
🔑 Using family seed: nat0
✅ Decryption successful

[Fossil contents displayed...]
```

#### Spore Encrypted Logs
```rust
// When creating spore
let spore_config = SporeConfig {
    node_id: "node-alpha",
    enable_encrypted_logs: true,  // Requires .family.seed
    log_retention_days: 30,
    // ...
};
```

---

## 🎯 Success Criteria

### Phase 1 Complete When:
- ✅ No stale logs in `/tmp/primals/`
- ✅ Active logs clearly identified by node ID
- ✅ Fossil records searchable and browsable
- ✅ Automatic archival on shutdown/reboot
- ✅ Issue detection and summary working

### Phase 2 Complete When:
- ✅ Each spore has `.spore.logs/` directory
- ✅ Deployment history tracked
- ✅ Spore forensics CLI working
- ✅ Self-healing log management

### Phase 3 Complete When:
- ✅ BearDog encryption integration working
- ✅ Parent-seed-only log access enforced
- ✅ Distributed forensics operational
- ✅ Compliance-ready audit trail

---

## 📝 Next Steps

1. **Implement Core Module** (`crates/biomeos-spore/src/logs.rs`)
2. **Tower Integration** (auto-archival on shutdown)
3. **CLI Commands** (`biomeos logs ...`)
4. **Migration Script** (clean up existing logs)
5. **Spore Integration** (`.spore.logs/` directory)
6. **Testing** (unit, E2E, chaos)
7. **Documentation** (user guide, forensics guide)
8. **BearDog Encryption** (Phase 3, future)

---

**Status:** Ready to implement Phase 1 🚀

