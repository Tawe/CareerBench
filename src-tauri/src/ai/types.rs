use serde::{Deserialize, Serialize};

/// Input for generating resume suggestions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResumeInput {
    pub profile_data: serde_json::Value, // User profile JSON
    pub job_description: String,
    pub options: Option<ResumeOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResumeOptions {
    pub tone: Option<String>,
    pub length: Option<String>,
    pub focus: Option<String>,
}

/// Output from resume suggestions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResumeSuggestions {
    pub summary: Option<String>,
    pub headline: Option<String>,
    pub sections: Vec<ResumeSection>,
    pub highlights: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResumeSection {
    pub title: String,
    pub items: Vec<ResumeSectionItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResumeSectionItem {
    pub heading: String,
    pub subheading: Option<String>,
    pub bullets: Vec<String>,
}

/// Input for generating cover letters
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoverLetterInput {
    pub profile_data: serde_json::Value,
    pub job_description: String,
    pub company_name: Option<String>,
    pub options: Option<CoverLetterOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoverLetterOptions {
    pub tone: Option<String>,
    pub length: Option<String>,
    pub audience: Option<String>,
}

/// Output from cover letter generation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoverLetter {
    pub subject: Option<String>,
    pub greeting: Option<String>,
    pub body_paragraphs: Vec<String>,
    pub closing: Option<String>,
    pub signature: Option<String>,
}

/// Input for skill suggestions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillSuggestionsInput {
    pub current_skills: Vec<String>,
    pub job_description: String,
    pub experience: Option<serde_json::Value>,
}

/// Output from skill suggestions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillSuggestions {
    pub missing_skills: Vec<String>,
    pub skill_gaps: Vec<SkillGap>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillGap {
    pub skill: String,
    pub importance: String, // "high", "medium", "low"
    pub reason: String,
}

/// Input for parsing job descriptions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobParsingInput {
    pub job_description: String,
    pub job_meta: Option<JobMeta>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobMeta {
    pub source: Option<String>,
    pub url: Option<String>,
}

/// Output from job parsing
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedJobOutput {
    #[serde(default)]
    pub title_suggestion: Option<String>,
    #[serde(default)]
    pub company_suggestion: Option<String>,
    #[serde(default)]
    pub seniority: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub responsibilities: Vec<String>,
    #[serde(default)]
    pub required_skills: Vec<String>,
    #[serde(default)]
    pub nice_to_have_skills: Vec<String>,
    #[serde(default)]
    pub domain_tags: Vec<String>,
    #[serde(default)]
    pub seniority_score: Option<f32>,
    #[serde(default)]
    pub remote_friendly: Option<bool>,
}

