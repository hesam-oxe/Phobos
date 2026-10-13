// Phobos IDE - Crypto utilities (hashing, hex, base64-free minimal)
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

pub fn hash_hex(s: &str) -> String {
    format!("{:016x}", hash_str(s))
}

pub fn xor_encode(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter().enumerate().map(|(i,b)| b ^ key[i % key.len()]).collect()
}

pub fn to_hex(data: &[u8]) -> String { hex::encode(data) }
pub fn from_hex(s: &str) -> Result<Vec<u8>, hex::FromHexError> { hex::decode(s) }

/// Simple checksum (FNV-1a 64)
pub fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in data { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}
