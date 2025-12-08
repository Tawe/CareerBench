use crate::ai::provider::AiProvider;
use crate::ai::types::*;
use crate::ai::errors::AiProviderError;
use crate::ai::settings::CloudProvider;
use crate::ai::retry::{retry_with_backoff, RetryConfig};
use crate::ai::rate_limiter::RateLimiter;
use crate::ai::validation::{validate_parsed_job, validate_resume_suggestions, validate_cover_letter, validate_skill_suggestions};
use async_trait::async_trait;
use serde_json::{json, Value};
use reqwest::Client;
use std::sync::Arc;

/// Cloud AI Provider
/// Supports multiple cloud providers (OpenAI, Anthropic, etc.)
pub struct CloudAiProvider {
    provider: CloudProvider,
    api_key: String,
    model_name: String,
    client: Client,
    rate_limiter: Arc<RateLimiter>,
}

impl CloudAiProvider {
    pub fn new(provider: CloudProvider, api_key: String, model_name: String) -> Self {
        // Create rate limiter based on provider
        let rate_limiter = match provider {
            CloudProvider::OpenAI => RateLimiter::openai_default(),
            CloudProvider::Anthropic => RateLimiter::anthropic_default(),
        };
        
        Self {
            provider,
            api_key,
            model_name,
            client: Client::new(),
            rate_limiter: Arc::new(rate_limiter),
        }
    }
    
    async fn call_anthropic(&self, system_prompt: &str, user_prompt: &str) -> Result<Value, AiProviderError> {
        // Acquire rate limit token before making the request
        self.rate_limiter.acquire().await;
        
        let url = "https://api.anthropic.com/v1/messages";
        let client = &self.client;
        let api_key = &self.api_key;
        let model_name = &self.model_name;
        
        // Use retry logic for the API call
        let retry_config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 500,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
        };
        
        retry_with_backoff(
            || {
                let client = client.clone();
                let url = url.to_string();
                let api_key = api_key.clone();
                let model_name = model_name.clone();
                let system_prompt = system_prompt.to_string();
                let user_prompt = user_prompt.to_string();
                
                async move {
                    let response = client
                        .post(&url)
                        .header("x-api-key", api_key)
                        .header("anthropic-version", "2023-06-01")
                        .header("Content-Type", "application/json")
                        .json(&json!({
                            "model": model_name,
                            "max_tokens": 4096,
                            "system": system_prompt,
                            "messages": [
                                {
                                    "role": "user",
                                    "content": user_prompt
                                }
                            ],
                            "temperature": 0.3
                        }))
                        .send()
                        .await
                        .map_err(|e| AiProviderError::NetworkError(e.to_string()))?;
                    
                    if response.status() == 401 {
                        return Err(AiProviderError::InvalidApiKey);
                    }
                    
                    if response.status() == 429 {
                        return Err(AiProviderError::RateLimitExceeded);
                    }
                    
                    if !response.status().is_success() {
                        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        return Err(AiProviderError::NetworkError(format!("API error: {}", error_text)));
                    }
                    
                    let json_response: Value = response
                        .json()
                        .await
                        .map_err(|e| AiProviderError::InvalidResponse(e.to_string()))?;
                    
                    // Extract content from Anthropic response format
                    // Anthropic returns: { "content": [{"type": "text", "text": "..."}] }
                    let content = json_response
                        .get("content")
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.get(0))
                        .and_then(|item| item.get("text"))
                        .and_then(|t| t.as_str())
                        .ok_or_else(|| AiProviderError::InvalidResponse("Missing content in response".to_string()))?;
                    
                    // Parse the JSON content
                    serde_json::from_str(content)
                        .map_err(|e| AiProviderError::InvalidResponse(format!("Failed to parse JSON: {}", e)))
                }
            },
            retry_config,
        )
        .await
    }
    
    async fn call_openai(&self, system_prompt: &str, user_prompt: &str) -> Result<Value, AiProviderError> {
        // Acquire rate limit token before making the request
        self.rate_limiter.acquire().await;
        
        let url = "https://api.openai.com/v1/chat/completions";
        let client = &self.client;
        let api_key = &self.api_key;
        let model_name = &self.model_name;
        
        // Use retry logic for the API call
        let retry_config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 500,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
        };
        
        retry_with_backoff(
            || {
                let client = client.clone();
                let url = url.to_string();
                let api_key = api_key.clone();
                let model_name = model_name.clone();
                let system_prompt = system_prompt.to_string();
                let user_prompt = user_prompt.to_string();
                
                async move {
                    let response = client
                        .post(&url)
                        .header("Authorization", format!("Bearer {}", api_key))
                        .header("Content-Type", "application/json")
                        .json(&json!({
                            "model": model_name,
                            "messages": [
                                {
                                    "role": "system",
                                    "content": system_prompt
                                },
                                {
                                    "role": "user",
                                    "content": user_prompt
                                }
                            ],
                            "temperature": 0.3, // Lower temperature for more deterministic output
                            "response_format": {
                                "type": "json_object"
                            }
                        }))
                        .send()
                        .await
                        .map_err(|e| AiProviderError::NetworkError(e.to_string()))?;
                    
                    if response.status() == 401 {
                        return Err(AiProviderError::InvalidApiKey);
                    }
                    
                    if response.status() == 429 {
                        return Err(AiProviderError::RateLimitExceeded);
                    }
                    
                    if !response.status().is_success() {
                        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        return Err(AiProviderError::NetworkError(format!("API error: {}", error_text)));
                    }
                    
                    let json_response: Value = response
                        .json()
                        .await
                        .map_err(|e| AiProviderError::InvalidResponse(e.to_string()))?;
                    
                    // Extract content from OpenAI response format
                    let content = json_response
                        .get("choices")
                        .and_then(|c| c.get(0))
                        .and_then(|c| c.get("message"))
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| AiProviderError::InvalidResponse("Missing content in response".to_string()))?;
                    
                    // Parse the JSON content
                    serde_json::from_str(content)
                        .map_err(|e| AiProviderError::InvalidResponse(format!("Failed to parse JSON: {}", e)))
                }
            },
            retry_config,
        )
        .await
    }
    
    fn build_resume_system_prompt() -> String {
        "You are a resume writing assistant. Your task is to help reorganize and improve existing resume content. 
CRITICAL RULES:
- NEVER invent skills, companies, dates, or experiences that don't exist in the input
- ONLY reorganize, rephrase, or restructure existing information
- Output MUST be valid JSON matching the ResumeSuggestions schema
- Be concise and professional
- Focus on achievements and impact".to_string()
    }
    
    fn build_cover_letter_system_prompt() -> String {
        "You are a cover letter writing assistant. Your task is to write a professional cover letter based on the user's profile and job description.
CRITICAL RULES:
- NEVER invent skills, companies, dates, or experiences
- ONLY use information provided in the user's profile
- Output MUST be valid JSON matching the CoverLetter schema
- Be professional and tailored to the specific job".to_string()
    }
    
    fn build_skill_suggestions_system_prompt() -> String {
        "You are a career advisor. Your task is to analyze skill gaps between the user's current skills and job requirements.
CRITICAL RULES:
- Identify missing skills that are mentioned in the job description
- Assess importance (high/medium/low) based on how frequently mentioned
- Provide actionable recommendations
- Output MUST be valid JSON matching the SkillSuggestions schema".to_string()
    }
    
    fn build_job_parsing_system_prompt() -> String {
        "You are a job description parser. Your task is to extract structured information from job postings.
CRITICAL RULES:
- Extract only information that is explicitly stated in the job description
- NEVER invent or infer skills, responsibilities, or requirements that aren't mentioned
- Output MUST be valid JSON matching the ParsedJob schema
- Be thorough but accurate - only extract what you can clearly identify".to_string()
    }
}


#[async_trait]
impl AiProvider for CloudAiProvider {
    async fn generate_resume_suggestions(&self, input: ResumeInput) -> Result<ResumeSuggestions, AiProviderError> {
        let system_prompt = Self::build_resume_system_prompt();
        let user_prompt = format!(
            "Profile data:\n{}\n\nJob description:\n{}\n\nGenerate resume suggestions in JSON format.",
            serde_json::to_string_pretty(&input.profile_data).unwrap_or_default(),
            input.job_description
        );
        
        let json_response = match self.provider {
            CloudProvider::OpenAI => {
                self.call_openai(&system_prompt, &user_prompt).await?
            }
            CloudProvider::Anthropic => {
                self.call_anthropic(&system_prompt, &user_prompt).await?
            }
        };
        
        // Validate response using validation module
        validate_resume_suggestions(&json_response)
    }
    
    async fn generate_cover_letter(&self, input: CoverLetterInput) -> Result<CoverLetter, AiProviderError> {
        let system_prompt = Self::build_cover_letter_system_prompt();
        let user_prompt = format!(
            "Profile data:\n{}\n\nJob description:\n{}\n\nCompany: {}\n\nGenerate a cover letter in JSON format.",
            serde_json::to_string_pretty(&input.profile_data).unwrap_or_default(),
            input.job_description,
            input.company_name.as_deref().unwrap_or("the company")
        );
        
        let json_response = match self.provider {
            CloudProvider::OpenAI => {
                self.call_openai(&system_prompt, &user_prompt).await?
            }
            CloudProvider::Anthropic => {
                self.call_anthropic(&system_prompt, &user_prompt).await?
            }
        };
        
        // Validate response using validation module
        validate_cover_letter(&json_response)
    }
    
    async fn generate_skill_suggestions(&self, input: SkillSuggestionsInput) -> Result<SkillSuggestions, AiProviderError> {
        let system_prompt = Self::build_skill_suggestions_system_prompt();
        let user_prompt = format!(
            "Current skills: {}\n\nJob description:\n{}\n\nGenerate skill suggestions in JSON format.",
            input.current_skills.join(", "),
            input.job_description
        );
        
        let json_response = match self.provider {
            CloudProvider::OpenAI => {
                self.call_openai(&system_prompt, &user_prompt).await?
            }
            CloudProvider::Anthropic => {
                self.call_anthropic(&system_prompt, &user_prompt).await?
            }
        };
        
        // Validate response using validation module
        validate_skill_suggestions(&json_response)
    }
    
    async fn parse_job(&self, input: JobParsingInput) -> Result<ParsedJobOutput, AiProviderError> {
        let system_prompt = Self::build_job_parsing_system_prompt();
        let user_prompt = format!(
            "Job description:\n{}\n\nParse this job description and extract structured information in JSON format.",
            input.job_description
        );
        
        let json_response = match self.provider {
            CloudProvider::OpenAI => {
                self.call_openai(&system_prompt, &user_prompt).await?
            }
            CloudProvider::Anthropic => {
                self.call_anthropic(&system_prompt, &user_prompt).await?
            }
        };
        
        // Validate response using validation module
        validate_parsed_job(&json_response)
    }
    
    async fn call_llm(&self, system_prompt: Option<&str>, user_prompt: &str) -> Result<String, AiProviderError> {
        let system = system_prompt.unwrap_or("You are a helpful AI assistant. Always respond with valid JSON when requested.");
        
        let json_response = match self.provider {
            CloudProvider::OpenAI => {
                self.call_openai(system, user_prompt).await?
            }
            CloudProvider::Anthropic => {
                self.call_anthropic(system, user_prompt).await?
            }
        };
        
        // Extract text content from JSON response
        // The response might be a JSON object with a "content" field, or just a string
        if let Some(content) = json_response.get("content").and_then(|v| v.as_str()) {
            Ok(content.to_string())
        } else if let Some(text) = json_response.as_str() {
            Ok(text.to_string())
        } else {
            // Try to extract from common response formats
            if let Some(choices) = json_response.get("choices").and_then(|c| c.as_array()) {
                if let Some(first) = choices.first() {
                    if let Some(content) = first.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                        return Ok(content.to_string());
                    }
                }
            }
            
            // Fallback: serialize the whole response as JSON string
            serde_json::to_string(&json_response)
                .map_err(|e| AiProviderError::InvalidResponse(format!("Failed to serialize response: {}", e)))
        }
    }
}

