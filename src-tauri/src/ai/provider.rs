use crate::ai::types::*;
use crate::ai::errors::AiProviderError;

/// Main AI Provider trait (async version)
/// All AI functionality goes through this abstraction
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    /// Generate resume suggestions based on profile and job description
    async fn generate_resume_suggestions(&self, input: ResumeInput) -> Result<ResumeSuggestions, AiProviderError>;
    
    /// Generate a cover letter based on profile and job description
    async fn generate_cover_letter(&self, input: CoverLetterInput) -> Result<CoverLetter, AiProviderError>;
    
    /// Generate skill suggestions based on current skills and job requirements
    async fn generate_skill_suggestions(&self, input: SkillSuggestionsInput) -> Result<SkillSuggestions, AiProviderError>;
    
    /// Parse a job description into structured data
    async fn parse_job(&self, input: JobParsingInput) -> Result<ParsedJobOutput, AiProviderError>;
    
    /// Generic LLM call for custom prompts
    /// This allows for flexible AI operations beyond the standard methods
    /// system_prompt: Optional system message to set context
    /// user_prompt: The main user prompt/question
    async fn call_llm(&self, system_prompt: Option<&str>, user_prompt: &str) -> Result<String, AiProviderError>;
}

