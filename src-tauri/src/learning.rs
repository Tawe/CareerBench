//! Learning Plan Generator module for skill gap analysis and learning track generation

use crate::db::get_connection;
use crate::errors::CareerBenchError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillGap {
    pub skill: String,
    pub frequency: i64,
    pub priority: String, // "high", "medium", "low"
    pub user_has_skill: bool,
    pub user_rating: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningPlan {
    pub id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub target_job_id: Option<i64>,
    pub skill_gaps: Option<String>, // JSON string of skill gaps
    pub estimated_duration_days: Option<i32>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningTrack {
    pub id: Option<i64>,
    pub learning_plan_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub skill_focus: Option<String>,
    pub order_index: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningTask {
    pub id: Option<i64>,
    pub learning_track_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub task_type: String,
    pub resource_url: Option<String>,
    pub estimated_hours: Option<i32>,
    pub completed: bool,
    pub completed_at: Option<String>,
    pub due_date: Option<String>,
    pub order_index: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningResource {
    pub id: Option<i64>,
    pub learning_task_id: Option<i64>,
    pub title: String,
    pub url: Option<String>,
    pub resource_type: String,
    pub description: Option<String>,
    pub created_at: String,
}

/// Analyze skill gaps by comparing user skills to job requirements
pub fn analyze_skill_gaps(
    job_id: Option<i64>,
    include_all_jobs: bool,
) -> Result<Vec<SkillGap>, CareerBenchError> {
    let conn = get_connection()?;

    // Get user skills
    let mut user_skills = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT name, self_rating FROM skills WHERE user_profile_id = 1"
    )?;

    let skill_rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<i32>>(1)?))
    })?;

    for row_result in skill_rows {
        let (name, rating) = row_result?;
        user_skills.insert(name.to_lowercase(), rating);
    }

    // Get required skills from jobs
    let query = if let Some(jid) = job_id {
        format!(
            "SELECT parsed_json FROM jobs WHERE id = {} AND parsed_json IS NOT NULL",
            jid
        )
    } else if include_all_jobs {
        "SELECT parsed_json FROM jobs WHERE parsed_json IS NOT NULL AND is_active = 1".to_string()
    } else {
        return Ok(Vec::new());
    };

    let mut skill_frequency: HashMap<String, i64> = HashMap::new();

    let mut stmt = conn.prepare(&query)?;
    let job_rows = stmt.query_map([], |row| {
        row.get::<_, Option<String>>(0)
    })?;

    for row_result in job_rows {
        if let Some(json_str) = row_result? {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                // Extract required skills
                if let Some(required) = parsed.get("required_skills").and_then(|v| v.as_array()) {
                    for skill in required {
                        if let Some(skill_str) = skill.as_str() {
                            *skill_frequency.entry(skill_str.to_lowercase()).or_insert(0) += 1;
                        }
                    }
                }
                // Extract nice-to-have skills (weighted less)
                if let Some(nice_to_have) = parsed.get("nice_to_have_skills").and_then(|v| v.as_array()) {
                    for skill in nice_to_have {
                        if let Some(skill_str) = skill.as_str() {
                            *skill_frequency.entry(skill_str.to_lowercase()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    // Convert to SkillGap list
    let mut gaps: Vec<SkillGap> = skill_frequency
        .into_iter()
        .map(|(skill, frequency)| {
            let user_has = user_skills.contains_key(&skill);
            let user_rating = user_skills.get(&skill).copied().flatten();
            
            // Determine priority based on frequency
            let priority = if frequency >= 5 {
                "high"
            } else if frequency >= 2 {
                "medium"
            } else {
                "low"
            };

            SkillGap {
                skill: skill.split_whitespace().map(|s| {
                    let mut chars = s.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                }).collect::<Vec<_>>().join(" "),
                frequency,
                priority: priority.to_string(),
                user_has_skill: user_has,
                user_rating,
            }
        })
        .collect();

    // Sort by priority and frequency
    gaps.sort_by(|a, b| {
        let priority_order = |p: &str| match p {
            "high" => 3,
            "medium" => 2,
            "low" => 1,
            _ => 0,
        };
        let order = priority_order(&b.priority).cmp(&priority_order(&a.priority));
        if order == std::cmp::Ordering::Equal {
            b.frequency.cmp(&a.frequency)
        } else {
            order
        }
    });

    Ok(gaps)
}

/// Create a learning plan from skill gaps
pub fn create_learning_plan(
    title: String,
    description: Option<String>,
    target_job_id: Option<i64>,
    skill_gaps: &[SkillGap],
    estimated_duration_days: Option<i32>,
) -> Result<i64, CareerBenchError> {
    let conn = get_connection()?;

    let skill_gaps_json = serde_json::to_string(skill_gaps)
        .map_err(|e| CareerBenchError::Configuration(crate::errors::ConfigurationError::Other(
            format!("Failed to serialize skill gaps: {}", e)
        )))?;

    conn.execute(
        "INSERT INTO learning_plans 
         (title, description, target_job_id, skill_gaps, estimated_duration_days, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 'active', datetime('now'), datetime('now'))",
        rusqlite::params![
            title,
            description,
            target_job_id,
            skill_gaps_json,
            estimated_duration_days
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get all learning plans
pub fn get_learning_plans(
    status: Option<&str>,
) -> Result<Vec<LearningPlan>, CareerBenchError> {
    let conn = get_connection()?;

    let query = if let Some(s) = status {
        format!("SELECT id, title, description, target_job_id, skill_gaps, estimated_duration_days, status, created_at, updated_at FROM learning_plans WHERE status = '{}' ORDER BY created_at DESC", s)
    } else {
        "SELECT id, title, description, target_job_id, skill_gaps, estimated_duration_days, status, created_at, updated_at FROM learning_plans ORDER BY created_at DESC".to_string()
    };

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map([], |row| {
        Ok(LearningPlan {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            target_job_id: row.get(3)?,
            skill_gaps: row.get(4)?,
            estimated_duration_days: row.get(5)?,
            status: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    })?;

    let mut plans = Vec::new();
    for row_result in rows {
        plans.push(row_result?);
    }

    Ok(plans)
}

/// Get learning tracks for a plan
pub fn get_learning_tracks(
    learning_plan_id: i64,
) -> Result<Vec<LearningTrack>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, learning_plan_id, title, description, skill_focus, order_index, created_at
         FROM learning_tracks
         WHERE learning_plan_id = ?
         ORDER BY order_index ASC, created_at ASC"
    )?;

    let rows = stmt.query_map([learning_plan_id], |row| {
        Ok(LearningTrack {
            id: row.get(0)?,
            learning_plan_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            skill_focus: row.get(4)?,
            order_index: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    let mut tracks = Vec::new();
    for row_result in rows {
        tracks.push(row_result?);
    }

    Ok(tracks)
}

/// Get learning tasks for a track
pub fn get_learning_tasks(
    learning_track_id: i64,
) -> Result<Vec<LearningTask>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, learning_track_id, title, description, task_type, resource_url, 
         estimated_hours, completed, completed_at, due_date, order_index, created_at, updated_at
         FROM learning_tasks
         WHERE learning_track_id = ?
         ORDER BY order_index ASC, created_at ASC"
    )?;

    let rows = stmt.query_map([learning_track_id], |row| {
        Ok(LearningTask {
            id: row.get(0)?,
            learning_track_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            task_type: row.get(4)?,
            resource_url: row.get(5)?,
            estimated_hours: row.get(6)?,
            completed: row.get::<_, i64>(7)? != 0,
            completed_at: row.get(8)?,
            due_date: row.get(9)?,
            order_index: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    let mut tasks = Vec::new();
    for row_result in rows {
        tasks.push(row_result?);
    }

    Ok(tasks)
}

/// Create a learning track
pub fn create_learning_track(
    learning_plan_id: i64,
    title: String,
    description: Option<String>,
    skill_focus: Option<String>,
    order_index: i32,
) -> Result<i64, CareerBenchError> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO learning_tracks 
         (learning_plan_id, title, description, skill_focus, order_index, created_at)
         VALUES (?, ?, ?, ?, ?, datetime('now'))",
        rusqlite::params![learning_plan_id, title, description, skill_focus, order_index],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Create a learning task
pub fn create_learning_task(
    learning_track_id: i64,
    title: String,
    description: Option<String>,
    task_type: String,
    resource_url: Option<String>,
    estimated_hours: Option<i32>,
    due_date: Option<String>,
    order_index: i32,
) -> Result<i64, CareerBenchError> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO learning_tasks 
         (learning_track_id, title, description, task_type, resource_url, estimated_hours, 
          completed, due_date, order_index, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?, datetime('now'), datetime('now'))",
        rusqlite::params![
            learning_track_id,
            title,
            description,
            task_type,
            resource_url,
            estimated_hours,
            due_date,
            order_index
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Mark a learning task as completed
pub fn complete_learning_task(
    task_id: i64,
    completed: bool,
) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;

    if completed {
        conn.execute(
            "UPDATE learning_tasks 
             SET completed = 1, completed_at = datetime('now'), updated_at = datetime('now')
             WHERE id = ?",
            [task_id],
        )?;
    } else {
        conn.execute(
            "UPDATE learning_tasks 
             SET completed = 0, completed_at = NULL, updated_at = datetime('now')
             WHERE id = ?",
            [task_id],
        )?;
    }

    Ok(())
}

/// Add a learning resource to a task
pub fn add_learning_resource(
    learning_task_id: Option<i64>,
    title: String,
    url: Option<String>,
    resource_type: String,
    description: Option<String>,
) -> Result<i64, CareerBenchError> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO learning_resources 
         (learning_task_id, title, url, resource_type, description, created_at)
         VALUES (?, ?, ?, ?, ?, datetime('now'))",
        rusqlite::params![learning_task_id, title, url, resource_type, description],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get learning resources for a task
pub fn get_learning_resources(
    learning_task_id: i64,
) -> Result<Vec<LearningResource>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, learning_task_id, title, url, resource_type, description, created_at
         FROM learning_resources
         WHERE learning_task_id = ?
         ORDER BY created_at ASC"
    )?;

    let rows = stmt.query_map([learning_task_id], |row| {
        Ok(LearningResource {
            id: row.get(0)?,
            learning_task_id: row.get(1)?,
            title: row.get(2)?,
            url: row.get(3)?,
            resource_type: row.get(4)?,
            description: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    let mut resources = Vec::new();
    for row_result in rows {
        resources.push(row_result?);
    }

    Ok(resources)
}

/// Delete a learning plan
pub fn delete_learning_plan(plan_id: i64) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;
    conn.execute("DELETE FROM learning_plans WHERE id = ?", [plan_id])?;
    Ok(())
}

/// Update learning plan status
pub fn update_learning_plan_status(
    plan_id: i64,
    status: &str,
) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE learning_plans SET status = ?, updated_at = datetime('now') WHERE id = ?",
        rusqlite::params![status, plan_id],
    )?;
    Ok(())
}

/// Generate learning tracks and tasks using AI
/// This function uses the AI provider to create structured learning content
pub async fn generate_learning_content(
    learning_plan_id: i64,
    skill_gaps: &[SkillGap],
) -> Result<(), CareerBenchError> {
    use crate::ai::resolver::ResolvedProvider;

    let provider = ResolvedProvider::resolve()
        .map_err(|e| CareerBenchError::Configuration(crate::errors::ConfigurationError::Other(
            format!("Failed to resolve AI provider: {}", e)
        )))?;

    // Prepare skill gaps summary for AI
    let high_priority_gaps: Vec<&SkillGap> = skill_gaps
        .iter()
        .filter(|gap| gap.priority == "high" && !gap.user_has_skill)
        .take(10)
        .collect();

    if high_priority_gaps.is_empty() {
        return Ok(()); // No gaps to address
    }

    let skills_list: Vec<String> = high_priority_gaps
        .iter()
        .map(|gap| gap.skill.clone())
        .collect();

    let system_prompt = Some(
        "You are a learning path generator. Create structured learning tracks and tasks to help users learn new skills. \
         Always return valid JSON with learning tracks and tasks."
    );

    let user_prompt = format!(
        "Create a learning plan for the following skills: {}. \
         Return a JSON object with this structure: \
         {{ \
           \"tracks\": [ \
             {{ \
               \"title\": \"Track name\", \
               \"description\": \"Track description\", \
               \"skillFocus\": \"Primary skill\", \
               \"orderIndex\": 0, \
               \"tasks\": [ \
                 {{ \
                   \"title\": \"Task name\", \
                   \"description\": \"Task description\", \
                   \"taskType\": \"learning\", \
                   \"resourceUrl\": \"https://example.com\", \
                   \"estimatedHours\": 5, \
                   \"orderIndex\": 0 \
                 }} \
               ] \
             }} \
           ] \
         }}",
        skills_list.join(", ")
    );

    let response = provider.as_provider()
        .call_llm(system_prompt, &user_prompt)
        .await
        .map_err(|e| CareerBenchError::Configuration(crate::errors::ConfigurationError::Other(
            format!("AI generation failed: {}", e)
        )))?;

    // Extract JSON from response
    let json_str = extract_json_from_text(&response);
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| CareerBenchError::Configuration(crate::errors::ConfigurationError::Other(
            format!("Failed to parse AI response: {}", e)
        )))?;

    // Create tracks and tasks
    if let Some(tracks_array) = parsed.get("tracks").and_then(|v| v.as_array()) {
        for (track_idx, track_data) in tracks_array.iter().enumerate() {
            let track_title = track_data.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Learning Track")
                .to_string();
            let track_description = track_data.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let skill_focus = track_data.get("skillFocus")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let track_id = create_learning_track(
                learning_plan_id,
                track_title,
                track_description,
                skill_focus,
                track_idx as i32,
            )?;

            // Create tasks for this track
            if let Some(tasks_array) = track_data.get("tasks").and_then(|v| v.as_array()) {
                for (task_idx, task_data) in tasks_array.iter().enumerate() {
                    let task_title = task_data.get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Learning Task")
                        .to_string();
                    let task_description = task_data.get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let task_type = task_data.get("taskType")
                        .and_then(|v| v.as_str())
                        .unwrap_or("learning")
                        .to_string();
                    let resource_url = task_data.get("resourceUrl")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let estimated_hours = task_data.get("estimatedHours")
                        .and_then(|v| v.as_i64())
                        .map(|h| h as i32);

                    create_learning_task(
                        track_id,
                        task_title,
                        task_description,
                        task_type,
                        resource_url,
                        estimated_hours,
                        None, // due_date
                        task_idx as i32,
                    )?;
                }
            }
        }
    }

    Ok(())
}

/// Helper function to extract JSON from text (handles markdown code blocks)
fn extract_json_from_text(text: &str) -> String {
    // Remove markdown code blocks if present
    let text = text.trim();
    if text.starts_with("```json") {
        text.strip_prefix("```json")
            .and_then(|s| s.strip_suffix("```"))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| text.to_string())
    } else if text.starts_with("```") {
        text.strip_prefix("```")
            .and_then(|s| s.strip_suffix("```"))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| text.to_string())
    } else {
        text.to_string()
    }
}
