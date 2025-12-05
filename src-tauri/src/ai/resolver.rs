use crate::ai::provider::AiProvider;
use crate::ai::settings::{AiMode, CloudProvider, load_ai_settings};
use crate::ai::local_provider::LocalProvider;
use crate::ai::cloud_provider::CloudAiProvider;
use std::sync::Arc;

/// Provider resolver
/// Determines which AI provider to use based on settings
pub enum ResolvedProvider {
    Local(Arc<LocalProvider>),
    Cloud(Arc<CloudAiProvider>),
}

impl ResolvedProvider {
    /// Resolve the provider based on current settings
    pub fn resolve() -> Result<Self, String> {
        let settings = load_ai_settings()?;
        
        match settings.mode {
            AiMode::Local => {
                // Check if local model path is configured
                let model_path = settings.local_model_path
                    .as_ref()
                    .map(|s| std::path::PathBuf::from(s));
                
                if model_path.is_none() {
                    return Err("Local AI mode requires a model path to be configured. Please go to Settings and either:\n1. Configure a local model path (download a GGUF model from Hugging Face), or\n2. Switch to Cloud mode and add an OpenAI API key.".to_string());
                }
                
                let provider = if let Some(path) = model_path {
                    LocalProvider::with_model_path(path)
                } else {
                    LocalProvider::new()
                };
                
                Ok(ResolvedProvider::Local(Arc::new(provider)))
            }
            AiMode::Cloud => {
                let api_key = settings.api_key
                    .ok_or_else(|| "AI provider is not set up. Please go to Settings and add an OpenAI API key to use Cloud mode.".to_string())?;
                let provider = settings.cloud_provider
                    .unwrap_or(CloudProvider::OpenAI);
                let model_name = settings.model_name
                    .unwrap_or_else(|| "gpt-4o-mini".to_string());
                
                Ok(ResolvedProvider::Cloud(Arc::new(
                    CloudAiProvider::new(provider, api_key, model_name)
                )))
            }
            AiMode::Hybrid => {
                // For hybrid mode, prefer cloud if configured, otherwise try local
                if let Some(api_key) = &settings.api_key {
                    let provider = settings.cloud_provider
                        .unwrap_or(CloudProvider::OpenAI);
                    let model_name = settings.model_name
                        .unwrap_or_else(|| "gpt-4o-mini".to_string());
                    
                    Ok(ResolvedProvider::Cloud(Arc::new(
                        CloudAiProvider::new(provider, api_key.clone(), model_name)
                    )))
                } else if let Some(model_path_str) = &settings.local_model_path {
                    let model_path = std::path::PathBuf::from(model_path_str);
                    Ok(ResolvedProvider::Local(Arc::new(
                        LocalProvider::with_model_path(model_path)
                    )))
                } else {
                    Err("Hybrid mode requires either a cloud API key or a local model path to be configured. Please go to Settings to configure one.".to_string())
                }
            }
        }
    }
    
    /// Get the provider as a trait object
    pub fn as_provider(&self) -> Arc<dyn AiProvider> {
        match self {
            ResolvedProvider::Local(provider) => provider.clone() as Arc<dyn AiProvider>,
            ResolvedProvider::Cloud(provider) => provider.clone() as Arc<dyn AiProvider>,
        }
    }
}

