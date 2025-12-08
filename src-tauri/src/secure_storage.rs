//! Secure storage for sensitive data using OS keychains
//! 
//! This module provides secure storage for sensitive data like API keys
//! using platform-specific secure storage mechanisms:
//! - macOS: Keychain
//! - Windows: Credential Manager
//! - Linux: Secret Service API (libsecret)
//! 
//! Falls back to encrypted file storage if OS keychain is unavailable.

use std::path::PathBuf;
use crate::db::get_app_data_dir;

/// Store a secret value securely
/// 
/// Uses OS keychain when available, otherwise falls back to encrypted file storage.
/// 
/// # Arguments
/// * `key` - Unique identifier for the secret
/// * `value` - The secret value to store
/// 
/// # Returns
/// `Ok(())` if successful, `Err(String)` if storage failed
pub fn store_secret(key: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        // Remove secret if value is empty
        return remove_secret(key);
    }
    
    // Store metadata about the key rotation
    let now = chrono::Utc::now().to_rfc3339();
    let metadata_key = format!("{}_metadata", key);
    let metadata = KeyMetadata {
        created_at: get_key_metadata(key)
            .map(|m| m.created_at)
            .unwrap_or_else(|_| now.clone()),
        last_rotated_at: now,
        rotation_count: get_key_metadata(key)
            .map(|m| m.rotation_count + 1)
            .unwrap_or(1),
    };
    
    // Store metadata
    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|e| format!("Failed to serialize key metadata: {}", e))?;
    let _ = store_secret_internal(&metadata_key, &metadata_json);
    
    // Try OS keychain first (if available)
    #[cfg(target_os = "macos")]
    {
        if let Ok(()) = store_in_keychain(key, value) {
            return Ok(());
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(()) = store_in_credential_manager(key, value) {
            return Ok(());
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(()) = store_in_secret_service(key, value) {
            return Ok(());
        }
    }
    
    // Fallback to encrypted file storage
    store_in_encrypted_file(key, value)
}

/// Internal function to store secret without metadata updates
fn store_secret_internal(key: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return remove_secret_internal(key);
    }
    
    // Try OS keychain first (if available)
    #[cfg(target_os = "macos")]
    {
        if let Ok(()) = store_in_keychain(key, value) {
            return Ok(());
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(()) = store_in_credential_manager(key, value) {
            return Ok(());
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(()) = store_in_secret_service(key, value) {
            return Ok(());
        }
    }
    
    // Fallback to encrypted file storage
    store_in_encrypted_file(key, value)
}

/// Internal function to remove secret without metadata updates
fn remove_secret_internal(key: &str) -> Result<(), String> {
    // Try OS keychain first (if available)
    #[cfg(target_os = "macos")]
    {
        let _ = remove_from_keychain(key);
    }
    
    #[cfg(target_os = "windows")]
    {
        let _ = remove_from_credential_manager(key);
    }
    
    #[cfg(target_os = "linux")]
    {
        let _ = remove_from_secret_service(key);
    }
    
    // Also try to remove from encrypted file storage
    remove_from_encrypted_file(key)
}

/// Key metadata for rotation tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyMetadata {
    /// When the key was first created
    pub created_at: String,
    /// When the key was last rotated
    pub last_rotated_at: String,
    /// Number of times the key has been rotated
    pub rotation_count: u32,
}

/// Get metadata for a key
pub fn get_key_metadata(key: &str) -> Result<KeyMetadata, String> {
    let metadata_key = format!("{}_metadata", key);
    if let Ok(Some(metadata_json)) = get_secret_internal(&metadata_key) {
        serde_json::from_str(&metadata_json)
            .map_err(|e| format!("Failed to parse key metadata: {}", e))
    } else {
        // Return default metadata if not found
        let now = chrono::Utc::now().to_rfc3339();
        Ok(KeyMetadata {
            created_at: now.clone(),
            last_rotated_at: now,
            rotation_count: 0,
        })
    }
}

/// Internal function to get secret without metadata
fn get_secret_internal(key: &str) -> Result<Option<String>, String> {
    // Try OS keychain first (if available)
    #[cfg(target_os = "macos")]
    {
        if let Ok(Some(value)) = get_from_keychain(key) {
            return Ok(Some(value));
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(Some(value)) = get_from_credential_manager(key) {
            return Ok(Some(value));
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(Some(value)) = get_from_secret_service(key) {
            return Ok(Some(value));
        }
    }
    
    // Fallback to encrypted file storage
    get_from_encrypted_file(key)
}

/// Retrieve a secret value from secure storage
/// 
/// # Arguments
/// * `key` - Unique identifier for the secret
/// 
/// # Returns
/// `Ok(Some(String))` if found, `Ok(None)` if not found, `Err(String)` on error
pub fn get_secret(key: &str) -> Result<Option<String>, String> {
    // Try OS keychain first (if available)
    #[cfg(target_os = "macos")]
    {
        if let Ok(Some(value)) = get_from_keychain(key) {
            return Ok(Some(value));
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(Some(value)) = get_from_credential_manager(key) {
            return Ok(Some(value));
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(Some(value)) = get_from_secret_service(key) {
            return Ok(Some(value));
        }
    }
    
    // Fallback to encrypted file storage
    get_from_encrypted_file(key)
}

/// Remove a secret from secure storage
/// 
/// # Arguments
/// * `key` - Unique identifier for the secret
/// 
/// # Returns
/// `Ok(())` if successful, `Err(String)` if removal failed
pub fn remove_secret(key: &str) -> Result<(), String> {
    // Remove metadata
    let metadata_key = format!("{}_metadata", key);
    let _ = remove_secret_internal(&metadata_key);
    
    // Remove the actual secret
    remove_secret_internal(key)
}

/// Rotate a secret key (replace old key with new key)
/// 
/// This function validates the new key before replacing the old one.
/// The old key is kept until the new key is successfully validated.
/// 
/// # Arguments
/// * `key` - Unique identifier for the secret
/// * `new_value` - The new secret value
/// * `validator` - Optional function to validate the new key before rotation
/// 
/// # Returns
/// `Ok(())` if rotation successful, `Err(String)` if rotation failed
pub fn rotate_secret<F>(key: &str, new_value: &str, validator: Option<F>) -> Result<(), String>
where
    F: Fn(&str) -> Result<(), String>,
{
    if new_value.is_empty() {
        return Err("Cannot rotate to an empty key".to_string());
    }
    
    // Get current key to ensure it exists (for validation)
    let _current_key = get_secret_internal(key)?
        .ok_or_else(|| format!("Key '{}' not found for rotation", key))?;
    
    // Validate new key if validator provided
    if let Some(validate) = validator {
        validate(new_value)
            .map_err(|e| format!("New key validation failed: {}", e))?;
    }
    
    // Store new key (this will update metadata)
    store_secret(key, new_value)?;
    
    log::info!("Successfully rotated key '{}' (rotation count: {})", 
        key, 
        get_key_metadata(key).map(|m| m.rotation_count).unwrap_or(0)
    );
    
    Ok(())
}

/// Check if a key should be rotated based on age
/// 
/// # Arguments
/// * `key` - Unique identifier for the secret
/// * `max_age_days` - Maximum age in days before rotation is recommended
/// 
/// # Returns
/// `Ok(Some(days_old))` if rotation recommended, `Ok(None)` if not needed, `Err` on error
pub fn should_rotate_key(key: &str, max_age_days: u32) -> Result<Option<u32>, String> {
    let metadata = get_key_metadata(key)?;
    
    let last_rotated = chrono::DateTime::parse_from_rfc3339(&metadata.last_rotated_at)
        .map_err(|e| format!("Failed to parse last_rotated_at: {}", e))?
        .with_timezone(&chrono::Utc);
    
    let now = chrono::Utc::now();
    let age_days = (now - last_rotated).num_days() as u32;
    
    if age_days >= max_age_days {
        Ok(Some(age_days))
    } else {
        Ok(None)
    }
}

// macOS Keychain implementation
#[cfg(target_os = "macos")]
fn store_in_keychain(key: &str, value: &str) -> Result<(), String> {
    use std::process::Command;
    
    let service = "com.careerbench.app";
    let account = format!("api_key_{}", key);
    
    // Use security command to add/update keychain item
    let output = Command::new("security")
        .arg("add-generic-password")
        .arg("-a")
        .arg(&account)
        .arg("-s")
        .arg(service)
        .arg("-w")
        .arg(value)
        .arg("-U") // Update if exists
        .output()
        .map_err(|e| format!("Failed to execute security command: {}", e))?;
    
    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Keychain storage failed: {}", error))
    }
}

#[cfg(target_os = "macos")]
fn get_from_keychain(key: &str) -> Result<Option<String>, String> {
    use std::process::Command;
    
    let service = "com.careerbench.app";
    let account = format!("api_key_{}", key);
    
    let output = Command::new("security")
        .arg("find-generic-password")
        .arg("-a")
        .arg(&account)
        .arg("-s")
        .arg(service)
        .arg("-w") // Print password only
        .output()
        .map_err(|e| format!("Failed to execute security command: {}", e))?;
    
    if output.status.success() {
        let value = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 in keychain value: {}", e))?;
        Ok(Some(value.trim().to_string()))
    } else {
        // Not found is not an error
        Ok(None)
    }
}

#[cfg(target_os = "macos")]
fn remove_from_keychain(key: &str) -> Result<(), String> {
    use std::process::Command;
    
    let service = "com.careerbench.app";
    let account = format!("api_key_{}", key);
    
    let output = Command::new("security")
        .arg("delete-generic-password")
        .arg("-a")
        .arg(&account)
        .arg("-s")
        .arg(service)
        .output()
        .map_err(|e| format!("Failed to execute security command: {}", e))?;
    
    if output.status.success() {
        Ok(())
    } else {
        // Not found is not an error
        Ok(())
    }
}

// Windows Credential Manager implementation
#[cfg(target_os = "windows")]
fn store_in_credential_manager(key: &str, value: &str) -> Result<(), String> {
    // For Windows, we would use the Windows API or a crate like `winapi`
    // For now, fall back to encrypted file storage
    // TODO: Implement Windows Credential Manager integration
    Err("Windows Credential Manager not yet implemented".to_string())
}

#[cfg(target_os = "windows")]
fn get_from_credential_manager(_key: &str) -> Result<Option<String>, String> {
    // TODO: Implement Windows Credential Manager integration
    Ok(None)
}

#[cfg(target_os = "windows")]
fn remove_from_credential_manager(_key: &str) -> Result<(), String> {
    // TODO: Implement Windows Credential Manager integration
    Ok(())
}

// Linux Secret Service implementation
#[cfg(target_os = "linux")]
fn store_in_secret_service(key: &str, value: &str) -> Result<(), String> {
    // For Linux, we would use libsecret or a crate like `secret-service`
    // For now, fall back to encrypted file storage
    // TODO: Implement Linux Secret Service integration
    Err("Linux Secret Service not yet implemented".to_string())
}

#[cfg(target_os = "linux")]
fn get_from_secret_service(_key: &str) -> Result<Option<String>, String> {
    // TODO: Implement Linux Secret Service integration
    Ok(None)
}

#[cfg(target_os = "linux")]
fn remove_from_secret_service(_key: &str) -> Result<(), String> {
    // TODO: Implement Linux Secret Service integration
    Ok(())
}

// Fallback: Encrypted file storage
fn get_secure_storage_path() -> PathBuf {
    get_app_data_dir().join("secure_storage")
}

fn store_in_encrypted_file(key: &str, value: &str) -> Result<(), String> {
    use crate::encryption::encrypt;
    
    let encrypted = encrypt(value)?;
    let storage_dir = get_secure_storage_path();
    std::fs::create_dir_all(&storage_dir)
        .map_err(|e| format!("Failed to create secure storage directory: {}", e))?;
    
    let file_path = storage_dir.join(sanitize_key(key));
    std::fs::write(&file_path, encrypted)
        .map_err(|e| format!("Failed to write secure storage file: {}", e))?;
    
    // Set restrictive permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&file_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .permissions();
        perms.set_mode(0o600); // rw-------
        std::fs::set_permissions(&file_path, perms)
            .map_err(|e| format!("Failed to set file permissions: {}", e))?;
    }
    
    Ok(())
}

fn get_from_encrypted_file(key: &str) -> Result<Option<String>, String> {
    use crate::encryption::decrypt;
    
    let file_path = get_secure_storage_path().join(sanitize_key(key));
    
    if !file_path.exists() {
        return Ok(None);
    }
    
    let encrypted = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read secure storage file: {}", e))?;
    
    let decrypted = decrypt(&encrypted)?;
    Ok(Some(decrypted))
}

fn remove_from_encrypted_file(key: &str) -> Result<(), String> {
    let file_path = get_secure_storage_path().join(sanitize_key(key));
    
    if file_path.exists() {
        std::fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to remove secure storage file: {}", e))?;
    }
    
    Ok(())
}

/// Sanitize a key to be safe for use as a filename
fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_key() {
        assert_eq!(sanitize_key("test-key"), "test-key");
        assert_eq!(sanitize_key("test/key"), "test_key");
        assert_eq!(sanitize_key("test.key"), "test_key");
    }
}

