pub mod types;
pub mod provider;
pub mod errors;
pub mod local_provider;
pub mod cloud_provider;
pub mod hybrid_provider;
pub mod settings;
pub mod resolver;
pub mod retry;
pub mod rate_limiter;
pub mod error_messages;
pub mod validation;
pub mod llama_wrapper;
pub mod key_rotation;

// Mock provider for testing - always available for integration tests
pub mod mock_provider;

// AiProvider trait is used internally by LocalProvider and CloudAiProvider
// but not exported for external use (accessed via ResolvedProvider instead)

