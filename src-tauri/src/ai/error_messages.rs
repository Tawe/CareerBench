use crate::ai::errors::AiProviderError;

/// User-friendly error message with recovery suggestions
#[derive(Debug, Clone)]
pub struct UserFriendlyError {
    /// Main error message for the user
    pub message: String,
    /// Suggested recovery actions
    #[allow(dead_code)]
    pub suggestions: Vec<String>,
    /// Whether the error is recoverable (user can try again)
    #[allow(dead_code)]
    pub recoverable: bool,
    /// Whether the error requires user action (e.g., configure settings)
    #[allow(dead_code)]
    pub requires_action: bool,
}

impl UserFriendlyError {
    pub fn new(message: String, suggestions: Vec<String>, recoverable: bool, requires_action: bool) -> Self {
        Self {
            message,
            suggestions,
            recoverable,
            requires_action,
        }
    }
}

/// Convert AI provider errors to user-friendly messages with recovery suggestions
pub fn to_user_friendly_error(error: &AiProviderError) -> UserFriendlyError {
    match error {
        AiProviderError::InvalidApiKey => UserFriendlyError::new(
            "Your API key is invalid or has expired".to_string(),
            vec![
                "Check your API key in Settings".to_string(),
                "Verify the key is correct and hasn't been revoked".to_string(),
                "Generate a new API key if needed".to_string(),
            ],
            false,
            true,
        ),
        
        AiProviderError::RateLimitExceeded => UserFriendlyError::new(
            "Rate limit exceeded. Too many requests in a short time".to_string(),
            vec![
                "Wait a few moments and try again".to_string(),
                "The system will automatically retry with a delay".to_string(),
                "Consider upgrading your API plan for higher limits".to_string(),
            ],
            true,
            false,
        ),
        
        AiProviderError::NetworkError(msg) => {
            // Try to extract more specific information from the error message
            let message = if msg.contains("timeout") || msg.contains("timed out") {
                "Connection timed out. The AI service may be slow or unavailable".to_string()
            } else if msg.contains("connection") || msg.contains("refused") {
                "Cannot connect to the AI service. Check your internet connection".to_string()
            } else if msg.contains("429") {
                "Too many requests. Please wait a moment and try again".to_string()
            } else {
                format!("Network error: {}", msg)
            };
            
            UserFriendlyError::new(
                message,
                vec![
                    "Check your internet connection".to_string(),
                    "Wait a moment and try again".to_string(),
                    "If the problem persists, the AI service may be temporarily unavailable".to_string(),
                ],
                true,
                false,
            )
        },
        
        AiProviderError::InvalidResponse(_msg) => UserFriendlyError::new(
            "The AI service returned an unexpected response".to_string(),
            vec![
                "Try generating again - this is usually a temporary issue".to_string(),
                "If the problem continues, the AI model may be experiencing issues".to_string(),
            ],
            true,
            false,
        ),
        
        AiProviderError::ValidationError(msg) => UserFriendlyError::new(
            format!("Validation error: {}", msg),
            vec![
                "The AI response didn't match the expected format".to_string(),
                "Try generating again".to_string(),
            ],
            true,
            false,
        ),
        
        AiProviderError::ModelNotFound => UserFriendlyError::new(
            "The specified AI model was not found".to_string(),
            vec![
                "Check your AI settings and verify the model name".to_string(),
                "For local models, ensure the model file exists at the specified path".to_string(),
            ],
            false,
            true,
        ),
        
        AiProviderError::Unknown(msg) => {
            // Check for common error patterns
            if msg.contains("not configured") || msg.contains("not set up") || msg.contains("not yet implemented") {
                UserFriendlyError::new(
                    "AI provider is not configured".to_string(),
                    vec![
                        "Go to Settings to configure your AI provider".to_string(),
                        "For cloud providers, enter your API key".to_string(),
                        "For local providers, specify the model file path".to_string(),
                    ],
                    false,
                    true,
                )
            } else if msg.contains("model path") || msg.contains("model file") {
                UserFriendlyError::new(
                    "Local AI model file not found".to_string(),
                    vec![
                        "Check the model path in Settings".to_string(),
                        "Ensure the model file exists and is accessible".to_string(),
                        "Download a GGUF model file if you haven't already".to_string(),
                    ],
                    false,
                    true,
                )
            } else {
                UserFriendlyError::new(
                    format!("An unexpected error occurred: {}", msg),
                    vec![
                        "Try again in a moment".to_string(),
                        "If the problem persists, check your AI settings".to_string(),
                    ],
                    true,
                    false,
                )
            }
        }
    }
}

/// Format error message for display in UI
#[allow(dead_code)]
pub fn format_error_for_ui(error: &AiProviderError) -> String {
    let friendly = to_user_friendly_error(error);
    let mut result = friendly.message.clone();
    
    if !friendly.suggestions.is_empty() {
        result.push_str("\n\nSuggestions:");
        for (_i, suggestion) in friendly.suggestions.iter().enumerate() {
            result.push_str(&format!("\n• {}", suggestion));
        }
    }
    
    result
}

/// Get a short error message (without suggestions) for toasts/notifications
pub fn get_short_error_message(error: &AiProviderError) -> String {
    to_user_friendly_error(error).message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_api_key_error() {
        let error = AiProviderError::InvalidApiKey;
        let friendly = to_user_friendly_error(&error);
        
        assert!(friendly.message.contains("API key"));
        assert!(!friendly.recoverable);
        assert!(friendly.requires_action);
        assert!(!friendly.suggestions.is_empty());
    }

    #[test]
    fn test_rate_limit_error() {
        let error = AiProviderError::RateLimitExceeded;
        let friendly = to_user_friendly_error(&error);
        
        assert!(friendly.message.contains("Rate limit"));
        assert!(friendly.recoverable);
        assert!(!friendly.requires_action);
    }

    #[test]
    fn test_network_error_with_timeout() {
        let error = AiProviderError::NetworkError("Connection timed out".to_string());
        let friendly = to_user_friendly_error(&error);
        
        assert!(friendly.message.contains("timed out") || friendly.message.contains("timeout"));
        assert!(friendly.recoverable);
    }

    #[test]
    fn test_unknown_error_with_not_configured() {
        let error = AiProviderError::Unknown("AI provider not configured".to_string());
        let friendly = to_user_friendly_error(&error);
        
        assert!(friendly.message.contains("not configured"));
        assert!(friendly.requires_action);
        assert!(friendly.suggestions.iter().any(|s| s.contains("Settings")));
    }
}

