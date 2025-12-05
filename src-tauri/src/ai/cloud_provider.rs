use crate::ai::provider::AiProvider;
use crate::ai::types::*;
use crate::ai::errors::AiProviderError;
use crate::ai::settings::CloudProvider;
use async_trait::async_trait;
use serde_json::{json, Value};
use reqwest::Client;

/// Cloud AI Provider
/// Supports multiple cloud providers (OpenAI, Anthropic, etc.)
pub struct CloudAiProvider {
    provider: CloudProvider,
    api_key: String,
    model_name: String,
    client: Client,
}

impl CloudAiProvider {
    pub fn new(provider: CloudProvider, api_key: String, model_name: String) -> Self {
        Self {
            provider,
            api_key,
            model_name,
            client: Client::new(),
        }
    }
    
    async fn call_openai(&self, system_prompt: &str, user_prompt: &str) -> Result<Value, AiProviderError> {
        let url = "https://api.openai.com/v1/chat/completions";
        
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": self.model_name,
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
        
        match self.provider {
            CloudProvider::OpenAI => {
                let json_response = self.call_openai(&system_prompt, &user_prompt).await?;
                serde_json::from_value(json_response)
                    .map_err(|e| AiProviderError::ValidationError(format!("Failed to deserialize resume suggestions: {}", e)))
            }
            CloudProvider::Anthropic => {
                // TODO: Implement Anthropic API
                Err(AiProviderError::Unknown("Anthropic provider not yet implemented".to_string()))
            }
        }
    }
    
    async fn generate_cover_letter(&self, input: CoverLetterInput) -> Result<CoverLetter, AiProviderError> {
        let system_prompt = Self::build_cover_letter_system_prompt();
        let user_prompt = format!(
            "Profile data:\n{}\n\nJob description:\n{}\n\nCompany: {}\n\nGenerate a cover letter in JSON format.",
            serde_json::to_string_pretty(&input.profile_data).unwrap_or_default(),
            input.job_description,
            input.company_name.as_deref().unwrap_or("the company")
        );
        
        match self.provider {
            CloudProvider::OpenAI => {
                let json_response = self.call_openai(&system_prompt, &user_prompt).await?;
                serde_json::from_value(json_response)
                    .map_err(|e| AiProviderError::ValidationError(format!("Failed to deserialize cover letter: {}", e)))
            }
            CloudProvider::Anthropic => {
                Err(AiProviderError::Unknown("Anthropic provider not yet implemented".to_string()))
            }
        }
    }
    
    async fn generate_skill_suggestions(&self, input: SkillSuggestionsInput) -> Result<SkillSuggestions, AiProviderError> {
        let system_prompt = Self::build_skill_suggestions_system_prompt();
        let user_prompt = format!(
            "Current skills: {}\n\nJob description:\n{}\n\nGenerate skill suggestions in JSON format.",
            input.current_skills.join(", "),
            input.job_description
        );
        
        match self.provider {
            CloudProvider::OpenAI => {
                let json_response = self.call_openai(&system_prompt, &user_prompt).await?;
                serde_json::from_value(json_response)
                    .map_err(|e| AiProviderError::ValidationError(format!("Failed to deserialize skill suggestions: {}", e)))
            }
            CloudProvider::Anthropic => {
                Err(AiProviderError::Unknown("Anthropic provider not yet implemented".to_string()))
            }
        }
    }
    
    async fn parse_job(&self, input: JobParsingInput) -> Result<ParsedJobOutput, AiProviderError> {
        let system_prompt = Self::build_job_parsing_system_prompt();
        let user_prompt = format!(
            "Job description:\n{}\n\nParse this job description and extract structured information in JSON format.",
            input.job_description
        );
        
        match self.provider {
            CloudProvider::OpenAI => {
                let json_response = self.call_openai(&system_prompt, &user_prompt).await?;
                serde_json::from_value(json_response)
                    .map_err(|e| AiProviderError::ValidationError(format!("Failed to deserialize parsed job: {}", e)))
            }
            CloudProvider::Anthropic => {
                Err(AiProviderError::Unknown("Anthropic provider not yet implemented".to_string()))
            }
        }
    }
}

