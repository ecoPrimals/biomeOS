# 🔐 BearDog Encrypted Seed Request - Genetic Material Security

**Date**: January 3, 2026  
**From**: biomeOS Team  
**To**: BearDog Team  
**Priority**: CRITICAL (Security)  
**Status**: 🔴 **SECURITY ISSUE** - Plaintext Genetics Unacceptable

---

## 🎯 Issue Summary

**Problem**: Family seed (genetic material) stored and transmitted in plaintext.

**Current State**:
```bash
# Plaintext seed visible everywhere
BEARDOG_FAMILY_SEED="iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88="
./beardog-server

# Visible in:
ps aux | grep beardog  # ❌ Process environment
history | grep BEARDOG  # ❌ Shell history
cat logs/beardog.log    # ❌ Log files
cat usb/family-seed.conf # ❌ USB config files
```

**Impact**:
- ❌ Genetic material exposed in process environment
- ❌ Visible in shell history
- ❌ Logged in debug output
- ❌ Stored plaintext on USB
- ❌ No access control
- ❌ Violates "genetics require effort" principle

**User Quote**:
> "the family seed being plaintext is unacceptable. that's a genetic beardog key and we should treat it as such. it takes effort to see my genetics"

---

## 🔒 Required Solution

### 1. Encrypted Seed File Support

**Environment Variable**: `BEARDOG_FAMILY_SEED_FILE`  
**File Format**: Encrypted binary (BearDog-specific)  
**Decryption**: Passphrase, HSM, or TPM

**Usage**:
```bash
# Encrypted seed file (secure!)
BEARDOG_FAMILY_SEED_FILE=/path/to/family-seed.enc \
./beardog-server

# Prompts for passphrase:
"🔐 Enter passphrase to unlock genetic material: "

# User types passphrase (not echoed)
# BearDog decrypts seed
# Starts with unlocked genetics

# Genetics NEVER visible in ps/history/logs!
```

---

## 🔧 Implementation Guide

### 1. Encrypted Seed File Format

**File Structure**:
```rust
#[derive(Serialize, Deserialize)]
struct EncryptedFamilySeed {
    /// Format version for future compatibility
    version: u32,  // 1
    
    /// Family ID (plaintext for identification)
    family_id: String,  // "iidn"
    
    /// Encryption algorithm used
    algorithm: String,  // "aes-256-gcm"
    
    /// Salt for key derivation
    salt: Vec<u8>,  // 32 bytes random
    
    /// Nonce for AES-GCM
    nonce: Vec<u8>,  // 12 bytes
    
    /// Encrypted family seed
    ciphertext: Vec<u8>,  // Encrypted base64 seed
    
    /// Authentication tag
    tag: Vec<u8>,  // 16 bytes
    
    /// Optional: HSM key ID
    hsm_key_id: Option<String>,
    
    /// Optional: TPM key handle
    tpm_handle: Option<u32>,
}
```

**On Disk** (binary format):
```
family-seed.enc:
  - Magic bytes: 0x42 0x44 0x47 0x53 ("BDGS" - BearDog Genetic Seed)
  - Version: u32 (1)
  - JSON payload length: u32
  - JSON payload: EncryptedFamilySeed (above)
```

### 2. Encryption Implementation

```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};

pub struct SeedEncryptor {
    argon2: Argon2<'static>,
}

impl SeedEncryptor {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }
    
    /// Encrypt family seed with passphrase
    pub fn encrypt_seed(
        &self,
        family_id: &str,
        seed: &str,
        passphrase: &str,
    ) -> Result<EncryptedFamilySeed> {
        // Generate random salt
        let salt = SaltString::generate(&mut OsRng);
        
        // Derive key from passphrase using Argon2
        let password_hash = self.argon2
            .hash_password(passphrase.as_bytes(), &salt)
            .map_err(|e| anyhow!("Key derivation failed: {}", e))?;
        
        let key_bytes = &password_hash.hash.unwrap().as_bytes()[..32];
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        
        // Create cipher
        let cipher = Aes256Gcm::new(key);
        
        // Generate nonce
        let nonce_bytes = Nonce::from_slice(&random_nonce());
        
        // Encrypt seed
        let ciphertext = cipher
            .encrypt(nonce_bytes, seed.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        
        Ok(EncryptedFamilySeed {
            version: 1,
            family_id: family_id.to_string(),
            algorithm: "aes-256-gcm".to_string(),
            salt: salt.as_bytes().to_vec(),
            nonce: nonce_bytes.to_vec(),
            ciphertext,
            tag: vec![], // Included in ciphertext for AES-GCM
            hsm_key_id: None,
            tpm_handle: None,
        })
    }
    
    /// Decrypt family seed with passphrase
    pub fn decrypt_seed(
        &self,
        encrypted: &EncryptedFamilySeed,
        passphrase: &str,
    ) -> Result<String> {
        // Verify version
        if encrypted.version != 1 {
            bail!("Unsupported seed format version: {}", encrypted.version);
        }
        
        // Derive key from passphrase
        let salt = SaltString::from_b64(&encrypted.salt)?;
        let password_hash = self.argon2
            .hash_password(passphrase.as_bytes(), &salt)
            .map_err(|e| anyhow!("Key derivation failed: {}", e))?;
        
        let key_bytes = &password_hash.hash.unwrap().as_bytes()[..32];
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        
        // Create cipher
        let cipher = Aes256Gcm::new(key);
        
        // Decrypt
        let nonce = Nonce::from_slice(&encrypted.nonce);
        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|_| anyhow!("Decryption failed - wrong passphrase?"))?;
        
        String::from_utf8(plaintext)
            .map_err(|e| anyhow!("Invalid UTF-8 in decrypted seed: {}", e))
    }
}
```

### 3. BearDog Server Integration

**In `beardog-server` main.rs**:

```rust
use rpassword::prompt_password;

async fn load_family_seed() -> Result<(String, String)> {
    // Option 1: Encrypted file (SECURE!)
    if let Ok(seed_file) = std::env::var("BEARDOG_FAMILY_SEED_FILE") {
        info!("🔐 Loading encrypted family seed from: {}", seed_file);
        
        // Read encrypted file
        let encrypted_data = std::fs::read(&seed_file)
            .context("Failed to read encrypted seed file")?;
        
        // Parse encrypted seed
        let encrypted = parse_encrypted_seed(&encrypted_data)?;
        
        let family_id = encrypted.family_id.clone();
        
        // Try HSM first (if configured)
        if let Some(hsm_key_id) = &encrypted.hsm_key_id {
            info!("🔑 Attempting HSM decryption...");
            if let Ok(seed) = decrypt_with_hsm(hsm_key_id, &encrypted) {
                info!("✅ Unlocked genetics via HSM");
                return Ok((family_id, seed));
            }
            warn!("⚠️  HSM decryption failed, falling back to passphrase");
        }
        
        // Try TPM (if configured)
        if let Some(tpm_handle) = encrypted.tpm_handle {
            info!("🔑 Attempting TPM decryption...");
            if let Ok(seed) = decrypt_with_tpm(tpm_handle, &encrypted) {
                info!("✅ Unlocked genetics via TPM");
                return Ok((family_id, seed));
            }
            warn!("⚠️  TPM decryption failed, falling back to passphrase");
        }
        
        // Prompt for passphrase
        let passphrase = prompt_password("🔐 Enter passphrase to unlock genetic material: ")
            .context("Failed to read passphrase")?;
        
        // Decrypt with passphrase
        let encryptor = SeedEncryptor::new();
        let seed = encryptor.decrypt_seed(&encrypted, &passphrase)
            .context("Failed to decrypt seed - wrong passphrase?")?;
        
        // Clear passphrase from memory
        drop(passphrase);
        
        info!("✅ Genetic material unlocked successfully");
        info!("   Family: {}", family_id);
        info!("   Seed: [REDACTED - {} bytes]", seed.len());
        
        return Ok((family_id, seed));
    }
    
    // Option 2: Plaintext env var (DEPRECATED!)
    if let Ok(seed) = std::env::var("BEARDOG_FAMILY_SEED") {
        warn!("⚠️  Using plaintext family seed from environment");
        warn!("⚠️  This is DEPRECATED and INSECURE");
        warn!("⚠️  Use BEARDOG_FAMILY_SEED_FILE instead");
        
        let family_id = std::env::var("BEARDOG_FAMILY_ID")
            .unwrap_or_else(|_| "unknown".to_string());
        
        return Ok((family_id, seed));
    }
    
    // Option 3: Generate new genesis seed
    info!("🌱 No family seed provided, generating genesis seed...");
    let family_id = std::env::var("BEARDOG_FAMILY_ID")
        .unwrap_or_else(|_| format!("genesis-{}", uuid::Uuid::new_v4()));
    let seed = generate_random_seed();
    
    info!("✅ Genesis seed created");
    info!("   Family: {}", family_id);
    info!("   To preserve this lineage, encrypt and save the seed:");
    info!("   beardog genetics encrypt-seed --family-id {} --seed [seed] --output family-seed.enc", family_id);
    
    Ok((family_id, seed))
}
```

### 4. CLI Tool for Encryption

**Command**: `beardog genetics encrypt-seed`

```rust
// In beardog CLI
use clap::{Args, Subcommand};

#[derive(Subcommand)]
enum GeneticsCommand {
    /// Encrypt family seed for secure storage
    EncryptSeed(EncryptSeedArgs),
    
    /// Decrypt and display family seed (for backup)
    DecryptSeed(DecryptSeedArgs),
    
    /// Rotate passphrase (re-encrypt with new passphrase)
    RotatePassphrase(RotatePassphraseArgs),
}

#[derive(Args)]
struct EncryptSeedArgs {
    /// Family ID
    #[arg(long)]
    family_id: String,
    
    /// Family seed (base64)
    #[arg(long)]
    seed: Option<String>,
    
    /// Input file (plaintext seed)
    #[arg(long)]
    input: Option<PathBuf>,
    
    /// Output file (encrypted)
    #[arg(long)]
    output: PathBuf,
    
    /// Use HSM for encryption
    #[arg(long)]
    hsm: bool,
    
    /// Use TPM for encryption
    #[arg(long)]
    tpm: bool,
}

impl EncryptSeedArgs {
    async fn execute(&self) -> Result<()> {
        // Get seed
        let seed = if let Some(ref seed_str) = self.seed {
            seed_str.clone()
        } else if let Some(ref input_file) = self.input {
            std::fs::read_to_string(input_file)?
                .trim()
                .to_string()
        } else {
            bail!("Either --seed or --input required");
        };
        
        // Prompt for passphrase
        let passphrase = rpassword::prompt_password("Enter passphrase: ")?;
        let passphrase_confirm = rpassword::prompt_password("Confirm passphrase: ")?;
        
        if passphrase != passphrase_confirm {
            bail!("Passphrases do not match!");
        }
        
        // Encrypt
        let encryptor = SeedEncryptor::new();
        let encrypted = encryptor.encrypt_seed(&self.family_id, &seed, &passphrase)?;
        
        // Save
        let output_data = serialize_encrypted_seed(&encrypted)?;
        std::fs::write(&self.output, output_data)?;
        
        println!("✅ Family seed encrypted successfully");
        println!("   Family ID: {}", self.family_id);
        println!("   Output: {}", self.output.display());
        println!("");
        println!("🔐 To use this seed:");
        println!("   BEARDOG_FAMILY_SEED_FILE={} ./beardog-server", self.output.display());
        
        Ok(())
    }
}
```

**Usage**:
```bash
# Encrypt existing plaintext seed
beardog genetics encrypt-seed \
    --family-id iidn \
    --seed "iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=" \
    --output /secure/path/family-seed.enc

# Prompts:
Enter passphrase: [user types]
Confirm passphrase: [user types]

# Output:
✅ Family seed encrypted successfully
   Family ID: iidn
   Output: /secure/path/family-seed.enc

🔐 To use this seed:
   BEARDOG_FAMILY_SEED_FILE=/secure/path/family-seed.enc ./beardog-server
```

---

## 🧪 Testing Strategy

### Test 1: Encryption/Decryption Roundtrip

```bash
# Encrypt
beardog genetics encrypt-seed \
    --family-id test \
    --seed "dGVzdC1zZWVkLTEyMzQ1Ng==" \
    --output /tmp/test-seed.enc
# Passphrase: "test123"

# Decrypt (via server)
BEARDOG_FAMILY_SEED_FILE=/tmp/test-seed.enc ./beardog-server
# Prompts for passphrase: "test123"
# Expected: Starts with family_id="test"
```

### Test 2: Wrong Passphrase

```bash
BEARDOG_FAMILY_SEED_FILE=/tmp/test-seed.enc ./beardog-server
# Prompts for passphrase: "wrong"
# Expected: Error "Decryption failed - wrong passphrase?"
# Expected: Server does NOT start
```

### Test 3: Passphrase Not Visible

```bash
BEARDOG_FAMILY_SEED_FILE=/tmp/test-seed.enc ./beardog-server &
BEARDOG_PID=$!

# Check process environment
cat /proc/$BEARDOG_PID/environ | tr '\0' '\n' | grep -i seed
# Expected: NO plaintext seed visible
# Expected: Only BEARDOG_FAMILY_SEED_FILE=/tmp/test-seed.enc

# Check logs
grep -i seed /var/log/beardog.log
# Expected: Logs show "[REDACTED]", NOT plaintext seed
```

### Test 4: USB Deployment

```bash
# USB contains encrypted seed
ls /media/usb/secrets/
# family-seed.enc (encrypted)

# Deploy
BEARDOG_FAMILY_SEED_FILE=/media/usb/secrets/family-seed.enc \
./beardog-server
# Prompts for passphrase
# User enters passphrase (effort required!)
# Genetics unlocked

# Result:
# ✅ Genetics require effort to access
# ✅ No plaintext on USB
# ✅ No plaintext in process env
```

---

## 📊 Security Benefits

### 1. Confidentiality ✅
- Genetics encrypted at rest
- Never in process environment
- Not in logs or history

### 2. Access Control ✅
- Passphrase required
- HSM/TPM optional
- Audit trail possible

### 3. Compliance ✅
- No plaintext secrets
- Defense in depth
- Cryptographic protection

### 4. Philosophy ✅
- **"It takes effort to see my genetics"**
- User must provide passphrase
- Intentional access only

---

## 🎯 Acceptance Criteria

### Must Have

- [ ] `BEARDOG_FAMILY_SEED_FILE` environment variable
- [ ] Passphrase-based encryption (AES-256-GCM)
- [ ] Passphrase prompt on startup
- [ ] Clear error on wrong passphrase
- [ ] CLI tool for encryption (`beardog genetics encrypt-seed`)
- [ ] CLI tool for decryption (backup/recovery)
- [ ] Deprecation warning for plaintext `BEARDOG_FAMILY_SEED`
- [ ] Seed never logged in plaintext
- [ ] All tests passing

### Nice to Have

- [ ] HSM integration (PKCS#11)
- [ ] TPM integration (TPM 2.0)
- [ ] Passphrase rotation tool
- [ ] Multiple encryption backends
- [ ] Seed backup/recovery guide

---

## 📚 Dependencies

**Add to `Cargo.toml`**:
```toml
[dependencies]
aes-gcm = "0.10"
argon2 = "0.5"
rpassword = "7.2"
zeroize = "1.6"
```

---

## 🚀 Timeline

**Estimated Effort**: 4-6 hours  
**Priority**: CRITICAL (Security)  
**Target**: Next BearDog release (urgent!)

**Breakdown**:
- Encryption/decryption impl: 2 hours
- Server integration: 1 hour
- CLI tool: 1 hour
- Testing: 1 hour
- Documentation: 1 hour

---

## 🎊 Bottom Line

**User is 100% correct**:

> "it takes effort to see my genetics"

Plaintext seeds violate this fundamental principle.

**After This Fix**:
- ✅ Genetics encrypted at rest
- ✅ Passphrase required to unlock
- ✅ No plaintext in process/logs/USB
- ✅ Philosophically aligned
- ✅ Security best practices

**This is not just a feature request - it's a security requirement!**

---

**Status**: 🔴 CRITICAL  
**Priority**: P0 (Security)  
**Timeline**: ASAP

🔐 **Protect the genetics!** 🧬

