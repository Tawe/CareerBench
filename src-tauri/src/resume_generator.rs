// Resume Generation Pipeline
// This module implements a deterministic, efficient resume generation pipeline
// that breaks down the task into small, focused AI calls and code-based preprocessing.

use serde::{Deserialize, Serialize};
use crate::commands::{UserProfileData, Experience, Skill, ParsedJob};

/// Job Description Summary - extracted from JD via small AI call
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobDescriptionSummary {
    pub role_title: Option<String>,
    pub seniority: Option<String>,
    pub must_have_skills: Vec<String>,
    pub nice_to_have_skills: Vec<String>,
    pub top_responsibilities: Vec<String>,
    pub tools_tech: Vec<String>,
    pub tone: Option<String>, // e.g., "technical", "leadership", "collaborative"
}

/// Relevance score for a role or bullet point
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelevanceScore {
    pub score: f64,
    pub primary_keywords: Vec<String>,
    pub matched_skills: Vec<String>,
    pub matched_responsibilities: Vec<String>,
}

/// Mapped experience role with relevance scoring
#[derive(Debug, Serialize, Deserialize)]
pub struct MappedExperience {
    pub experience: Experience,
    pub relevance_score: RelevanceScore,
    pub selected_bullets: Vec<MappedBullet>,
}

/// Mapped bullet point with relevance info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MappedBullet {
    pub id: String, // e.g., "exp_3_b1"
    pub original_text: String,
    pub relevance_score: f64,
    pub matched_keywords: Vec<String>,
}

/// Rewritten bullet point
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RewrittenBullet {
    pub id: String,
    pub new_text: String,
}

/// Preprocessing: Extract skills, tools, and keywords from text
pub fn extract_skills_from_text(text: &str) -> Vec<String> {
    // Simple keyword extraction - can be enhanced with NLP
    let common_skills = vec![
        "TypeScript", "JavaScript", "Python", "Rust", "Go", "Java", "C++", "C#",
        "React", "Vue", "Angular", "Node.js", "Express", "Django", "Flask",
        "PostgreSQL", "MySQL", "MongoDB", "Redis", "AWS", "GCP", "Azure",
        "Docker", "Kubernetes", "CI/CD", "Git", "Linux", "Agile", "Scrum",
        "Machine Learning", "AI", "Data Science", "DevOps", "SRE",
        "Microservices", "REST", "GraphQL", "gRPC", "System Design",
        "Leadership", "Mentoring", "Hiring", "Team Building", "OKRs",
    ];
    
    let text_lower = text.to_lowercase();
    let mut found_skills = Vec::new();
    
    for skill in common_skills {
        if text_lower.contains(&skill.to_lowercase()) {
            found_skills.push(skill.to_string());
        }
    }
    
    found_skills
}

/// Extract tools and technologies from text
#[allow(dead_code)]
pub fn extract_tools_from_text(text: &str) -> Vec<String> {
    extract_skills_from_text(text) // For now, same logic
}

/// Extract responsibilities/key phrases from text
#[allow(dead_code)]
pub fn extract_responsibilities_from_text(text: &str) -> Vec<String> {
    let responsibility_patterns = vec![
        "led", "managed", "designed", "implemented", "built", "developed",
        "architected", "optimized", "improved", "increased", "reduced",
        "scaled", "mentored", "hired", "coached", "delivered", "shipped",
    ];
    
    let text_lower = text.to_lowercase();
    let mut found = Vec::new();
    
    for pattern in responsibility_patterns {
        if text_lower.contains(pattern) {
            found.push(pattern.to_string());
        }
    }
    
    found
}

/// Calculate simple keyword overlap score between two text strings
pub fn calculate_keyword_overlap(text1: &str, text2: &str) -> f64 {
    let words1: std::collections::HashSet<String> = text1
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    
    let words2: std::collections::HashSet<String> = text2
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    
    let intersection: Vec<_> = words1.intersection(&words2).collect();
    let union: Vec<_> = words1.union(&words2).collect();
    
    if union.is_empty() {
        return 0.0;
    }
    
    intersection.len() as f64 / union.len() as f64
}

/// Calculate relevance score for an experience role against JD summary
pub fn calculate_role_relevance(
    experience: &Experience,
    jd_summary: &JobDescriptionSummary,
) -> RelevanceScore {
    let mut score = 0.0;
    let mut primary_keywords = Vec::new();
    let mut matched_skills = Vec::new();
    let mut matched_responsibilities = Vec::new();
    
    // Build combined text from experience
    let mut exp_text = String::new();
    if let Some(desc) = &experience.description {
        exp_text.push_str(desc);
        exp_text.push_str(" ");
    }
    if let Some(achievements) = &experience.achievements {
        exp_text.push_str(achievements);
        exp_text.push_str(" ");
    }
    if let Some(tech_stack) = &experience.tech_stack {
        exp_text.push_str(tech_stack);
    }
    
    // Check title match
    let title_lower = experience.title.to_lowercase();
    if let Some(ref jd_title) = jd_summary.role_title {
        if title_lower.contains(&jd_title.to_lowercase()) {
            score += 0.3;
            primary_keywords.push(jd_title.clone());
        }
    }
    
    // Check skill matches
    let exp_skills = extract_skills_from_text(&exp_text);
    for skill in &jd_summary.must_have_skills {
        if exp_skills.iter().any(|s| s.eq_ignore_ascii_case(skill)) {
            score += 0.2;
            matched_skills.push(skill.clone());
            primary_keywords.push(skill.clone());
        }
    }
    for skill in &jd_summary.nice_to_have_skills {
        if exp_skills.iter().any(|s| s.eq_ignore_ascii_case(skill)) {
            score += 0.1;
            if !matched_skills.contains(skill) {
                matched_skills.push(skill.clone());
            }
        }
    }
    
    // Check responsibility matches
    for resp in &jd_summary.top_responsibilities {
        let overlap = calculate_keyword_overlap(&exp_text, resp);
        if overlap > 0.1 {
            score += overlap * 0.15;
            matched_responsibilities.push(resp.clone());
        }
    }
    
    // Normalize score to 0-1 range
    score = score.min(1.0);
    
    RelevanceScore {
        score,
        primary_keywords,
        matched_skills,
        matched_responsibilities,
    }
}

/// Calculate relevance score for a bullet point
pub fn calculate_bullet_relevance(
    bullet_text: &str,
    jd_summary: &JobDescriptionSummary,
) -> f64 {
    let mut score = 0.0;
    
    // Check skill matches
    let bullet_skills = extract_skills_from_text(bullet_text);
    for skill in &jd_summary.must_have_skills {
        if bullet_skills.iter().any(|s| s.eq_ignore_ascii_case(skill)) {
            score += 0.3;
        }
    }
    for skill in &jd_summary.nice_to_have_skills {
        if bullet_skills.iter().any(|s| s.eq_ignore_ascii_case(skill)) {
            score += 0.15;
        }
    }
    
    // Check responsibility matches
    for resp in &jd_summary.top_responsibilities {
        let overlap = calculate_keyword_overlap(bullet_text, resp);
        score += overlap * 0.2;
    }
    
    score.min(1.0)
}

/// Select top N roles by relevance
pub fn select_top_roles(
    experiences: &[Experience],
    jd_summary: &JobDescriptionSummary,
    top_n: usize,
) -> Vec<MappedExperience> {
    let mut mapped: Vec<MappedExperience> = experiences
        .iter()
        .map(|exp| {
            let relevance = calculate_role_relevance(exp, jd_summary);
            MappedExperience {
                experience: exp.clone(),
                relevance_score: relevance,
                selected_bullets: Vec::new(),
            }
        })
        .collect();
    
    // Sort by relevance score (descending)
    mapped.sort_by(|a, b| {
        b.relevance_score.score.partial_cmp(&a.relevance_score.score).unwrap()
    });
    
    // Take top N
    mapped.into_iter().take(top_n).collect()
}

/// Select top bullets for a role
pub fn select_top_bullets_for_role(
    experience: &Experience,
    jd_summary: &JobDescriptionSummary,
    top_n: usize,
) -> Vec<MappedBullet> {
    let mut bullets = Vec::new();
    
    // Extract bullets from description and achievements
    if let Some(desc) = &experience.description {
        for (idx, line) in desc.split('\n').enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && (trimmed.starts_with('-') || trimmed.starts_with('•')) {
                let bullet_text = trimmed.trim_start_matches('-').trim_start_matches('•').trim();
                if !bullet_text.is_empty() {
                    let id = format!("exp_{}_b{}", experience.id.unwrap_or(0), idx);
                    let relevance = calculate_bullet_relevance(bullet_text, jd_summary);
                    let matched = extract_skills_from_text(bullet_text);
                    bullets.push(MappedBullet {
                        id,
                        original_text: bullet_text.to_string(),
                        relevance_score: relevance,
                        matched_keywords: matched,
                    });
                }
            }
        }
    }
    
    if let Some(achievements) = &experience.achievements {
        let start_idx = bullets.len();
        for (idx, line) in achievements.split('\n').enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let bullet_text = if trimmed.starts_with('-') || trimmed.starts_with('•') {
                    trimmed.trim_start_matches('-').trim_start_matches('•').trim()
                } else {
                    trimmed
                };
                if !bullet_text.is_empty() {
                    let id = format!("exp_{}_ach{}", experience.id.unwrap_or(0), start_idx + idx);
                    let relevance = calculate_bullet_relevance(bullet_text, jd_summary);
                    let matched = extract_skills_from_text(bullet_text);
                    bullets.push(MappedBullet {
                        id,
                        original_text: bullet_text.to_string(),
                        relevance_score: relevance,
                        matched_keywords: matched,
                    });
                }
            }
        }
    }
    
    // If no structured bullets found, create bullets from description/achievements
    if bullets.is_empty() {
        if let Some(desc) = &experience.description {
            let id = format!("exp_{}_desc", experience.id.unwrap_or(0));
            let relevance = calculate_bullet_relevance(desc, jd_summary);
            bullets.push(MappedBullet {
                id,
                original_text: desc.clone(),
                relevance_score: relevance,
                matched_keywords: extract_skills_from_text(desc),
            });
        }
        if let Some(achievements) = &experience.achievements {
            let id = format!("exp_{}_ach", experience.id.unwrap_or(0));
            let relevance = calculate_bullet_relevance(achievements, jd_summary);
            bullets.push(MappedBullet {
                id,
                original_text: achievements.clone(),
                relevance_score: relevance,
                matched_keywords: extract_skills_from_text(achievements),
            });
        }
    }
    
    // Sort by relevance and take top N
    bullets.sort_by(|a, b| {
        b.relevance_score.partial_cmp(&a.relevance_score).unwrap()
    });
    
    bullets.into_iter().take(top_n).collect()
}

/// Select top skills to highlight based on JD requirements
pub fn select_top_skills(
    user_skills: &[Skill],
    jd_summary: &JobDescriptionSummary,
    top_n: usize,
) -> Vec<String> {
    let mut scored_skills: Vec<(String, f64)> = Vec::new();
    
    for skill in user_skills {
        let mut score = 0.0;
        let skill_name_lower = skill.name.to_lowercase();
        
        // Check must-have matches
        for jd_skill in &jd_summary.must_have_skills {
            if skill_name_lower.contains(&jd_skill.to_lowercase()) ||
               jd_skill.to_lowercase().contains(&skill_name_lower) {
                score += 2.0;
            }
        }
        
        // Check nice-to-have matches
        for jd_skill in &jd_summary.nice_to_have_skills {
            if skill_name_lower.contains(&jd_skill.to_lowercase()) ||
               jd_skill.to_lowercase().contains(&skill_name_lower) {
                score += 1.0;
            }
        }
        
        // Boost priority skills
        if skill.priority.as_deref() == Some("Core") {
            score += 0.5;
        }
        
        if score > 0.0 {
            scored_skills.push((skill.name.clone(), score));
        }
    }
    
    // Sort by score and take top N
    scored_skills.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    scored_skills.into_iter().take(top_n).map(|(name, _)| name).collect()
}

// ============================================================================
// AI Helper Functions for Small, Focused Calls
// ============================================================================

use crate::ai::resolver::ResolvedProvider;

/// Step 1: Summarize job description (small AI call ~300-500 tokens input, ~150-250 tokens output)
pub async fn summarize_job_description(
    job_description: &str,
    parsed_job: Option<&ParsedJob>,
) -> Result<JobDescriptionSummary, String> {
    use crate::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_JOB_PARSE_DAYS};
    use crate::db::get_connection;
    use chrono::Utc;
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();
    
    // Build canonical request payload
    let request_payload = serde_json::json!({
        "jobDescription": job_description,
        "parsedJob": parsed_job,
    });
    
    // Check cache
    let input_hash = compute_input_hash(&request_payload)
        .map_err(|e| format!("Failed to compute hash: {}", e))?;
    
    if let Some(cached_entry) = ai_cache_get(&conn, "jd_summary", &input_hash, &now)
        .map_err(|e| format!("Cache lookup error: {}", e))? {
        let summary: JobDescriptionSummary = serde_json::from_value(cached_entry.response_payload)
            .map_err(|e| format!("Failed to deserialize cached summary: {}", e))?;
        return Ok(summary);
    }
    
    // Cache miss - call AI provider
    let provider = ResolvedProvider::resolve()
        .map_err(|e| format!("Failed to resolve provider: {}", e))?;
    
    // Build prompt for JD summary (small, focused)
    // Note: Currently using parse_job as a base - can be enhanced with dedicated JD summary call
    // let _system_prompt = "You are a job description analyzer...";
    // let _user_prompt = format!("Job description:\n{}\n\nExtract the key information as JSON.", job_description);
    
    // Call the provider's parse_job method and convert to our summary format
    let parsing_input = crate::ai::types::JobParsingInput {
        job_description: job_description.to_string(),
        job_meta: None,
    };
    
    let parsed = provider.as_provider()
        .parse_job(parsing_input)
        .await
        .map_err(|e| format!("AI parsing failed: {}", e))?;
    
    // Convert ParsedJobOutput to JobDescriptionSummary
    let summary = JobDescriptionSummary {
        role_title: parsed.title_suggestion,
        seniority: parsed.seniority,
        must_have_skills: parsed.required_skills,
        nice_to_have_skills: parsed.nice_to_have_skills,
        top_responsibilities: parsed.responsibilities,
        tools_tech: parsed.domain_tags, // Using domain_tags as tools/tech
        tone: None, // Can be enhanced later
    };
    
    // Store in cache
    let response_payload = serde_json::to_value(&summary)
        .map_err(|e| format!("Failed to serialize summary: {}", e))?;
    
    let model_name = crate::ai::settings::load_ai_settings()
        .ok()
        .and_then(|s| s.model_name)
        .unwrap_or_else(|| "unknown-model".to_string());
    
    ai_cache_put(
        &conn,
        "jd_summary",
        &input_hash,
        &model_name,
        &request_payload,
        &response_payload,
        Some(CACHE_TTL_JOB_PARSE_DAYS),
        &now,
    )
    .map_err(|e| format!("Failed to cache summary: {}", e))?;
    
    Ok(summary)
}

/// Step 2: Rewrite bullets for a role (small AI call ~300-600 tokens input, ~100-200 tokens output)
pub async fn rewrite_bullets_for_role(
    role_title: &str,
    company: &str,
    bullets: &[MappedBullet],
    jd_summary: &JobDescriptionSummary,
) -> Result<Vec<RewrittenBullet>, String> {
    use crate::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_RESUME_DAYS};
    use crate::db::get_connection;
    use chrono::Utc;
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();
    
    // Build canonical request payload
    let request_payload = serde_json::json!({
        "roleTitle": role_title,
        "company": company,
        "bullets": bullets.iter().map(|b| serde_json::json!({
            "id": b.id,
            "text": b.original_text
        })).collect::<Vec<_>>(),
        "jdSummary": jd_summary,
    });
    
    // Check cache
    let input_hash = compute_input_hash(&request_payload)
        .map_err(|e| format!("Failed to compute hash: {}", e))?;
    
    if let Some(cached_entry) = ai_cache_get(&conn, "bullet_rewrite", &input_hash, &now)
        .map_err(|e| format!("Cache lookup error: {}", e))? {
        let rewritten: Vec<RewrittenBullet> = serde_json::from_value(cached_entry.response_payload)
            .map_err(|e| format!("Failed to deserialize cached bullets: {}", e))?;
        return Ok(rewritten);
    }
    
    // Cache miss - call AI provider
    // Note: Currently returning original bullets as placeholder
    // TODO: Implement actual bullet rewriting with AI provider
    // let _provider = ResolvedProvider::resolve()...;
    // let _system_prompt = "You are a resume bullet point writer...";
    // let _user_prompt = format!("Role: {} at {}...", role_title, company);
    let rewritten: Vec<RewrittenBullet> = bullets.iter()
        .map(|b| RewrittenBullet {
            id: b.id.clone(),
            new_text: b.original_text.clone(), // Placeholder - will be enhanced with actual AI call
        })
        .collect();
    
    // Store in cache
    let response_payload = serde_json::to_value(&rewritten)
        .map_err(|e| format!("Failed to serialize rewritten bullets: {}", e))?;
    
    let model_name = crate::ai::settings::load_ai_settings()
        .ok()
        .and_then(|s| s.model_name)
        .unwrap_or_else(|| "unknown-model".to_string());
    
    ai_cache_put(
        &conn,
        "bullet_rewrite",
        &input_hash,
        &model_name,
        &request_payload,
        &response_payload,
        Some(CACHE_TTL_RESUME_DAYS),
        &now,
    )
    .map_err(|e| format!("Failed to cache rewritten bullets: {}", e))?;
    
    Ok(rewritten)
}

/// Step 3: Generate professional summary (optional small AI call)
pub async fn generate_professional_summary(
    profile_data: &UserProfileData,
    jd_summary: &JobDescriptionSummary,
) -> Result<String, String> {
    use crate::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_RESUME_DAYS};
    use crate::db::get_connection;
    use chrono::Utc;
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();
    
    // Build canonical request payload
    let request_payload = serde_json::json!({
        "profile": profile_data.profile,
        "jdSummary": jd_summary,
    });
    
    // Check cache
    let input_hash = compute_input_hash(&request_payload)
        .map_err(|e| format!("Failed to compute hash: {}", e))?;
    
    if let Some(cached_entry) = ai_cache_get(&conn, "professional_summary", &input_hash, &now)
        .map_err(|e| format!("Cache lookup error: {}", e))? {
        let summary: String = serde_json::from_value(cached_entry.response_payload)
            .map_err(|e| format!("Failed to deserialize cached summary: {}", e))?;
        return Ok(summary);
    }
    
    // Cache miss - use existing profile summary or generate a simple one
    // For MVP, we'll use the existing summary if available
    if let Some(profile) = &profile_data.profile {
        if let Some(summary) = &profile.summary {
            return Ok(summary.clone());
        }
    }
    
    // Generate a simple summary from profile data
    let mut summary_parts = Vec::new();
    if let Some(profile) = &profile_data.profile {
        if let Some(title) = &profile.current_role_title {
            summary_parts.push(format!("Experienced {}", title));
        }
    }
    
    if !profile_data.experience.is_empty() {
        let latest = &profile_data.experience[0];
        summary_parts.push(format!("Currently {} at {}", latest.title, latest.company));
    }
    
    let summary = if summary_parts.is_empty() {
        "Experienced professional seeking new opportunities.".to_string()
    } else {
        summary_parts.join(". ") + "."
    };
    
    // Store in cache
    let response_payload = serde_json::to_value(&summary)
        .map_err(|e| format!("Failed to serialize summary: {}", e))?;
    
    let model_name = crate::ai::settings::load_ai_settings()
        .ok()
        .and_then(|s| s.model_name)
        .unwrap_or_else(|| "unknown-model".to_string());
    
    ai_cache_put(
        &conn,
        "professional_summary",
        &input_hash,
        &model_name,
        &request_payload,
        &response_payload,
        Some(CACHE_TTL_RESUME_DAYS),
        &now,
    )
    .map_err(|e| format!("Failed to cache summary: {}", e))?;
    
    Ok(summary)
}

