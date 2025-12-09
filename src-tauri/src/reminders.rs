//! Reminder notification system for interviews and events

use crate::db::get_connection;
use crate::errors::CareerBenchError;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: Option<i64>,
    pub application_id: Option<i64>,
    pub event_id: Option<i64>,
    pub reminder_type: String,
    pub reminder_date: String,
    pub message: Option<String>,
    pub is_sent: bool,
    pub sent_at: Option<String>,
    pub created_at: String,
}

/// Create a reminder for an interview or event
pub fn create_reminder(
    application_id: Option<i64>,
    event_id: Option<i64>,
    reminder_type: &str,
    reminder_date: &str,
    message: Option<&str>,
) -> Result<i64, CareerBenchError> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO reminders (application_id, event_id, reminder_type, reminder_date, message, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
        rusqlite::params![application_id, event_id, reminder_type, reminder_date, message],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get all reminders for a date range
pub fn get_reminders(
    start_date: &str,
    end_date: &str,
    include_sent: bool,
) -> Result<Vec<Reminder>, CareerBenchError> {
    let conn = get_connection()?;

    let query = if include_sent {
        "SELECT id, application_id, event_id, reminder_type, reminder_date, message, is_sent, sent_at, created_at
         FROM reminders
         WHERE reminder_date >= ? AND reminder_date <= ?
         ORDER BY reminder_date ASC"
    } else {
        "SELECT id, application_id, event_id, reminder_type, reminder_date, message, is_sent, sent_at, created_at
         FROM reminders
         WHERE reminder_date >= ? AND reminder_date <= ? AND is_sent = 0
         ORDER BY reminder_date ASC"
    };

    let mut stmt = conn.prepare(query)?;
    let rows = stmt.query_map([start_date, end_date], |row| {
        Ok(Reminder {
            id: row.get(0)?,
            application_id: row.get(1)?,
            event_id: row.get(2)?,
            reminder_type: row.get(3)?,
            reminder_date: row.get(4)?,
            message: row.get(5)?,
            is_sent: row.get::<_, i64>(6)? != 0,
            sent_at: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;

    let mut reminders = Vec::new();
    for row_result in rows {
        reminders.push(row_result?);
    }

    Ok(reminders)
}

/// Get reminders that are due (reminder_date <= now and not sent)
pub fn get_due_reminders() -> Result<Vec<Reminder>, CareerBenchError> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, application_id, event_id, reminder_type, reminder_date, message, is_sent, sent_at, created_at
         FROM reminders
         WHERE reminder_date <= ? AND is_sent = 0
         ORDER BY reminder_date ASC"
    )?;

    let rows = stmt.query_map([&now], |row| {
        Ok(Reminder {
            id: row.get(0)?,
            application_id: row.get(1)?,
            event_id: row.get(2)?,
            reminder_type: row.get(3)?,
            reminder_date: row.get(4)?,
            message: row.get(5)?,
            is_sent: row.get::<_, i64>(6)? != 0,
            sent_at: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;

    let mut reminders = Vec::new();
    for row_result in rows {
        reminders.push(row_result?);
    }

    Ok(reminders)
}

/// Mark a reminder as sent
pub fn mark_reminder_sent(reminder_id: i64) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;

    conn.execute(
        "UPDATE reminders SET is_sent = 1, sent_at = datetime('now') WHERE id = ?",
        [reminder_id],
    )?;

    Ok(())
}

/// Delete a reminder
pub fn delete_reminder(reminder_id: i64) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;

    conn.execute("DELETE FROM reminders WHERE id = ?", [reminder_id])?;

    Ok(())
}

/// Get reminders for a specific application
pub fn get_reminders_for_application(application_id: i64) -> Result<Vec<Reminder>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, application_id, event_id, reminder_type, reminder_date, message, is_sent, sent_at, created_at
         FROM reminders
         WHERE application_id = ?
         ORDER BY reminder_date ASC"
    )?;

    let rows = stmt.query_map([application_id], |row| {
        Ok(Reminder {
            id: row.get(0)?,
            application_id: row.get(1)?,
            event_id: row.get(2)?,
            reminder_type: row.get(3)?,
            reminder_date: row.get(4)?,
            message: row.get(5)?,
            is_sent: row.get::<_, i64>(6)? != 0,
            sent_at: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;

    let mut reminders = Vec::new();
    for row_result in rows {
        reminders.push(row_result?);
    }

    Ok(reminders)
}
