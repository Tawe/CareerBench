//! Email integration module for connecting to email providers and tracking application-related emails

use crate::db::get_connection;
use crate::errors::CareerBenchError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailAccount {
    pub id: Option<i64>,
    pub email_address: String,
    pub provider: String,
    pub imap_server: Option<String>,
    pub imap_port: Option<i32>,
    pub smtp_server: Option<String>,
    pub smtp_port: Option<i32>,
    pub use_ssl: bool,
    pub is_active: bool,
    pub last_sync_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailThread {
    pub id: Option<i64>,
    pub application_id: Option<i64>,
    pub thread_id: String,
    pub subject: Option<String>,
    pub participants: Option<String>,
    pub last_message_date: Option<String>,
    pub message_count: i64,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailMessage {
    pub id: Option<i64>,
    pub thread_id: i64,
    pub email_account_id: i64,
    pub message_id: String,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub subject: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub received_date: String,
    pub is_read: bool,
    pub is_application_event: bool,
    pub event_type: Option<String>,
    pub extracted_data: Option<String>,
    pub created_at: String,
}

/// Get IMAP server settings for common email providers
#[allow(dead_code)]
pub fn get_provider_settings(provider: &str, email: &str) -> (String, u16, bool) {
    match provider.to_lowercase().as_str() {
        "gmail" => ("imap.gmail.com".to_string(), 993, true),
        "outlook" | "hotmail" | "live" => ("outlook.office365.com".to_string(), 993, true),
        "yahoo" => ("imap.mail.yahoo.com".to_string(), 993, true),
        "icloud" => ("imap.mail.me.com".to_string(), 993, true),
        _ => {
            // Try to extract domain and use generic IMAP
            if let Some(domain) = email.split('@').nth(1) {
                (format!("imap.{}", domain), 993, true)
            } else {
                ("imap.example.com".to_string(), 993, true)
            }
        }
    }
}

/// Create or update an email account
pub fn save_email_account(account: &EmailAccount) -> Result<i64, CareerBenchError> {
    let conn = get_connection()?;

    if let Some(id) = account.id {
        // Update existing account
        conn.execute(
            "UPDATE email_accounts 
             SET email_address = ?, provider = ?, imap_server = ?, imap_port = ?, 
                 smtp_server = ?, smtp_port = ?, use_ssl = ?, is_active = ?, 
                 updated_at = datetime('now')
             WHERE id = ?",
            rusqlite::params![
                account.email_address,
                account.provider,
                account.imap_server,
                account.imap_port,
                account.smtp_server,
                account.smtp_port,
                account.use_ssl as i64,
                account.is_active as i64,
                id
            ],
        )?;
        Ok(id)
    } else {
        // Insert new account
        conn.execute(
            "INSERT INTO email_accounts 
             (email_address, provider, imap_server, imap_port, smtp_server, smtp_port, use_ssl, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
            rusqlite::params![
                account.email_address,
                account.provider,
                account.imap_server,
                account.imap_port,
                account.smtp_server,
                account.smtp_port,
                account.use_ssl as i64,
                account.is_active as i64
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

/// Get all email accounts
pub fn get_email_accounts() -> Result<Vec<EmailAccount>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, email_address, provider, imap_server, imap_port, smtp_server, smtp_port, 
         use_ssl, is_active, last_sync_at, created_at, updated_at
         FROM email_accounts
         ORDER BY created_at DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(EmailAccount {
            id: row.get(0)?,
            email_address: row.get(1)?,
            provider: row.get(2)?,
            imap_server: row.get(3)?,
            imap_port: row.get(4)?,
            smtp_server: row.get(5)?,
            smtp_port: row.get(6)?,
            use_ssl: row.get::<_, i64>(7)? != 0,
            is_active: row.get::<_, i64>(8)? != 0,
            last_sync_at: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;

    let mut accounts = Vec::new();
    for row_result in rows {
        accounts.push(row_result?);
    }

    Ok(accounts)
}

/// Delete an email account
pub fn delete_email_account(account_id: i64) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;
    conn.execute("DELETE FROM email_accounts WHERE id = ?", [account_id])?;
    Ok(())
}

/// Parse email content to extract application-related events
#[allow(dead_code)]
pub fn parse_email_for_events(
    subject: &str,
    body: &str,
    from: &str,
) -> Option<(String, HashMap<String, String>)> {
    let subject_lower = subject.to_lowercase();
    let body_lower = body.to_lowercase();
    let _from_lower = from.to_lowercase();

    let mut extracted_data = HashMap::new();
    extracted_data.insert("from".to_string(), from.to_string());
    extracted_data.insert("subject".to_string(), subject.to_string());

    // Check for interview scheduling
    if subject_lower.contains("interview") || body_lower.contains("interview scheduled") {
        // Try to extract date/time
        if let Some(date_match) = extract_date(&body) {
            extracted_data.insert("interview_date".to_string(), date_match);
        }
        return Some(("InterviewScheduled".to_string(), extracted_data));
    }

    // Check for offer
    if subject_lower.contains("offer") || body_lower.contains("job offer") || body_lower.contains("congratulations") {
        if let Some(date_match) = extract_date(&body) {
            extracted_data.insert("offer_date".to_string(), date_match);
        }
        return Some(("OfferReceived".to_string(), extracted_data));
    }

    // Check for rejection
    if subject_lower.contains("reject") || body_lower.contains("not moving forward") || body_lower.contains("unfortunately") {
        return Some(("Rejected".to_string(), extracted_data));
    }

    // Check for follow-up requests
    if subject_lower.contains("follow up") || body_lower.contains("next steps") {
        return Some(("FollowUpSent".to_string(), extracted_data));
    }

    None
}

/// Simple date extraction from text (basic implementation)
#[allow(dead_code)]
fn extract_date(text: &str) -> Option<String> {
    // Look for common date patterns
    let patterns = vec![
        r"\d{1,2}/\d{1,2}/\d{4}",
        r"\d{1,2}-\d{1,2}-\d{4}",
        r"[A-Z][a-z]+ \d{1,2}, \d{4}",
    ];

    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(captures) = re.find(text) {
                return Some(captures.as_str().to_string());
            }
        }
    }

    None
}

/// Link an email thread to an application
pub fn link_thread_to_application(
    thread_id: i64,
    application_id: i64,
) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE email_threads SET application_id = ?, updated_at = datetime('now') WHERE id = ?",
        [application_id, thread_id],
    )?;
    Ok(())
}

/// Get email threads for an application
pub fn get_threads_for_application(
    application_id: i64,
) -> Result<Vec<EmailThread>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, application_id, thread_id, subject, participants, last_message_date, 
         message_count, is_archived, created_at, updated_at
         FROM email_threads
         WHERE application_id = ?
         ORDER BY last_message_date DESC"
    )?;

    let rows = stmt.query_map([application_id], |row| {
        Ok(EmailThread {
            id: row.get(0)?,
            application_id: row.get(1)?,
            thread_id: row.get(2)?,
            subject: row.get(3)?,
            participants: row.get(4)?,
            last_message_date: row.get(5)?,
            message_count: row.get(6)?,
            is_archived: row.get::<_, i64>(7)? != 0,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })?;

    let mut threads = Vec::new();
    for row_result in rows {
        threads.push(row_result?);
    }

    Ok(threads)
}

/// Get email messages for a thread
pub fn get_messages_for_thread(
    thread_id: i64,
) -> Result<Vec<EmailMessage>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, thread_id, email_account_id, message_id, from_address, to_address, 
         subject, body_text, body_html, received_date, is_read, is_application_event, 
         event_type, extracted_data, created_at
         FROM email_messages
         WHERE thread_id = ?
         ORDER BY received_date ASC"
    )?;

    let rows = stmt.query_map([thread_id], |row| {
        Ok(EmailMessage {
            id: row.get(0)?,
            thread_id: row.get(1)?,
            email_account_id: row.get(2)?,
            message_id: row.get(3)?,
            from_address: row.get(4)?,
            to_address: row.get(5)?,
            subject: row.get(6)?,
            body_text: row.get(7)?,
            body_html: row.get(8)?,
            received_date: row.get(9)?,
            is_read: row.get::<_, i64>(10)? != 0,
            is_application_event: row.get::<_, i64>(11)? != 0,
            event_type: row.get(12)?,
            extracted_data: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;

    let mut messages = Vec::new();
    for row_result in rows {
        messages.push(row_result?);
    }

    Ok(messages)
}
