// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Cryptographic genetics - inherited from BearDog lineage
//!
//! This module contains genetic material from BearDog, adapted
//! for the Platypus organism's unique niche.

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// Genetic keys - cryptographic identity that can evolve
/// 
/// Unlike static keys, genetic keys carry lineage information
/// that allows verification of evolutionary history.
#[derive(Clone)]
pub struct GeneticKeys {
    /// The current signing key
    signing_key: SigningKey,
    
    /// Lineage chain - hashes of ancestor keys
    lineage: Vec<[u8; 32]>,
    
    /// Generation number
    generation: u64,
}

impl GeneticKeys {
    /// Create a new root key (generation 0)
    pub fn new_root() -> Self {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key();
        
        // Root lineage is just the hash of the public key
        let mut hasher = Hasher::new();
        hasher.update(verifying_key.as_bytes());
        let root_hash = hasher.finalize();
        
        Self {
            signing_key,
            lineage: vec![*root_hash.as_bytes()],
            generation: 0,
        }
    }
    
    /// Evolve to next generation (create child key)
    pub fn evolve(&self) -> Self {
        let child_signing = SigningKey::generate(&mut rand::thread_rng());
        let child_verifying = child_signing.verifying_key();
        
        // Child lineage includes parent lineage + new hash
        let mut hasher = Hasher::new();
        hasher.update(child_verifying.as_bytes());
        let parent_hash = self.lineage.last().copied().unwrap_or([0u8; 32]);
        hasher.update(&parent_hash);
        let child_hash = hasher.finalize();
        
        let mut lineage = self.lineage.clone();
        lineage.push(*child_hash.as_bytes());
        
        Self {
            signing_key: child_signing,
            lineage,
            generation: self.generation + 1,
        }
    }
    
    /// Verify that another key shares lineage with this one
    pub fn verify_lineage(&self, other: &GeneticKeys) -> bool {
        // Find common ancestor
        let min_len = self.lineage.len().min(other.lineage.len());
        
        for i in 0..min_len {
            if self.lineage[i] != other.lineage[i] {
                return i > 0; // Must share at least root
            }
        }
        
        true // Shared lineage up to min length
    }
    
    /// Sign data with genetic key
    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }
    
    /// Get the verifying (public) key
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
    
    /// Get the generation number
    pub fn generation(&self) -> u64 {
        self.generation
    }
    
    /// Get lineage depth
    pub fn lineage_depth(&self) -> usize {
        self.lineage.len()
    }
}

/// Identity - a DID-like identifier with genetic lineage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// The identifier (did:platypus:...)
    pub id: String,
    
    /// Public key bytes
    pub public_key: Vec<u8>,
    
    /// Generation
    pub generation: u64,
    
    /// Lineage hashes (for verification)
    pub lineage_hashes: Vec<String>,
}

impl Identity {
    /// Create identity from genetic keys
    pub fn from_keys(keys: &GeneticKeys) -> Self {
        let verifying = keys.verifying_key();
        let mut hasher = Hasher::new();
        hasher.update(verifying.as_bytes());
        let id_hash = hasher.finalize();
        
        Self {
            id: format!("did:platypus:{}", hex::encode(&id_hash.as_bytes()[..16])),
            public_key: verifying.as_bytes().to_vec(),
            generation: keys.generation(),
            lineage_hashes: keys.lineage.iter()
                .map(|h| hex::encode(&h[..8]))
                .collect(),
        }
    }
    
    /// Verify a signature from this identity
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        if signature.len() != 64 {
            return false;
        }
        
        let key_bytes: [u8; 32] = match self.public_key.as_slice().try_into() {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
        
        let Ok(verifying_key) = VerifyingKey::from_bytes(&key_bytes) else {
            return false;
        };
        
        let sig_bytes: [u8; 64] = match signature.try_into() {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
        
        let sig = Signature::from_bytes(&sig_bytes);
        
        verifying_key.verify(data, &sig).is_ok()
    }
}

// Hex encoding helper
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lineage_verification() {
        let root = GeneticKeys::new_root();
        let child1 = root.evolve();
        let child2 = root.evolve();
        let grandchild = child1.evolve();
        
        // Same lineage
        assert!(root.verify_lineage(&child1));
        assert!(child1.verify_lineage(&grandchild));
        
        // Siblings share lineage through root
        assert!(child1.verify_lineage(&child2));
    }
    
    #[test]
    fn test_identity_creation() {
        let keys = GeneticKeys::new_root();
        let identity = Identity::from_keys(&keys);
        
        assert!(identity.id.starts_with("did:platypus:"));
        assert_eq!(identity.generation, 0);
    }
}

