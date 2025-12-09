//! Calendar integration module for syncing interviews and events with system calendar

use crate::db::get_connection;
use crate::errors::CareerBenchError;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: Option<i64>,
    pub application_id: i64,
    pub job_title: Option<String>,
    pub company: Option<String>,
    pub event_type: String,
    pub event_date: String,
    pub title: Option<String>,
    pub details: Option<String>,
    pub next_action_date: Option<String>,
    pub next_action_note: Option<String>,
}

/// Get all calendar events (interviews, follow-ups, etc.) for a date range
pub fn get_calendar_events(
    start_date: &str,
    end_date: &str,
) -> Result<Vec<CalendarEvent>, CareerBenchError> {
    let conn = get_connection()?;

    // Get interview events and next action dates from applications
    let mut events = Vec::new();

    // Get interview events from application_events
    let mut stmt = conn.prepare(
        "SELECT 
            e.id,
            e.application_id,
            e.event_type,
            e.event_date,
            e.title,
            e.details,
            j.title as job_title,
            j.company
        FROM application_events e
        JOIN applications a ON e.application_id = a.id
        LEFT JOIN jobs j ON a.job_id = j.id
        WHERE e.event_type IN ('InterviewScheduled', 'InterviewCompleted', 'FollowUpSent', 'OfferReceived')
          AND e.event_date >= ? AND e.event_date <= ?
          AND a.archived = 0
        ORDER BY e.event_date ASC"
    )?;

    let rows = stmt.query_map([start_date, end_date], |row| {
        Ok(CalendarEvent {
            id: row.get(0)?,
            application_id: row.get(1)?,
            job_title: row.get(6)?,
            company: row.get(7)?,
            event_type: row.get(2)?,
            event_date: row.get(3)?,
            title: row.get(4)?,
            details: row.get(5)?,
            next_action_date: None,
            next_action_note: None,
        })
    })?;

    for row_result in rows {
        events.push(row_result?);
    }

    // Get next action dates from applications
    let mut stmt = conn.prepare(
        "SELECT 
            a.id as application_id,
            a.next_action_date,
            a.next_action_note,
            j.title as job_title,
            j.company
        FROM applications a
        LEFT JOIN jobs j ON a.job_id = j.id
        WHERE a.next_action_date IS NOT NULL
          AND a.next_action_date >= ? AND a.next_action_date <= ?
          AND a.archived = 0"
    )?;

    let rows = stmt.query_map([start_date, end_date], |row| {
        Ok(CalendarEvent {
            id: None,
            application_id: row.get(0)?,
            job_title: row.get(3)?,
            company: row.get(4)?,
            event_type: "NextAction".to_string(),
            event_date: row.get::<_, String>(1)?,
            title: Some("Follow-up".to_string()),
            details: row.get(2)?,
            next_action_date: Some(row.get::<_, String>(1)?),
            next_action_note: row.get(2)?,
        })
    })?;

    for row_result in rows {
        events.push(row_result?);
    }

    // Sort by date
    events.sort_by_key(|e| e.event_date.clone());

    Ok(events)
}

/// Get events for a specific date
pub fn get_events_for_date(date: &str) -> Result<Vec<CalendarEvent>, CareerBenchError> {
    get_calendar_events(date, date)
}

/// Sync an interview event to system calendar
/// Returns the calendar event ID if successful, or ICS content as fallback
pub fn sync_interview_to_calendar(
    application_id: i64,
    event_id: Option<i64>,
    title: &str,
    start_time: &str,
    end_time: Option<&str>,
    location: Option<&str>,
    notes: Option<&str>,
) -> Result<String, CareerBenchError> {
    let start_dt = DateTime::parse_from_rfc3339(start_time)
        .map_err(|e| CareerBenchError::Validation(crate::errors::ValidationError::InvalidFormat(
            format!("Invalid start time: {}", e)
        )))?;
    
    let end_dt = if let Some(et) = end_time {
        DateTime::parse_from_rfc3339(et)
            .map_err(|e| CareerBenchError::Validation(crate::errors::ValidationError::InvalidFormat(
                format!("Invalid end time: {}", e)
            )))?
    } else {
        // Default to 1 hour after start
        start_dt + chrono::Duration::hours(1)
    };

    // Try platform-specific calendar sync first
    #[cfg(target_os = "macos")]
    {
        if let Ok(_) = sync_to_macos_calendar(title, &start_dt, &end_dt, location, notes) {
            return Ok(format!("Synced to macOS Calendar: {}", title));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(_) = sync_to_windows_calendar(title, &start_dt, &end_dt, location, notes) {
            return Ok(format!("Synced to Windows Calendar: {}", title));
        }
    }

    // Fallback to ICS file generation
    generate_ics_content(application_id, event_id, title, &start_dt, &end_dt, location, notes)
}

/// Generate ICS file content for calendar import
fn generate_ics_content(
    application_id: i64,
    event_id: Option<i64>,
    title: &str,
    start_dt: &DateTime<chrono::FixedOffset>,
    end_dt: &DateTime<chrono::FixedOffset>,
    location: Option<&str>,
    notes: Option<&str>,
) -> Result<String, CareerBenchError> {
    let ics_content = format!(
        "BEGIN:VCALENDAR\r\n\
        VERSION:2.0\r\n\
        PRODID:-//CareerBench//Interview Calendar//EN\r\n\
        BEGIN:VEVENT\r\n\
        UID:careerbench-interview-{}-{}\r\n\
        DTSTART:{}\r\n\
        DTEND:{}\r\n\
        SUMMARY:{}\r\n\
        {}\
        {}\
        STATUS:CONFIRMED\r\n\
        END:VEVENT\r\n\
        END:VCALENDAR\r\n",
        application_id,
        event_id.unwrap_or(0),
        start_dt.format("%Y%m%dT%H%M%SZ"),
        end_dt.format("%Y%m%dT%H%M%SZ"),
        title,
        if let Some(loc) = location {
            format!("LOCATION:{}\r\n", loc)
        } else {
            String::new()
        },
        if let Some(n) = notes {
            format!("DESCRIPTION:{}\r\n", n.replace("\r\n", "\\n").replace("\n", "\\n"))
        } else {
            String::new()
        }
    );

    Ok(ics_content)
}

/// Sync to macOS Calendar using AppleScript
#[cfg(target_os = "macos")]
fn sync_to_macos_calendar(
    title: &str,
    start_dt: &DateTime<chrono::FixedOffset>,
    end_dt: &DateTime<chrono::FixedOffset>,
    location: Option<&str>,
    notes: Option<&str>,
) -> Result<(), CareerBenchError> {
    use std::process::Command;

    // Convert to local time for AppleScript
    let start_local = start_dt.with_timezone(&chrono::Local);
    let end_local = end_dt.with_timezone(&chrono::Local);

    let start_str = start_local.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_str = end_local.format("%Y-%m-%d %H:%M:%S").to_string();

    let location_str = location.unwrap_or("");
    let notes_str = notes.unwrap_or("");

    // AppleScript to create calendar event
    let script = format!(
        r#"
        tell application "Calendar"
            tell calendar "Home"
                make new event at end with properties {{
                    summary: "{}",
                    start date: date "{}",
                    end date: date "{}",
                    location: "{}",
                    description: "{}"
                }}
            end tell
        end tell
        "#,
        title.replace("\"", "\\\""),
        start_str,
        end_str,
        location_str.replace("\"", "\\\""),
        notes_str.replace("\"", "\\\"")
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| CareerBenchError::Configuration(crate::errors::ConfigurationError::Other(
            format!("Failed to execute AppleScript: {}", e)
        )))?;

    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(CareerBenchError::Configuration(crate::errors::ConfigurationError::Other(
            format!("Calendar sync failed: {}", error)
        )))
    }
}

/// Sync to Windows Calendar using PowerShell
#[cfg(target_os = "windows")]
fn sync_to_windows_calendar(
    title: &str,
    start_dt: &DateTime<chrono::FixedOffset>,
    end_dt: &DateTime<chrono::FixedOffset>,
    location: Option<&str>,
    notes: Option<&str>,
) -> Result<(), CareerBenchError> {
    use std::process::Command;

    // Convert to local time
    let start_local = start_dt.with_timezone(&chrono::Local);
    let end_local = end_dt.with_timezone(&chrono::Local);

    let start_str = start_local.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_str = end_local.format("%Y-%m-%d %H:%M:%S").to_string();

    let location_str = location.unwrap_or("");
    let notes_str = notes.unwrap_or("");

    // PowerShell command to create calendar event
    let ps_command = format!(
        r#"$start = Get-Date "{}"; $end = Get-Date "{}"; $appt = New-Object -ComObject Outlook.Application; $apptItem = $appt.CreateItem(1); $apptItem.Subject = "{}"; $apptItem.Start = $start; $apptItem.End = $end; $apptItem.Location = "{}"; $apptItem.Body = "{}"; $apptItem.Save()"#,
        start_str, end_str, title, location_str, notes_str
    );

    let output = Command::new("powershell")
        .arg("-Command")
        .arg(&ps_command)
        .output()
        .map_err(|e| CareerBenchError::Configuration(crate::errors::ConfigurationError::Other(
            format!("Failed to execute PowerShell: {}", e)
        )))?;

    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(CareerBenchError::Configuration(crate::errors::ConfigurationError::Other(
            format!("Calendar sync failed: {}", error)
        )))
    }
}

