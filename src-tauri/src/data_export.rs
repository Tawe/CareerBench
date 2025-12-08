//! Data export functionality
//! 
//! This module provides functionality to export all user data in a structured format
//! for backup, migration, or privacy compliance purposes.

use crate::db::get_connection;
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// Complete data export structure
#[derive(Debug, Serialize, Deserialize)]
pub struct DataExport {
    /// Export metadata
    pub metadata: ExportMetadata,
    /// User profile data
    pub profile: Option<ProfileExport>,
    /// Jobs
    pub jobs: Vec<JobExport>,
    /// Applications
    pub applications: Vec<ApplicationExport>,
    /// Artifacts (resumes, cover letters)
    pub artifacts: Vec<ArtifactExport>,
}

/// Export metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Export timestamp
    pub exported_at: String,
    /// Application version (if available)
    pub version: String,
    /// Total records exported
    pub record_counts: RecordCounts,
}

/// Record counts in export
#[derive(Debug, Serialize, Deserialize)]
pub struct RecordCounts {
    pub jobs: usize,
    pub applications: usize,
    pub artifacts: usize,
}

/// Profile export data
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileExport {
    pub profile: crate::commands::UserProfile,
    pub experience: Vec<crate::commands::Experience>,
    pub skills: Vec<crate::commands::Skill>,
    pub education: Vec<crate::commands::Education>,
    pub certifications: Vec<crate::commands::Certification>,
    pub portfolio: Vec<crate::commands::PortfolioItem>,
}

/// Job export data
#[derive(Debug, Serialize, Deserialize)]
pub struct JobExport {
    pub id: i64,
    pub title: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub job_source: Option<String>,
    pub posting_url: Option<String>,
    pub raw_description: Option<String>,
    pub parsed_json: Option<String>,
    pub is_active: bool,
    pub date_added: String,
    pub last_updated: String,
}

/// Application export data
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationExport {
    pub id: i64,
    pub job_id: i64,
    pub status: String,
    pub channel: Option<String>,
    pub priority: Option<String>,
    pub date_applied: Option<String>,
    pub next_action_date: Option<String>,
    pub next_action_note: Option<String>,
    pub notes_summary: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_linkedin: Option<String>,
    pub location_override: Option<String>,
    pub offer_compensation: Option<String>,
    pub archived: bool,
    pub date_saved: String,
    pub created_at: String,
    pub updated_at: String,
    pub events: Vec<ApplicationEventExport>,
}

/// Application event export data
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationEventExport {
    pub id: i64,
    pub event_type: String,
    pub event_date: String,
    pub from_status: Option<String>,
    pub to_status: Option<String>,
    pub title: Option<String>,
    pub details: Option<String>,
    pub created_at: String,
}

/// Artifact export data
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactExport {
    pub id: i64,
    pub job_id: Option<i64>,
    pub application_id: Option<i64>,
    pub artifact_type: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Export all user data
/// 
/// # Returns
/// `Ok(DataExport)` with all user data, `Err(String)` on error
pub fn export_all_data() -> Result<DataExport, String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    // Export profile data
    let profile = export_profile_data(&conn)?;
    
    // Export jobs
    let jobs = export_jobs(&conn)?;
    
    // Export applications with events
    let applications = export_applications(&conn)?;
    
    // Export artifacts
    let artifacts = export_artifacts(&conn)?;
    
    // Create metadata
    let metadata = ExportMetadata {
        exported_at: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        record_counts: RecordCounts {
            jobs: jobs.len(),
            applications: applications.len(),
            artifacts: artifacts.len(),
        },
    };
    
    Ok(DataExport {
        metadata,
        profile,
        jobs,
        applications,
        artifacts,
    })
}

/// Export profile data
fn export_profile_data(conn: &rusqlite::Connection) -> Result<Option<ProfileExport>, String> {
    // Get profile
    let profile_result: Result<crate::commands::UserProfile, _> = conn
        .query_row(
            "SELECT id, full_name, headline, location, summary, current_role_title, current_company, seniority, open_to_roles, created_at, updated_at FROM user_profile WHERE id = 1",
            [],
            |row| {
                Ok(crate::commands::UserProfile {
                    id: Some(row.get(0)?),
                    full_name: row.get(1)?,
                    headline: row.get(2)?,
                    location: row.get(3)?,
                    summary: row.get(4)?,
                    current_role_title: row.get(5)?,
                    current_company: row.get(6)?,
                    seniority: row.get(7)?,
                    open_to_roles: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            },
        );
    
    let profile = match profile_result {
        Ok(p) => Some(p),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => return Err(format!("Failed to load profile: {}", e)),
    };
    
    // Get experience
    let mut stmt = conn
        .prepare("SELECT id, company, title, location, start_date, end_date, is_current, description, achievements, tech_stack FROM experience WHERE user_profile_id = 1 ORDER BY start_date DESC")
        .map_err(|e| format!("Failed to prepare experience query: {}", e))?;
    
    let experience: Result<Vec<_>, _> = stmt
        .query_map([], |row| {
            Ok(crate::commands::Experience {
                id: Some(row.get(0)?),
                company: row.get(1)?,
                title: row.get(2)?,
                location: row.get(3)?,
                start_date: row.get(4)?,
                end_date: row.get(5)?,
                is_current: row.get::<_, i32>(6)? != 0,
                description: row.get(7)?,
                achievements: row.get(8)?,
                tech_stack: row.get(9)?,
            })
        })
        .and_then(|rows| rows.collect());
    
    let experience = experience.map_err(|e| format!("Failed to load experience: {}", e))?;
    
    // Get skills
    let mut stmt = conn
        .prepare("SELECT id, name, category, self_rating, priority, years_experience, notes FROM skills ORDER BY name")
        .map_err(|e| format!("Failed to prepare skills query: {}", e))?;
    
    let skills: Result<Vec<_>, _> = stmt
        .query_map([], |row| {
            Ok(crate::commands::Skill {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                category: row.get(2)?,
                self_rating: row.get(3)?,
                priority: row.get(4)?,
                years_experience: row.get(5)?,
                notes: row.get(6)?,
            })
        })
        .and_then(|rows| rows.collect());
    
    let skills = skills.map_err(|e| format!("Failed to load skills: {}", e))?;
    
    // Get education
    let mut stmt = conn
        .prepare("SELECT id, institution, degree, field_of_study, start_date, end_date, grade, description FROM education WHERE user_profile_id = 1 ORDER BY end_date DESC")
        .map_err(|e| format!("Failed to prepare education query: {}", e))?;
    
    let education: Result<Vec<_>, _> = stmt
        .query_map([], |row| {
            Ok(crate::commands::Education {
                id: Some(row.get(0)?),
                institution: row.get(1)?,
                degree: row.get(2)?,
                field_of_study: row.get(3)?,
                start_date: row.get(4)?,
                end_date: row.get(5)?,
                grade: row.get(6)?,
                description: row.get(7)?,
            })
        })
        .and_then(|rows| rows.collect());
    
    let education = education.map_err(|e| format!("Failed to load education: {}", e))?;
    
    // Get certifications
    let mut stmt = conn
        .prepare("SELECT id, name, issuing_organization, issue_date, expiration_date, credential_id, credential_url FROM certifications WHERE user_profile_id = 1 ORDER BY issue_date DESC")
        .map_err(|e| format!("Failed to prepare certifications query: {}", e))?;
    
    let certifications: Result<Vec<_>, _> = stmt
        .query_map([], |row| {
            Ok(crate::commands::Certification {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                issuing_organization: row.get(2)?,
                issue_date: row.get(3)?,
                expiration_date: row.get(4)?,
                credential_id: row.get(5)?,
                credential_url: row.get(6)?,
            })
        })
        .and_then(|rows| rows.collect());
    
    let certifications = certifications.map_err(|e| format!("Failed to load certifications: {}", e))?;
    
    // Get portfolio
    let mut stmt = conn
        .prepare("SELECT id, title, url, description, role, tech_stack, highlighted FROM portfolio WHERE user_profile_id = 1 ORDER BY highlighted DESC, title")
        .map_err(|e| format!("Failed to prepare portfolio query: {}", e))?;
    
    let portfolio: Result<Vec<_>, _> = stmt
        .query_map([], |row| {
            Ok(crate::commands::PortfolioItem {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                role: row.get(4)?,
                tech_stack: row.get(5)?,
                highlighted: row.get::<_, i32>(6)? != 0,
            })
        })
        .and_then(|rows| rows.collect());
    
    let portfolio = portfolio.map_err(|e| format!("Failed to load portfolio: {}", e))?;
    
    if profile.is_none() && experience.is_empty() && skills.is_empty() && education.is_empty() && certifications.is_empty() && portfolio.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ProfileExport {
            profile: profile.unwrap_or_else(|| crate::commands::UserProfile {
                id: None,
                full_name: String::new(),
                headline: None,
                location: None,
                summary: None,
                current_role_title: None,
                current_company: None,
                seniority: None,
                open_to_roles: None,
                created_at: None,
                updated_at: None,
            }),
            experience,
            skills,
            education,
            certifications,
            portfolio,
        }))
    }
}

/// Export jobs
fn export_jobs(conn: &rusqlite::Connection) -> Result<Vec<JobExport>, String> {
    let mut stmt = conn
        .prepare("SELECT id, title, company, location, job_source, posting_url, raw_description, parsed_json, is_active, date_added, last_updated FROM jobs ORDER BY date_added DESC")
        .map_err(|e| format!("Failed to prepare jobs query: {}", e))?;
    
    let jobs: Result<Vec<_>, _> = stmt
        .query_map([], |row| {
            Ok(JobExport {
                id: row.get(0)?,
                title: row.get(1)?,
                company: row.get(2)?,
                location: row.get(3)?,
                job_source: row.get(4)?,
                posting_url: row.get(5)?,
                raw_description: row.get(6)?,
                parsed_json: row.get(7)?,
                is_active: row.get::<_, i32>(8)? != 0,
                date_added: row.get(9)?,
                last_updated: row.get(10)?,
            })
        })
        .and_then(|rows| rows.collect());
    
    jobs.map_err(|e| format!("Failed to export jobs: {}", e))
}

/// Export applications with events
fn export_applications(conn: &rusqlite::Connection) -> Result<Vec<ApplicationExport>, String> {
    let mut stmt = conn
        .prepare("SELECT id, job_id, status, channel, priority, date_applied, next_action_date, next_action_note, notes_summary, contact_name, contact_email, contact_linkedin, location_override, offer_compensation, archived, date_saved, created_at, updated_at FROM applications ORDER BY date_saved DESC")
        .map_err(|e| format!("Failed to prepare applications query: {}", e))?;
    
    let applications: Result<Vec<_>, _> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?, // id
                row.get::<_, i64>(1)?, // job_id
                row.get::<_, String>(2)?, // status
                row.get::<_, Option<String>>(3)?, // channel
                row.get::<_, Option<String>>(4)?, // priority
                row.get::<_, Option<String>>(5)?, // date_applied
                row.get::<_, Option<String>>(6)?, // next_action_date
                row.get::<_, Option<String>>(7)?, // next_action_note
                row.get::<_, Option<String>>(8)?, // notes_summary
                row.get::<_, Option<String>>(9)?, // contact_name
                row.get::<_, Option<String>>(10)?, // contact_email
                row.get::<_, Option<String>>(11)?, // contact_linkedin
                row.get::<_, Option<String>>(12)?, // location_override
                row.get::<_, Option<String>>(13)?, // offer_compensation
                row.get::<_, i32>(14)? != 0, // archived
                row.get::<_, String>(15)?, // date_saved
                row.get::<_, String>(16)?, // created_at
                row.get::<_, String>(17)?, // updated_at
            ))
        })
        .and_then(|rows| rows.collect());
    
    let applications = applications.map_err(|e| format!("Failed to export applications: {}", e))?;
    
    // Load events for each application
    let mut result = Vec::new();
    for (id, job_id, status, channel, priority, date_applied, next_action_date, next_action_note, notes_summary, contact_name, contact_email, contact_linkedin, location_override, offer_compensation, archived, date_saved, created_at, updated_at) in applications {
        // Get events for this application
        let mut event_stmt = conn
            .prepare("SELECT id, event_type, event_date, from_status, to_status, title, details, created_at FROM application_events WHERE application_id = ? ORDER BY event_date, created_at")
            .map_err(|e| format!("Failed to prepare events query: {}", e))?;
        
        let events: Result<Vec<_>, _> = event_stmt
            .query_map([id], |row| {
                Ok(ApplicationEventExport {
                    id: row.get(0)?,
                    event_type: row.get(1)?,
                    event_date: row.get(2)?,
                    from_status: row.get(3)?,
                    to_status: row.get(4)?,
                    title: row.get(5)?,
                    details: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .and_then(|rows| rows.collect());
        
        let events = events.map_err(|e| format!("Failed to export events for application {}: {}", id, e))?;
        
        result.push(ApplicationExport {
            id,
            job_id,
            status,
            channel,
            priority,
            date_applied,
            next_action_date,
            next_action_note,
            notes_summary,
            contact_name,
            contact_email,
            contact_linkedin,
            location_override,
            offer_compensation,
            archived,
            date_saved,
            created_at,
            updated_at,
            events,
        });
    }
    
    Ok(result)
}

/// Export artifacts
fn export_artifacts(conn: &rusqlite::Connection) -> Result<Vec<ArtifactExport>, String> {
    let mut stmt = conn
        .prepare("SELECT id, job_id, application_id, artifact_type, title, content, created_at, updated_at FROM artifacts ORDER BY created_at DESC")
        .map_err(|e| format!("Failed to prepare artifacts query: {}", e))?;
    
    let artifacts: Result<Vec<_>, _> = stmt
        .query_map([], |row| {
            Ok(ArtifactExport {
                id: row.get(0)?,
                job_id: row.get(1)?,
                application_id: row.get(2)?,
                artifact_type: row.get(3)?,
                title: row.get(4)?,
                content: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .and_then(|rows| rows.collect());
    
    artifacts.map_err(|e| format!("Failed to export artifacts: {}", e))
}

/// Export data to JSON string
/// 
/// # Returns
/// `Ok(String)` with JSON-encoded export data, `Err(String)` on error
pub fn export_to_json() -> Result<String, String> {
    let data = export_all_data()?;
    serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Failed to serialize export data: {}", e))
}

