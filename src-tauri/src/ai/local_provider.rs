use crate::ai::provider::AiProvider;
use crate::ai::types::*;
use crate::ai::errors::AiProviderError;
use crate::ai::llama_wrapper::{LlamaModel, SharedModel, get_or_load_model};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;

/// Local AI Provider
/// Uses a bundled local model via llama.cpp (GGUF format)
/// This is the default provider for privacy-friendly offline use
/// 
/// IMPLEMENTATION STATUS:
/// - Structure and interface: ✅ Complete
/// - Prompt formatting: ✅ Complete (matches cloud provider)
/// - Model loading: ⏳ Structure ready, needs llama-cpp-sys-3 integration
/// - Inference pipeline: ⏳ Structure ready, needs llama-cpp-sys-3 integration
/// 
/// NEXT STEPS:
/// 1. Complete llama_wrapper.rs implementation with actual llama-cpp-sys-3 calls
/// 2. Test with a Phi-3-mini GGUF model file
/// 3. Test all four operations end-to-end
pub struct LocalProvider {
    // Model path - can be set via settings or use default bundled model
    model_path: Option<PathBuf>,
    // Lazy-loaded model instance (wrapped in Arc<Mutex> for thread safety)
    // This will be populated on first use
    model_cache: SharedModel,
}

impl LocalProvider {
    pub fn new() -> Self {
        Self {
            model_path: None,
            model_cache: Arc::new(Mutex::new(None)),
        }
    }
    
    pub fn with_model_path(path: PathBuf) -> Self {
        Self {
            model_path: Some(path),
            model_cache: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Load the model if not already loaded
    /// This is called lazily on first inference request
    async fn ensure_model_loaded(&self) -> Result<Arc<LlamaModel>, AiProviderError> {
        let path = self.model_path.as_ref()
            .ok_or_else(|| AiProviderError::Unknown(
                "Local model path not configured. Please set a model path in Settings. \
                Recommended: Download Phi-3-mini GGUF model from Hugging Face and configure the path.".to_string()
            ))?;
        
        if !path.exists() {
            return Err(AiProviderError::Unknown(
                format!("Model file not found at: {}. Please download a GGUF model (e.g., Phi-3-mini) and configure the path in settings.", path.display())
            ));
        }
        
        // Load or get cached model
        get_or_load_model(&self.model_cache, path.clone()).await
    }
    
    /// Run inference on the local model
    /// Formats the prompt and returns JSON response
    async fn run_inference(&self, system_prompt: &str, user_prompt: &str) -> Result<serde_json::Value, AiProviderError> {
        // Ensure model is loaded
        let model = self.ensure_model_loaded().await?;
        
        // Format the full prompt for local models
        // Local models typically need a single prompt string rather than system/user separation
        let full_prompt = format!("{}\n\n{}", system_prompt, user_prompt);
        
        // Run inference
        // Use a reasonable max_tokens for JSON output (typically 500-1000 tokens is enough)
        let response = model.generate(&full_prompt, 1000).await?;
        
        // Extract JSON from response (may need to parse markdown code blocks)
        let json_str = Self::extract_json_from_response(&response);
        
        // Parse JSON
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| AiProviderError::InvalidResponse(
                format!("Failed to parse JSON from model response: {}. Response was: {}", e, response)
            ))?;
        
        Ok(json)
    }
    
    /// Extract JSON from model response
    /// Handles cases where model wraps JSON in markdown code blocks
    fn extract_json_from_response(response: &str) -> String {
        // Try to find JSON in the response
        // Models sometimes wrap JSON in ```json ... ``` blocks
        
        // First, try to find JSON object boundaries
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_candidate = &response[start..=end];
                // Try to parse it to validate
                if serde_json::from_str::<serde_json::Value>(json_candidate).is_ok() {
                    return json_candidate.to_string();
                }
            }
        }
        
        // If no valid JSON found, try extracting from markdown code blocks
        if let Some(start) = response.find("```json") {
            let after_start = &response[start + 7..];
            if let Some(end) = after_start.find("```") {
                return after_start[..end].trim().to_string();
            }
        }
        
        // Fallback: return the whole response and let the caller handle parsing errors
        response.to_string()
    }
    
    /// Build system prompt for resume generation
    /// Same format as cloud provider for consistency
    fn build_resume_system_prompt() -> String {
        "You are a resume writing assistant. Your task is to help reorganize and improve existing resume content. 
CRITICAL RULES:
- NEVER invent skills, companies, dates, or experiences that don't exist in the input
- ONLY reorganize, rephrase, or restructure existing information
- Output MUST be valid JSON matching the ResumeSuggestions schema
- Be concise and professional
- Focus on achievements and impact".to_string()
    }
    
    /// Build system prompt for cover letter generation
    fn build_cover_letter_system_prompt() -> String {
        "You are a cover letter writing assistant. Your task is to write a professional cover letter based on the user's profile and job description.
CRITICAL RULES:
- NEVER invent skills, companies, dates, or experiences
- ONLY use information provided in the user's profile
- Output MUST be valid JSON matching the CoverLetter schema
- Be professional and tailored to the specific job".to_string()
    }
    
    /// Build system prompt for skill suggestions
    fn build_skill_suggestions_system_prompt() -> String {
        "You are a career advisor. Your task is to analyze skill gaps between the user's current skills and job requirements.
CRITICAL RULES:
- Identify missing skills that are mentioned in the job description
- Assess importance (high/medium/low) based on how frequently mentioned
- Provide actionable recommendations
- Output MUST be valid JSON matching the SkillSuggestions schema".to_string()
    }
    
    /// Build system prompt for job parsing
    fn build_job_parsing_system_prompt() -> String {
        "You are a job description parser. Your task is to extract structured information from job postings.
CRITICAL RULES:
- Extract only information that is explicitly stated in the job description
- NEVER invent or infer skills, responsibilities, or requirements that aren't mentioned
- Output MUST be valid JSON matching the ParsedJob schema
- Be thorough but accurate - only extract what you can clearly identify".to_string()
    }
}

impl Default for LocalProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProvider for LocalProvider {
    async fn generate_resume_suggestions(&self, input: ResumeInput) -> Result<ResumeSuggestions, AiProviderError> {
        let system_prompt = Self::build_resume_system_prompt();
        let user_prompt = format!(
            "Profile data:\n{}\n\nJob description:\n{}\n\nGenerate resume suggestions in JSON format.",
            serde_json::to_string_pretty(&input.profile_data).unwrap_or_default(),
            input.job_description
        );
        
        let json_response = self.run_inference(&system_prompt, &user_prompt).await?;
        serde_json::from_value(json_response)
            .map_err(|e| AiProviderError::ValidationError(format!("Failed to deserialize resume suggestions: {}", e)))
    }
    
    async fn generate_cover_letter(&self, input: CoverLetterInput) -> Result<CoverLetter, AiProviderError> {
        let system_prompt = Self::build_cover_letter_system_prompt();
        let user_prompt = format!(
            "Profile data:\n{}\n\nJob description:\n{}\n\nCompany: {}\n\nGenerate a cover letter in JSON format.",
            serde_json::to_string_pretty(&input.profile_data).unwrap_or_default(),
            input.job_description,
            input.company_name.as_deref().unwrap_or("the company")
        );
        
        let json_response = self.run_inference(&system_prompt, &user_prompt).await?;
        serde_json::from_value(json_response)
            .map_err(|e| AiProviderError::ValidationError(format!("Failed to deserialize cover letter: {}", e)))
    }
    
    async fn generate_skill_suggestions(&self, input: SkillSuggestionsInput) -> Result<SkillSuggestions, AiProviderError> {
        let system_prompt = Self::build_skill_suggestions_system_prompt();
        let user_prompt = format!(
            "Current skills: {}\n\nJob description:\n{}\n\nGenerate skill suggestions in JSON format.",
            input.current_skills.join(", "),
            input.job_description
        );
        
        let json_response = self.run_inference(&system_prompt, &user_prompt).await?;
        serde_json::from_value(json_response)
            .map_err(|e| AiProviderError::ValidationError(format!("Failed to deserialize skill suggestions: {}", e)))
    }
    
    async fn parse_job(&self, input: JobParsingInput) -> Result<ParsedJobOutput, AiProviderError> {
        let system_prompt = Self::build_job_parsing_system_prompt();
        let user_prompt = format!(
            "Job description:\n{}\n\nParse this job description and extract structured information in JSON format.",
            input.job_description
        );
        
        let json_response = self.run_inference(&system_prompt, &user_prompt).await?;
        serde_json::from_value(json_response)
            .map_err(|e| AiProviderError::ValidationError(format!("Failed to deserialize parsed job: {}", e)))
    }
}
