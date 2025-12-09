//! Recruiter CRM module for managing recruiter contacts, interactions, and relationships

use crate::db::get_connection;
use crate::errors::CareerBenchError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruiterContact {
    pub id: Option<i64>,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub linkedin_url: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub relationship_strength: String,
    pub last_contact_date: Option<String>,
    pub tags: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruiterInteraction {
    pub id: Option<i64>,
    pub contact_id: i64,
    pub interaction_type: String,
    pub interaction_date: String,
    pub subject: Option<String>,
    pub notes: Option<String>,
    pub linked_application_id: Option<i64>,
    pub linked_job_id: Option<i64>,
    pub outcome: Option<String>,
    pub follow_up_date: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ContactApplicationLink {
    pub id: Option<i64>,
    pub contact_id: i64,
    pub application_id: i64,
    pub role: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

/// Create a new recruiter contact
pub fn create_recruiter_contact(
    name: String,
    email: Option<String>,
    phone: Option<String>,
    linkedin_url: Option<String>,
    company: Option<String>,
    title: Option<String>,
    notes: Option<String>,
    relationship_strength: Option<String>,
    tags: Option<String>,
) -> Result<i64, CareerBenchError> {
    let conn = get_connection()?;

    let relationship = relationship_strength.unwrap_or_else(|| "neutral".to_string());

    conn.execute(
        "INSERT INTO recruiter_contacts 
         (name, email, phone, linkedin_url, company, title, notes, relationship_strength, tags, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        rusqlite::params![
            name,
            email,
            phone,
            linkedin_url,
            company,
            title,
            notes,
            relationship,
            tags
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get all recruiter contacts
pub fn get_recruiter_contacts(
    company_filter: Option<&str>,
    search_query: Option<&str>,
) -> Result<Vec<RecruiterContact>, CareerBenchError> {
    let conn = get_connection()?;

    let mut query = "SELECT id, name, email, phone, linkedin_url, company, title, notes, 
                     relationship_strength, last_contact_date, tags, created_at, updated_at
                     FROM recruiter_contacts".to_string();
    let mut params = Vec::new();

    let mut conditions = Vec::new();
    if let Some(company) = company_filter {
        conditions.push("company = ?");
        params.push(company.to_string());
    }
    if let Some(search) = search_query {
        conditions.push("(name LIKE ? OR email LIKE ? OR company LIKE ?)");
        let search_pattern = format!("%{}%", search);
        params.push(search_pattern.clone());
        params.push(search_pattern.clone());
        params.push(search_pattern);
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY last_contact_date DESC NULLS LAST, name ASC");

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(RecruiterContact {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            phone: row.get(3)?,
            linkedin_url: row.get(4)?,
            company: row.get(5)?,
            title: row.get(6)?,
            notes: row.get(7)?,
            relationship_strength: row.get(8)?,
            last_contact_date: row.get(9)?,
            tags: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    let mut contacts = Vec::new();
    for row_result in rows {
        contacts.push(row_result?);
    }

    Ok(contacts)
}

/// Get a single recruiter contact by ID
pub fn get_recruiter_contact(contact_id: i64) -> Result<RecruiterContact, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, name, email, phone, linkedin_url, company, title, notes, 
         relationship_strength, last_contact_date, tags, created_at, updated_at
         FROM recruiter_contacts WHERE id = ?"
    )?;

    let contact = stmt.query_row([contact_id], |row| {
        Ok(RecruiterContact {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            phone: row.get(3)?,
            linkedin_url: row.get(4)?,
            company: row.get(5)?,
            title: row.get(6)?,
            notes: row.get(7)?,
            relationship_strength: row.get(8)?,
            last_contact_date: row.get(9)?,
            tags: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    Ok(contact)
}

/// Update a recruiter contact
pub fn update_recruiter_contact(
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
) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;

    // Get current contact to merge with updates
    let current = get_recruiter_contact(contact_id)?;

    let final_name = name.unwrap_or(current.name);
    let final_email = email.or(current.email);
    let final_phone = phone.or(current.phone);
    let final_linkedin = linkedin_url.or(current.linkedin_url);
    let final_company = company.or(current.company);
    let final_title = title.or(current.title);
    let final_notes = notes.or(current.notes);
    let final_relationship = relationship_strength.unwrap_or(current.relationship_strength);
    let final_tags = tags.or(current.tags);

    conn.execute(
        "UPDATE recruiter_contacts SET 
         name = ?, email = ?, phone = ?, linkedin_url = ?, company = ?, 
         title = ?, notes = ?, relationship_strength = ?, tags = ?, 
         updated_at = datetime('now')
         WHERE id = ?",
        rusqlite::params![
            final_name,
            final_email,
            final_phone,
            final_linkedin,
            final_company,
            final_title,
            final_notes,
            final_relationship,
            final_tags,
            contact_id
        ],
    )?;

    Ok(())
}

/// Delete a recruiter contact
pub fn delete_recruiter_contact(contact_id: i64) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;
    conn.execute("DELETE FROM recruiter_contacts WHERE id = ?", [contact_id])?;
    Ok(())
}

/// Create a new interaction
pub fn create_interaction(
    contact_id: i64,
    interaction_type: String,
    interaction_date: String,
    subject: Option<String>,
    notes: Option<String>,
    linked_application_id: Option<i64>,
    linked_job_id: Option<i64>,
    outcome: Option<String>,
    follow_up_date: Option<String>,
) -> Result<i64, CareerBenchError> {
    let conn = get_connection()?;

    // Update last_contact_date on the contact
    conn.execute(
        "UPDATE recruiter_contacts SET last_contact_date = ?, updated_at = datetime('now') WHERE id = ?",
        [&interaction_date, &contact_id.to_string()],
    )?;

    conn.execute(
        "INSERT INTO recruiter_interactions 
         (contact_id, interaction_type, interaction_date, subject, notes, 
          linked_application_id, linked_job_id, outcome, follow_up_date, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))",
        rusqlite::params![
            contact_id,
            interaction_type,
            interaction_date,
            subject,
            notes,
            linked_application_id,
            linked_job_id,
            outcome,
            follow_up_date
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get interactions for a contact
pub fn get_interactions_for_contact(
    contact_id: i64,
) -> Result<Vec<RecruiterInteraction>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, contact_id, interaction_type, interaction_date, subject, notes,
         linked_application_id, linked_job_id, outcome, follow_up_date, created_at
         FROM recruiter_interactions
         WHERE contact_id = ?
         ORDER BY interaction_date DESC, created_at DESC"
    )?;

    let rows = stmt.query_map([contact_id], |row| {
        Ok(RecruiterInteraction {
            id: row.get(0)?,
            contact_id: row.get(1)?,
            interaction_type: row.get(2)?,
            interaction_date: row.get(3)?,
            subject: row.get(4)?,
            notes: row.get(5)?,
            linked_application_id: row.get(6)?,
            linked_job_id: row.get(7)?,
            outcome: row.get(8)?,
            follow_up_date: row.get(9)?,
            created_at: row.get(10)?,
        })
    })?;

    let mut interactions = Vec::new();
    for row_result in rows {
        interactions.push(row_result?);
    }

    Ok(interactions)
}

/// Get interactions for an application
pub fn get_interactions_for_application(
    application_id: i64,
) -> Result<Vec<RecruiterInteraction>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, contact_id, interaction_type, interaction_date, subject, notes,
         linked_application_id, linked_job_id, outcome, follow_up_date, created_at
         FROM recruiter_interactions
         WHERE linked_application_id = ?
         ORDER BY interaction_date DESC"
    )?;

    let rows = stmt.query_map([application_id], |row| {
        Ok(RecruiterInteraction {
            id: row.get(0)?,
            contact_id: row.get(1)?,
            interaction_type: row.get(2)?,
            interaction_date: row.get(3)?,
            subject: row.get(4)?,
            notes: row.get(5)?,
            linked_application_id: row.get(6)?,
            linked_job_id: row.get(7)?,
            outcome: row.get(8)?,
            follow_up_date: row.get(9)?,
            created_at: row.get(10)?,
        })
    })?;

    let mut interactions = Vec::new();
    for row_result in rows {
        interactions.push(row_result?);
    }

    Ok(interactions)
}

/// Link a contact to an application
pub fn link_contact_to_application(
    contact_id: i64,
    application_id: i64,
    role: Option<String>,
    notes: Option<String>,
) -> Result<i64, CareerBenchError> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT OR REPLACE INTO contact_application_links 
         (contact_id, application_id, role, notes, created_at)
         VALUES (?, ?, ?, ?, datetime('now'))",
        rusqlite::params![contact_id, application_id, role, notes],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get contacts linked to an application
pub fn get_contacts_for_application(
    application_id: i64,
) -> Result<Vec<RecruiterContact>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, c.email, c.phone, c.linkedin_url, c.company, c.title, c.notes,
         c.relationship_strength, c.last_contact_date, c.tags, c.created_at, c.updated_at
         FROM recruiter_contacts c
         INNER JOIN contact_application_links l ON c.id = l.contact_id
         WHERE l.application_id = ?
         ORDER BY c.name"
    )?;

    let rows = stmt.query_map([application_id], |row| {
        Ok(RecruiterContact {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            phone: row.get(3)?,
            linkedin_url: row.get(4)?,
            company: row.get(5)?,
            title: row.get(6)?,
            notes: row.get(7)?,
            relationship_strength: row.get(8)?,
            last_contact_date: row.get(9)?,
            tags: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    let mut contacts = Vec::new();
    for row_result in rows {
        contacts.push(row_result?);
    }

    Ok(contacts)
}

/// Get applications linked to a contact
pub fn get_applications_for_contact(
    contact_id: i64,
) -> Result<Vec<i64>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT application_id FROM contact_application_links WHERE contact_id = ?"
    )?;

    let rows = stmt.query_map([contact_id], |row| {
        row.get::<_, i64>(0)
    })?;

    let mut application_ids = Vec::new();
    for row_result in rows {
        application_ids.push(row_result?);
    }

    Ok(application_ids)
}

/// Unlink a contact from an application
pub fn unlink_contact_from_application(
    contact_id: i64,
    application_id: i64,
) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;
    conn.execute(
        "DELETE FROM contact_application_links WHERE contact_id = ? AND application_id = ?",
        [contact_id, application_id],
    )?;
    Ok(())
}

/// Delete an interaction
pub fn delete_interaction(interaction_id: i64) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;
    conn.execute("DELETE FROM recruiter_interactions WHERE id = ?", [interaction_id])?;
    Ok(())
}
