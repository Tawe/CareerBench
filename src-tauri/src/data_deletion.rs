//! Data deletion and privacy controls
//! 
//! This module provides functionality for deleting user data, supporting
//! privacy compliance (GDPR, etc.) and user control over their data.

use crate::db::get_connection;

/// Delete a specific job and all related data
/// 
/// This will delete:
/// - The job record
/// - All applications linked to this job
/// - All artifacts linked to this job
/// 
/// # Arguments
/// * `job_id` - ID of the job to delete
/// 
/// # Returns
/// `Ok(())` if successful, `Err(String)` on error
pub fn delete_job(job_id: i64) -> Result<(), String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    // Delete artifacts linked to this job
    conn.execute(
        "DELETE FROM artifacts WHERE job_id = ?",
        [job_id],
    )
    .map_err(|e| format!("Failed to delete artifacts for job {}: {}", job_id, e))?;
    
    // Delete application events for applications linked to this job
    conn.execute(
        "DELETE FROM application_events WHERE application_id IN (SELECT id FROM applications WHERE job_id = ?)",
        [job_id],
    )
    .map_err(|e| format!("Failed to delete application events for job {}: {}", job_id, e))?;
    
    // Delete applications linked to this job
    conn.execute(
        "DELETE FROM applications WHERE job_id = ?",
        [job_id],
    )
    .map_err(|e| format!("Failed to delete applications for job {}: {}", job_id, e))?;
    
    // Delete the job itself
    conn.execute(
        "DELETE FROM jobs WHERE id = ?",
        [job_id],
    )
    .map_err(|e| format!("Failed to delete job {}: {}", job_id, e))?;
    
    log::info!("Deleted job {} and all related data", job_id);
    Ok(())
}

/// Delete a specific application and all related data
/// 
/// This will delete:
/// - The application record
/// - All events for this application
/// - All artifacts linked to this application
/// 
/// # Arguments
/// * `application_id` - ID of the application to delete
/// 
/// # Returns
/// `Ok(())` if successful, `Err(String)` on error
pub fn delete_application(application_id: i64) -> Result<(), String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    // Delete artifacts linked to this application
    conn.execute(
        "DELETE FROM artifacts WHERE application_id = ?",
        [application_id],
    )
    .map_err(|e| format!("Failed to delete artifacts for application {}: {}", application_id, e))?;
    
    // Delete application events
    conn.execute(
        "DELETE FROM application_events WHERE application_id = ?",
        [application_id],
    )
    .map_err(|e| format!("Failed to delete events for application {}: {}", application_id, e))?;
    
    // Delete the application itself
    conn.execute(
        "DELETE FROM applications WHERE id = ?",
        [application_id],
    )
    .map_err(|e| format!("Failed to delete application {}: {}", application_id, e))?;
    
    log::info!("Deleted application {} and all related data", application_id);
    Ok(())
}

/// Delete a specific artifact
/// 
/// # Arguments
/// * `artifact_id` - ID of the artifact to delete
/// 
/// # Returns
/// `Ok(())` if successful, `Err(String)` on error
pub fn delete_artifact(artifact_id: i64) -> Result<(), String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    conn.execute(
        "DELETE FROM artifacts WHERE id = ?",
        [artifact_id],
    )
    .map_err(|e| format!("Failed to delete artifact {}: {}", artifact_id, e))?;
    
    log::info!("Deleted artifact {}", artifact_id);
    Ok(())
}

/// Delete profile section data
/// 
/// # Arguments
/// * `section` - Section to delete ("experience", "skills", "education", "certifications", "portfolio")
/// * `item_id` - Optional ID of specific item to delete (if None, deletes all items in section)
/// 
/// # Returns
/// `Ok(())` if successful, `Err(String)` on error
pub fn delete_profile_section(section: &str, item_id: Option<i64>) -> Result<(), String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    match section {
        "experience" => {
            if let Some(id) = item_id {
                conn.execute(
                    "DELETE FROM experience WHERE id = ? AND user_profile_id = 1",
                    [id],
                )
                .map_err(|e| format!("Failed to delete experience {}: {}", id, e))?;
            } else {
                conn.execute(
                    "DELETE FROM experience WHERE user_profile_id = 1",
                    [],
                )
                .map_err(|e| format!("Failed to delete all experience: {}", e))?;
            }
        }
        "skills" => {
            if let Some(id) = item_id {
                conn.execute(
                    "DELETE FROM skills WHERE id = ? AND user_profile_id = 1",
                    [id],
                )
                .map_err(|e| format!("Failed to delete skill {}: {}", id, e))?;
            } else {
                conn.execute(
                    "DELETE FROM skills WHERE user_profile_id = 1",
                    [],
                )
                .map_err(|e| format!("Failed to delete all skills: {}", e))?;
            }
        }
        "education" => {
            if let Some(id) = item_id {
                conn.execute(
                    "DELETE FROM education WHERE id = ? AND user_profile_id = 1",
                    [id],
                )
                .map_err(|e| format!("Failed to delete education {}: {}", id, e))?;
            } else {
                conn.execute(
                    "DELETE FROM education WHERE user_profile_id = 1",
                    [],
                )
                .map_err(|e| format!("Failed to delete all education: {}", e))?;
            }
        }
        "certifications" => {
            if let Some(id) = item_id {
                conn.execute(
                    "DELETE FROM certifications WHERE id = ? AND user_profile_id = 1",
                    [id],
                )
                .map_err(|e| format!("Failed to delete certification {}: {}", id, e))?;
            } else {
                conn.execute(
                    "DELETE FROM certifications WHERE user_profile_id = 1",
                    [],
                )
                .map_err(|e| format!("Failed to delete all certifications: {}", e))?;
            }
        }
        "portfolio" => {
            if let Some(id) = item_id {
                conn.execute(
                    "DELETE FROM portfolio_items WHERE id = ? AND user_profile_id = 1",
                    [id],
                )
                .map_err(|e| format!("Failed to delete portfolio item {}: {}", id, e))?;
            } else {
                conn.execute(
                    "DELETE FROM portfolio_items WHERE user_profile_id = 1",
                    [],
                )
                .map_err(|e| format!("Failed to delete all portfolio items: {}", e))?;
            }
        }
        _ => {
            return Err(format!("Unknown profile section: {}", section));
        }
    }
    
    log::info!("Deleted profile section '{}' (item_id: {:?})", section, item_id);
    Ok(())
}

/// Delete all user data (GDPR "Right to be Forgotten")
/// 
/// This function deletes ALL user data from the database:
/// - User profile
/// - All experience, skills, education, certifications, portfolio
/// - All jobs
/// - All applications and events
/// - All artifacts
/// - AI cache (optional)
/// 
/// WARNING: This is irreversible! Use with extreme caution.
/// 
/// # Arguments
/// * `include_ai_cache` - Whether to also delete AI cache entries
/// 
/// # Returns
/// `Ok(())` if successful, `Err(String)` on error
pub fn delete_all_user_data(include_ai_cache: bool) -> Result<(), String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    log::warn!("DELETING ALL USER DATA - This is irreversible!");
    
    // Delete in order to respect foreign key constraints
    
    // 1. Delete artifacts (no dependencies)
    conn.execute("DELETE FROM artifacts", [])
        .map_err(|e| format!("Failed to delete artifacts: {}", e))?;
    log::info!("Deleted all artifacts");
    
    // 2. Delete application events
    conn.execute("DELETE FROM application_events", [])
        .map_err(|e| format!("Failed to delete application events: {}", e))?;
    log::info!("Deleted all application events");
    
    // 3. Delete applications
    conn.execute("DELETE FROM applications", [])
        .map_err(|e| format!("Failed to delete applications: {}", e))?;
    log::info!("Deleted all applications");
    
    // 4. Delete jobs
    conn.execute("DELETE FROM jobs", [])
        .map_err(|e| format!("Failed to delete jobs: {}", e))?;
    log::info!("Deleted all jobs");
    
    // 5. Delete profile-related data
    conn.execute("DELETE FROM portfolio_items WHERE user_profile_id = 1", [])
        .map_err(|e| format!("Failed to delete portfolio: {}", e))?;
    log::info!("Deleted all portfolio items");
    
    conn.execute("DELETE FROM certifications WHERE user_profile_id = 1", [])
        .map_err(|e| format!("Failed to delete certifications: {}", e))?;
    log::info!("Deleted all certifications");
    
    conn.execute("DELETE FROM education WHERE user_profile_id = 1", [])
        .map_err(|e| format!("Failed to delete education: {}", e))?;
    log::info!("Deleted all education");
    
    conn.execute("DELETE FROM skills WHERE user_profile_id = 1", [])
        .map_err(|e| format!("Failed to delete skills: {}", e))?;
    log::info!("Deleted all skills");
    
    conn.execute("DELETE FROM experience WHERE user_profile_id = 1", [])
        .map_err(|e| format!("Failed to delete experience: {}", e))?;
    log::info!("Deleted all experience");
    
    // 6. Delete user profile
    conn.execute("DELETE FROM user_profile WHERE id = 1", [])
        .map_err(|e| format!("Failed to delete user profile: {}", e))?;
    log::info!("Deleted user profile");
    
    // 7. Optionally delete AI cache
    if include_ai_cache {
        conn.execute("DELETE FROM ai_cache", [])
            .map_err(|e| format!("Failed to delete AI cache: {}", e))?;
        log::info!("Deleted all AI cache entries");
    }
    
    log::warn!("All user data has been deleted");
    Ok(())
}

/// Get data deletion summary
/// 
/// Returns counts of records that would be deleted, useful for confirmation dialogs.
/// 
/// # Returns
/// `Ok(DeletionSummary)` with record counts, `Err(String)` on error
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeletionSummary {
    pub jobs: i64,
    pub applications: i64,
    pub artifacts: i64,
    pub profile_sections: ProfileSectionCounts,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ProfileSectionCounts {
    pub experience: i64,
    pub skills: i64,
    pub education: i64,
    pub certifications: i64,
    pub portfolio: i64,
}

pub fn get_deletion_summary() -> Result<DeletionSummary, String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    let jobs: i64 = conn
        .query_row("SELECT COUNT(*) FROM jobs", [], |row| row.get(0))
        .unwrap_or(0);
    
    let applications: i64 = conn
        .query_row("SELECT COUNT(*) FROM applications", [], |row| row.get(0))
        .unwrap_or(0);
    
    let artifacts: i64 = conn
        .query_row("SELECT COUNT(*) FROM artifacts", [], |row| row.get(0))
        .unwrap_or(0);
    
    let experience: i64 = conn
        .query_row("SELECT COUNT(*) FROM experience WHERE user_profile_id = 1", [], |row| row.get(0))
        .unwrap_or(0);
    
    let skills: i64 = conn
        .query_row("SELECT COUNT(*) FROM skills WHERE user_profile_id = 1", [], |row| row.get(0))
        .unwrap_or(0);
    
    let education: i64 = conn
        .query_row("SELECT COUNT(*) FROM education WHERE user_profile_id = 1", [], |row| row.get(0))
        .unwrap_or(0);
    
    let certifications: i64 = conn
        .query_row("SELECT COUNT(*) FROM certifications WHERE user_profile_id = 1", [], |row| row.get(0))
        .unwrap_or(0);
    
    let portfolio: i64 = conn
        .query_row("SELECT COUNT(*) FROM portfolio_items WHERE user_profile_id = 1", [], |row| row.get(0))
        .unwrap_or(0);
    
    Ok(DeletionSummary {
        jobs,
        applications,
        artifacts,
        profile_sections: ProfileSectionCounts {
            experience,
            skills,
            education,
            certifications,
            portfolio,
        },
    })
}

