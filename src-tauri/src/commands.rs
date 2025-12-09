use crate::db::get_connection;
use crate::errors::CareerBenchError;
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
pub struct DateRange {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub kpis: DashboardKpis,
    pub status_breakdown: Vec<StatusBucket>,
    pub activity_last_30_days: Vec<DailyActivityPoint>,
    pub funnel: Vec<FunnelStep>,
    pub date_range: Option<DateRange>,
}

#[tauri::command]
pub async fn get_dashboard_data(
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<DashboardData, String> {
    let conn = get_connection()
        .map_err(|e| CareerBenchError::from(e).to_string_for_tauri())?;

    // Default to last 30 days if no dates provided
    let start_date_str = start_date.unwrap_or_else(|| {
        (Utc::now() - chrono::Duration::days(30)).format("%Y-%m-%d").to_string()
    });
    let end_date_str = end_date.unwrap_or_else(|| {
        Utc::now().format("%Y-%m-%d").to_string()
    });

    // KPIs - Optimized: Single query with conditional aggregation
    let kpi_row = conn
        .query_row(
            "SELECT 
                (SELECT COUNT(*) FROM jobs) as total_jobs,
                (SELECT COUNT(*) FROM applications) as total_applications,
                (SELECT COUNT(*) FROM applications WHERE archived = 0) as active_applications,
                (SELECT COUNT(*) FROM applications WHERE date_saved >= ? AND date_saved <= ?) as applications_in_range,
                (SELECT COUNT(*) FROM applications WHERE status = 'Offer') as offers_received",
            [&start_date_str, &end_date_str],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,  // total_jobs
                    row.get::<_, i64>(1)?,  // total_applications
                    row.get::<_, i64>(2)?,  // active_applications
                    row.get::<_, i64>(3)?,  // applications_in_range
                    row.get::<_, i64>(4)?,  // offers_received
                ))
            },
        )
        .map_err(|e| format!("Failed to get KPIs: {}", e))?;

    let (total_jobs, total_applications, active_applications, applications_in_range, offers_received) = kpi_row;

    let kpis = DashboardKpis {
        total_jobs_tracked: total_jobs,
        total_applications,
        active_applications,
        applications_last_30_days: applications_in_range,
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

    // Activity for date range
    let mut activity_map: HashMap<String, DailyActivityPoint> = HashMap::new();

    // Parse dates and initialize all dates in range
    let start = chrono::NaiveDate::parse_from_str(&start_date_str, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date: {}", e))?;
    let end = chrono::NaiveDate::parse_from_str(&end_date_str, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end date: {}", e))?;
    
    let mut current = start;
    while current <= end {
        let date_str = current.format("%Y-%m-%d").to_string();
        activity_map.insert(
            date_str.clone(),
            DailyActivityPoint {
                date: date_str,
                applications_created: 0,
                interviews_completed: 0,
                offers_received: 0,
            },
        );
        current = current.succ_opt()
            .ok_or_else(|| "Date range too large".to_string())?;
    }

    // Activity data - Optimized: Single query with UNION ALL for all activity types
    // This reduces database round trips from 3 to 1
    let mut stmt = conn
        .prepare(
            "SELECT date(date_saved) as day, COUNT(*) as count, 'applications' as type
             FROM applications
             WHERE date_saved >= ? AND date_saved <= ?
             GROUP BY day
             UNION ALL
             SELECT date(event_date) as day, COUNT(*) as count, 'interviews' as type
             FROM application_events
             WHERE event_type = 'InterviewCompleted'
               AND event_date >= ? AND event_date <= ?
             GROUP BY day
             UNION ALL
             SELECT date(event_date) as day, COUNT(*) as count, 'offers' as type
             FROM application_events
             WHERE event_type = 'OfferReceived'
               AND event_date >= ? AND event_date <= ?
             GROUP BY day",
        )
        .map_err(|e| format!("Failed to prepare activity query: {}", e))?;

    let activity_rows = stmt
        .query_map(
            [&start_date_str, &end_date_str, &start_date_str, &end_date_str, &start_date_str, &end_date_str],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,  // day
                    row.get::<_, i64>(1)?,     // count
                    row.get::<_, String>(2)?,  // type
                ))
            },
        )
        .map_err(|e| format!("Failed to get activity data: {}", e))?;

    for row_result in activity_rows {
        let (day, count, activity_type) = row_result.map_err(|e| format!("Error: {}", e))?;
        if let Some(point) = activity_map.get_mut(&day) {
            match activity_type.as_str() {
                "applications" => point.applications_created = count,
                "interviews" => point.interviews_completed = count,
                "offers" => point.offers_received = count,
                _ => {}
            }
        }
    }

    let mut activity_last_30_days: Vec<DailyActivityPoint> = activity_map.into_values().collect();
    activity_last_30_days.sort_by_key(|p| p.date.clone());

    // Funnel - Optimized: Single query with conditional aggregation
    let funnel_row = conn
        .query_row(
            "SELECT 
                COUNT(CASE WHEN status IN ('Applied', 'Interviewing', 'Offer', 'Rejected', 'Ghosted', 'Withdrawn') THEN 1 END) as applied,
                COUNT(CASE WHEN status IN ('Interviewing', 'Offer', 'Rejected', 'Ghosted', 'Withdrawn') THEN 1 END) as interviewing,
                COUNT(CASE WHEN status = 'Offer' THEN 1 END) as offer
             FROM applications",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,  // applied
                    row.get::<_, i64>(1)?,  // interviewing
                    row.get::<_, i64>(2)?,  // offer
                ))
            },
        )
        .map_err(|e| format!("Failed to get funnel data: {}", e))?;

    let (applied, interviewing, offer) = funnel_row;

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
        date_range: Some(DateRange {
            start_date: start_date_str.clone(),
            end_date: end_date_str.clone(),
        }),
    })
}

/// Export dashboard data as CSV
#[tauri::command]
pub async fn export_dashboard_data(
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<String, String> {
    let dashboard_data = get_dashboard_data(start_date, end_date).await?;
    
    let mut csv = String::new();
    
    // Header
    csv.push_str("CareerBench Dashboard Export\n");
    if let Some(range) = &dashboard_data.date_range {
        csv.push_str(&format!("Date Range: {} to {}\n", range.start_date, range.end_date));
    }
    csv.push_str("\n");
    
    // KPIs Section
    csv.push_str("KPIs\n");
    csv.push_str("Metric,Value\n");
    csv.push_str(&format!("Total Jobs Tracked,{}\n", dashboard_data.kpis.total_jobs_tracked));
    csv.push_str(&format!("Total Applications,{}\n", dashboard_data.kpis.total_applications));
    csv.push_str(&format!("Active Applications,{}\n", dashboard_data.kpis.active_applications));
    csv.push_str(&format!("Applications in Range,{}\n", dashboard_data.kpis.applications_last_30_days));
    csv.push_str(&format!("Offers Received,{}\n", dashboard_data.kpis.offers_received));
    csv.push_str("\n");
    
    // Status Breakdown
    csv.push_str("Status Breakdown\n");
    csv.push_str("Status,Count\n");
    for status in &dashboard_data.status_breakdown {
        csv.push_str(&format!("{},{}\n", status.status, status.count));
    }
    csv.push_str("\n");
    
    // Funnel
    csv.push_str("Pipeline Funnel\n");
    csv.push_str("Stage,Count\n");
    for step in &dashboard_data.funnel {
        csv.push_str(&format!("{},{}\n", step.label, step.count));
    }
    csv.push_str("\n");
    
    // Activity Over Time
    csv.push_str("Daily Activity\n");
    csv.push_str("Date,Applications Created,Interviews Completed,Offers Received\n");
    for point in &dashboard_data.activity_last_30_days {
        csv.push_str(&format!("{},{},{},{}\n", 
            point.date, 
            point.applications_created, 
            point.interviews_completed, 
            point.offers_received
        ));
    }
    
    Ok(csv)
}

/// Get calendar events for a date range
#[tauri::command]
pub async fn get_calendar_events(
    start_date: String,
    end_date: String,
) -> Result<Vec<crate::calendar::CalendarEvent>, String> {
    crate::calendar::get_calendar_events(&start_date, &end_date)
        .map_err(|e| e.to_string_for_tauri())
}

/// Get calendar events for a specific date
#[tauri::command]
pub async fn get_events_for_date(date: String) -> Result<Vec<crate::calendar::CalendarEvent>, String> {
    crate::calendar::get_events_for_date(&date)
        .map_err(|e| e.to_string_for_tauri())
}

/// Generate ICS file content for an interview event
#[tauri::command]
pub async fn sync_interview_to_calendar(
    application_id: i64,
    event_id: Option<i64>,
    title: String,
    start_time: String,
    end_time: Option<String>,
    location: Option<String>,
    notes: Option<String>,
) -> Result<String, String> {
    crate::calendar::sync_interview_to_calendar(
        application_id,
        event_id,
        &title,
        &start_time,
        end_time.as_deref(),
        location.as_deref(),
        notes.as_deref(),
    )
    .map_err(|e| e.to_string_for_tauri())
}

// ============================================================================
// Reminder Commands
// ============================================================================

/// Create a reminder for an interview or event
#[tauri::command]
pub async fn create_reminder(
    application_id: Option<i64>,
    event_id: Option<i64>,
    reminder_type: String,
    reminder_date: String,
    message: Option<String>,
) -> Result<i64, String> {
    crate::reminders::create_reminder(
        application_id,
        event_id,
        &reminder_type,
        &reminder_date,
        message.as_deref(),
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Get reminders for a date range
#[tauri::command]
pub async fn get_reminders(
    start_date: String,
    end_date: String,
    include_sent: bool,
) -> Result<Vec<crate::reminders::Reminder>, String> {
    crate::reminders::get_reminders(&start_date, &end_date, include_sent)
        .map_err(|e| e.to_string_for_tauri())
}

/// Get reminders that are due (should be sent now)
#[tauri::command]
pub async fn get_due_reminders() -> Result<Vec<crate::reminders::Reminder>, String> {
    crate::reminders::get_due_reminders()
        .map_err(|e| e.to_string_for_tauri())
}

/// Get reminders for a specific application
#[tauri::command]
pub async fn get_reminders_for_application(
    application_id: i64,
) -> Result<Vec<crate::reminders::Reminder>, String> {
    crate::reminders::get_reminders_for_application(application_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Mark a reminder as sent
#[tauri::command]
pub async fn mark_reminder_sent(reminder_id: i64) -> Result<(), String> {
    crate::reminders::mark_reminder_sent(reminder_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Delete a reminder
#[tauri::command]
pub async fn delete_reminder(reminder_id: i64) -> Result<(), String> {
    crate::reminders::delete_reminder(reminder_id)
        .map_err(|e| e.to_string_for_tauri())
}

// ============================================================================
// Portfolio Export Commands
// ============================================================================

/// Export portfolio as HTML
#[tauri::command]
pub async fn export_portfolio_html(
    include_highlighted_only: bool,
) -> Result<String, String> {
    let conn = get_connection()
        .map_err(|e| CareerBenchError::from(e).to_string_for_tauri())?;

    let mut stmt = conn.prepare(
        "SELECT id, title, url, description, role, tech_stack, highlighted
         FROM portfolio_items WHERE user_profile_id = 1
         ORDER BY highlighted DESC, id DESC"
    )
    .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt.query_map([], |row| {
        Ok(crate::portfolio_export::PortfolioItem {
            id: row.get(0)?,
            title: row.get(1)?,
            url: row.get(2)?,
            description: row.get(3)?,
            role: row.get(4)?,
            tech_stack: row.get(5)?,
            highlighted: row.get::<_, i64>(6)? != 0,
        })
    })
    .map_err(|e| format!("Failed to query portfolio: {}", e))?;

    let mut items = Vec::new();
    for row_result in rows {
        items.push(row_result.map_err(|e| format!("Failed to parse portfolio item: {}", e))?);
    }

    Ok(crate::portfolio_export::export_portfolio_html(&items, include_highlighted_only))
}

/// Export portfolio as Markdown
#[tauri::command]
pub async fn export_portfolio_markdown(
    include_highlighted_only: bool,
) -> Result<String, String> {
    let conn = get_connection()
        .map_err(|e| CareerBenchError::from(e).to_string_for_tauri())?;

    let mut stmt = conn.prepare(
        "SELECT id, title, url, description, role, tech_stack, highlighted
         FROM portfolio_items WHERE user_profile_id = 1
         ORDER BY highlighted DESC, id DESC"
    )
    .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt.query_map([], |row| {
        Ok(crate::portfolio_export::PortfolioItem {
            id: row.get(0)?,
            title: row.get(1)?,
            url: row.get(2)?,
            description: row.get(3)?,
            role: row.get(4)?,
            tech_stack: row.get(5)?,
            highlighted: row.get::<_, i64>(6)? != 0,
        })
    })
    .map_err(|e| format!("Failed to query portfolio: {}", e))?;

    let mut items = Vec::new();
    for row_result in rows {
        items.push(row_result.map_err(|e| format!("Failed to parse portfolio item: {}", e))?);
    }

    Ok(crate::portfolio_export::export_portfolio_markdown(&items, include_highlighted_only))
}

/// Export portfolio as plain text
#[tauri::command]
pub async fn export_portfolio_text(
    include_highlighted_only: bool,
) -> Result<String, String> {
    let conn = get_connection()
        .map_err(|e| CareerBenchError::from(e).to_string_for_tauri())?;

    let mut stmt = conn.prepare(
        "SELECT id, title, url, description, role, tech_stack, highlighted
         FROM portfolio_items WHERE user_profile_id = 1
         ORDER BY highlighted DESC, id DESC"
    )
    .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt.query_map([], |row| {
        Ok(crate::portfolio_export::PortfolioItem {
            id: row.get(0)?,
            title: row.get(1)?,
            url: row.get(2)?,
            description: row.get(3)?,
            role: row.get(4)?,
            tech_stack: row.get(5)?,
            highlighted: row.get::<_, i64>(6)? != 0,
        })
    })
    .map_err(|e| format!("Failed to query portfolio: {}", e))?;

    let mut items = Vec::new();
    for row_result in rows {
        items.push(row_result.map_err(|e| format!("Failed to parse portfolio item: {}", e))?);
    }

    Ok(crate::portfolio_export::export_portfolio_text(&items, include_highlighted_only))
}

/// Get portfolio items linked to an application
#[tauri::command]
pub async fn get_portfolio_for_application(
    application_id: i64,
) -> Result<Vec<crate::portfolio_export::PortfolioItem>, String> {
    crate::portfolio_export::get_portfolio_for_application(application_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Link portfolio items to an application
#[tauri::command]
pub async fn link_portfolio_to_application(
    application_id: i64,
    portfolio_item_ids: Vec<i64>,
) -> Result<(), String> {
    crate::portfolio_export::link_portfolio_to_application(application_id, &portfolio_item_ids)
        .map_err(|e| e.to_string_for_tauri())
}

/// Get applications linked to a portfolio item
#[tauri::command]
pub async fn get_applications_for_portfolio(
    portfolio_item_id: i64,
) -> Result<Vec<i64>, String> {
    crate::portfolio_export::get_applications_for_portfolio(portfolio_item_id)
        .map_err(|e| e.to_string_for_tauri())
}

// ============================================================================
// Analytics Commands
// ============================================================================

/// Get conversion rate analytics
#[tauri::command]
pub async fn get_conversion_rates(
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<crate::analytics::ConversionRates, String> {
    crate::analytics::calculate_conversion_rates(
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Get time-in-stage metrics
#[tauri::command]
pub async fn get_time_in_stage(
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<crate::analytics::TimeInStage>, String> {
    crate::analytics::calculate_time_in_stage(
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Get channel effectiveness analysis
#[tauri::command]
pub async fn get_channel_effectiveness(
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<crate::analytics::ChannelEffectiveness>, String> {
    crate::analytics::analyze_channel_effectiveness(
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Generate AI insights and recommendations
#[tauri::command]
pub async fn get_analytics_insights(
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<crate::analytics::Insight>, String> {
    crate::analytics::generate_insights(
        start_date.as_deref(),
        end_date.as_deref(),
    )
    .map_err(|e| e.to_string_for_tauri())
}

// ============================================================================
// Email Integration Commands
// ============================================================================

/// Save an email account
#[tauri::command]
pub async fn save_email_account(
    account: crate::email::EmailAccount,
) -> Result<i64, String> {
    crate::email::save_email_account(&account)
        .map_err(|e| e.to_string_for_tauri())
}

/// Get all email accounts
#[tauri::command]
pub async fn get_email_accounts() -> Result<Vec<crate::email::EmailAccount>, String> {
    crate::email::get_email_accounts()
        .map_err(|e| e.to_string_for_tauri())
}

/// Delete an email account
#[tauri::command]
pub async fn delete_email_account(account_id: i64) -> Result<(), String> {
    crate::email::delete_email_account(account_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Get email threads for an application
#[tauri::command]
pub async fn get_email_threads_for_application(
    application_id: i64,
) -> Result<Vec<crate::email::EmailThread>, String> {
    crate::email::get_threads_for_application(application_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Link an email thread to an application
#[tauri::command]
pub async fn link_email_thread_to_application(
    thread_id: i64,
    application_id: i64,
) -> Result<(), String> {
    crate::email::link_thread_to_application(thread_id, application_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Get email messages for a thread
#[tauri::command]
pub async fn get_email_messages_for_thread(
    thread_id: i64,
) -> Result<Vec<crate::email::EmailMessage>, String> {
    crate::email::get_messages_for_thread(thread_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Test email account connection (stub - full IMAP implementation would go here)
#[tauri::command]
pub async fn test_email_connection(
    email: String,
    _password: String,
    _provider: String,
) -> Result<String, String> {
    // This is a placeholder - full IMAP connection would be implemented here
    // For now, we'll just validate the email format
    if email.contains('@') && email.contains('.') {
        Ok(format!("Email account '{}' format is valid. IMAP connection testing requires additional setup.", email))
    } else {
        Err("Invalid email format".to_string())
    }
}

/// Sync emails from an account (stub - full IMAP sync would go here)
#[tauri::command]
pub async fn sync_email_account(account_id: i64) -> Result<String, String> {
    // This is a placeholder - full IMAP sync would be implemented here
    // The implementation would:
    // 1. Connect to IMAP server
    // 2. Fetch recent emails
    // 3. Parse emails for application events
    // 4. Create/update email threads and messages
    // 5. Auto-link to applications when possible
    Ok(format!("Email sync for account {} would be implemented here. This requires IMAP library setup and OAuth/app password configuration.", account_id))
}

// ============================================================================
// Learning Plan Commands
// ============================================================================

/// Analyze skill gaps comparing user skills to job requirements
#[tauri::command]
pub async fn analyze_skill_gaps(
    job_id: Option<i64>,
    include_all_jobs: bool,
) -> Result<Vec<crate::learning::SkillGap>, String> {
    crate::learning::analyze_skill_gaps(job_id, include_all_jobs)
        .map_err(|e| e.to_string_for_tauri())
}

/// Create a learning plan from skill gaps
#[tauri::command]
pub async fn create_learning_plan(
    title: String,
    description: Option<String>,
    target_job_id: Option<i64>,
    skill_gaps: Vec<crate::learning::SkillGap>,
    estimated_duration_days: Option<i32>,
) -> Result<i64, String> {
    crate::learning::create_learning_plan(
        title,
        description,
        target_job_id,
        &skill_gaps,
        estimated_duration_days,
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Get all learning plans
#[tauri::command]
pub async fn get_learning_plans(
    status: Option<String>,
) -> Result<Vec<crate::learning::LearningPlan>, String> {
    crate::learning::get_learning_plans(status.as_deref())
        .map_err(|e| e.to_string_for_tauri())
}

/// Get learning tracks for a plan
#[tauri::command]
pub async fn get_learning_tracks(
    learning_plan_id: i64,
) -> Result<Vec<crate::learning::LearningTrack>, String> {
    crate::learning::get_learning_tracks(learning_plan_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Get learning tasks for a track
#[tauri::command]
pub async fn get_learning_tasks(
    learning_track_id: i64,
) -> Result<Vec<crate::learning::LearningTask>, String> {
    crate::learning::get_learning_tasks(learning_track_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Create a learning track
#[tauri::command]
pub async fn create_learning_track(
    learning_plan_id: i64,
    title: String,
    description: Option<String>,
    skill_focus: Option<String>,
    order_index: i32,
) -> Result<i64, String> {
    crate::learning::create_learning_track(
        learning_plan_id,
        title,
        description,
        skill_focus,
        order_index,
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Create a learning task
#[tauri::command]
pub async fn create_learning_task(
    learning_track_id: i64,
    title: String,
    description: Option<String>,
    task_type: String,
    resource_url: Option<String>,
    estimated_hours: Option<i32>,
    due_date: Option<String>,
    order_index: i32,
) -> Result<i64, String> {
    crate::learning::create_learning_task(
        learning_track_id,
        title,
        description,
        task_type,
        resource_url,
        estimated_hours,
        due_date,
        order_index,
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Mark a learning task as completed
#[tauri::command]
pub async fn complete_learning_task(
    task_id: i64,
    completed: bool,
) -> Result<(), String> {
    crate::learning::complete_learning_task(task_id, completed)
        .map_err(|e| e.to_string_for_tauri())
}

/// Add a learning resource
#[tauri::command]
pub async fn add_learning_resource(
    learning_task_id: Option<i64>,
    title: String,
    url: Option<String>,
    resource_type: String,
    description: Option<String>,
) -> Result<i64, String> {
    crate::learning::add_learning_resource(
        learning_task_id,
        title,
        url,
        resource_type,
        description,
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Get learning resources for a task
#[tauri::command]
pub async fn get_learning_resources(
    learning_task_id: i64,
) -> Result<Vec<crate::learning::LearningResource>, String> {
    crate::learning::get_learning_resources(learning_task_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Delete a learning plan
#[tauri::command]
pub async fn delete_learning_plan(plan_id: i64) -> Result<(), String> {
    crate::learning::delete_learning_plan(plan_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Update learning plan status
#[tauri::command]
pub async fn update_learning_plan_status(
    plan_id: i64,
    status: String,
) -> Result<(), String> {
    crate::learning::update_learning_plan_status(plan_id, &status)
        .map_err(|e| e.to_string_for_tauri())
}

/// Generate learning tracks and tasks using AI
#[tauri::command]
pub async fn generate_learning_content(
    learning_plan_id: i64,
    skill_gaps: Vec<crate::learning::SkillGap>,
) -> Result<(), String> {
    crate::learning::generate_learning_content(learning_plan_id, &skill_gaps)
        .await
        .map_err(|e| e.to_string_for_tauri())
}

// ============================================================================
// Recruiter CRM Commands
// ============================================================================

/// Create a new recruiter contact
#[tauri::command]
pub async fn create_recruiter_contact(
    name: String,
    email: Option<String>,
    phone: Option<String>,
    linkedin_url: Option<String>,
    company: Option<String>,
    title: Option<String>,
    notes: Option<String>,
    relationship_strength: Option<String>,
    tags: Option<String>,
) -> Result<i64, String> {
    crate::recruiter_crm::create_recruiter_contact(
        name,
        email,
        phone,
        linkedin_url,
        company,
        title,
        notes,
        relationship_strength,
        tags,
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Get all recruiter contacts
#[tauri::command]
pub async fn get_recruiter_contacts(
    company_filter: Option<String>,
    search_query: Option<String>,
) -> Result<Vec<crate::recruiter_crm::RecruiterContact>, String> {
    crate::recruiter_crm::get_recruiter_contacts(
        company_filter.as_deref(),
        search_query.as_deref(),
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Get a single recruiter contact
#[tauri::command]
pub async fn get_recruiter_contact(
    contact_id: i64,
) -> Result<crate::recruiter_crm::RecruiterContact, String> {
    crate::recruiter_crm::get_recruiter_contact(contact_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Update a recruiter contact
#[tauri::command]
pub async fn update_recruiter_contact(
    contact_id: i64,
    name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    linkedin_url: Option<String>,
    company: Option<String>,
    title: Option<String>,
    notes: Option<String>,
    relationship_strength: Option<String>,
    tags: Option<String>,
) -> Result<(), String> {
    crate::recruiter_crm::update_recruiter_contact(
        contact_id,
        name,
        email,
        phone,
        linkedin_url,
        company,
        title,
        notes,
        relationship_strength,
        tags,
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Delete a recruiter contact
#[tauri::command]
pub async fn delete_recruiter_contact(contact_id: i64) -> Result<(), String> {
    crate::recruiter_crm::delete_recruiter_contact(contact_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Create a new interaction
#[tauri::command]
pub async fn create_interaction(
    contact_id: i64,
    interaction_type: String,
    interaction_date: String,
    subject: Option<String>,
    notes: Option<String>,
    linked_application_id: Option<i64>,
    linked_job_id: Option<i64>,
    outcome: Option<String>,
    follow_up_date: Option<String>,
) -> Result<i64, String> {
    crate::recruiter_crm::create_interaction(
        contact_id,
        interaction_type,
        interaction_date,
        subject,
        notes,
        linked_application_id,
        linked_job_id,
        outcome,
        follow_up_date,
    )
    .map_err(|e| e.to_string_for_tauri())
}

/// Get interactions for a contact
#[tauri::command]
pub async fn get_interactions_for_contact(
    contact_id: i64,
) -> Result<Vec<crate::recruiter_crm::RecruiterInteraction>, String> {
    crate::recruiter_crm::get_interactions_for_contact(contact_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Get interactions for an application
#[tauri::command]
pub async fn get_interactions_for_application(
    application_id: i64,
) -> Result<Vec<crate::recruiter_crm::RecruiterInteraction>, String> {
    crate::recruiter_crm::get_interactions_for_application(application_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Link a contact to an application
#[tauri::command]
pub async fn link_contact_to_application(
    contact_id: i64,
    application_id: i64,
    role: Option<String>,
    notes: Option<String>,
) -> Result<i64, String> {
    crate::recruiter_crm::link_contact_to_application(contact_id, application_id, role, notes)
        .map_err(|e| e.to_string_for_tauri())
}

/// Get contacts linked to an application
#[tauri::command]
pub async fn get_contacts_for_application(
    application_id: i64,
) -> Result<Vec<crate::recruiter_crm::RecruiterContact>, String> {
    crate::recruiter_crm::get_contacts_for_application(application_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Get applications linked to a contact
#[tauri::command]
pub async fn get_applications_for_contact(contact_id: i64) -> Result<Vec<i64>, String> {
    crate::recruiter_crm::get_applications_for_contact(contact_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Unlink a contact from an application
#[tauri::command]
pub async fn unlink_contact_from_application(
    contact_id: i64,
    application_id: i64,
) -> Result<(), String> {
    crate::recruiter_crm::unlink_contact_from_application(contact_id, application_id)
        .map_err(|e| e.to_string_for_tauri())
}

/// Delete an interaction
#[tauri::command]
pub async fn delete_interaction(interaction_id: i64) -> Result<(), String> {
    crate::recruiter_crm::delete_interaction(interaction_id)
        .map_err(|e| e.to_string_for_tauri())
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    
    // Invalidate profile-related caches before saving
    // This ensures resume/cover letter caches are cleared when profile changes
    let _ = crate::ai_cache::ai_cache_invalidate_profile(&conn);

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
        // Invalidate job parsing cache when description changes
        let _ = crate::ai_cache::ai_cache_invalidate_job(&conn, id);
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedJobList {
    pub jobs: Vec<JobSummary>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[tauri::command]
pub async fn get_job_list(
    search: Option<String>,
    active_only: Option<bool>,
    source: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<PaginatedJobList, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;

    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(50).max(1).min(100); // Limit to 100 per page
    let offset = (page - 1) * page_size;

    // Build WHERE clause
    let mut where_clauses = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if active_only.unwrap_or(true) {
        where_clauses.push("is_active = 1".to_string());
    }

    if let Some(source_filter) = &source {
        where_clauses.push("job_source = ?".to_string());
        params.push(source_filter.clone());
    }

    if let Some(search_term) = &search {
        where_clauses.push("(title LIKE ? OR company LIKE ? OR location LIKE ? OR raw_description LIKE ?)".to_string());
        let search_pattern = format!("%{}%", search_term);
        for _ in 0..4 {
            params.push(search_pattern.clone());
        }
    }

    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Get total count
    let count_query = format!("SELECT COUNT(*) FROM jobs {}", where_clause);
    let total: i64 = if params.is_empty() {
        conn.query_row(&count_query, [], |row| row.get(0))
            .map_err(|e| format!("Failed to get total count: {}", e))?
    } else {
        let rusqlite_params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        conn.query_row(&count_query, rusqlite::params_from_iter(rusqlite_params.iter().cloned()), |row| row.get(0))
            .map_err(|e| format!("Failed to get total count: {}", e))?
    };

    // Get paginated results
    let query = format!(
        "SELECT id, title, company, location, seniority, domain_tags, date_added FROM jobs {} ORDER BY date_added DESC LIMIT ? OFFSET ?",
        where_clause
    );

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    // Add limit and offset to params
    let mut all_params = params.clone();
    all_params.push(page_size.to_string());
    all_params.push(offset.to_string());
    
    let rusqlite_params: Vec<&dyn rusqlite::ToSql> = all_params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
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

    let total_pages = if total > 0 {
        ((total as f64 / page_size as f64).ceil() as i64).max(1)
    } else {
        0
    };

    Ok(PaginatedJobList {
        jobs,
        total,
        page,
        page_size,
        total_pages,
    })
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
    use crate::ai::resolver::ResolvedProvider;
    use crate::ai::types::{JobParsingInput, JobMeta};
    
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

    // Step 5: Cache miss - call AI provider using new provider system
    let provider = ResolvedProvider::resolve()
        .map_err(|e| format!("Failed to resolve provider: {}", e))?;
    
    // Build job parsing input
    let parsing_input = JobParsingInput {
        job_description: raw_description.to_string(),
        job_meta: Some(JobMeta {
            source: job.job_source.clone(),
            url: job.posting_url.clone(),
        }),
    };
    
    // Call AI provider
    let parsed_output = provider.as_provider()
        .parse_job(parsing_input)
        .await
        .map_err(|e| {
            // Convert AI provider error to user-friendly message
            use crate::ai::error_messages::get_short_error_message;
            let error_string = e.to_string();
            // Check error patterns and convert to appropriate AiProviderError
            if error_string.contains("Invalid API key") || error_string.contains("InvalidApiKey") || error_string.contains("401") {
                get_short_error_message(&crate::ai::errors::AiProviderError::InvalidApiKey)
            } else if error_string.contains("Rate limit") || error_string.contains("RateLimitExceeded") || error_string.contains("429") {
                get_short_error_message(&crate::ai::errors::AiProviderError::RateLimitExceeded)
            } else if error_string.contains("Network error") || error_string.contains("NetworkError") {
                get_short_error_message(&crate::ai::errors::AiProviderError::NetworkError(error_string.clone()))
            } else if error_string.contains("Invalid response") || error_string.contains("InvalidResponse") {
                get_short_error_message(&crate::ai::errors::AiProviderError::InvalidResponse(error_string.clone()))
            } else {
                format!("AI parsing failed: {}", error_string)
            }
        })?;
    
    // Convert ParsedJobOutput to ParsedJob (they have the same structure)
    let parsed = ParsedJob {
        title_suggestion: parsed_output.title_suggestion,
        company_suggestion: parsed_output.company_suggestion,
        seniority: parsed_output.seniority,
        location: parsed_output.location,
        summary: parsed_output.summary,
        responsibilities: parsed_output.responsibilities,
        required_skills: parsed_output.required_skills,
        nice_to_have_skills: parsed_output.nice_to_have_skills,
        domain_tags: parsed_output.domain_tags,
        seniority_score: parsed_output.seniority_score,
        remote_friendly: parsed_output.remote_friendly,
    };

    // Step 6: Store in cache
    let response_payload = serde_json::to_value(&parsed)
        .map_err(|e| format!("Failed to serialize parsed job: {}", e))?;
    
    // Get model name from settings for cache
    let model_name = crate::ai::settings::load_ai_settings()
        .ok()
        .and_then(|s| s.model_name)
        .unwrap_or_else(|| "unknown-model".to_string());

    ai_cache_put(
        &conn,
        "job_parse",
        &input_hash,
        &model_name,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedApplicationList {
    pub applications: Vec<ApplicationSummary>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[tauri::command]
pub async fn get_applications(
    status: Option<String>,
    job_id: Option<i64>,
    active_only: Option<bool>,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<PaginatedApplicationList, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;

    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(50).max(1).min(100); // Limit to 100 per page
    let offset = (page - 1) * page_size;

    // Build WHERE clause
    let mut where_clauses = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if active_only.unwrap_or(true) {
        where_clauses.push("a.archived = 0".to_string());
    }

    if let Some(status_filter) = &status {
        where_clauses.push("a.status = ?".to_string());
        params.push(status_filter.clone());
    }

    if let Some(job_id_filter) = job_id {
        where_clauses.push("a.job_id = ?".to_string());
        params.push(job_id_filter.to_string());
    }

    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Get total count
    let count_query = format!("SELECT COUNT(*) FROM applications a LEFT JOIN jobs j ON a.job_id = j.id {}", where_clause);
    let total: i64 = if params.is_empty() {
        conn.query_row(&count_query, [], |row| row.get(0))
            .map_err(|e| format!("Failed to get total count: {}", e))?
    } else {
        let rusqlite_params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        conn.query_row(&count_query, rusqlite::params_from_iter(rusqlite_params.iter().cloned()), |row| row.get(0))
            .map_err(|e| format!("Failed to get total count: {}", e))?
    };

    // Get paginated results
    let query = format!(
        "SELECT a.id, a.job_id, j.title, j.company, a.status, a.priority, a.date_saved, a.date_applied, a.last_activity_date FROM applications a LEFT JOIN jobs j ON a.job_id = j.id {} ORDER BY a.date_saved DESC LIMIT ? OFFSET ?",
        where_clause
    );

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    // Add limit and offset to params
    let mut all_params = params.clone();
    all_params.push(page_size.to_string());
    all_params.push(offset.to_string());

    let rusqlite_params: Vec<&dyn rusqlite::ToSql> = all_params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
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

    let total_pages = if total > 0 {
        ((total as f64 / page_size as f64).ceil() as i64).max(1)
    } else {
        0
    };

    Ok(PaginatedApplicationList {
        applications,
        total,
        page,
        page_size,
        total_pages,
    })
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
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LetterGenerationResult {
    pub letter: GeneratedLetter,
    pub content: String,
}

#[tauri::command]
pub async fn generate_resume_for_job(
    job_id: i64,
    _application_id: Option<i64>,
    options: Option<GenerationOptions>,
) -> Result<ResumeGenerationResult, String> {
    use crate::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_RESUME_DAYS};
    use crate::resume_generator::*;
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();

    // Load user profile
    let profile_data = get_user_profile_data().await?;
    if profile_data.profile.is_none() {
        return Err("User profile not found. Please set up your profile first.".to_string());
    }

    // Load job
    let job = get_job_detail(job_id).await?;
    let job_description = job.raw_description.as_deref().unwrap_or("");

    // Parse job if not already parsed
    let parsed_job = if let Some(parsed_json) = &job.parsed_json {
        serde_json::from_str::<ParsedJob>(parsed_json).ok()
    } else {
        None
    };

    // Build canonical request payload for final resume cache
    let request_payload = serde_json::json!({
        "userProfile": profile_data.profile,
        "experience": profile_data.experience,
        "skills": profile_data.skills,
        "education": profile_data.education,
        "job": {
            "title": job.title,
            "company": job.company,
            "rawDescription": job.raw_description,
        },
        "options": options
    });

    // Check final resume cache
    let input_hash = compute_input_hash(&request_payload)
        .map_err(|e| format!("Failed to compute hash: {}", e))?;

    if let Some(cached_entry) = ai_cache_get(&conn, "resume_generation", &input_hash, &now)
        .map_err(|e| format!("Cache lookup error: {}", e))? {
        let resume: GeneratedResume = serde_json::from_value(cached_entry.response_payload)
            .map_err(|e| format!("Failed to deserialize cached response: {}", e))?;
        
        let content = render_resume_to_text(&resume);

        return Ok(ResumeGenerationResult {
            resume,
            content,
        });
    }

    // ============================================================================
    // NEW PIPELINE: Small, focused AI calls + code-based preprocessing
    // ============================================================================

    // Step 1: Summarize job description (small AI call, cached)
    let jd_summary = summarize_job_description(job_description, parsed_job.as_ref()).await?;

    // Step 2: Preprocess and select relevant roles/bullets (code-based, no AI)
    let top_roles = select_top_roles(&profile_data.experience, &jd_summary, 3);
    
    // Step 3: Select top bullets for each role and rewrite them (small AI calls per role)
    let mut experience_sections = Vec::new();
    for mapped_role in &top_roles {
        // Select top bullets for this role
        let selected_bullets = select_top_bullets_for_role(&mapped_role.experience, &jd_summary, 5);
        
        // Rewrite bullets (small AI call per role)
        let rewritten_bullets = rewrite_bullets_for_role(
            &mapped_role.experience.title,
            &mapped_role.experience.company,
            &selected_bullets,
            &jd_summary,
        ).await?;
        
        // Build subheading with dates and location
        let mut subheading = String::new();
        if let Some(start) = &mapped_role.experience.start_date {
            subheading.push_str(&crate::commands::format_date(start));
        }
        if mapped_role.experience.is_current {
            subheading.push_str(" – Present");
        } else if let Some(end) = &mapped_role.experience.end_date {
            subheading.push_str(&format!(" – {}", crate::commands::format_date(end)));
        }
        if let Some(loc) = &mapped_role.experience.location {
            subheading.push_str(&format!(" | {}", loc));
        }
        
        // Create section item with rewritten bullets
        let bullets: Vec<String> = rewritten_bullets.iter()
            .map(|b| b.new_text.clone())
            .collect();
        
        experience_sections.push(ResumeSectionItem {
            heading: format!("{} – {}", mapped_role.experience.title, mapped_role.experience.company),
            subheading: if subheading.is_empty() { None } else { Some(subheading) },
            bullets,
        });
    }

    // Step 4: Generate professional summary (optional small AI call, cached)
    let summary = generate_professional_summary(&profile_data, &jd_summary).await?;

    // Step 5: Select top skills (code-based, no AI)
    let top_skills = select_top_skills(&profile_data.skills, &jd_summary, 10);

    // Step 6: Assemble final resume in code (no AI)
    let mut sections = Vec::new();
    
    // Experience section
    if !experience_sections.is_empty() {
        sections.push(ResumeSection {
            title: "Experience".to_string(),
            items: experience_sections,
        });
    }
    
    // Skills section
    if !top_skills.is_empty() {
        sections.push(ResumeSection {
            title: "Skills".to_string(),
            items: vec![ResumeSectionItem {
                heading: "Key Skills".to_string(),
                subheading: None,
                bullets: vec![top_skills.join(", ")],
            }],
        });
    }
    
    // Education section (raw from user, no AI)
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
    
    // Build headline
    let headline = if let Some(profile) = &profile_data.profile {
        if let Some(title) = &profile.current_role_title {
            format!("{} – {}", profile.full_name, title)
        } else {
            profile.full_name.clone()
        }
    } else {
        "Professional Resume".to_string()
    };
    
    let resume = GeneratedResume {
        summary: Some(summary),
        headline: Some(headline),
        sections,
        highlights: vec![
            format!("Tailored for {} role", jd_summary.role_title.as_deref().unwrap_or("this position")),
            format!("Emphasizes: {}", jd_summary.must_have_skills.join(", ")),
        ],
    };

    // Store in cache
    let response_payload = serde_json::to_value(&resume)
        .map_err(|e| format!("Failed to serialize resume: {}", e))?;
    
    let model_name = crate::ai::settings::load_ai_settings()
        .ok()
        .and_then(|s| s.model_name)
        .unwrap_or_else(|| "unknown-model".to_string());

    ai_cache_put(
        &conn,
        "resume_generation",
        &input_hash,
        &model_name,
        &request_payload,
        &response_payload,
        Some(CACHE_TTL_RESUME_DAYS),
        &now,
    )
    .map_err(|e| format!("Failed to cache result: {}", e))?;

    // Don't create artifact automatically - user will save it if they want
    let content = render_resume_to_text(&resume);

    Ok(ResumeGenerationResult {
        resume,
        content,
    })
}

#[tauri::command]
pub async fn generate_cover_letter_for_job(
    job_id: i64,
    _application_id: Option<i64>,
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

        return Ok(LetterGenerationResult {
            letter,
            content,
        });
    }

    // Cache miss - generate letter using AI provider
    let provider = ResolvedProvider::resolve()
        .map_err(|e| format!("Failed to resolve provider: {}", e))?;
    
    // Build profile data JSON for AI provider
    let profile_json = serde_json::json!({
        "profile": profile_data.profile,
        "experience": profile_data.experience,
        "skills": profile_data.skills,
        "education": profile_data.education,
    });
    
    // Build job description
    let job_description = job.raw_description
        .as_deref()
        .unwrap_or("")
        .to_string();
    
    // Convert GenerationOptions to CoverLetterOptions
    let letter_options = options.as_ref().map(|opt| crate::ai::types::CoverLetterOptions {
        tone: opt.tone.clone(),
        length: opt.length.clone(),
        audience: opt.audience.clone(),
    });
    
    // Call AI provider
    let letter_input = crate::ai::types::CoverLetterInput {
        profile_data: profile_json,
        job_description,
        company_name: job.company.clone(),
        options: letter_options,
    };
    
    let cover_letter = provider.as_provider()
        .generate_cover_letter(letter_input)
        .await
        .map_err(|e| format!("AI generation failed: {}", e))?;

    // Convert CoverLetter to GeneratedLetter (they have the same structure)
    let letter = GeneratedLetter {
        subject: cover_letter.subject,
        greeting: cover_letter.greeting,
        body_paragraphs: cover_letter.body_paragraphs,
        closing: cover_letter.closing,
        signature: cover_letter.signature,
    };

    // Store in cache
    let response_payload = serde_json::to_value(&letter)
        .map_err(|e| format!("Failed to serialize letter: {}", e))?;
    
    // Get model name from settings for cache
    let model_name = crate::ai::settings::load_ai_settings()
        .ok()
        .and_then(|s| s.model_name)
        .unwrap_or_else(|| "unknown-model".to_string());

    ai_cache_put(
        &conn,
        "cover_letter_generation",
        &input_hash,
        &model_name,
        &request_payload,
        &response_payload,
        Some(CACHE_TTL_COVER_LETTER_DAYS),
        &now,
    )
    .map_err(|e| format!("Failed to cache result: {}", e))?;

    // Don't create artifact automatically - user will save it if they want
    let content = render_letter_to_text(&letter);

    Ok(LetterGenerationResult {
        letter,
        content,
    })
}

// Helper function to create a new artifact (always creates new, allows multiple per job)
fn create_artifact(
    conn: &rusqlite::Connection,
    application_id: Option<i64>,
    job_id: Option<i64>,
    artifact_type: &str,
    title: &str,
    content: &str,
    ai_payload: &str,
    now: &str,
) -> Result<i64, String> {
    // Get model name for tracking
    let model_name = crate::ai::settings::load_ai_settings()
        .ok()
        .and_then(|s| s.model_name)
        .unwrap_or_else(|| "unknown-model".to_string());

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
            model_name,
            "ai_generated",
            now,
            now
        ],
    )
    .map_err(|e| format!("Failed to create artifact: {}", e))?;

    Ok(conn.last_insert_rowid())
}

// Placeholder AI generation functions
#[allow(dead_code)]
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

#[allow(dead_code)]
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

pub fn format_date(date_str: &str) -> String {
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

pub fn render_resume_to_text(resume: &GeneratedResume) -> String {
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

pub fn render_letter_to_text(letter: &GeneratedLetter) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date() {
        // Test YYYY-MM format
        assert_eq!(format_date("2024-01"), "Jan 2024");
        assert_eq!(format_date("2023-12"), "Dec 2023");
        assert_eq!(format_date("2022-06"), "Jun 2022");
        
        // Test edge cases
        assert_eq!(format_date("2024-13"), "13 2024"); // Invalid month
        assert_eq!(format_date("2024"), "2024"); // Too short
        assert_eq!(format_date(""), ""); // Empty string
        assert_eq!(format_date("invalid"), "invalid"); // Invalid format
    }

    #[test]
    fn test_render_resume_to_text() {
        let resume = GeneratedResume {
            summary: Some("Test summary".to_string()),
            headline: Some("Test Headline".to_string()),
            sections: vec![
                ResumeSection {
                    title: "Experience".to_string(),
                    items: vec![
                        ResumeSectionItem {
                            heading: "Software Engineer".to_string(),
                            subheading: Some("2020-2024".to_string()),
                            bullets: vec![
                                "Built amazing features".to_string(),
                                "Led a team".to_string(),
                            ],
                        },
                    ],
                },
                ResumeSection {
                    title: "Skills".to_string(),
                    items: vec![
                        ResumeSectionItem {
                            heading: "Programming Languages".to_string(),
                            subheading: None,
                            bullets: vec!["Rust, TypeScript".to_string()],
                        },
                    ],
                },
            ],
            highlights: vec![],
        };

        let text = render_resume_to_text(&resume);
        
        // Check that all components are present
        assert!(text.contains("Test Headline"));
        assert!(text.contains("Test summary"));
        assert!(text.contains("## Experience"));
        assert!(text.contains("### Software Engineer"));
        assert!(text.contains("2020-2024"));
        assert!(text.contains("- Built amazing features"));
        assert!(text.contains("- Led a team"));
        assert!(text.contains("## Skills"));
        assert!(text.contains("### Programming Languages"));
        assert!(text.contains("- Rust, TypeScript"));
    }

    #[test]
    fn test_render_resume_to_text_minimal() {
        // Test with minimal data
        let resume = GeneratedResume {
            summary: None,
            headline: None,
            sections: vec![],
            highlights: vec![],
        };

        let text = render_resume_to_text(&resume);
        assert_eq!(text, "");
    }

    #[test]
    fn test_render_letter_to_text() {
        let letter = GeneratedLetter {
            subject: Some("Application for Software Engineer".to_string()),
            greeting: Some("Dear Hiring Manager,".to_string()),
            body_paragraphs: vec![
                "First paragraph".to_string(),
                "Second paragraph".to_string(),
            ],
            closing: Some("Best regards,".to_string()),
            signature: Some("John Doe".to_string()),
        };

        let text = render_letter_to_text(&letter);
        
        // Check that all components are present
        assert!(text.contains("Subject: Application for Software Engineer"));
        assert!(text.contains("Dear Hiring Manager,"));
        assert!(text.contains("First paragraph"));
        assert!(text.contains("Second paragraph"));
        assert!(text.contains("Best regards,"));
        assert!(text.contains("John Doe"));
    }

    #[test]
    fn test_render_letter_to_text_minimal() {
        // Test with minimal data
        let letter = GeneratedLetter {
            subject: None,
            greeting: None,
            body_paragraphs: vec![],
            closing: None,
            signature: None,
        };

        let text = render_letter_to_text(&letter);
        assert_eq!(text, "");
    }

    #[test]
    fn test_user_profile_serialization_roundtrip() {
        // Ensure camelCase JSON maps correctly and roundtrips
        let json = serde_json::json!({
            "id": 1,
            "full_name": "Test User",
            "headline": "Engineer",
            "location": "Remote",
            "summary": "Summary",
            "current_role_title": "Senior Engineer",
            "current_company": "Example Co",
            "seniority": "Senior",
            "open_to_roles": "Backend",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        });

        let profile: UserProfile = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(profile.full_name, "Test User");
        assert_eq!(profile.headline.as_deref(), Some("Engineer"));

        let serialized = serde_json::to_value(&profile).unwrap();
        // Check snake_case keys are preserved
        assert!(serialized.get("full_name").is_some());
        assert_eq!(serialized.get("full_name").unwrap(), "Test User");
    }

    #[test]
    fn test_application_serialization() {
        // Ensure Application serializes/deserializes without losing fields
        let app = Application {
            id: Some(5),
            job_id: 10,
            status: "Applied".to_string(),
            channel: Some("Referral".to_string()),
            priority: Some("High".to_string()),
            date_saved: "2024-01-01T00:00:00Z".to_string(),
            date_applied: Some("2024-01-02T00:00:00Z".to_string()),
            last_activity_date: Some("2024-01-03T00:00:00Z".to_string()),
            next_action_date: Some("2024-01-10".to_string()),
            next_action_note: Some("Follow up".to_string()),
            notes_summary: Some("Summary".to_string()),
            contact_name: Some("Hiring Manager".to_string()),
            contact_email: Some("hm@example.com".to_string()),
            contact_linkedin: Some("linkedin.com/hm".to_string()),
            location_override: Some("Remote".to_string()),
            offer_compensation: Some("150k".to_string()),
            archived: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-03T00:00:00Z".to_string(),
        };

        let serialized = serde_json::to_value(&app).unwrap();
        assert_eq!(serialized.get("status").unwrap(), "Applied");
        assert_eq!(serialized.get("channel").unwrap(), "Referral");

        let deserialized: Application = serde_json::from_value(serialized).unwrap();
        assert_eq!(deserialized.id, Some(5));
        assert_eq!(deserialized.status, "Applied");
        assert_eq!(deserialized.contact_email.as_deref(), Some("hm@example.com"));
    }
}

// ============================================================================
// AI Provider Commands
// ============================================================================

use crate::ai::resolver::ResolvedProvider;
use crate::ai::types::*;
use crate::ai::settings::AiSettings;

#[tauri::command]
pub async fn ai_resume_suggestions(input: ResumeInput) -> Result<ResumeSuggestions, String> {
    let provider = ResolvedProvider::resolve()
        .map_err(|e| format!("Failed to resolve provider: {}", e))?;
    
    let result = provider.as_provider()
        .generate_resume_suggestions(input)
        .await
        .map_err(|e| format!("AI error: {}", e))?;
    
    Ok(result)
}

#[tauri::command]
pub async fn ai_cover_letter(input: CoverLetterInput) -> Result<CoverLetter, String> {
    let provider = ResolvedProvider::resolve()
        .map_err(|e| format!("Failed to resolve provider: {}", e))?;
    
    let result = provider.as_provider()
        .generate_cover_letter(input)
        .await
        .map_err(|e| format!("AI error: {}", e))?;
    
    Ok(result)
}

#[tauri::command]
pub async fn ai_skill_suggestions(input: SkillSuggestionsInput) -> Result<SkillSuggestions, String> {
    let provider = ResolvedProvider::resolve()
        .map_err(|e| format!("Failed to resolve provider: {}", e))?;
    
    let result = provider.as_provider()
        .generate_skill_suggestions(input)
        .await
        .map_err(|e| format!("AI error: {}", e))?;
    
    Ok(result)
}

#[tauri::command]
pub async fn get_ai_settings() -> Result<AiSettings, String> {
    crate::ai::settings::load_ai_settings()
        .map_err(|e| format!("Failed to load settings: {}", e))
}

#[tauri::command]
pub async fn save_ai_settings(settings: AiSettings) -> Result<(), String> {
    crate::ai::settings::save_ai_settings(&settings)
        .map_err(|e| format!("Failed to save settings: {}", e))
}

/// Rotate the AI API key with validation
#[tauri::command]
pub async fn rotate_api_key(
    new_api_key: String,
    provider: crate::ai::settings::CloudProvider,
) -> Result<(), String> {
    crate::ai::key_rotation::rotate_api_key(&new_api_key, provider).await
}

/// Get API key rotation metadata
#[tauri::command]
pub async fn get_api_key_metadata() -> Result<crate::secure_storage::KeyMetadata, String> {
    crate::ai::key_rotation::get_api_key_metadata()
}

/// Check if API key rotation is needed
#[tauri::command]
pub async fn check_api_key_rotation_needed(max_age_days: Option<u32>) -> Result<Option<u32>, String> {
    crate::ai::key_rotation::check_api_key_rotation_needed(max_age_days)
}

#[tauri::command]
pub async fn test_ai_connection() -> Result<String, String> {
    let provider = ResolvedProvider::resolve()
        .map_err(|e| format!("Failed to resolve provider: {}", e))?;
    
    // Test with a simple skill suggestions request
    let test_input = SkillSuggestionsInput {
        current_skills: vec!["Rust".to_string(), "TypeScript".to_string()],
        job_description: "Looking for a software engineer with Python and React experience.".to_string(),
        experience: None,
    };
    
    match provider.as_provider().generate_skill_suggestions(test_input).await {
        Ok(_) => Ok("Connection successful!".to_string()),
        Err(e) => Err(format!("Connection test failed: {}", e)),
    }
}

#[tauri::command]
pub async fn check_local_provider_availability() -> Result<bool, String> {
    use crate::ai::settings::load_ai_settings;
    use std::path::PathBuf;
    
    let settings = load_ai_settings()
        .map_err(|e| format!("Failed to load settings: {}", e))?;
    
    // Check if local mode is selected
    if settings.mode != crate::ai::settings::AiMode::Local {
        return Ok(false);
    }
    
    // Check if model path is configured
    let model_path = match settings.local_model_path {
        Some(path_str) => PathBuf::from(path_str),
        None => return Ok(false),
    };
    
    // Check if file exists
    Ok(model_path.exists() && model_path.is_file())
}

// ============================================================================
// Artifact Management Commands
// ============================================================================

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Artifact {
    pub id: i64,
    pub application_id: Option<i64>,
    pub job_id: Option<i64>,
    pub r#type: String,
    pub title: String,
    pub content: Option<String>,
    pub format: Option<String>,
    pub ai_payload: Option<String>,
    pub ai_model: Option<String>,
    pub source: Option<String>,
    pub version: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub fn get_artifacts_for_application(application_id: i64) -> Result<Vec<Artifact>, String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    let mut stmt = conn.prepare(
        "SELECT id, application_id, job_id, type, title, content, format, ai_payload, ai_model, source, version, created_at, updated_at
         FROM artifacts
         WHERE application_id = ?
         ORDER BY created_at DESC"
    )
    .map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let artifacts = stmt.query_map([application_id], |row| {
        Ok(Artifact {
            id: row.get(0)?,
            application_id: row.get(1)?,
            job_id: row.get(2)?,
            r#type: row.get(3)?,
            title: row.get(4)?,
            content: row.get(5)?,
            format: row.get(6)?,
            ai_payload: row.get(7)?,
            ai_model: row.get(8)?,
            source: row.get(9)?,
            version: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })
    .map_err(|e| format!("Failed to query artifacts: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect artifacts: {}", e))?;
    
    Ok(artifacts)
}

#[tauri::command]
pub fn get_artifacts_for_job(job_id: i64) -> Result<Vec<Artifact>, String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    let mut stmt = conn.prepare(
        "SELECT id, application_id, job_id, type, title, content, format, ai_payload, ai_model, source, version, created_at, updated_at
         FROM artifacts
         WHERE job_id = ?
         ORDER BY created_at DESC"
    )
    .map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let artifacts = stmt.query_map([job_id], |row| {
        Ok(Artifact {
            id: row.get(0)?,
            application_id: row.get(1)?,
            job_id: row.get(2)?,
            r#type: row.get(3)?,
            title: row.get(4)?,
            content: row.get(5)?,
            format: row.get(6)?,
            ai_payload: row.get(7)?,
            ai_model: row.get(8)?,
            source: row.get(9)?,
            version: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })
    .map_err(|e| format!("Failed to query artifacts: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect artifacts: {}", e))?;
    
    Ok(artifacts)
}

#[tauri::command]
pub fn get_artifact(id: i64) -> Result<Artifact, String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    let artifact = conn.query_row(
        "SELECT id, application_id, job_id, type, title, content, format, ai_payload, ai_model, source, version, created_at, updated_at
         FROM artifacts
         WHERE id = ?",
        [id],
        |row| {
            Ok(Artifact {
                id: row.get(0)?,
                application_id: row.get(1)?,
                job_id: row.get(2)?,
                r#type: row.get(3)?,
                title: row.get(4)?,
                content: row.get(5)?,
                format: row.get(6)?,
                ai_payload: row.get(7)?,
                ai_model: row.get(8)?,
                source: row.get(9)?,
                version: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        },
    )
    .map_err(|e| format!("Failed to get artifact: {}", e))?;
    
    Ok(artifact)
}

#[tauri::command]
pub fn update_artifact(id: i64, content: String) -> Result<Artifact, String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    let now = chrono::Utc::now().to_rfc3339();
    
    conn.execute(
        "UPDATE artifacts SET content = ?, updated_at = ? WHERE id = ?",
        rusqlite::params![content, now, id],
    )
    .map_err(|e| format!("Failed to update artifact: {}", e))?;
    
    get_artifact(id)
}

#[tauri::command]
pub fn update_artifact_title(id: i64, title: String) -> Result<Artifact, String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    let now = chrono::Utc::now().to_rfc3339();
    
    conn.execute(
        "UPDATE artifacts SET title = ?, updated_at = ? WHERE id = ?",
        rusqlite::params![title, now, id],
    )
    .map_err(|e| format!("Failed to update artifact title: {}", e))?;
    
    get_artifact(id)
}

#[tauri::command]
pub async fn save_resume(
    job_id: i64,
    application_id: Option<i64>,
    resume: GeneratedResume,
    title: String,
) -> Result<Artifact, String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    let now = chrono::Utc::now().to_rfc3339();
    let content = render_resume_to_text(&resume);
    let ai_payload = serde_json::to_string(&resume)
        .map_err(|e| format!("Failed to serialize resume: {}", e))?;
    
    let artifact_id = create_artifact(
        &conn,
        application_id,
        Some(job_id),
        "Resume",
        &title,
        &content,
        &ai_payload,
        &now,
    )?;
    
    get_artifact(artifact_id)
}

#[tauri::command]
pub async fn save_cover_letter(
    job_id: i64,
    application_id: Option<i64>,
    letter: GeneratedLetter,
    title: String,
) -> Result<Artifact, String> {
    let conn = get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    let now = chrono::Utc::now().to_rfc3339();
    let content = render_letter_to_text(&letter);
    let ai_payload = serde_json::to_string(&letter)
        .map_err(|e| format!("Failed to serialize letter: {}", e))?;
    
    let artifact_id = create_artifact(
        &conn,
        application_id,
        Some(job_id),
        "CoverLetter",
        &title,
        &content,
        &ai_payload,
        &now,
    )?;
    
    get_artifact(artifact_id)
}


/// Generate AI-assisted professional summary from user profile
#[tauri::command]
pub async fn generate_profile_summary() -> Result<String, String> {
    use crate::ai::resolver::ResolvedProvider;
    use crate::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_RESUME_DAYS};
    use crate::db::get_connection;
    use chrono::Utc;
    
    // Load current profile data
    let profile_data = get_user_profile_data().await
        .map_err(|e| format!("Failed to load profile data: {}", e))?;
    
    let conn = get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    let now = Utc::now().to_rfc3339();
    
    // Build canonical input for caching
    let request_payload = serde_json::json!({
        "operation": "generate_profile_summary",
        "profile": profile_data.profile,
        "experience": profile_data.experience,
        "skills": profile_data.skills,
    });
    
    // Check cache
    let input_hash = compute_input_hash(&request_payload)
        .map_err(|e| format!("Failed to compute hash: {}", e))?;
    
    if let Some(cached_entry) = ai_cache_get(&conn, "profile_summary", &input_hash, &now)
        .map_err(|e| format!("Cache lookup error: {}", e))? {
        // Cache hit - deserialize and return
        let summary: String = serde_json::from_value(cached_entry.response_payload)
            .map_err(|e| format!("Failed to deserialize cached response: {}", e))?;
        return Ok(summary);
    }
    
    // Cache miss - call AI provider
    let provider = ResolvedProvider::resolve()
        .map_err(|e| format!("Failed to resolve provider: {}", e))?;
    
    // Build prompt for summary generation
    let mut profile_context = String::new();
    
    if let Some(profile) = &profile_data.profile {
        if !profile.full_name.is_empty() {
            profile_context.push_str(&format!("Name: {}\n", profile.full_name));
        }
        if let Some(title) = &profile.current_role_title {
            profile_context.push_str(&format!("Current Role: {}\n", title));
        }
        if let Some(company) = &profile.current_company {
            profile_context.push_str(&format!("Current Company: {}\n", company));
        }
        if let Some(headline) = &profile.headline {
            profile_context.push_str(&format!("Headline: {}\n", headline));
        }
    }
    
    // Add experience summary
    if !profile_data.experience.is_empty() {
        profile_context.push_str("\nExperience:\n");
        for exp in &profile_data.experience {
            profile_context.push_str(&format!("- {} at {} ({})\n", 
                exp.title, 
                exp.company,
                if exp.is_current { "Current" } else { "Previous" }
            ));
            if let Some(achievements) = &exp.achievements {
                profile_context.push_str(&format!("  Key achievements: {}\n", achievements));
            }
        }
    }
    
    // Add skills summary
    if !profile_data.skills.is_empty() {
        profile_context.push_str("\nSkills:\n");
        let skill_names: Vec<String> = profile_data.skills.iter()
            .map(|s| s.name.clone())
            .collect();
        profile_context.push_str(&format!("{}\n", skill_names.join(", ")));
    }
    
    let prompt = format!(
        r#"Generate a professional summary (2-6 paragraphs) for this profile. 
The summary should:
- Be concise and impactful
- Highlight key achievements and experience
- Emphasize relevant skills and expertise
- Use a professional, confident tone
- Be tailored for job applications

Profile information:
{}

Return only the summary text, no markdown formatting or additional commentary."#,
        profile_context
    );
    
    let system_prompt = Some("You are a professional resume writer. Generate compelling professional summaries that highlight achievements and expertise.");
    
    let response = provider.as_provider()
        .call_llm(system_prompt, &prompt)
        .await
        .map_err(|e| format!("AI error: {}", e))?;
    
    // Extract text from response (may contain markdown code blocks)
    let summary = extract_json_from_text(&response).trim().to_string();
    
    // If the response looks like JSON, try to parse it
    let final_summary = if summary.starts_with('{') || summary.starts_with('[') {
        // Try to extract text from JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&summary) {
            if let Some(text) = json.get("summary").and_then(|v| v.as_str()) {
                text.to_string()
            } else if let Some(text) = json.as_str() {
                text.to_string()
            } else {
                summary
            }
        } else {
            summary
        }
    } else {
        summary
    };
    
    // Cache the result
    let response_payload = serde_json::to_value(&final_summary)
        .map_err(|e| format!("Failed to serialize response: {}", e))?;
    
    // Get model name from settings for cache
    let model_name = crate::ai::settings::load_ai_settings()
        .ok()
        .and_then(|s| s.model_name)
        .unwrap_or_else(|| "unknown-model".to_string());
    
    ai_cache_put(
        &conn,
        "profile_summary",
        &input_hash,
        &model_name,
        &request_payload,
        &response_payload,
        Some(CACHE_TTL_RESUME_DAYS),
        &now,
    )
    .map_err(|e| format!("Failed to cache result: {}", e))?;
    
    Ok(final_summary)
}

/// Extract skills from experience entries using AI
#[tauri::command]
pub async fn extract_skills_from_experience() -> Result<Vec<String>, String> {
    // TODO: Implement full AI skill extraction using AI provider
    // For now, return a helpful message
    Err("AI skill extraction is not yet fully implemented. This feature requires additional AI provider methods and will be available in a future update.".to_string())
}

/// Rewrite portfolio description using AI
#[tauri::command]
pub async fn rewrite_portfolio_description(_portfolio_id: i64, _description: String) -> Result<String, String> {
    // TODO: Implement full AI portfolio rewriting using AI provider
    // For now, return a helpful message
    Err("AI portfolio rewriting is not yet fully implemented. This feature requires additional AI provider methods and will be available in a future update.".to_string())
}

/// Export all user data as JSON
#[tauri::command]
pub async fn export_all_data() -> Result<String, String> {
    crate::data_export::export_to_json()
}

/// Delete a job and all related data
#[tauri::command]
pub async fn delete_job(job_id: i64) -> Result<(), String> {
    crate::data_deletion::delete_job(job_id)
}

/// Delete an application and all related data
#[tauri::command]
pub async fn delete_application(application_id: i64) -> Result<(), String> {
    crate::data_deletion::delete_application(application_id)
}

/// Delete an artifact
#[tauri::command]
pub async fn delete_artifact(artifact_id: i64) -> Result<(), String> {
    crate::data_deletion::delete_artifact(artifact_id)
}

/// Delete a profile section or specific item
#[tauri::command]
pub async fn delete_profile_section(section: String, item_id: Option<i64>) -> Result<(), String> {
    crate::data_deletion::delete_profile_section(&section, item_id)
}

/// Delete all user data (GDPR "Right to be Forgotten")
/// 
/// WARNING: This is irreversible! All user data will be permanently deleted.
#[tauri::command]
pub async fn delete_all_user_data(include_ai_cache: bool) -> Result<(), String> {
    crate::data_deletion::delete_all_user_data(include_ai_cache)
}

/// Get deletion summary (counts of records that would be deleted)
#[tauri::command]
pub async fn get_deletion_summary() -> Result<crate::data_deletion::DeletionSummary, String> {
    crate::data_deletion::get_deletion_summary()
}

/// Get storage information (verify local-first storage)
#[tauri::command]
pub async fn get_storage_info() -> Result<crate::local_storage::StorageInfo, String> {
    crate::local_storage::get_storage_info()
}

/// Verify that storage is local
#[tauri::command]
pub async fn verify_local_storage() -> Result<bool, String> {
    crate::local_storage::verify_local_storage()
}

/// Get total size of local data storage
#[tauri::command]
pub async fn get_storage_size() -> Result<u64, String> {
    crate::local_storage::get_storage_size()
}

// ============================================================================
// Profile Import Commands
// ============================================================================

/// Extract text from a resume file (PDF, DOCX, or TXT)
#[tauri::command]
pub async fn extract_resume_text(file_path: String) -> Result<crate::profile_import::ParsedResumeText, String> {
    use crate::profile_import::extract_text_from_resume;
    use std::path::PathBuf;
    
    let path = PathBuf::from(&file_path);
    let text = extract_text_from_resume(&path)
        .map_err(|e| e.to_string_for_tauri())?;
    
    Ok(crate::profile_import::ParsedResumeText {
        text,
        file_path,
    })
}

/// Extract profile data from resume text using AI
#[tauri::command]
pub async fn extract_profile_from_resume(resume_text: String) -> Result<crate::profile_import::ExtractedProfileData, String> {
    use crate::ai::resolver::ResolvedProvider;
    use crate::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_RESUME_DAYS};
    use crate::db::get_connection;
    use chrono::Utc;
    
    let conn = get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    let now = Utc::now().to_rfc3339();
    
    // Build canonical input for caching
    let request_payload = serde_json::json!({
        "resumeText": resume_text,
        "operation": "extract_profile"
    });
    
    // Check cache
    let input_hash = compute_input_hash(&request_payload)
        .map_err(|e| format!("Failed to compute hash: {}", e))?;
    
    if let Some(cached_entry) = ai_cache_get(&conn, "profile_extract", &input_hash, &now)
        .map_err(|e| format!("Cache lookup error: {}", e))? {
        // Cache hit - deserialize and return
        let extracted: crate::profile_import::ExtractedProfileData = serde_json::from_value(cached_entry.response_payload)
            .map_err(|e| format!("Failed to deserialize cached response: {}", e))?;
        return Ok(extracted);
    }
    
    // Cache miss - call AI provider
    let provider = ResolvedProvider::resolve()
        .map_err(|e| format!("Failed to resolve provider: {}", e))?;
    
    // Create prompt for profile extraction
    let prompt = format!(
        r#"Extract professional profile information from the following resume text. 
Return a JSON object with the following structure:
{{
  "profile": {{
    "full_name": "string (required)",
    "headline": "string (optional)",
    "location": "string (optional)",
    "summary": "string (optional, professional summary)",
    "current_role_title": "string (optional)",
    "current_company": "string (optional)",
    "seniority": "string (optional: Junior, Mid, Senior, Lead, Manager, Director, VP, C-Level)",
    "open_to_roles": "string (optional, comma-separated)"
  }},
  "experience": [
    {{
      "company": "string (required)",
      "title": "string (required)",
      "location": "string (optional)",
      "start_date": "string (optional, YYYY-MM format)",
      "end_date": "string (optional, YYYY-MM format, or null for current)",
      "is_current": "boolean",
      "description": "string (optional)",
      "achievements": "string (optional, newline-separated bullets)",
      "tech_stack": "string (optional, comma-separated)"
    }}
  ],
  "skills": [
    {{
      "name": "string (required)",
      "category": "string (optional: Technical, Soft, Domain, Tool)",
      "self_rating": "integer (optional, 1-5)",
      "priority": "string (optional: Core, Supporting, Learning)",
      "years_experience": "number (optional)",
      "notes": "string (optional)"
    }}
  ],
  "education": [
    {{
      "institution": "string (required)",
      "degree": "string (optional)",
      "field_of_study": "string (optional)",
      "start_date": "string (optional, YYYY-MM format)",
      "end_date": "string (optional, YYYY-MM format)",
      "grade": "string (optional)",
      "description": "string (optional)"
    }}
  ],
  "certifications": [
    {{
      "name": "string (required)",
      "issuing_organization": "string (optional)",
      "issue_date": "string (optional, YYYY-MM format)",
      "expiration_date": "string (optional, YYYY-MM format)",
      "credential_id": "string (optional)",
      "credential_url": "string (optional)"
    }}
  ],
  "portfolio": [
    {{
      "title": "string (required)",
      "url": "string (optional)",
      "description": "string (optional)",
      "role": "string (optional)",
      "tech_stack": "string (optional, comma-separated)",
      "highlighted": "boolean"
    }}
  ]
}}

Resume text:
{}

Return only valid JSON, no markdown formatting."#,
        resume_text
    );
    
    // Call AI provider using the new generic call_llm method
    let system_prompt = Some("You are a professional profile extraction assistant. Extract structured profile information from resume text. Always return valid JSON matching the specified schema.");
    
    let response = provider.as_provider()
        .call_llm(system_prompt, &prompt)
        .await
        .map_err(|e| format!("AI error: {}", e))?;
    
    // Extract JSON from response (may contain markdown code blocks)
    let json_str = extract_json_from_text(&response);
    
    // Parse JSON response
    let extracted: crate::profile_import::ExtractedProfileData = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse AI response as JSON: {}. Response was: {}", e, json_str))?;
    
    // Cache the result
    let response_payload = serde_json::to_value(&extracted)
        .map_err(|e| format!("Failed to serialize response: {}", e))?;
    
    // Get model name from settings for cache
    let model_name = crate::ai::settings::load_ai_settings()
        .ok()
        .and_then(|s| s.model_name)
        .unwrap_or_else(|| "unknown-model".to_string());
    
    ai_cache_put(
        &conn,
        "profile_extract",
        &input_hash,
        &model_name,
        &request_payload,
        &response_payload,
        Some(CACHE_TTL_RESUME_DAYS),
        &now,
    )
    .map_err(|e| format!("Failed to cache result: {}", e))?;
    
    Ok(extracted)
}

/// Helper function to extract JSON from text (handles markdown code blocks)
fn extract_json_from_text(text: &str) -> String {
    // Try to find JSON object boundaries
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            let json_candidate = &text[start..=end];
            // Try to parse it to validate
            if serde_json::from_str::<serde_json::Value>(json_candidate).is_ok() {
                return json_candidate.to_string();
            }
        }
    }
    
    // If no valid JSON found, try extracting from markdown code blocks
    if let Some(start) = text.find("```json") {
        let after_start = &text[start + 7..];
        if let Some(end) = after_start.find("```") {
            return after_start[..end].trim().to_string();
        }
    }
    
    // Also try ``` without json
    if let Some(start) = text.find("```") {
        let after_start = &text[start + 3..];
        if let Some(end) = after_start.find("```") {
            let candidate = after_start[..end].trim();
            if candidate.starts_with('{') && candidate.ends_with('}') {
                if serde_json::from_str::<serde_json::Value>(candidate).is_ok() {
                    return candidate.to_string();
                }
            }
        }
    }
    
    // Fallback: return the whole text
    text.to_string()
}

// ============================================================================
// Job URL Scraping Commands
// ============================================================================

/// Scrape job data from a URL
#[tauri::command]
pub async fn scrape_job_url(url: String) -> Result<crate::job_scraper::ScrapedJobData, String> {
    crate::job_scraper::scrape_job_url(&url)
        .await
        .map_err(|e| e.to_string_for_tauri())
}

// ============================================================================
// Cache Management Commands
// ============================================================================

/// Get cache statistics
#[tauri::command]
pub async fn get_cache_stats() -> Result<crate::ai_cache::CacheStats, String> {
    use crate::ai_cache::ai_cache_get_stats;
    use crate::db::get_connection;
    use chrono::Utc;
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();
    
    ai_cache_get_stats(&conn, &now)
        .map_err(|e| format!("Failed to get cache stats: {}", e))
}

/// Clear cache by purpose
#[tauri::command]
pub async fn clear_cache_by_purpose(purpose: String) -> Result<u64, String> {
    use crate::ai_cache::ai_cache_clear_purpose;
    use crate::db::get_connection;
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    
    ai_cache_clear_purpose(&conn, &purpose)
        .map_err(|e| format!("Failed to clear cache: {}", e))
}

/// Clear all cache entries
#[tauri::command]
pub async fn clear_all_cache() -> Result<u64, String> {
    use crate::ai_cache::ai_cache_clear_all;
    use crate::db::get_connection;
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    
    ai_cache_clear_all(&conn)
        .map_err(|e| format!("Failed to clear cache: {}", e))
}

/// Cleanup expired cache entries
#[tauri::command]
pub async fn cleanup_expired_cache() -> Result<u64, String> {
    use crate::ai_cache::ai_cache_cleanup_expired;
    use crate::db::get_connection;
    use chrono::Utc;
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = Utc::now().to_rfc3339();
    
    ai_cache_cleanup_expired(&conn, &now)
        .map_err(|e| format!("Failed to cleanup cache: {}", e))
}

/// Evict cache entries to stay under size limit
#[tauri::command]
pub async fn evict_cache_by_size(max_size_mb: u64) -> Result<u64, String> {
    use crate::ai_cache::ai_cache_evict_by_size;
    use crate::db::get_connection;
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let max_size_bytes = max_size_mb * 1024 * 1024;
    
    ai_cache_evict_by_size(&conn, max_size_bytes)
        .map_err(|e| format!("Failed to evict cache: {}", e))
}

/// Evict cache entries to stay under entry count limit
#[tauri::command]
pub async fn evict_cache_by_count(max_entries: u64) -> Result<u64, String> {
    use crate::ai_cache::ai_cache_evict_lru;
    use crate::db::get_connection;
    
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    
    ai_cache_evict_lru(&conn, max_entries)
        .map_err(|e| format!("Failed to evict cache: {}", e))
}
