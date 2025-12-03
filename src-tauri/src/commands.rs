use crate::db::get_connection;
use chrono::Utc;
use rusqlite;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Dashboard types
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardKpis {
    pub total_jobs_tracked: i64,
    pub total_applications: i64,
    pub active_applications: i64,
    pub applications_last_30_days: i64,
    pub offers_received: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusBucket {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyActivityPoint {
    pub date: String,
    pub applications_created: i64,
    pub interviews_completed: i64,
    pub offers_received: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunnelStep {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub kpis: DashboardKpis,
    pub status_breakdown: Vec<StatusBucket>,
    pub activity_last_30_days: Vec<DailyActivityPoint>,
    pub funnel: Vec<FunnelStep>,
}

#[tauri::command]
pub async fn get_dashboard_data() -> Result<DashboardData, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;

    // KPIs
    let total_jobs: i64 = conn
        .query_row("SELECT COUNT(*) FROM jobs", [], |row| row.get(0))
        .map_err(|e| format!("Failed to get total jobs: {}", e))?;

    let total_applications: i64 = conn
        .query_row("SELECT COUNT(*) FROM applications", [], |row| row.get(0))
        .map_err(|e| format!("Failed to get total applications: {}", e))?;

    let active_applications: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM applications WHERE archived = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get active applications: {}", e))?;

    let applications_last_30_days: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM applications WHERE date_saved >= date('now', '-30 day')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get recent applications: {}", e))?;

    let offers_received: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM applications WHERE status = 'Offer'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get offers: {}", e))?;

    let kpis = DashboardKpis {
        total_jobs_tracked: total_jobs,
        total_applications,
        active_applications,
        applications_last_30_days,
        offers_received,
    };

    // Status breakdown
    let mut stmt = conn
        .prepare(
            "SELECT status, COUNT(*) as count
             FROM applications
             WHERE archived = 0
             GROUP BY status",
        )
        .map_err(|e| format!("Failed to prepare status query: {}", e))?;

    let status_rows = stmt
        .query_map([], |row| {
            Ok(StatusBucket {
                status: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| format!("Failed to get status breakdown: {}", e))?;

    let mut status_breakdown = Vec::new();
    for row_result in status_rows {
        status_breakdown.push(row_result.map_err(|e| format!("Error: {}", e))?);
    }

    // Activity last 30 days
    let mut activity_map: HashMap<String, DailyActivityPoint> = HashMap::new();

    // Initialize all dates
    let now = Utc::now();
    for i in 0..30 {
        let date = now - chrono::Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();
        activity_map.insert(
            date_str.clone(),
            DailyActivityPoint {
                date: date_str,
                applications_created: 0,
                interviews_completed: 0,
                offers_received: 0,
            },
        );
    }

    // Applications created
    let mut stmt = conn
        .prepare(
            "SELECT date(date_saved) as day, COUNT(*) as count
             FROM applications
             WHERE date_saved >= date('now', '-30 day')
             GROUP BY day",
        )
        .map_err(|e| format!("Failed to prepare applications query: {}", e))?;

    let app_rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| format!("Failed to get applications: {}", e))?;

    for row_result in app_rows {
        let (day, count) = row_result.map_err(|e| format!("Error: {}", e))?;
        if let Some(point) = activity_map.get_mut(&day) {
            point.applications_created = count;
        }
    }

    // Interviews completed
    let mut stmt = conn
        .prepare(
            "SELECT date(event_date) as day, COUNT(*) as count
             FROM application_events
             WHERE event_type = 'InterviewCompleted'
               AND event_date >= date('now', '-30 day')
             GROUP BY day",
        )
        .map_err(|e| format!("Failed to prepare interviews query: {}", e))?;

    let interview_rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| format!("Failed to get interviews: {}", e))?;

    for row_result in interview_rows {
        let (day, count) = row_result.map_err(|e| format!("Error: {}", e))?;
        if let Some(point) = activity_map.get_mut(&day) {
            point.interviews_completed = count;
        }
    }

    // Offers received
    let mut stmt = conn
        .prepare(
            "SELECT date(event_date) as day, COUNT(*) as count
             FROM application_events
             WHERE event_type = 'OfferReceived'
               AND event_date >= date('now', '-30 day')
             GROUP BY day",
        )
        .map_err(|e| format!("Failed to prepare offers query: {}", e))?;

    let offer_rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| format!("Failed to get offers: {}", e))?;

    for row_result in offer_rows {
        let (day, count) = row_result.map_err(|e| format!("Error: {}", e))?;
        if let Some(point) = activity_map.get_mut(&day) {
            point.offers_received = count;
        }
    }

    let mut activity_last_30_days: Vec<DailyActivityPoint> = activity_map.into_values().collect();
    activity_last_30_days.sort_by_key(|p| p.date.clone());

    // Funnel
    let applied: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM applications WHERE status IN ('Applied', 'Interviewing', 'Offer', 'Rejected', 'Ghosted', 'Withdrawn')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get applied count: {}", e))?;

    let interviewing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM applications WHERE status IN ('Interviewing', 'Offer', 'Rejected', 'Ghosted', 'Withdrawn')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get interviewing count: {}", e))?;

    let offer: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM applications WHERE status = 'Offer'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get offer count: {}", e))?;

    let funnel = vec![
        FunnelStep {
            label: "Applied".to_string(),
            count: applied,
        },
        FunnelStep {
            label: "Interviewing".to_string(),
            count: interviewing,
        },
        FunnelStep {
            label: "Offer".to_string(),
            count: offer,
        },
    ];

    Ok(DashboardData {
        kpis,
        status_breakdown,
        activity_last_30_days,
        funnel,
    })
}

// User Profile types
#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Option<i64>,
    pub full_name: String,
    pub headline: Option<String>,
    pub location: Option<String>,
    pub summary: Option<String>,
    pub current_role_title: Option<String>,
    pub current_company: Option<String>,
    pub seniority: Option<String>,
    pub open_to_roles: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Experience {
    pub id: Option<i64>,
    pub company: String,
    pub title: String,
    pub location: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_current: bool,
    pub description: Option<String>,
    pub achievements: Option<String>,
    pub tech_stack: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Skill {
    pub id: Option<i64>,
    pub name: String,
    pub category: Option<String>,
    pub self_rating: Option<i32>,
    pub priority: Option<String>,
    pub years_experience: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Education {
    pub id: Option<i64>,
    pub institution: String,
    pub degree: Option<String>,
    pub field_of_study: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub grade: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Certification {
    pub id: Option<i64>,
    pub name: String,
    pub issuing_organization: Option<String>,
    pub issue_date: Option<String>,
    pub expiration_date: Option<String>,
    pub credential_id: Option<String>,
    pub credential_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioItem {
    pub id: Option<i64>,
    pub title: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub role: Option<String>,
    pub tech_stack: Option<String>,
    pub highlighted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileData {
    pub profile: Option<UserProfile>,
    pub experience: Vec<Experience>,
    pub skills: Vec<Skill>,
    pub education: Vec<Education>,
    pub certifications: Vec<Certification>,
    pub portfolio: Vec<PortfolioItem>,
}

#[tauri::command]
pub async fn get_user_profile_data() -> Result<UserProfileData, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;

    // Get or create user profile (id = 1)
    let profile: Option<UserProfile> = {
        let mut stmt = conn
            .prepare("SELECT id, full_name, headline, location, summary, current_role_title, current_company, seniority, open_to_roles, created_at, updated_at FROM user_profile WHERE id = 1")
            .map_err(|e| format!("Failed to prepare profile query: {}", e))?;

        let profile_result = stmt.query_row([], |row| {
            Ok(UserProfile {
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
        });

        match profile_result {
            Ok(p) => Some(p),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => return Err(format!("Failed to get profile: {}", e)),
        }
    };

    // Get experience
    let experience: Vec<Experience> = {
        let mut stmt = conn
            .prepare("SELECT id, company, title, location, start_date, end_date, is_current, description, achievements, tech_stack FROM experience WHERE user_profile_id = 1 ORDER BY start_date DESC, id DESC")
            .map_err(|e| format!("Failed to prepare experience query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Experience {
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
            .map_err(|e| format!("Failed to get experience: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Error: {}", e))?
    };

    // Get skills
    let skills: Vec<Skill> = {
        let mut stmt = conn
            .prepare("SELECT id, name, category, self_rating, priority, years_experience, notes FROM skills WHERE user_profile_id = 1 ORDER BY name")
            .map_err(|e| format!("Failed to prepare skills query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Skill {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    category: row.get(2)?,
                    self_rating: row.get(3)?,
                    priority: row.get(4)?,
                    years_experience: row.get(5)?,
                    notes: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to get skills: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Error: {}", e))?
    };

    // Get education
    let education: Vec<Education> = {
        let mut stmt = conn
            .prepare("SELECT id, institution, degree, field_of_study, start_date, end_date, grade, description FROM education WHERE user_profile_id = 1 ORDER BY end_date DESC, start_date DESC")
            .map_err(|e| format!("Failed to prepare education query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Education {
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
            .map_err(|e| format!("Failed to get education: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Error: {}", e))?
    };

    // Get certifications
    let certifications: Vec<Certification> = {
        let mut stmt = conn
            .prepare("SELECT id, name, issuing_organization, issue_date, expiration_date, credential_id, credential_url FROM certifications WHERE user_profile_id = 1 ORDER BY issue_date DESC")
            .map_err(|e| format!("Failed to prepare certifications query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Certification {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    issuing_organization: row.get(2)?,
                    issue_date: row.get(3)?,
                    expiration_date: row.get(4)?,
                    credential_id: row.get(5)?,
                    credential_url: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to get certifications: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Error: {}", e))?
    };

    // Get portfolio
    let portfolio: Vec<PortfolioItem> = {
        let mut stmt = conn
            .prepare("SELECT id, title, url, description, role, tech_stack, highlighted FROM portfolio_items WHERE user_profile_id = 1 ORDER BY highlighted DESC, id DESC")
            .map_err(|e| format!("Failed to prepare portfolio query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(PortfolioItem {
                    id: Some(row.get(0)?),
                    title: row.get(1)?,
                    url: row.get(2)?,
                    description: row.get(3)?,
                    role: row.get(4)?,
                    tech_stack: row.get(5)?,
                    highlighted: row.get::<_, i32>(6)? != 0,
                })
            })
            .map_err(|e| format!("Failed to get portfolio: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Error: {}", e))?
    };

    Ok(UserProfileData {
        profile,
        experience,
        skills,
        education,
        certifications,
        portfolio,
    })
}

#[tauri::command]
pub async fn save_user_profile_data(data: UserProfileData) -> Result<UserProfileData, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();

    // Save or update profile
    if let Some(profile) = &data.profile {
        if profile.id.is_some() {
            // Update existing
            conn.execute(
                "UPDATE user_profile SET full_name = ?, headline = ?, location = ?, summary = ?, current_role_title = ?, current_company = ?, seniority = ?, open_to_roles = ?, updated_at = ? WHERE id = 1",
                rusqlite::params![
                    profile.full_name,
                    profile.headline,
                    profile.location,
                    profile.summary,
                    profile.current_role_title,
                    profile.current_company,
                    profile.seniority,
                    profile.open_to_roles,
                    now
                ],
            )
            .map_err(|e| format!("Failed to update profile: {}", e))?;
        } else {
            // Insert new
            conn.execute(
                "INSERT INTO user_profile (id, full_name, headline, location, summary, current_role_title, current_company, seniority, open_to_roles, created_at, updated_at) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    profile.full_name,
                    profile.headline,
                    profile.location,
                    profile.summary,
                    profile.current_role_title,
                    profile.current_company,
                    profile.seniority,
                    profile.open_to_roles,
                    now,
                    now
                ],
            )
            .map_err(|e| format!("Failed to insert profile: {}", e))?;
        }
    }

    // Save experience (delete all and reinsert for simplicity in MVP)
    conn.execute("DELETE FROM experience WHERE user_profile_id = 1", [])
        .map_err(|e| format!("Failed to delete experience: {}", e))?;

    for exp in &data.experience {
        conn.execute(
            "INSERT INTO experience (user_profile_id, company, title, location, start_date, end_date, is_current, description, achievements, tech_stack, created_at, updated_at) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                exp.company,
                exp.title,
                exp.location,
                exp.start_date,
                exp.end_date,
                if exp.is_current { 1 } else { 0 },
                exp.description,
                exp.achievements,
                exp.tech_stack,
                now,
                now
            ],
        )
        .map_err(|e| format!("Failed to insert experience: {}", e))?;
    }

    // Save skills
    conn.execute("DELETE FROM skills WHERE user_profile_id = 1", [])
        .map_err(|e| format!("Failed to delete skills: {}", e))?;

    for skill in &data.skills {
        conn.execute(
            "INSERT INTO skills (user_profile_id, name, category, self_rating, priority, years_experience, notes) VALUES (1, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                skill.name,
                skill.category,
                skill.self_rating,
                skill.priority,
                skill.years_experience,
                skill.notes
            ],
        )
        .map_err(|e| format!("Failed to insert skill: {}", e))?;
    }

    // Save education
    conn.execute("DELETE FROM education WHERE user_profile_id = 1", [])
        .map_err(|e| format!("Failed to delete education: {}", e))?;

    for edu in &data.education {
        conn.execute(
            "INSERT INTO education (user_profile_id, institution, degree, field_of_study, start_date, end_date, grade, description) VALUES (1, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                edu.institution,
                edu.degree,
                edu.field_of_study,
                edu.start_date,
                edu.end_date,
                edu.grade,
                edu.description
            ],
        )
        .map_err(|e| format!("Failed to insert education: {}", e))?;
    }

    // Save certifications
    conn.execute("DELETE FROM certifications WHERE user_profile_id = 1", [])
        .map_err(|e| format!("Failed to delete certifications: {}", e))?;

    for cert in &data.certifications {
        conn.execute(
            "INSERT INTO certifications (user_profile_id, name, issuing_organization, issue_date, expiration_date, credential_id, credential_url) VALUES (1, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                cert.name,
                cert.issuing_organization,
                cert.issue_date,
                cert.expiration_date,
                cert.credential_id,
                cert.credential_url
            ],
        )
        .map_err(|e| format!("Failed to insert certification: {}", e))?;
    }

    // Save portfolio
    conn.execute("DELETE FROM portfolio_items WHERE user_profile_id = 1", [])
        .map_err(|e| format!("Failed to delete portfolio: {}", e))?;

    for item in &data.portfolio {
        conn.execute(
            "INSERT INTO portfolio_items (user_profile_id, title, url, description, role, tech_stack, highlighted) VALUES (1, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                item.title,
                item.url,
                item.description,
                item.role,
                item.tech_stack,
                if item.highlighted { 1 } else { 0 }
            ],
        )
        .map_err(|e| format!("Failed to insert portfolio item: {}", e))?;
    }

    // Return updated data
    get_user_profile_data().await
}

// Job types
#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: Option<i64>,
    pub title: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub job_source: Option<String>,
    pub posting_url: Option<String>,
    pub raw_description: Option<String>,
    pub parsed_json: Option<String>,
    pub seniority: Option<String>,
    pub domain_tags: Option<String>,
    pub is_active: bool,
    pub date_added: String,
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobSummary {
    pub id: i64,
    pub title: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub seniority: Option<String>,
    pub domain_tags: Option<String>,
    pub date_added: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateJobInput {
    pub title: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub job_source: Option<String>,
    pub posting_url: Option<String>,
    pub raw_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateJobInput {
    pub title: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub job_source: Option<String>,
    pub posting_url: Option<String>,
    pub raw_description: Option<String>,
    pub is_active: Option<bool>,
}

#[tauri::command]
pub async fn create_job(input: CreateJobInput) -> Result<Job, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();

    // Validate that at least title, company, or raw_description is provided
    if input.title.is_none() && input.company.is_none() && input.raw_description.is_none() {
        return Err("At least one of title, company, or description must be provided".to_string());
    }

    conn.execute(
        "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)",
        rusqlite::params![
            input.title,
            input.company,
            input.location,
            input.job_source,
            input.posting_url,
            input.raw_description,
            now,
            now
        ],
    )
    .map_err(|e| format!("Failed to create job: {}", e))?;

    let id = conn.last_insert_rowid();
    get_job_detail(id).await
}

#[tauri::command]
pub async fn update_job(id: i64, input: UpdateJobInput) -> Result<Job, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();

    // Build update query dynamically based on provided fields
    let mut updates = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(title) = &input.title {
        updates.push("title = ?");
        params.push(title.clone());
    }
    if let Some(company) = &input.company {
        updates.push("company = ?");
        params.push(company.clone());
    }
    if let Some(location) = &input.location {
        updates.push("location = ?");
        params.push(location.clone());
    }
    if let Some(job_source) = &input.job_source {
        updates.push("job_source = ?");
        params.push(job_source.clone());
    }
    if let Some(posting_url) = &input.posting_url {
        updates.push("posting_url = ?");
        params.push(posting_url.clone());
    }
    if let Some(raw_description) = &input.raw_description {
        updates.push("raw_description = ?");
        updates.push("parsed_json = NULL"); // Clear parsed data when description changes
        params.push(raw_description.clone());
    }
    if let Some(is_active) = input.is_active {
        updates.push("is_active = ?");
        params.push(if is_active { "1".to_string() } else { "0".to_string() });
    }

    if updates.is_empty() {
        return get_job_detail(id).await;
    }

    updates.push("last_updated = ?");
    params.push(now.clone());

    // Build query with placeholders and execute - avoid holding references across await
    let query = format!("UPDATE jobs SET {} WHERE id = ?", updates.join(", "));
    
    // Use rusqlite::params! macro which is Send-safe
    let param_count = params.len();
    match param_count {
        1 => {
            conn.execute(&query, rusqlite::params![params[0], id])
                .map_err(|e| format!("Failed to update job: {}", e))?;
        }
        2 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], id])
                .map_err(|e| format!("Failed to update job: {}", e))?;
        }
        3 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], id])
                .map_err(|e| format!("Failed to update job: {}", e))?;
        }
        4 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], params[3], id])
                .map_err(|e| format!("Failed to update job: {}", e))?;
        }
        5 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], params[3], params[4], id])
                .map_err(|e| format!("Failed to update job: {}", e))?;
        }
        6 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], params[3], params[4], params[5], id])
                .map_err(|e| format!("Failed to update job: {}", e))?;
        }
        7 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], params[3], params[4], params[5], params[6], id])
                .map_err(|e| format!("Failed to update job: {}", e))?;
        }
        8 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], params[3], params[4], params[5], params[6], params[7], id])
                .map_err(|e| format!("Failed to update job: {}", e))?;
        }
        _ => {
            // Fallback for many params - use a helper to build params array
            // We'll manually handle up to 15 params, which should be more than enough
            if param_count <= 15 {
                let mut all_params = params.clone();
                all_params.push(id.to_string());
                // Use a macro-like approach with match for safety
                let result = match all_params.len() {
                    9 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8]]),
                    10 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9]]),
                    11 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9], all_params[10]]),
                    12 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9], all_params[10], all_params[11]]),
                    13 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9], all_params[10], all_params[11], all_params[12]]),
                    14 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9], all_params[10], all_params[11], all_params[12], all_params[13]]),
                    15 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9], all_params[10], all_params[11], all_params[12], all_params[13], all_params[14]]),
                    _ => return Err("Too many parameters for update query".to_string()),
                };
                result.map_err(|e| format!("Failed to update job: {}", e))?;
            } else {
                return Err("Too many parameters for update query".to_string());
            }
        }
    }

    get_job_detail(id).await
}

#[tauri::command]
pub async fn get_job_list(
    search: Option<String>,
    active_only: Option<bool>,
    source: Option<String>,
) -> Result<Vec<JobSummary>, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;

    let mut query = "SELECT id, title, company, location, seniority, domain_tags, date_added FROM jobs WHERE 1=1".to_string();
    let mut params: Vec<String> = Vec::new();

    if active_only.unwrap_or(true) {
        query.push_str(" AND is_active = 1");
    }

    if let Some(source_filter) = &source {
        query.push_str(" AND job_source = ?");
        params.push(source_filter.clone());
    }

    if let Some(search_term) = &search {
        query.push_str(" AND (title LIKE ? OR company LIKE ? OR location LIKE ? OR raw_description LIKE ?)");
        let search_pattern = format!("%{}%", search_term);
        for _ in 0..4 {
            params.push(search_pattern.clone());
        }
    }

    query.push_str(" ORDER BY date_added DESC");

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rusqlite_params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
    let rows = stmt
        .query_map(rusqlite::params_from_iter(rusqlite_params.iter().cloned()), |row| {
            Ok(JobSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                company: row.get(2)?,
                location: row.get(3)?,
                seniority: row.get(4)?,
                domain_tags: row.get(5)?,
                date_added: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to get jobs: {}", e))?;

    let mut jobs = Vec::new();
    for row_result in rows {
        jobs.push(row_result.map_err(|e| format!("Error: {}", e))?);
    }

    Ok(jobs)
}

#[tauri::command]
pub async fn get_job_detail(id: i64) -> Result<Job, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, company, location, job_source, posting_url, raw_description, parsed_json, seniority, domain_tags, is_active, date_added, last_updated FROM jobs WHERE id = ?"
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let job = stmt
        .query_row([id], |row| {
            Ok(Job {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                company: row.get(2)?,
                location: row.get(3)?,
                job_source: row.get(4)?,
                posting_url: row.get(5)?,
                raw_description: row.get(6)?,
                parsed_json: row.get(7)?,
                seniority: row.get(8)?,
                domain_tags: row.get(9)?,
                is_active: row.get::<_, i32>(10)? != 0,
                date_added: row.get(11)?,
                last_updated: row.get(12)?,
            })
        })
        .map_err(|e| format!("Job not found: {}", e))?;

    Ok(job)
}

// ParsedJob struct for AI parsing
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedJob {
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

#[tauri::command]
pub async fn parse_job_with_ai(job_id: i64) -> Result<ParsedJob, String> {
    use crate::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_JOB_PARSE_DAYS};
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();

    // Step 1: Load job from DB
    let job = get_job_detail(job_id).await?;

    // Step 2: Validate raw_description exists
    let raw_description = job.raw_description.as_ref()
        .ok_or_else(|| "Job description is empty; cannot parse.".to_string())?;
    
    if raw_description.trim().is_empty() {
        return Err("Job description is empty; cannot parse.".to_string());
    }

    // Step 3: Build canonical input JSON for caching
    let request_payload = serde_json::json!({
        "jobDescription": raw_description,
        "jobMeta": {
            "source": job.job_source,
            "url": job.posting_url
        }
    });

    // Step 4: Compute input hash and check cache
    let input_hash = compute_input_hash(&request_payload)
        .map_err(|e| format!("Failed to compute hash: {}", e))?;

    if let Some(cached_entry) = ai_cache_get(&conn, "job_parse", &input_hash, &now)
        .map_err(|e| format!("Cache lookup error: {}", e))? {
        // Cache hit - deserialize and return
        let parsed: ParsedJob = serde_json::from_value(cached_entry.response_payload)
            .map_err(|e| format!("Failed to deserialize cached response: {}", e))?;
        
        // Update job with cached parsed data
        update_job_with_parsed_data(&conn, job_id, &parsed, &now)?;
        
        return Ok(parsed);
    }

    // Step 5: Cache miss - call AI provider (placeholder for now)
    // TODO: Replace with actual AI provider call
    let parsed = call_ai_provider_for_parsing(raw_description).await?;

    // Step 6: Store in cache
    let response_payload = serde_json::to_value(&parsed)
        .map_err(|e| format!("Failed to serialize parsed job: {}", e))?;

    ai_cache_put(
        &conn,
        "job_parse",
        &input_hash,
        "placeholder-model", // TODO: Use actual model name
        &request_payload,
        &response_payload,
        Some(CACHE_TTL_JOB_PARSE_DAYS),
        &now,
    )
    .map_err(|e| format!("Failed to cache result: {}", e))?;

    // Step 7: Update job with parsed data
    update_job_with_parsed_data(&conn, job_id, &parsed, &now)?;

    Ok(parsed)
}

// Helper function to update job with parsed data
fn update_job_with_parsed_data(
    conn: &rusqlite::Connection,
    job_id: i64,
    parsed: &ParsedJob,
    now: &str,
) -> Result<(), String> {
    let parsed_json = serde_json::to_string(parsed)
        .map_err(|e| format!("Failed to serialize parsed job: {}", e))?;

    let domain_tags_str = parsed.domain_tags.join(", ");

    conn.execute(
        "UPDATE jobs SET parsed_json = ?, seniority = COALESCE(?, seniority), domain_tags = COALESCE(?, domain_tags), last_updated = ? WHERE id = ?",
        rusqlite::params![
            parsed_json,
            parsed.seniority,
            if domain_tags_str.is_empty() { None } else { Some(domain_tags_str) },
            now,
            job_id
        ],
    )
    .map_err(|e| format!("Failed to update job with parsed data: {}", e))?;

    Ok(())
}

// Placeholder AI provider call - will be replaced with actual implementation
async fn call_ai_provider_for_parsing(raw_description: &str) -> Result<ParsedJob, String> {
    // TODO: Implement actual AI provider call
    // For now, return a basic parsed structure with some heuristics
    
    let description_lower = raw_description.to_lowercase();
    
    // Simple heuristics for demonstration
    let mut responsibilities = Vec::new();
    let mut required_skills = Vec::new();
    let mut nice_to_have_skills = Vec::new();
    
    // Extract some basic patterns (very simple - real AI would do much better)
    if description_lower.contains("node") || description_lower.contains("node.js") {
        required_skills.push("Node.js".to_string());
    }
    if description_lower.contains("react") {
        required_skills.push("React".to_string());
    }
    if description_lower.contains("typescript") {
        required_skills.push("TypeScript".to_string());
    }
    if description_lower.contains("python") {
        required_skills.push("Python".to_string());
    }
    if description_lower.contains("aws") {
        required_skills.push("AWS".to_string());
    }
    if description_lower.contains("docker") || description_lower.contains("kubernetes") {
        nice_to_have_skills.push("Containerization".to_string());
    }
    
    // Try to extract responsibilities from common patterns
    for line in raw_description.lines() {
        let line = line.trim();
        if line.starts_with("-") || line.starts_with("•") || line.starts_with("*") {
            let cleaned = line.trim_start_matches(|c: char| c == '-' || c == '•' || c == '*').trim();
            if cleaned.len() > 20 && cleaned.len() < 200 {
                responsibilities.push(cleaned.to_string());
            }
        }
    }
    
    // Limit responsibilities
    if responsibilities.len() > 10 {
        responsibilities.truncate(10);
    }
    
    // Determine seniority from keywords
    let seniority = if description_lower.contains("senior") || description_lower.contains("sr.") {
        Some("Senior".to_string())
    } else if description_lower.contains("junior") || description_lower.contains("jr.") {
        Some("Junior".to_string())
    } else if description_lower.contains("lead") || description_lower.contains("principal") {
        Some("Lead".to_string())
    } else {
        None
    };
    
    // Check for remote
    let remote_friendly = description_lower.contains("remote") || 
                         description_lower.contains("work from home") ||
                         description_lower.contains("wfh");
    
    Ok(ParsedJob {
        title_suggestion: None,
        company_suggestion: None,
        seniority,
        location: None,
        summary: None,
        responsibilities,
        required_skills,
        nice_to_have_skills,
        domain_tags: Vec::new(),
        seniority_score: None,
        remote_friendly: Some(remote_friendly),
    })
}

// Application types
#[derive(Debug, Serialize, Deserialize)]
pub struct Application {
    pub id: Option<i64>,
    pub job_id: i64,
    pub status: String,
    pub channel: Option<String>,
    pub priority: Option<String>,
    pub date_saved: String,
    pub date_applied: Option<String>,
    pub last_activity_date: Option<String>,
    pub next_action_date: Option<String>,
    pub next_action_note: Option<String>,
    pub notes_summary: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_linkedin: Option<String>,
    pub location_override: Option<String>,
    pub offer_compensation: Option<String>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationSummary {
    pub id: i64,
    pub job_id: i64,
    pub job_title: Option<String>,
    pub company: Option<String>,
    pub status: String,
    pub priority: Option<String>,
    pub date_saved: String,
    pub date_applied: Option<String>,
    pub last_activity_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationEvent {
    pub id: Option<i64>,
    pub application_id: i64,
    pub event_type: String,
    pub event_date: String,
    pub from_status: Option<String>,
    pub to_status: Option<String>,
    pub title: Option<String>,
    pub details: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationDetail {
    pub application: Application,
    pub events: Vec<ApplicationEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApplicationInput {
    pub job_id: i64,
    pub status: Option<String>,
    pub channel: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateApplicationInput {
    pub status: Option<String>,
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddEventInput {
    pub application_id: i64,
    pub event_type: String,
    pub event_date: Option<String>,
    pub title: Option<String>,
    pub details: Option<String>,
}

#[tauri::command]
pub async fn create_application(input: CreateApplicationInput) -> Result<Application, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();
    let status = input.status.unwrap_or_else(|| "Saved".to_string());

    // Check if job exists
    let job_exists: bool = conn
        .query_row("SELECT COUNT(*) FROM jobs WHERE id = ?", [input.job_id], |row| {
            row.get(0)
        })
        .map_err(|e| format!("Failed to check job: {}", e))?;

    if !job_exists {
        return Err("Job not found".to_string());
    }

    // Insert application
    conn.execute(
        "INSERT INTO applications (job_id, status, channel, priority, date_saved, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            input.job_id,
            status,
            input.channel,
            input.priority,
            now,
            now,
            now
        ],
    )
    .map_err(|e| format!("Failed to create application: {}", e))?;

    let application_id = conn.last_insert_rowid();

    // Create ApplicationCreated event
    conn.execute(
        "INSERT INTO application_events (application_id, event_type, event_date, created_at) VALUES (?, ?, ?, ?)",
        rusqlite::params![application_id, "ApplicationCreated", now, now],
    )
    .map_err(|e| format!("Failed to create event: {}", e))?;

    get_application_detail(application_id).await.map(|d| d.application)
}

#[tauri::command]
pub async fn update_application(id: i64, input: UpdateApplicationInput) -> Result<Application, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();

    // Get current application to check status change
    let current_app = get_application_detail(id).await
        .map_err(|_| "Application not found".to_string())?;
    let old_status = current_app.application.status.clone();

    // Build update query
    let mut updates = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(status) = &input.status {
        updates.push("status = ?");
        params.push(status.clone());
    }
    if let Some(channel) = &input.channel {
        updates.push("channel = ?");
        params.push(channel.clone());
    }
    if let Some(priority) = &input.priority {
        updates.push("priority = ?");
        params.push(priority.clone());
    }
    if let Some(date_applied) = &input.date_applied {
        updates.push("date_applied = ?");
        params.push(date_applied.clone());
    }
    if let Some(next_action_date) = &input.next_action_date {
        updates.push("next_action_date = ?");
        params.push(next_action_date.clone());
    }
    if let Some(next_action_note) = &input.next_action_note {
        updates.push("next_action_note = ?");
        params.push(next_action_note.clone());
    }
    if let Some(notes_summary) = &input.notes_summary {
        updates.push("notes_summary = ?");
        params.push(notes_summary.clone());
    }
    if let Some(contact_name) = &input.contact_name {
        updates.push("contact_name = ?");
        params.push(contact_name.clone());
    }
    if let Some(contact_email) = &input.contact_email {
        updates.push("contact_email = ?");
        params.push(contact_email.clone());
    }
    if let Some(contact_linkedin) = &input.contact_linkedin {
        updates.push("contact_linkedin = ?");
        params.push(contact_linkedin.clone());
    }
    if let Some(location_override) = &input.location_override {
        updates.push("location_override = ?");
        params.push(location_override.clone());
    }
    if let Some(offer_compensation) = &input.offer_compensation {
        updates.push("offer_compensation = ?");
        params.push(offer_compensation.clone());
    }

    if updates.is_empty() {
        return Ok(current_app.application);
    }

    // Handle status change
    let new_status = input.status.as_ref().unwrap_or(&old_status);
    if new_status != &old_status {
        updates.push("last_activity_date = ?");
        params.push(now.clone());

        // If status becomes Applied and date_applied is empty, set it
        if new_status == "Applied" && input.date_applied.is_none() && current_app.application.date_applied.is_none() {
            updates.push("date_applied = ?");
            params.push(now.clone());
        }

        // Create StatusChanged event
        conn.execute(
            "INSERT INTO application_events (application_id, event_type, event_date, from_status, to_status, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                id,
                "StatusChanged",
                now,
                old_status,
                new_status,
                now
            ],
        )
        .map_err(|e| format!("Failed to create status change event: {}", e))?;
    }

    updates.push("updated_at = ?");
    params.push(now.clone());

    // Build query and execute - avoid holding references across await
    let query = format!("UPDATE applications SET {} WHERE id = ?", updates.join(", "));
    
    // Use rusqlite::params! macro which is Send-safe
    let param_count = params.len();
    match param_count {
        1 => {
            conn.execute(&query, rusqlite::params![params[0], id])
                .map_err(|e| format!("Failed to update application: {}", e))?;
        }
        2 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], id])
                .map_err(|e| format!("Failed to update application: {}", e))?;
        }
        3 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], id])
                .map_err(|e| format!("Failed to update application: {}", e))?;
        }
        4 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], params[3], id])
                .map_err(|e| format!("Failed to update application: {}", e))?;
        }
        5 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], params[3], params[4], id])
                .map_err(|e| format!("Failed to update application: {}", e))?;
        }
        6 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], params[3], params[4], params[5], id])
                .map_err(|e| format!("Failed to update application: {}", e))?;
        }
        7 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], params[3], params[4], params[5], params[6], id])
                .map_err(|e| format!("Failed to update application: {}", e))?;
        }
        8 => {
            conn.execute(&query, rusqlite::params![params[0], params[1], params[2], params[3], params[4], params[5], params[6], params[7], id])
                .map_err(|e| format!("Failed to update application: {}", e))?;
        }
        _ => {
            // Fallback for many params - use a helper to build params array
            // We'll manually handle up to 15 params, which should be more than enough
            if param_count <= 15 {
                let mut all_params = params.clone();
                all_params.push(id.to_string());
                // Use a macro-like approach with match for safety
                let result = match all_params.len() {
                    9 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8]]),
                    10 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9]]),
                    11 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9], all_params[10]]),
                    12 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9], all_params[10], all_params[11]]),
                    13 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9], all_params[10], all_params[11], all_params[12]]),
                    14 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9], all_params[10], all_params[11], all_params[12], all_params[13]]),
                    15 => conn.execute(&query, rusqlite::params![all_params[0], all_params[1], all_params[2], all_params[3], all_params[4], all_params[5], all_params[6], all_params[7], all_params[8], all_params[9], all_params[10], all_params[11], all_params[12], all_params[13], all_params[14]]),
                    _ => return Err("Too many parameters for update query".to_string()),
                };
                result.map_err(|e| format!("Failed to update application: {}", e))?;
            } else {
                return Err("Too many parameters for update query".to_string());
            }
        }
    }

    get_application_detail(id).await.map(|d| d.application)
}

#[tauri::command]
pub async fn get_applications(
    status: Option<String>,
    job_id: Option<i64>,
    active_only: Option<bool>,
) -> Result<Vec<ApplicationSummary>, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;

    let mut query = "SELECT a.id, a.job_id, j.title, j.company, a.status, a.priority, a.date_saved, a.date_applied, a.last_activity_date FROM applications a LEFT JOIN jobs j ON a.job_id = j.id WHERE 1=1".to_string();
    let mut params: Vec<String> = Vec::new();

    if active_only.unwrap_or(true) {
        query.push_str(" AND a.archived = 0");
    }

    if let Some(status_filter) = &status {
        query.push_str(" AND a.status = ?");
        params.push(status_filter.clone());
    }

    if let Some(job_id_filter) = job_id {
        query.push_str(" AND a.job_id = ?");
        params.push(job_id_filter.to_string());
    }

    query.push_str(" ORDER BY a.date_saved DESC");

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rusqlite_params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
    let rows = stmt
        .query_map(rusqlite::params_from_iter(rusqlite_params.iter().cloned()), |row| {
            Ok(ApplicationSummary {
                id: row.get(0)?,
                job_id: row.get(1)?,
                job_title: row.get(2)?,
                company: row.get(3)?,
                status: row.get(4)?,
                priority: row.get(5)?,
                date_saved: row.get(6)?,
                date_applied: row.get(7)?,
                last_activity_date: row.get(8)?,
            })
        })
        .map_err(|e| format!("Failed to get applications: {}", e))?;

    let mut applications = Vec::new();
    for row_result in rows {
        applications.push(row_result.map_err(|e| format!("Error: {}", e))?);
    }

    Ok(applications)
}

#[tauri::command]
pub async fn get_application_detail(id: i64) -> Result<ApplicationDetail, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;

    // Get application
    let mut stmt = conn
        .prepare(
            "SELECT id, job_id, status, channel, priority, date_saved, date_applied, last_activity_date, next_action_date, next_action_note, notes_summary, contact_name, contact_email, contact_linkedin, location_override, offer_compensation, archived, created_at, updated_at FROM applications WHERE id = ?"
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let application = stmt
        .query_row([id], |row| {
            Ok(Application {
                id: Some(row.get(0)?),
                job_id: row.get(1)?,
                status: row.get(2)?,
                channel: row.get(3)?,
                priority: row.get(4)?,
                date_saved: row.get(5)?,
                date_applied: row.get(6)?,
                last_activity_date: row.get(7)?,
                next_action_date: row.get(8)?,
                next_action_note: row.get(9)?,
                notes_summary: row.get(10)?,
                contact_name: row.get(11)?,
                contact_email: row.get(12)?,
                contact_linkedin: row.get(13)?,
                location_override: row.get(14)?,
                offer_compensation: row.get(15)?,
                archived: row.get::<_, i32>(16)? != 0,
                created_at: row.get(17)?,
                updated_at: row.get(18)?,
            })
        })
        .map_err(|e| format!("Application not found: {}", e))?;

    // Get events
    let mut stmt = conn
        .prepare(
            "SELECT id, application_id, event_type, event_date, from_status, to_status, title, details, created_at FROM application_events WHERE application_id = ? ORDER BY event_date ASC"
        )
        .map_err(|e| format!("Failed to prepare events query: {}", e))?;

    let rows = stmt
        .query_map([id], |row| {
            Ok(ApplicationEvent {
                id: Some(row.get(0)?),
                application_id: row.get(1)?,
                event_type: row.get(2)?,
                event_date: row.get(3)?,
                from_status: row.get(4)?,
                to_status: row.get(5)?,
                title: row.get(6)?,
                details: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("Failed to get events: {}", e))?;

    let mut events = Vec::new();
    for row_result in rows {
        events.push(row_result.map_err(|e| format!("Error: {}", e))?);
    }

    Ok(ApplicationDetail { application, events })
}

#[tauri::command]
pub async fn add_application_event(input: AddEventInput) -> Result<ApplicationEvent, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();
    let event_date = input.event_date.unwrap_or_else(|| now.clone());

    conn.execute(
        "INSERT INTO application_events (application_id, event_type, event_date, title, details, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            input.application_id,
            input.event_type,
            event_date,
            input.title,
            input.details,
            now
        ],
    )
    .map_err(|e| format!("Failed to create event: {}", e))?;

    let event_id = conn.last_insert_rowid();

    // Update last_activity_date on application
    conn.execute(
        "UPDATE applications SET last_activity_date = ? WHERE id = ?",
        rusqlite::params![event_date, input.application_id],
    )
    .map_err(|e| format!("Failed to update last activity: {}", e))?;

    // Get the created event
    let mut stmt = conn
        .prepare("SELECT id, application_id, event_type, event_date, from_status, to_status, title, details, created_at FROM application_events WHERE id = ?")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let event = stmt
        .query_row([event_id], |row| {
            Ok(ApplicationEvent {
                id: Some(row.get(0)?),
                application_id: row.get(1)?,
                event_type: row.get(2)?,
                event_date: row.get(3)?,
                from_status: row.get(4)?,
                to_status: row.get(5)?,
                title: row.get(6)?,
                details: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("Failed to get event: {}", e))?;

    Ok(event)
}

#[tauri::command]
pub async fn archive_application(id: i64) -> Result<Application, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE applications SET archived = 1, updated_at = ? WHERE id = ?",
        rusqlite::params![now, id],
    )
    .map_err(|e| format!("Failed to archive application: {}", e))?;

    get_application_detail(id).await.map(|d| d.application)
}

// Resume & Cover Letter types
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedResume {
    pub summary: Option<String>,
    pub headline: Option<String>,
    pub sections: Vec<ResumeSection>,
    pub highlights: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedLetter {
    pub subject: Option<String>,
    pub greeting: Option<String>,
    pub body_paragraphs: Vec<String>,
    pub closing: Option<String>,
    pub signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationOptions {
    pub tone: Option<String>,
    pub length: Option<String>,
    pub focus: Option<String>,
    pub audience: Option<String>, // For cover letters
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResumeGenerationResult {
    pub resume: GeneratedResume,
    pub artifact_id: i64,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LetterGenerationResult {
    pub letter: GeneratedLetter,
    pub artifact_id: i64,
    pub content: String,
}

#[tauri::command]
pub async fn generate_resume_for_job(
    job_id: i64,
    application_id: Option<i64>,
    options: Option<GenerationOptions>,
) -> Result<ResumeGenerationResult, String> {
    use crate::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_RESUME_DAYS};
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();

    // Load user profile
    let profile_data = get_user_profile_data().await?;
    if profile_data.profile.is_none() {
        return Err("User profile not found. Please set up your profile first.".to_string());
    }

    // Load job
    let job = get_job_detail(job_id).await?;

    // Build canonical request payload
    let request_payload = serde_json::json!({
        "userProfile": profile_data.profile,
        "experience": profile_data.experience,
        "skills": profile_data.skills,
        "education": profile_data.education,
        "portfolio": profile_data.portfolio,
        "job": {
            "title": job.title,
            "company": job.company,
            "rawDescription": job.raw_description,
            "parsedJson": job.parsed_json
        },
        "options": options
    });

    // Check cache
    let input_hash = compute_input_hash(&request_payload)
        .map_err(|e| format!("Failed to compute hash: {}", e))?;

    if let Some(cached_entry) = ai_cache_get(&conn, "resume_generation", &input_hash, &now)
        .map_err(|e| format!("Cache lookup error: {}", e))? {
        let resume: GeneratedResume = serde_json::from_value(cached_entry.response_payload)
            .map_err(|e| format!("Failed to deserialize cached response: {}", e))?;
        
        let content = render_resume_to_text(&resume);
        let artifact_id = get_or_create_artifact(
            &conn,
            application_id,
            Some(job_id),
            "Resume",
            &content,
            &serde_json::to_string(&resume).unwrap(),
            &now,
        )?;

        return Ok(ResumeGenerationResult {
            resume,
            artifact_id,
            content,
        });
    }

    // Cache miss - generate resume (placeholder AI)
    let resume = generate_resume_with_ai(&profile_data, &job, options.as_ref()).await?;

    // Store in cache
    let response_payload = serde_json::to_value(&resume)
        .map_err(|e| format!("Failed to serialize resume: {}", e))?;

    ai_cache_put(
        &conn,
        "resume_generation",
        &input_hash,
        "placeholder-model",
        &request_payload,
        &response_payload,
        Some(CACHE_TTL_RESUME_DAYS),
        &now,
    )
    .map_err(|e| format!("Failed to cache result: {}", e))?;

    // Create artifact
    let content = render_resume_to_text(&resume);
    let artifact_id = get_or_create_artifact(
        &conn,
        application_id,
        Some(job_id),
        "Resume",
        &content,
        &serde_json::to_string(&resume).unwrap(),
        &now,
    )?;

    Ok(ResumeGenerationResult {
        resume,
        artifact_id,
        content,
    })
}

#[tauri::command]
pub async fn generate_cover_letter_for_job(
    job_id: i64,
    application_id: Option<i64>,
    options: Option<GenerationOptions>,
) -> Result<LetterGenerationResult, String> {
    use crate::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_COVER_LETTER_DAYS};
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();

    // Load user profile
    let profile_data = get_user_profile_data().await?;
    if profile_data.profile.is_none() {
        return Err("User profile not found. Please set up your profile first.".to_string());
    }

    // Load job
    let job = get_job_detail(job_id).await?;

    // Build canonical request payload
    let request_payload = serde_json::json!({
        "userProfile": profile_data.profile,
        "experience": profile_data.experience,
        "skills": profile_data.skills,
        "job": {
            "title": job.title,
            "company": job.company,
            "rawDescription": job.raw_description,
            "parsedJson": job.parsed_json
        },
        "options": options
    });

    // Check cache
    let input_hash = compute_input_hash(&request_payload)
        .map_err(|e| format!("Failed to compute hash: {}", e))?;

    if let Some(cached_entry) = ai_cache_get(&conn, "cover_letter_generation", &input_hash, &now)
        .map_err(|e| format!("Cache lookup error: {}", e))? {
        let letter: GeneratedLetter = serde_json::from_value(cached_entry.response_payload)
            .map_err(|e| format!("Failed to deserialize cached response: {}", e))?;
        
        let content = render_letter_to_text(&letter);
        let artifact_id = get_or_create_artifact(
            &conn,
            application_id,
            Some(job_id),
            "CoverLetter",
            &content,
            &serde_json::to_string(&letter).unwrap(),
            &now,
        )?;

        return Ok(LetterGenerationResult {
            letter,
            artifact_id,
            content,
        });
    }

    // Cache miss - generate letter (placeholder AI)
    let letter = generate_cover_letter_with_ai(&profile_data, &job, options.as_ref()).await?;

    // Store in cache
    let response_payload = serde_json::to_value(&letter)
        .map_err(|e| format!("Failed to serialize letter: {}", e))?;

    ai_cache_put(
        &conn,
        "cover_letter_generation",
        &input_hash,
        "placeholder-model",
        &request_payload,
        &response_payload,
        Some(CACHE_TTL_COVER_LETTER_DAYS),
        &now,
    )
    .map_err(|e| format!("Failed to cache result: {}", e))?;

    // Create artifact
    let content = render_letter_to_text(&letter);
    let artifact_id = get_or_create_artifact(
        &conn,
        application_id,
        Some(job_id),
        "CoverLetter",
        &content,
        &serde_json::to_string(&letter).unwrap(),
        &now,
    )?;

    Ok(LetterGenerationResult {
        letter,
        artifact_id,
        content,
    })
}

// Helper function to get or create artifact
fn get_or_create_artifact(
    conn: &rusqlite::Connection,
    application_id: Option<i64>,
    job_id: Option<i64>,
    artifact_type: &str,
    content: &str,
    ai_payload: &str,
    now: &str,
) -> Result<i64, String> {
    // Get job title and company for artifact title
    let (job_title, company) = if let Some(jid) = job_id {
        let job_result: Result<(Option<String>, Option<String>), _> = conn
            .query_row(
                "SELECT title, company FROM jobs WHERE id = ?",
                [jid],
                |row| Ok((row.get(0)?, row.get(1)?)),
            );
        match job_result {
            Ok((title, comp)) => (
                title.unwrap_or_else(|| "Untitled".to_string()),
                comp.unwrap_or_else(|| "Unknown".to_string()),
            ),
            Err(_) => ("Untitled".to_string(), "Unknown".to_string()),
        }
    } else {
        ("Untitled".to_string(), "Unknown".to_string())
    };

    // Check if artifact already exists
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM artifacts WHERE application_id = ? AND job_id = ? AND type = ? ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![application_id, job_id, artifact_type],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing {
        // Update existing
        conn.execute(
            "UPDATE artifacts SET content = ?, ai_payload = ?, updated_at = ? WHERE id = ?",
            rusqlite::params![content, ai_payload, now, id],
        )
        .map_err(|e| format!("Failed to update artifact: {}", e))?;
        return Ok(id);
    }

    // Create new
    let title = format!("{} – {} @ {}", artifact_type, job_title, company);

    conn.execute(
        "INSERT INTO artifacts (application_id, job_id, type, title, content, format, ai_payload, ai_model, source, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            application_id,
            job_id,
            artifact_type,
            title,
            content,
            "markdown",
            ai_payload,
            "placeholder-model",
            "ai_generated",
            now,
            now
        ],
    )
    .map_err(|e| format!("Failed to create artifact: {}", e))?;

    Ok(conn.last_insert_rowid())
}

// Placeholder AI generation functions
async fn generate_resume_with_ai(
    profile_data: &UserProfileData,
    _job: &Job,
    _options: Option<&GenerationOptions>,
) -> Result<GeneratedResume, String> {
    // TODO: Replace with actual AI provider call
    // For now, create a basic resume structure from profile data
    
    let profile = profile_data.profile.as_ref().unwrap();
    let mut sections = Vec::new();

    // Experience section
    if !profile_data.experience.is_empty() {
        let mut exp_items = Vec::new();
        for exp in &profile_data.experience {
            let mut bullets = Vec::new();
            if let Some(desc) = &exp.description {
                bullets.push(desc.clone());
            }
            if let Some(achievements) = &exp.achievements {
                for line in achievements.split('\n') {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        bullets.push(trimmed.to_string());
                    }
                }
            }
            
            let mut subheading = String::new();
            if let Some(start) = &exp.start_date {
                subheading.push_str(&format_date(start));
            }
            if exp.is_current {
                subheading.push_str(" – Present");
            } else if let Some(end) = &exp.end_date {
                subheading.push_str(&format!(" – {}", format_date(end)));
            }
            if let Some(loc) = &exp.location {
                subheading.push_str(&format!(" | {}", loc));
            }

            exp_items.push(ResumeSectionItem {
                heading: format!("{} – {}", exp.title, exp.company),
                subheading: if subheading.is_empty() { None } else { Some(subheading) },
                bullets,
            });
        }
        sections.push(ResumeSection {
            title: "Experience".to_string(),
            items: exp_items,
        });
    }

    // Skills section
    if !profile_data.skills.is_empty() {
        let mut skill_bullets = Vec::new();
        let core_skills: Vec<String> = profile_data.skills
            .iter()
            .filter(|s| s.priority.as_deref() == Some("Core"))
            .map(|s| s.name.clone())
            .collect();
        if !core_skills.is_empty() {
            skill_bullets.push(core_skills.join(", "));
        }
        
        sections.push(ResumeSection {
            title: "Skills".to_string(),
            items: vec![ResumeSectionItem {
                heading: "Core Skills".to_string(),
                subheading: None,
                bullets: skill_bullets,
            }],
        });
    }

    // Education section
    if !profile_data.education.is_empty() {
        let mut edu_items = Vec::new();
        for edu in &profile_data.education {
            let mut heading = edu.institution.clone();
            if let Some(degree) = &edu.degree {
                heading.push_str(&format!(" – {}", degree));
            }
            edu_items.push(ResumeSectionItem {
                heading,
                subheading: None,
                bullets: Vec::new(),
            });
        }
        sections.push(ResumeSection {
            title: "Education".to_string(),
            items: edu_items,
        });
    }

    Ok(GeneratedResume {
        summary: profile.summary.clone(),
        headline: profile.headline.clone(),
        sections,
        highlights: vec!["Tailored for this specific role".to_string()],
    })
}

async fn generate_cover_letter_with_ai(
    profile_data: &UserProfileData,
    job: &Job,
    _options: Option<&GenerationOptions>,
) -> Result<GeneratedLetter, String> {
    // TODO: Replace with actual AI provider call
    let profile = profile_data.profile.as_ref().unwrap();
    let name = &profile.full_name;
    let job_title = job.title.as_deref().unwrap_or("this position");
    let company = job.company.as_deref().unwrap_or("your company");

    let subject = format!("Application for {} – {}", job_title, name);
    let greeting = "Dear Hiring Manager,".to_string();
    
    let mut paragraphs = Vec::new();
    paragraphs.push(format!(
        "I am excited to apply for the {} role at {}.",
        job_title, company
    ));
    
    if let Some(summary) = &profile.summary {
        paragraphs.push(summary.clone());
    } else if !profile_data.experience.is_empty() {
        let latest_exp = &profile_data.experience[0];
        paragraphs.push(format!(
            "With my experience as {} at {}, I am confident I can contribute to your team.",
            latest_exp.title, latest_exp.company
        ));
    }

    paragraphs.push(format!(
        "I am particularly drawn to {} and would welcome the opportunity to discuss how my background aligns with your needs.",
        company
    ));

    let closing = "Thank you for your time and consideration.".to_string();
    let signature = format!("Sincerely,\n{}", name);

    Ok(GeneratedLetter {
        subject: Some(subject),
        greeting: Some(greeting),
        body_paragraphs: paragraphs,
        closing: Some(closing),
        signature: Some(signature),
    })
}

fn format_date(date_str: &str) -> String {
    if date_str.len() >= 7 {
        // Format: YYYY-MM
        let parts: Vec<&str> = date_str.split('-').collect();
        if parts.len() >= 2 {
            let month = match parts[1] {
                "01" => "Jan", "02" => "Feb", "03" => "Mar", "04" => "Apr",
                "05" => "May", "06" => "Jun", "07" => "Jul", "08" => "Aug",
                "09" => "Sep", "10" => "Oct", "11" => "Nov", "12" => "Dec",
                _ => parts[1],
            };
            return format!("{} {}", month, parts[0]);
        }
    }
    date_str.to_string()
}

fn render_resume_to_text(resume: &GeneratedResume) -> String {
    let mut output = String::new();
    
    if let Some(headline) = &resume.headline {
        output.push_str(headline);
        output.push_str("\n\n");
    }
    
    if let Some(summary) = &resume.summary {
        output.push_str(summary);
        output.push_str("\n\n");
    }
    
    for section in &resume.sections {
        output.push_str(&format!("## {}\n\n", section.title));
        for item in &section.items {
            output.push_str(&format!("### {}\n", item.heading));
            if let Some(subheading) = &item.subheading {
                output.push_str(&format!("{}\n", subheading));
            }
            for bullet in &item.bullets {
                output.push_str(&format!("- {}\n", bullet));
            }
            output.push_str("\n");
        }
    }
    
    output
}

fn render_letter_to_text(letter: &GeneratedLetter) -> String {
    let mut output = String::new();
    
    if let Some(subject) = &letter.subject {
        output.push_str(&format!("Subject: {}\n\n", subject));
    }
    
    if let Some(greeting) = &letter.greeting {
        output.push_str(greeting);
        output.push_str("\n\n");
    }
    
    for paragraph in &letter.body_paragraphs {
        output.push_str(paragraph);
        output.push_str("\n\n");
    }
    
    if let Some(closing) = &letter.closing {
        output.push_str(closing);
        output.push_str("\n\n");
    }
    
    if let Some(signature) = &letter.signature {
        output.push_str(signature);
    }
    
    output
}

