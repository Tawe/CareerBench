use std::fmt;

/// Error type for AI provider operations
#[derive(Debug, Clone)]
pub enum AiProviderError {
    NetworkError(String),
    InvalidResponse(String),
    RateLimitExceeded,
    InvalidApiKey,
    #[allow(dead_code)]
    ModelNotFound,
    ValidationError(String),
    Unknown(String),
}

impl fmt::Display for AiProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiProviderError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            AiProviderError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            AiProviderError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            AiProviderError::InvalidApiKey => write!(f, "Invalid API key"),
            AiProviderError::ModelNotFound => write!(f, "Model not found"),
            AiProviderError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AiProviderError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for AiProviderError {}

