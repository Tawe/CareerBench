use crate::ai::provider::AiProvider;
use crate::ai::types::*;
use crate::ai::errors::AiProviderError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock AI Provider for testing
/// Returns predefined responses based on registered expectations
pub struct MockProvider {
    // Store responses keyed by operation type and input hash
    parse_job_responses: Arc<Mutex<HashMap<String, ParsedJobOutput>>>,
    resume_responses: Arc<Mutex<HashMap<String, ResumeSuggestions>>>,
    cover_letter_responses: Arc<Mutex<HashMap<String, CoverLetter>>>,
    skill_suggestions_responses: Arc<Mutex<HashMap<String, SkillSuggestions>>>,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            parse_job_responses: Arc::new(Mutex::new(HashMap::new())),
            resume_responses: Arc::new(Mutex::new(HashMap::new())),
            cover_letter_responses: Arc::new(Mutex::new(HashMap::new())),
            skill_suggestions_responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a response for job parsing
    /// The key should be a hash or identifier for the job description
    #[allow(dead_code)]
    pub fn register_parse_job(&self, key: &str, response: ParsedJobOutput) {
        self.parse_job_responses.lock().unwrap().insert(key.to_string(), response);
    }

    /// Register a response for resume generation
    #[allow(dead_code)]
    pub fn register_resume(&self, key: &str, response: ResumeSuggestions) {
        self.resume_responses.lock().unwrap().insert(key.to_string(), response);
    }

    /// Register a response for cover letter generation
    #[allow(dead_code)]
    pub fn register_cover_letter(&self, key: &str, response: CoverLetter) {
        self.cover_letter_responses.lock().unwrap().insert(key.to_string(), response);
    }

    /// Register a response for skill suggestions
    #[allow(dead_code)]
    pub fn register_skill_suggestions(&self, key: &str, response: SkillSuggestions) {
        self.skill_suggestions_responses.lock().unwrap().insert(key.to_string(), response);
    }

    /// Generate a simple key from job description for matching
    pub fn job_key(job_description: &str) -> String {
        // Use first 50 chars as key for simple matching
        job_description.chars().take(50).collect()
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProvider for MockProvider {
    async fn generate_resume_suggestions(&self, input: ResumeInput) -> Result<ResumeSuggestions, AiProviderError> {
        let key = Self::job_key(&input.job_description);
        let responses = self.resume_responses.lock().unwrap();
        
        if let Some(response) = responses.get(&key) {
            Ok(response.clone())
        } else {
            // Return a default response if no mock is registered
            Ok(ResumeSuggestions {
                summary: Some("Mock resume summary".to_string()),
                headline: Some("Mock Headline".to_string()),
                sections: vec![],
                highlights: vec!["Mock highlight".to_string()],
            })
        }
    }

    async fn generate_cover_letter(&self, input: CoverLetterInput) -> Result<CoverLetter, AiProviderError> {
        let key = Self::job_key(&input.job_description);
        let responses = self.cover_letter_responses.lock().unwrap();
        
        if let Some(response) = responses.get(&key) {
            Ok(response.clone())
        } else {
            // Return a default response if no mock is registered
            Ok(CoverLetter {
                subject: Some("Mock Subject".to_string()),
                greeting: Some("Dear Hiring Manager,".to_string()),
                body_paragraphs: vec!["Mock paragraph".to_string()],
                closing: Some("Sincerely,".to_string()),
                signature: Some("Mock Signature".to_string()),
            })
        }
    }

    async fn generate_skill_suggestions(&self, input: SkillSuggestionsInput) -> Result<SkillSuggestions, AiProviderError> {
        let key = Self::job_key(&input.job_description);
        let responses = self.skill_suggestions_responses.lock().unwrap();
        
        if let Some(response) = responses.get(&key) {
            Ok(response.clone())
        } else {
            // Return a default response if no mock is registered
            Ok(SkillSuggestions {
                missing_skills: vec![],
                skill_gaps: vec![],
                recommendations: vec!["Mock recommendation".to_string()],
            })
        }
    }

    async fn parse_job(&self, input: JobParsingInput) -> Result<ParsedJobOutput, AiProviderError> {
        let key = Self::job_key(&input.job_description);
        let responses = self.parse_job_responses.lock().unwrap();
        
        if let Some(response) = responses.get(&key) {
            Ok(response.clone())
        } else {
            // Return a default response if no mock is registered
            Ok(ParsedJobOutput {
                title_suggestion: Some("Mock Job Title".to_string()),
                company_suggestion: Some("Mock Company".to_string()),
                location: Some("Mock Location".to_string()),
                seniority: Some("Mid".to_string()),
                required_skills: vec!["Mock Skill".to_string()],
                nice_to_have_skills: vec![],
                responsibilities: vec!["Mock responsibility".to_string()],
                domain_tags: vec![],
                remote_friendly: Some(false),
                summary: None,
                seniority_score: None,
            })
        }
    }
    
    async fn call_llm(&self, _system_prompt: Option<&str>, user_prompt: &str) -> Result<String, AiProviderError> {
        // For mock provider, return a simple JSON response based on prompt content
        // This is mainly for testing
        if user_prompt.contains("Extract professional profile") {
            // Return a mock profile extraction response
            Ok(r#"{
  "profile": {
    "full_name": "John Doe",
    "headline": "Software Engineer",
    "location": "San Francisco, CA"
  },
  "experience": [],
  "skills": [],
  "education": [],
  "certifications": [],
  "portfolio": []
}"#.to_string())
        } else {
            Ok(r#"{"result": "mock response"}"#.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_parse_job() {
        let provider = MockProvider::new();
        let response = ParsedJobOutput {
            title_suggestion: Some("Software Engineer".to_string()),
            company_suggestion: Some("Test Company".to_string()),
            location: Some("San Francisco, CA".to_string()),
            seniority: Some("Senior".to_string()),
            required_skills: vec!["Rust".to_string(), "TypeScript".to_string()],
            nice_to_have_skills: vec!["Python".to_string()],
            responsibilities: vec!["Build features".to_string()],
            domain_tags: vec!["Backend".to_string()],
            remote_friendly: Some(true),
            summary: None,
            seniority_score: None,
        };
        
        // Register with the key that will be generated from the job description
        let job_desc = "Software Engineer position";
        let key = MockProvider::job_key(job_desc);
        provider.register_parse_job(&key, response.clone());
        
        let input = JobParsingInput {
            job_description: job_desc.to_string(),
            job_meta: None,
        };
        
        let result = provider.parse_job(input).await;
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.title_suggestion, response.title_suggestion);
        assert_eq!(parsed.required_skills.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_provider_default_response() {
        let provider = MockProvider::new();
        
        let input = JobParsingInput {
            job_description: "Unknown job".to_string(),
            job_meta: None,
        };
        
        let result = provider.parse_job(input).await;
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.title_suggestion.is_some());
    }
}

