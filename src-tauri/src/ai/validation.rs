//! AI Response Validation
//! 
//! This module provides runtime validation for AI responses to ensure they
//! match expected schemas before being used in the application.

use crate::ai::types::*;
use crate::ai::errors::AiProviderError;
use serde_json::Value;

/// Validates a ParsedJobOutput response
pub fn validate_parsed_job(value: &Value) -> Result<ParsedJobOutput, AiProviderError> {
    // Deserialize using serde (this validates basic structure)
    let parsed: ParsedJobOutput = serde_json::from_value(value.clone())
        .map_err(|e| AiProviderError::ValidationError(
            format!("Failed to deserialize ParsedJobOutput: {}. Response: {}", e, value)
        ))?;
    
    // Additional validation rules
    validate_parsed_job_rules(&parsed)?;
    
    Ok(parsed)
}

/// Validates business rules for ParsedJobOutput
fn validate_parsed_job_rules(parsed: &ParsedJobOutput) -> Result<(), AiProviderError> {
    // Ensure arrays are present (even if empty)
    // This is already enforced by serde, but we can add custom rules here
    
    // Validate seniority_score is in valid range if present
    if let Some(score) = parsed.seniority_score {
        if score < 0.0 || score > 1.0 {
            return Err(AiProviderError::ValidationError(
                format!("seniority_score must be between 0.0 and 1.0, got {}", score)
            ));
        }
    }
    
    Ok(())
}

/// Validates a ResumeSuggestions response
pub fn validate_resume_suggestions(value: &Value) -> Result<ResumeSuggestions, AiProviderError> {
    // Deserialize using serde
    let resume: ResumeSuggestions = serde_json::from_value(value.clone())
        .map_err(|e| AiProviderError::ValidationError(
            format!("Failed to deserialize ResumeSuggestions: {}. Response: {}", e, value)
        ))?;
    
    // Additional validation rules
    validate_resume_suggestions_rules(&resume)?;
    
    Ok(resume)
}

/// Validates business rules for ResumeSuggestions
fn validate_resume_suggestions_rules(resume: &ResumeSuggestions) -> Result<(), AiProviderError> {
    // Ensure sections array is present (already enforced by serde)
    // Validate that sections have required fields
    for (idx, section) in resume.sections.iter().enumerate() {
        if section.title.is_empty() {
            return Err(AiProviderError::ValidationError(
                format!("Resume section {} has empty title", idx)
            ));
        }
        
        for (item_idx, item) in section.items.iter().enumerate() {
            if item.heading.is_empty() {
                return Err(AiProviderError::ValidationError(
                    format!("Resume section {} item {} has empty heading", idx, item_idx)
                ));
            }
        }
    }
    
    Ok(())
}

/// Validates a CoverLetter response
pub fn validate_cover_letter(value: &Value) -> Result<CoverLetter, AiProviderError> {
    // Deserialize using serde
    let letter: CoverLetter = serde_json::from_value(value.clone())
        .map_err(|e| AiProviderError::ValidationError(
            format!("Failed to deserialize CoverLetter: {}. Response: {}", e, value)
        ))?;
    
    // Additional validation rules
    validate_cover_letter_rules(&letter)?;
    
    Ok(letter)
}

/// Validates business rules for CoverLetter
fn validate_cover_letter_rules(letter: &CoverLetter) -> Result<(), AiProviderError> {
    // Ensure body_paragraphs array is present and not empty
    if letter.body_paragraphs.is_empty() {
        return Err(AiProviderError::ValidationError(
            "Cover letter must have at least one body paragraph".to_string()
        ));
    }
    
    // Validate paragraphs are not empty
    for (idx, paragraph) in letter.body_paragraphs.iter().enumerate() {
        if paragraph.trim().is_empty() {
            return Err(AiProviderError::ValidationError(
                format!("Cover letter body paragraph {} is empty", idx)
            ));
        }
    }
    
    Ok(())
}

/// Validates a SkillSuggestions response
pub fn validate_skill_suggestions(value: &Value) -> Result<SkillSuggestions, AiProviderError> {
    // Deserialize using serde
    let skills: SkillSuggestions = serde_json::from_value(value.clone())
        .map_err(|e| AiProviderError::ValidationError(
            format!("Failed to deserialize SkillSuggestions: {}. Response: {}", e, value)
        ))?;
    
    // Additional validation rules
    validate_skill_suggestions_rules(&skills)?;
    
    Ok(skills)
}

/// Validates business rules for SkillSuggestions
fn validate_skill_suggestions_rules(skills: &SkillSuggestions) -> Result<(), AiProviderError> {
    // Validate skill gaps have required fields
    for (idx, gap) in skills.skill_gaps.iter().enumerate() {
        if gap.skill.is_empty() {
            return Err(AiProviderError::ValidationError(
                format!("Skill gap {} has empty skill name", idx)
            ));
        }
        
        // Validate importance is one of the expected values
        let importance_lower = gap.importance.to_lowercase();
        if !["high", "medium", "low"].contains(&importance_lower.as_str()) {
            return Err(AiProviderError::ValidationError(
                format!("Skill gap {} has invalid importance: {}. Must be 'high', 'medium', or 'low'", 
                    idx, gap.importance)
            ));
        }
    }
    
    Ok(())
}

/// Validates that a JSON value is a valid object (not null, array, or primitive)
#[allow(dead_code)]
pub fn validate_json_object(value: &Value) -> Result<(), AiProviderError> {
    if !value.is_object() {
        return Err(AiProviderError::ValidationError(
            format!("Expected JSON object, got: {}", value)
        ));
    }
    Ok(())
}

/// Validates that required fields are present in a JSON object
#[allow(dead_code)]
pub fn validate_required_fields(value: &Value, required_fields: &[&str]) -> Result<(), AiProviderError> {
    let obj = value.as_object()
        .ok_or_else(|| AiProviderError::ValidationError("Expected JSON object".to_string()))?;
    
    for field in required_fields {
        if !obj.contains_key(*field) {
            return Err(AiProviderError::ValidationError(
                format!("Missing required field: {}", field)
            ));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_parsed_job_valid() {
        let value = json!({
            "titleSuggestion": "Software Engineer",
            "companySuggestion": "Tech Corp",
            "seniority": "Senior",
            "location": "San Francisco, CA",
            "summary": "We are looking for...",
            "responsibilities": ["Build features", "Write tests"],
            "requiredSkills": ["Rust", "TypeScript"],
            "niceToHaveSkills": ["Python"],
            "domainTags": ["backend", "fullstack"],
            "seniorityScore": 0.75,
            "remoteFriendly": true
        });
        
        let result = validate_parsed_job(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_parsed_job_invalid_score() {
        let value = json!({
            "responsibilities": [],
            "requiredSkills": [],
            "niceToHaveSkills": [],
            "domainTags": [],
            "seniorityScore": 1.5  // Invalid: > 1.0
        });
        
        let result = validate_parsed_job(&value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("seniority_score"));
    }

    #[test]
    fn test_validate_resume_suggestions_valid() {
        let value = json!({
            "summary": "Experienced engineer",
            "headline": "Senior Software Engineer",
            "sections": [
                {
                    "title": "Experience",
                    "items": [
                        {
                            "heading": "Software Engineer at Tech Corp",
                            "subheading": "2020-2024",
                            "bullets": ["Built features", "Wrote tests"]
                        }
                    ]
                }
            ],
            "highlights": ["10+ years experience"]
        });
        
        let result = validate_resume_suggestions(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_resume_suggestions_empty_title() {
        let value = json!({
            "sections": [
                {
                    "title": "",  // Invalid: empty title
                    "items": []
                }
            ],
            "highlights": []
        });
        
        let result = validate_resume_suggestions(&value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty title"));
    }

    #[test]
    fn test_validate_cover_letter_valid() {
        let value = json!({
            "subject": "Application for Software Engineer",
            "greeting": "Dear Hiring Manager",
            "bodyParagraphs": ["I am writing to apply...", "I have 10 years..."],
            "closing": "Sincerely",
            "signature": "John Doe"
        });
        
        let result = validate_cover_letter(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_cover_letter_empty_paragraphs() {
        let value = json!({
            "bodyParagraphs": []  // Invalid: must have at least one paragraph
        });
        
        let result = validate_cover_letter(&value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one body paragraph"));
    }

    #[test]
    fn test_validate_skill_suggestions_valid() {
        let value = json!({
            "missingSkills": ["Rust", "TypeScript"],
            "skillGaps": [
                {
                    "skill": "Rust",
                    "importance": "high",
                    "reason": "Frequently mentioned"
                }
            ],
            "recommendations": ["Learn Rust", "Practice TypeScript"]
        });
        
        let result = validate_skill_suggestions(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_skill_suggestions_invalid_importance() {
        let value = json!({
            "missingSkills": [],
            "skillGaps": [
                {
                    "skill": "Rust",
                    "importance": "critical",  // Invalid: not high/medium/low
                    "reason": "Frequently mentioned"
                }
            ],
            "recommendations": []
        });
        
        let result = validate_skill_suggestions(&value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid importance"));
    }
}

