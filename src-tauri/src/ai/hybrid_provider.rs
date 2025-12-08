use crate::ai::provider::AiProvider;
use crate::ai::errors::AiProviderError;
use crate::ai::types::*;
use crate::ai::local_provider::LocalProvider;
use crate::ai::cloud_provider::CloudAiProvider;
use crate::ai::settings::{CloudProvider, load_ai_settings};
use async_trait::async_trait;
use std::sync::Arc;
use std::path::PathBuf;

/// Hybrid AI Provider
/// Attempts to use cloud provider first, falls back to local provider on errors
pub struct HybridProvider {
    cloud_provider: Option<Arc<CloudAiProvider>>,
    local_provider: Option<Arc<LocalProvider>>,
    prefer_cloud: bool,
}

impl HybridProvider {
    /// Create a new hybrid provider
    /// 
    /// # Arguments
    /// * `prefer_cloud` - If true, tries cloud first; if false, tries local first
    pub fn new(prefer_cloud: bool) -> Result<Self, String> {
        let settings = load_ai_settings()
            .map_err(|e| format!("Failed to load AI settings: {}", e))?;
        
        // Initialize cloud provider if configured
        let cloud_provider = if let Some(api_key) = &settings.api_key {
            let provider = settings.cloud_provider.unwrap_or(CloudProvider::OpenAI);
            let model_name = settings.model_name
                .unwrap_or_else(|| "gpt-4o-mini".to_string());
            
            log::info!("[HybridProvider] Cloud provider configured: {:?}, model: {}", provider, model_name);
            Some(Arc::new(CloudAiProvider::new(provider, api_key.clone(), model_name)))
        } else {
            log::info!("[HybridProvider] Cloud provider not configured (no API key)");
            None
        };
        
        // Initialize local provider if configured
        let local_provider = if let Some(model_path_str) = &settings.local_model_path {
            let model_path = PathBuf::from(model_path_str);
            if model_path.exists() {
                log::info!("[HybridProvider] Local provider configured: {}", model_path.display());
                Some(Arc::new(LocalProvider::with_model_path(model_path)))
            } else {
                log::warn!("[HybridProvider] Local model path configured but file not found: {}", model_path.display());
                None
            }
        } else {
            log::info!("[HybridProvider] Local provider not configured (no model path)");
            None
        };
        
        if cloud_provider.is_none() && local_provider.is_none() {
            return Err(
                "Hybrid mode requires at least one provider to be configured. Please configure either:\n\
                1. A cloud API key in Settings, or\n\
                2. A local model path in Settings".to_string()
            );
        }
        
        Ok(Self {
            cloud_provider,
            local_provider,
            prefer_cloud,
        })
    }
    
    /// Try an operation with fallback logic
    /// 
    /// Attempts the operation with the preferred provider first,
    /// then falls back to the other provider if the first fails with a recoverable error.
    async fn try_with_fallback<F, Fut, T>(
        &self,
        operation: F,
    ) -> Result<T, AiProviderError>
    where
        F: Fn(Arc<dyn AiProvider>) -> Fut,
        Fut: std::future::Future<Output = Result<T, AiProviderError>>,
    {
        // Determine primary and fallback providers
        let (primary, fallback) = if self.prefer_cloud {
            (
                self.cloud_provider.as_ref().map(|p| p.clone() as Arc<dyn AiProvider>),
                self.local_provider.as_ref().map(|p| p.clone() as Arc<dyn AiProvider>),
            )
        } else {
            (
                self.local_provider.as_ref().map(|p| p.clone() as Arc<dyn AiProvider>),
                self.cloud_provider.as_ref().map(|p| p.clone() as Arc<dyn AiProvider>),
            )
        };
        
        // Try primary provider first
        if let Some(provider) = primary {
            log::debug!("[HybridProvider] Attempting operation with primary provider");
            match operation(provider).await {
                Ok(result) => {
                    log::info!("[HybridProvider] Operation succeeded with primary provider");
                    return Ok(result);
                }
                Err(error) => {
                    // Check if error is recoverable and we have a fallback
                    if Self::is_recoverable_error(&error) && fallback.is_some() {
                        log::warn!("[HybridProvider] Primary provider failed with recoverable error: {}. Falling back to secondary provider.", error);
                    } else {
                        // Non-recoverable error or no fallback - return immediately
                        log::error!("[HybridProvider] Primary provider failed: {}", error);
                        return Err(error);
                    }
                }
            }
        }
        
        // Try fallback provider
        if let Some(provider) = fallback {
            log::info!("[HybridProvider] Attempting operation with fallback provider");
            operation(provider).await
        } else {
            // No fallback available
            Err(AiProviderError::Unknown(
                "No AI provider available. Please configure at least one provider in Settings.".to_string()
            ))
        }
    }
    
    /// Check if an error is recoverable (should trigger fallback)
    fn is_recoverable_error(error: &AiProviderError) -> bool {
        match error {
            // Network errors are usually recoverable - try fallback
            AiProviderError::NetworkError(_) => true,
            // Rate limits are recoverable - try fallback
            AiProviderError::RateLimitExceeded => true,
            // Invalid API key is not recoverable - don't try fallback
            AiProviderError::InvalidApiKey => false,
            // Invalid response might be recoverable - try fallback
            AiProviderError::InvalidResponse(_) => true,
            // Validation errors are not recoverable - don't try fallback
            AiProviderError::ValidationError(_) => false,
            // Model not found is not recoverable - don't try fallback
            AiProviderError::ModelNotFound => false,
            // Unknown errors - be conservative, don't try fallback unless it's clearly a network issue
            AiProviderError::Unknown(msg) => {
                // Check if it's a network-related unknown error
                let msg_lower = msg.to_lowercase();
                msg_lower.contains("network") || 
                msg_lower.contains("connection") || 
                msg_lower.contains("timeout") ||
                msg_lower.contains("unavailable")
            }
        }
    }
}

#[async_trait]
impl AiProvider for HybridProvider {
    async fn generate_resume_suggestions(&self, input: ResumeInput) -> Result<ResumeSuggestions, AiProviderError> {
        self.try_with_fallback(|provider| {
            let input = input.clone();
            async move {
                provider.generate_resume_suggestions(input).await
            }
        })
        .await
    }
    
    async fn generate_cover_letter(&self, input: CoverLetterInput) -> Result<CoverLetter, AiProviderError> {
        self.try_with_fallback(|provider| {
            let input = input.clone();
            async move {
                provider.generate_cover_letter(input).await
            }
        })
        .await
    }
    
    async fn generate_skill_suggestions(&self, input: SkillSuggestionsInput) -> Result<SkillSuggestions, AiProviderError> {
        self.try_with_fallback(|provider| {
            let input = input.clone();
            async move {
                provider.generate_skill_suggestions(input).await
            }
        })
        .await
    }
    
    async fn parse_job(&self, input: JobParsingInput) -> Result<ParsedJobOutput, AiProviderError> {
        self.try_with_fallback(|provider| {
            let input = input.clone();
            async move {
                provider.parse_job(input).await
            }
        })
        .await
    }
    
    async fn call_llm(&self, system_prompt: Option<&str>, user_prompt: &str) -> Result<String, AiProviderError> {
        self.try_with_fallback(|provider| {
            let system_prompt = system_prompt.map(|s| s.to_string());
            let user_prompt = user_prompt.to_string();
            async move {
                provider.call_llm(system_prompt.as_deref(), &user_prompt).await
            }
        })
        .await
    }
}


