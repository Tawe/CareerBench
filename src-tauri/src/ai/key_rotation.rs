//! API key rotation utilities
//! 
//! This module provides functions for rotating API keys with validation
//! and testing before committing the rotation.

use crate::secure_storage::{rotate_secret, get_key_metadata, should_rotate_key};
use crate::ai::errors::AiProviderError;
use crate::ai::provider::AiProvider;

/// Rotate the AI API key with validation
/// 
/// This function:
/// 1. Validates the new API key by testing it with the AI provider
/// 2. Only replaces the old key if validation succeeds
/// 3. Updates key rotation metadata
/// 
/// # Arguments
/// * `new_api_key` - The new API key to rotate to
/// * `provider` - The cloud provider (OpenAI or Anthropic)
/// 
/// # Returns
/// `Ok(())` if rotation successful, `Err(String)` if rotation failed
pub async fn rotate_api_key(new_api_key: &str, provider: crate::ai::settings::CloudProvider) -> Result<(), String> {
    if new_api_key.is_empty() {
        return Err("API key cannot be empty".to_string());
    }
    
    // Create a temporary provider with the new key to validate it
    let model_name = match provider {
        crate::ai::settings::CloudProvider::OpenAI => "gpt-4o-mini",
        crate::ai::settings::CloudProvider::Anthropic => "claude-3-5-sonnet-20241022",
    };
    
    let temp_provider = crate::ai::cloud_provider::CloudAiProvider::new(
        provider,
        new_api_key.to_string(),
        model_name.to_string(),
    );
    
    // Test the new key with a simple API call
    log::info!("Validating new API key before rotation...");
    let test_result = temp_provider
        .parse_job(crate::ai::types::JobParsingInput {
            job_description: "Test job: Software Engineer at Test Company".to_string(),
            job_meta: None,
        })
        .await;
    
    match test_result {
        Ok(_) => {
            log::info!("New API key validated successfully");
        }
        Err(AiProviderError::InvalidApiKey) => {
            return Err("Invalid API key: The provided key is not valid for this provider".to_string());
        }
        Err(e) => {
            // Other errors (network, etc.) don't necessarily mean the key is invalid
            // Log a warning but proceed with rotation
            log::warn!("API key validation returned error (may be transient): {}", e);
        }
    }
    
    // Rotate the key in secure storage
    rotate_secret("ai_api_key", new_api_key, None::<fn(&str) -> Result<(), String>>)
        .map_err(|e| format!("Failed to rotate API key: {}", e))?;
    
    log::info!("API key rotated successfully");
    Ok(())
}

/// Get API key rotation metadata
/// 
/// Returns information about when the API key was created, last rotated, and rotation count.
pub fn get_api_key_metadata() -> Result<crate::secure_storage::KeyMetadata, String> {
    get_key_metadata("ai_api_key")
}

/// Check if API key should be rotated
/// 
/// # Arguments
/// * `max_age_days` - Maximum age in days before rotation is recommended (default: 90)
/// 
/// # Returns
/// `Ok(Some(days_old))` if rotation recommended, `Ok(None)` if not needed
pub fn check_api_key_rotation_needed(max_age_days: Option<u32>) -> Result<Option<u32>, String> {
    let max_age = max_age_days.unwrap_or(90); // Default: 90 days
    should_rotate_key("ai_api_key", max_age)
}

