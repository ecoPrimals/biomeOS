// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Shared mocks and crypto helpers for sovereign mesh integration tests.
#![allow(dead_code)] // Each integration test binary uses a subset of this module

use rand::Rng;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;

// ═══════════════════════════════════════════════════════════════════════
// CRYPTOGRAPHIC MOCK BEARDOG
// ═══════════════════════════════════════════════════════════════════════

/// A 32-byte family seed shared by all nodes in the same family
#[derive(Clone)]
pub struct FamilySeed(pub [u8; 32]);

impl FamilySeed {
    pub fn new(seed_bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(seed_bytes);
        let hash = hasher.finalize();
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&hash);
        Self(seed)
    }

    pub fn derive_key(&self, context: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.0);
        hasher.update(context.as_bytes());
        let hash = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash);
        key
    }

    pub fn lineage_hash(&self) -> String {
        let key = self.derive_key("lineage-hash-v1");
        hex::encode(&key[..16])
    }

    pub fn family_hash(&self) -> String {
        let key = self.derive_key("family-hash-v1");
        hex::encode(&key[..16])
    }
}

pub fn symmetric_encrypt(plaintext: &[u8], key: &[u8; 32]) -> String {
    let mut rng = rand::rng();
    let nonce: [u8; 8] = rng.random();

    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.update(nonce);
    let stream_seed = hasher.finalize();

    let mut ciphertext = Vec::with_capacity(plaintext.len());
    for (i, &byte) in plaintext.iter().enumerate() {
        let stream_byte = if i < 32 {
            stream_seed[i]
        } else {
            let mut h = Sha256::new();
            h.update(stream_seed);
            h.update((i as u64).to_le_bytes());
            let extended = h.finalize();
            extended[i % 32]
        };
        ciphertext.push(byte ^ stream_byte);
    }

    let mut hmac_hasher = Sha256::new();
    hmac_hasher.update(key);
    hmac_hasher.update(nonce);
    hmac_hasher.update(&ciphertext);
    let tag = hmac_hasher.finalize();

    let mut output = Vec::new();
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    output.extend_from_slice(&tag);

    hex::encode(output)
}

pub fn symmetric_decrypt(ciphertext_hex: &str, key: &[u8; 32]) -> Option<Vec<u8>> {
    let data = hex::decode(ciphertext_hex).ok()?;
    if data.len() < 8 + 32 {
        return None;
    }

    let nonce = &data[..8];
    let encrypted = &data[8..data.len() - 32];
    let tag = &data[data.len() - 32..];

    let mut hmac_hasher = Sha256::new();
    hmac_hasher.update(key);
    hmac_hasher.update(nonce);
    hmac_hasher.update(encrypted);
    let expected_tag = hmac_hasher.finalize();

    if tag != expected_tag.as_slice() {
        return None;
    }

    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.update(nonce);
    let stream_seed = hasher.finalize();

    let mut plaintext = Vec::with_capacity(encrypted.len());
    for (i, &byte) in encrypted.iter().enumerate() {
        let stream_byte = if i < 32 {
            stream_seed[i]
        } else {
            let mut h = Sha256::new();
            h.update(stream_seed);
            h.update((i as u64).to_le_bytes());
            let extended = h.finalize();
            extended[i % 32]
        };
        plaintext.push(byte ^ stream_byte);
    }

    Some(plaintext)
}

pub struct CryptoMockBearDog {
    pub family_seed: FamilySeed,
    pub beacon_id: String,
    #[expect(
        dead_code,
        reason = "serde may require the field shape; not read in tests"
    )]
    pub beacon_seed_hex: String,
}

impl CryptoMockBearDog {
    pub fn new(family_seed: FamilySeed, node_name: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(family_seed.0);
        hasher.update(node_name.as_bytes());
        hasher.update(b"beacon-id");
        let beacon_id = hex::encode(&hasher.finalize()[..16]);

        let mut hasher = Sha256::new();
        hasher.update(family_seed.0);
        hasher.update(node_name.as_bytes());
        hasher.update(b"beacon-seed");
        let beacon_seed_hex = hex::encode(hasher.finalize());

        Self {
            family_seed,
            beacon_id,
            beacon_seed_hex,
        }
    }

    pub fn try_decrypt_token(&self, token: &str) -> Option<Value> {
        let key = self.family_seed.derive_key("dark-forest-token-v1");
        let plaintext_bytes = symmetric_decrypt(token, &key)?;
        let plaintext_str = String::from_utf8(plaintext_bytes).ok()?;
        serde_json::from_str(&plaintext_str).ok()
    }

    pub fn create_dark_forest_token(&self, node_id: &str) -> String {
        let payload = serde_json::json!({
            "family_hash": self.family_seed.family_hash(),
            "node_id": node_id,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
        let key = self.family_seed.derive_key("dark-forest-token-v1");
        symmetric_encrypt(payload.to_string().as_bytes(), &key)
    }

    pub fn encrypt_beacon(&self, endpoint_info: &Value) -> String {
        let key = self.family_seed.derive_key("beacon-exchange-v1");
        symmetric_encrypt(endpoint_info.to_string().as_bytes(), &key)
    }

    pub fn decrypt_beacon(&self, encrypted: &str) -> Option<Value> {
        let key = self.family_seed.derive_key("beacon-exchange-v1");
        let plaintext_bytes = symmetric_decrypt(encrypted, &key)?;
        let plaintext_str = String::from_utf8(plaintext_bytes).ok()?;
        serde_json::from_str(&plaintext_str).ok()
    }

    pub fn encrypt_data(&self, data: &[u8]) -> String {
        let key = self.family_seed.derive_key("data-transfer-v1");
        symmetric_encrypt(data, &key)
    }

    pub fn decrypt_data(&self, ciphertext_hex: &str) -> Option<Vec<u8>> {
        let key = self.family_seed.derive_key("data-transfer-v1");
        symmetric_decrypt(ciphertext_hex, &key)
    }
}

pub struct SimulatedNode {
    pub name: String,
    pub beardog: CryptoMockBearDog,
    endpoint_info: Value,
}

impl SimulatedNode {
    pub fn new(name: &str, family_seed: FamilySeed, endpoint: &str) -> Self {
        let beardog = CryptoMockBearDog::new(family_seed, name);
        let endpoint_info = serde_json::json!({
            "node_id": name,
            "beacon_id": beardog.beacon_id,
            "endpoints": [endpoint],
            "capabilities": match name {
                "pixel8a" => vec!["mobile", "compute", "camera"],
                "usb" => vec!["storage", "compute", "portable"],
                "tower" => vec!["compute", "gpu", "sovereign-beacon"],
                _ => vec!["unknown"],
            },
        });
        Self {
            name: name.to_string(),
            beardog,
            endpoint_info,
        }
    }

    pub fn create_token(&self) -> String {
        self.beardog.create_dark_forest_token(&self.name)
    }

    pub fn encrypt_beacon(&self) -> String {
        self.beardog.encrypt_beacon(&self.endpoint_info)
    }

    pub fn decrypt_peer_beacon(&self, encrypted: &str) -> Option<Value> {
        self.beardog.decrypt_beacon(encrypted)
    }

    pub fn encrypt_data(&self, data: &[u8]) -> String {
        self.beardog.encrypt_data(data)
    }

    pub fn decrypt_data(&self, ciphertext: &str) -> Option<Vec<u8>> {
        self.beardog.decrypt_data(ciphertext)
    }
}

pub struct MockRendezvous {
    slots: Mutex<HashMap<String, Vec<(String, String)>>>,
    family_seed: FamilySeed,
}

impl MockRendezvous {
    pub fn new(family_seed: FamilySeed) -> Self {
        Self {
            slots: Mutex::new(HashMap::new()),
            family_seed,
        }
    }

    pub fn post_beacon(
        &self,
        dark_forest_token: &str,
        encrypted_beacon: &str,
        node_name: &str,
    ) -> Result<(bool, Option<String>, usize), &'static str> {
        let key = self.family_seed.derive_key("dark-forest-token-v1");
        let plaintext_bytes = symmetric_decrypt(dark_forest_token, &key);

        let plaintext = match plaintext_bytes {
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => return Err("token-not-family"),
            },
            None => return Err("token-not-family"),
        };

        let token_data: Value = match serde_json::from_str(&plaintext) {
            Ok(v) => v,
            Err(_) => return Err("token-malformed"),
        };

        let family_hash = token_data
            .get("family_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut hasher = Sha256::new();
        hasher.update(node_name.as_bytes());
        let node_hash = hex::encode(&hasher.finalize()[..8]);

        let mut slots = self.slots.lock().unwrap();
        let lineage_slots = slots.entry(family_hash).or_default();

        let peer_beacon = lineage_slots
            .iter()
            .find(|(hash, _)| *hash != node_hash)
            .map(|(_, beacon)| beacon.clone());

        let peers_waiting = lineage_slots.len();

        lineage_slots.push((node_hash, encrypted_beacon.to_string()));

        Ok((true, peer_beacon, peers_waiting))
    }

    pub fn check_peer(
        &self,
        dark_forest_token: &str,
        node_name: &str,
    ) -> Result<Option<String>, &'static str> {
        let key = self.family_seed.derive_key("dark-forest-token-v1");
        let plaintext_bytes =
            symmetric_decrypt(dark_forest_token, &key).ok_or("token-not-family")?;
        let plaintext = String::from_utf8(plaintext_bytes).map_err(|_| "token-not-family")?;
        let token_data: Value = serde_json::from_str(&plaintext).map_err(|_| "token-malformed")?;

        let family_hash = token_data
            .get("family_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let mut hasher = Sha256::new();
        hasher.update(node_name.as_bytes());
        let node_hash = hex::encode(&hasher.finalize()[..8]);

        let slots = self.slots.lock().unwrap();
        let Some(lineage_slots) = slots.get(&family_hash) else {
            return Ok(None);
        };

        let peer_beacon = lineage_slots
            .iter()
            .find(|(hash, _)| *hash != node_hash)
            .map(|(_, beacon)| beacon.clone());

        Ok(peer_beacon)
    }
}
