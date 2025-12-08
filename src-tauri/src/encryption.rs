//! Encryption utilities for sensitive data (API keys, etc.)
//! 
//! This module provides encryption/decryption functionality for storing
//! sensitive data like API keys in the database. Uses AES-GCM encryption
//! with a key derived from system information.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use std::sync::Mutex;

// Global encryption key (derived from system)
static ENCRYPTION_KEY: Mutex<Option<Key<Aes256Gcm>>> = Mutex::new(None);

/// Initialize the encryption key
/// 
/// Derives a key from system information. In a production environment,
/// this could be enhanced to use OS keychains or Tauri secure storage.
fn get_encryption_key() -> Result<Key<Aes256Gcm>, String> {
    let mut key_guard = ENCRYPTION_KEY.lock()
        .map_err(|e| format!("Failed to lock encryption key: {}", e))?;
    
    if let Some(key) = key_guard.as_ref() {
        return Ok(key.clone());
    }
    
    // Derive key from system information
    // In production, this should use a more secure method (OS keychain, Tauri secure storage)
    let system_id = get_system_id();
    let key_bytes = derive_key_from_system_id(&system_id);
    
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    *key_guard = Some(key.clone());
    
    Ok(key.clone())
}

/// Get a system identifier for key derivation
/// 
/// Uses a combination of system properties to create a unique identifier.
/// This ensures the encryption key is consistent across app restarts.
fn get_system_id() -> String {
    // Use a combination of system properties
    // In production, consider using Tauri's secure storage or OS keychain
    let mut id = String::new();
    
    // Add username if available
    if let Ok(user) = std::env::var("USER") {
        id.push_str(&user);
    } else if let Ok(user) = std::env::var("USERNAME") {
        id.push_str(&user);
    }
    
    // Add app data directory path (consistent per user)
    if let Ok(app_dir) = std::env::current_dir() {
        id.push_str(&app_dir.to_string_lossy());
    }
    
    // Fallback to a default if nothing is available
    if id.is_empty() {
        id = "careerbench-default-key".to_string();
    }
    
    id
}

/// Derive a 32-byte key from system ID using SHA-256
fn derive_key_from_system_id(system_id: &str) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(system_id.as_bytes());
    hasher.update(b"careerbench-encryption-salt-v1");
    let hash = hasher.finalize();
    
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash[..32]);
    key
}

/// Encrypt a string value
/// 
/// Returns a base64-encoded string containing the nonce and ciphertext.
/// Format: base64(nonce || ciphertext)
pub fn encrypt(plaintext: &str) -> Result<String, String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }
    
    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Combine nonce and ciphertext, then base64 encode
    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(nonce.as_slice());
    combined.extend_from_slice(&ciphertext);
    
    Ok(general_purpose::STANDARD.encode(&combined))
}

/// Decrypt a base64-encoded encrypted string
/// 
/// Expects the format: base64(nonce || ciphertext)
pub fn decrypt(encrypted: &str) -> Result<String, String> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }
    
    // Try to decode as base64
    let combined = general_purpose::STANDARD.decode(encrypted)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    
    // Check if this looks like encrypted data (has nonce + ciphertext)
    if combined.len() < 12 {
        // Too short to contain nonce (12 bytes) + ciphertext
        // This might be plaintext, return as-is for backward compatibility
        return Ok(String::from_utf8_lossy(&combined).to_string());
    }
    
    // Extract nonce (first 12 bytes) and ciphertext (rest)
    let nonce = Nonce::from_slice(&combined[..12]);
    let ciphertext = &combined[12..];
    
    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new(&key);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    String::from_utf8(plaintext)
        .map_err(|e| format!("Invalid UTF-8 in decrypted data: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let plaintext = "test-api-key-12345";
        let encrypted = encrypt(plaintext).unwrap();
        assert_ne!(encrypted, plaintext);
        
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_empty() {
        let encrypted = encrypt("").unwrap();
        assert_eq!(encrypted, "");
        
        let decrypted = decrypt("").unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_encrypt_long_string() {
        let plaintext = "a".repeat(1000);
        let encrypted = encrypt(&plaintext).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}

