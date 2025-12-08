use serde_json::Value;

#[cfg(test)]
use serde_json::json;

/// Error type for AI client operations
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AiError {
    NetworkError(String),
    InvalidResponse(String),
    RateLimitExceeded,
    Unknown(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            AiError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            AiError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            AiError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for AiError {}

/// Trait for AI client implementations
/// This abstraction allows us to swap between real and mock implementations
#[allow(dead_code)]
pub trait AiClient: Send + Sync {
    /// Generate structured JSON from a prompt
    /// Returns a serde_json::Value that should be deserialized into a specific struct
    fn generate_json(&self, prompt: &str) -> Result<Value, AiError>;
}

/// Real AI client implementation (for production)
/// This would call an actual AI provider API
#[allow(dead_code)]
pub struct RealAiClient {
    // Add configuration fields as needed (API key, endpoint, etc.)
}

impl RealAiClient {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}

impl AiClient for RealAiClient {
    fn generate_json(&self, _prompt: &str) -> Result<Value, AiError> {
        // TODO: Implement actual AI provider call
        // For now, return an error to indicate it's not implemented
        Err(AiError::Unknown(
            "Real AI client not yet implemented".to_string(),
        ))
    }
}

/// Mock AI client for testing
/// Returns fixed responses based on test keys or purpose
#[allow(dead_code)]
pub struct MockAiClient {
    responses: std::collections::HashMap<String, Value>,
}

impl MockAiClient {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            responses: std::collections::HashMap::new(),
        }
    }

    /// Register a response for a given test key
    #[allow(dead_code)]
    pub fn register_response(&mut self, key: &str, response: Value) {
        self.responses.insert(key.to_string(), response);
    }

    /// Register a response based on prompt content (simple pattern matching)
    #[allow(dead_code)]
    pub fn register_response_for_prompt(&mut self, prompt_pattern: &str, response: Value) {
        self.responses
            .insert(prompt_pattern.to_string(), response);
    }
}

impl Default for MockAiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AiClient for MockAiClient {
    fn generate_json(&self, prompt: &str) -> Result<Value, AiError> {
        // Try exact match first
        if let Some(response) = self.responses.get(prompt) {
            return Ok(response.clone());
        }

        // Try pattern matching (simple contains check)
        for (pattern, response) in &self.responses {
            if prompt.contains(pattern) {
                return Ok(response.clone());
            }
        }

        // Default: return an error indicating no mock response was found
        Err(AiError::Unknown(format!(
            "No mock response registered for prompt: {}",
            prompt
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_client_exact_match() {
        let mut client = MockAiClient::new();
        let test_response = json!({"result": "test"});
        client.register_response("test_prompt", test_response.clone());

        let result = client.generate_json("test_prompt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_response);
    }

    #[test]
    fn test_mock_client_pattern_match() {
        let mut client = MockAiClient::new();
        let test_response = json!({"result": "parsed_job"});
        client.register_response_for_prompt("parse this job", test_response.clone());

        let result = client.generate_json("Please parse this job description");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_response);
    }

    #[test]
    fn test_mock_client_no_match() {
        let client = MockAiClient::new();
        let result = client.generate_json("unknown prompt");
        assert!(result.is_err());
    }
}

