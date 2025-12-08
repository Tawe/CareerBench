use crate::ai::provider::AiProvider;
use crate::ai::settings::{AiMode, CloudProvider, load_ai_settings};
use crate::ai::local_provider::LocalProvider;
use crate::ai::cloud_provider::CloudAiProvider;
use crate::ai::hybrid_provider::HybridProvider;
use std::sync::Arc;

/// Provider resolver
/// Determines which AI provider to use based on settings
pub enum ResolvedProvider {
    Local(Arc<LocalProvider>),
    Cloud(Arc<CloudAiProvider>),
    Hybrid(Arc<HybridProvider>),
}

impl ResolvedProvider {
    /// Resolve the provider based on current settings
    pub fn resolve() -> Result<Self, String> {
        log::info!("[ResolvedProvider] Resolving AI provider from settings...");
        let settings = match load_ai_settings() {
            Ok(s) => {
                log::debug!("[ResolvedProvider] Settings loaded: mode={:?}", s.mode);
                s
            }
            Err(e) => {
                log::error!("[ResolvedProvider] Failed to load settings: {}", e);
                return Err(format!("Failed to load AI settings: {}", e));
            }
        };
        
        match settings.mode {
            AiMode::Local => {
                log::info!("[ResolvedProvider] Local mode selected");
                // Check if local model path is configured
                let model_path = settings.local_model_path
                    .as_ref()
                    .map(|s| std::path::PathBuf::from(s));
                
                if model_path.is_none() {
                    let msg = "Local AI mode requires a model path to be configured. Please go to Settings and either:\n1. Configure a local model path (download a GGUF model from Hugging Face), or\n2. Switch to Cloud mode and add an OpenAI API key.";
                    log::error!("[ResolvedProvider] {}", msg);
                    return Err(msg.to_string());
                }
                
                let path = model_path.unwrap();
                log::info!("[ResolvedProvider] Using local model at: {}", path.display());
                
                // Verify path exists
                if !path.exists() {
                    let msg = format!("Model file not found at: {}. Please verify the path in Settings.", path.display());
                    log::error!("[ResolvedProvider] {}", msg);
                    return Err(msg);
                }
                
                let provider = LocalProvider::with_model_path(path);
                log::info!("[ResolvedProvider] Local provider initialized successfully");
                Ok(ResolvedProvider::Local(Arc::new(provider)))
            }
            AiMode::Cloud => {
                log::info!("[ResolvedProvider] Cloud mode selected");
                let api_key = settings.api_key
                    .ok_or_else(|| {
                        let msg = "AI provider is not set up. Please go to Settings and add an OpenAI API key to use Cloud mode.";
                        log::error!("[ResolvedProvider] {}", msg);
                        msg.to_string()
                    })?;
                let provider = settings.cloud_provider
                    .unwrap_or(CloudProvider::OpenAI);
                let model_name = settings.model_name
                    .unwrap_or_else(|| "gpt-4o-mini".to_string());
                
                log::info!("[ResolvedProvider] Using cloud provider: {:?}, model: {}", provider, model_name);
                Ok(ResolvedProvider::Cloud(Arc::new(
                    CloudAiProvider::new(provider, api_key, model_name)
                )))
            }
            AiMode::Hybrid => {
                log::info!("[ResolvedProvider] Hybrid mode selected");
                // Use HybridProvider which handles intelligent routing and fallback
                // Prefer cloud if both are configured, otherwise use whichever is available
                let prefer_cloud = settings.api_key.is_some() && settings.local_model_path.is_some();
                let hybrid_provider = HybridProvider::new(prefer_cloud)
                    .map_err(|e| {
                        log::error!("[ResolvedProvider] Failed to create hybrid provider: {}", e);
                        e
                    })?;
                log::info!("[ResolvedProvider] Hybrid provider initialized (prefer_cloud: {})", prefer_cloud);
                Ok(ResolvedProvider::Hybrid(Arc::new(hybrid_provider)))
            }
        }
    }
    
    /// Get the provider as a trait object
    pub fn as_provider(&self) -> Arc<dyn AiProvider> {
        match self {
            ResolvedProvider::Local(provider) => provider.clone() as Arc<dyn AiProvider>,
            ResolvedProvider::Cloud(provider) => provider.clone() as Arc<dyn AiProvider>,
            ResolvedProvider::Hybrid(provider) => provider.clone() as Arc<dyn AiProvider>,
        }
    }
}

