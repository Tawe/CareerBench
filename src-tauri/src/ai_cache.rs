use rusqlite::Connection;
use serde_json::Value;
use sha2::{Digest, Sha256};
use chrono::DateTime;

pub const CACHE_TTL_JOB_PARSE_DAYS: i64 = 90;
pub const CACHE_TTL_RESUME_DAYS: i64 = 30;
pub const CACHE_TTL_COVER_LETTER_DAYS: i64 = 30;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AiCacheKey {
    pub purpose: String,
    pub input_hash: String,
}

#[derive(Debug, Clone)]
pub struct AiCacheEntry {
    #[allow(dead_code)]
    pub id: i64,
    #[allow(dead_code)]
    pub purpose: String,
    #[allow(dead_code)]
    pub input_hash: String,
    #[allow(dead_code)]
    pub model_name: String,
    #[allow(dead_code)]
    pub request_payload: Value,
    pub response_payload: Value, // This is used in commands.rs
    #[allow(dead_code)]
    pub created_at: String,
    #[allow(dead_code)]
    pub expires_at: Option<String>,
}

pub fn compute_input_hash(json_payload: &Value) -> Result<String, String> {
    let serialized = serde_json::to_string(json_payload)
        .map_err(|e| format!("Failed to serialize cache payload: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn ai_cache_get(
    conn: &Connection,
    purpose: &str,
    input_hash: &str,
    now_iso: &str,
) -> Result<Option<AiCacheEntry>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, purpose, input_hash, model_name, request_payload, response_payload, created_at, expires_at
         FROM ai_cache
         WHERE purpose = ? AND input_hash = ?"
    ).map_err(|e| format!("DB error: {}", e))?;

    let rows = stmt.query_map([purpose, input_hash], |row| {
        let expires_at: Option<String> = row.get(7)?;
        
        // Check expiration
        if let Some(expires_at_str) = &expires_at {
            if expires_at_str.as_str() < now_iso {
                return Ok(None);
            }
        }

        Ok(Some(AiCacheEntry {
            id: row.get(0)?,
            purpose: row.get(1)?,
            input_hash: row.get(2)?,
            model_name: row.get(3)?,
            request_payload: serde_json::from_str(row.get::<_, String>(4)?.as_str())
                .unwrap_or(Value::Null),
            response_payload: serde_json::from_str(row.get::<_, String>(5)?.as_str())
                .unwrap_or(Value::Null),
            created_at: row.get(6)?,
            expires_at,
        }))
    }).map_err(|e| format!("DB error: {}", e))?;

    for row_result in rows {
        if let Ok(Some(entry)) = row_result {
            return Ok(Some(entry));
        }
    }

    Ok(None)
}

pub fn ai_cache_put(
    conn: &Connection,
    purpose: &str,
    input_hash: &str,
    model_name: &str,
    request_payload: &Value,
    response_payload: &Value,
    ttl_days: Option<i64>,
    now_iso: &str,
) -> Result<(), String> {
    let request_json = serde_json::to_string(request_payload)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;
    let response_json = serde_json::to_string(response_payload)
        .map_err(|e| format!("Failed to serialize response: {}", e))?;

    let expires_at = if let Some(days) = ttl_days {
        let now = DateTime::parse_from_rfc3339(now_iso)
            .map_err(|e| format!("Invalid date: {}", e))?;
        let expires = now + chrono::Duration::days(days);
        Some(expires.to_rfc3339())
    } else {
        None
    };

    conn.execute(
        "INSERT INTO ai_cache (purpose, input_hash, model_name, request_payload, response_payload, created_at, expires_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            purpose,
            input_hash,
            model_name,
            request_json,
            response_json,
            now_iso,
            expires_at
        ],
    ).map_err(|e| format!("Failed to insert cache entry: {}", e))?;

    Ok(())
}

#[allow(dead_code)]
pub fn ai_cache_clear_purpose(conn: &Connection, purpose: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM ai_cache WHERE purpose = ?",
        [purpose],
    ).map_err(|e| format!("Failed to clear cache: {}", e))?;
    Ok(())
}

#[allow(dead_code)]
pub fn ai_cache_clear_all(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM ai_cache", [])
        .map_err(|e| format!("Failed to clear cache: {}", e))?;
    Ok(())
}

