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

/// Clear all cache entries for a specific purpose
pub fn ai_cache_clear_purpose(conn: &Connection, purpose: &str) -> Result<u64, String> {
    let count = conn.execute(
        "DELETE FROM ai_cache WHERE purpose = ?",
        [purpose],
    ).map_err(|e| format!("Failed to clear cache: {}", e))?;
    Ok(count as u64)
}

/// Clear all cache entries
pub fn ai_cache_clear_all(conn: &Connection) -> Result<u64, String> {
    let count = conn.execute("DELETE FROM ai_cache", [])
        .map_err(|e| format!("Failed to clear cache: {}", e))?;
    Ok(count as u64)
}

/// Clean up expired cache entries
/// Returns the number of entries deleted
pub fn ai_cache_cleanup_expired(conn: &Connection, now_iso: &str) -> Result<u64, String> {
    let count = conn.execute(
        "DELETE FROM ai_cache WHERE expires_at IS NOT NULL AND expires_at < ?",
        [now_iso],
    ).map_err(|e| format!("Failed to cleanup expired cache: {}", e))?;
    Ok(count as u64)
}

/// Invalidate cache entries related to a specific job
/// This clears job_parse entries that might be affected by job updates
/// Note: Currently clears all job_parse entries since we can't easily match them to specific jobs
/// In the future, we could add a job_id field to cache entries or store job_id in request_payload
pub fn ai_cache_invalidate_job(conn: &Connection, _job_id: i64) -> Result<u64, String> {
    ai_cache_clear_purpose(conn, "job_parse")
}

/// Invalidate cache entries related to profile changes
/// This clears resume and cover letter caches that depend on profile data
pub fn ai_cache_invalidate_profile(conn: &Connection) -> Result<u64, String> {
    let mut total = 0u64;
    
    // Clear resume-related caches
    total += ai_cache_clear_purpose(conn, "resume_generation")?;
    total += ai_cache_clear_purpose(conn, "bullet_rewrite")?;
    total += ai_cache_clear_purpose(conn, "professional_summary")?;
    total += ai_cache_clear_purpose(conn, "profile_summary")?;
    
    // Clear cover letter caches
    total += ai_cache_clear_purpose(conn, "cover_letter_generation")?;
    
    Ok(total)
}

/// Get cache statistics
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CacheStats {
    pub total_entries: u64,
    pub total_size_bytes: u64,
    pub entries_by_purpose: std::collections::HashMap<String, u64>,
    pub expired_entries: u64,
    pub oldest_entry: Option<String>,
    pub newest_entry: Option<String>,
}

/// Get statistics about the cache
pub fn ai_cache_get_stats(conn: &Connection, now_iso: &str) -> Result<CacheStats, String> {
    // Total entries
    let total_entries: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ai_cache",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to count entries: {}", e))?;
    
    // Total size (approximate - sum of response_payload lengths)
    let total_size: i64 = conn.query_row(
        "SELECT COALESCE(SUM(LENGTH(response_payload)), 0) FROM ai_cache",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to calculate size: {}", e))?;
    
    // Entries by purpose
    let mut entries_by_purpose = std::collections::HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT purpose, COUNT(*) FROM ai_cache GROUP BY purpose"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;
    
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    }).map_err(|e| format!("Failed to query: {}", e))?;
    
    for row in rows {
        let (purpose, count) = row.map_err(|e| format!("Failed to read row: {}", e))?;
        entries_by_purpose.insert(purpose, count as u64);
    }
    
    // Expired entries
    let expired_entries: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ai_cache WHERE expires_at IS NOT NULL AND expires_at < ?",
        [now_iso],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to count expired: {}", e))?;
    
    // Oldest entry
    let oldest_entry: Option<String> = conn.query_row(
        "SELECT MIN(created_at) FROM ai_cache",
        [],
        |row| row.get(0),
    ).ok();
    
    // Newest entry
    let newest_entry: Option<String> = conn.query_row(
        "SELECT MAX(created_at) FROM ai_cache",
        [],
        |row| row.get(0),
    ).ok();
    
    Ok(CacheStats {
        total_entries: total_entries as u64,
        total_size_bytes: total_size as u64,
        entries_by_purpose,
        expired_entries: expired_entries as u64,
        oldest_entry,
        newest_entry,
    })
}

/// Evict least recently used entries to stay under size limit
/// Uses created_at as a proxy for LRU (oldest entries first)
/// Returns number of entries evicted
pub fn ai_cache_evict_lru(conn: &Connection, max_entries: u64) -> Result<u64, String> {
    // Count current entries
    let current_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ai_cache",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to count entries: {}", e))?;
    
    if current_count <= max_entries as i64 {
        return Ok(0);
    }
    
    // Calculate how many to evict
    let to_evict = (current_count - max_entries as i64) as u64;
    
    // Delete oldest entries (by created_at)
    // Keep the most recent max_entries
    let count = conn.execute(
        "DELETE FROM ai_cache WHERE id IN (
            SELECT id FROM ai_cache ORDER BY created_at ASC LIMIT ?
        )",
        [to_evict as i64],
    ).map_err(|e| format!("Failed to evict entries: {}", e))?;
    
    Ok(count as u64)
}

/// Evict entries to stay under size limit (in bytes)
/// Uses created_at as a proxy for LRU
/// Returns number of entries evicted
pub fn ai_cache_evict_by_size(conn: &Connection, max_size_bytes: u64) -> Result<u64, String> {
    // Get current total size
    let current_size: i64 = conn.query_row(
        "SELECT COALESCE(SUM(LENGTH(response_payload)), 0) FROM ai_cache",
        [],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to calculate size: {}", e))?;
    
    if current_size <= max_size_bytes as i64 {
        return Ok(0);
    }
    
    // Delete oldest entries until we're under the limit
    let mut evicted = 0u64;
    let mut remaining_size = current_size;
    
    // Get entries ordered by age (oldest first)
    let mut stmt = conn.prepare(
        "SELECT id, LENGTH(response_payload) FROM ai_cache ORDER BY created_at ASC"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;
    
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
    }).map_err(|e| format!("Failed to query: {}", e))?;
    
    let mut ids_to_delete = Vec::new();
    for row in rows {
        let (id, size) = row.map_err(|e| format!("Failed to read row: {}", e))?;
        if remaining_size > max_size_bytes as i64 {
            ids_to_delete.push(id);
            remaining_size -= size;
            evicted += 1;
        } else {
            break;
        }
    }
    
    if !ids_to_delete.is_empty() {
        // Delete entries one by one (simple approach)
        // For better performance with many entries, we could batch, but this is simpler
        for id in ids_to_delete {
            conn.execute(
                "DELETE FROM ai_cache WHERE id = ?",
                [id],
            ).map_err(|e| format!("Failed to delete entry {}: {}", id, e))?;
        }
    }
    
    Ok(evicted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use serde_json::json;
    use chrono::Utc;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE ai_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                purpose TEXT NOT NULL,
                input_hash TEXT NOT NULL,
                model_name TEXT NOT NULL,
                request_payload TEXT NOT NULL,
                response_payload TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT
            )",
            [],
        ).unwrap();
        conn
    }

    #[test]
    fn test_compute_input_hash() {
        let payload = json!({"test": "data"});
        let hash1 = compute_input_hash(&payload).unwrap();
        let hash2 = compute_input_hash(&payload).unwrap();
        
        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 produces 64 hex chars
        
        // Different input should produce different hash
        let payload2 = json!({"test": "different"});
        let hash3 = compute_input_hash(&payload2).unwrap();
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_ai_cache_put_and_get() {
        let conn = setup_test_db();
        let now = Utc::now().to_rfc3339();
        
        let purpose = "test_purpose";
        let input_hash = "test_hash";
        let request = json!({"input": "test"});
        let response = json!({"output": "result"});
        
        // Put entry
        ai_cache_put(
            &conn,
            purpose,
            input_hash,
            "test_model",
            &request,
            &response,
            Some(30),
            &now,
        ).unwrap();
        
        // Get entry
        let entry = ai_cache_get(&conn, purpose, input_hash, &now).unwrap();
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.purpose, purpose);
        assert_eq!(entry.input_hash, input_hash);
        assert_eq!(entry.response_payload, response);
    }

    #[test]
    fn test_ai_cache_expiration() {
        let conn = setup_test_db();
        let now = Utc::now().to_rfc3339();
        
        // Create entry that expires in 1 day
        let purpose = "test_purpose";
        let input_hash = "test_hash";
        let request = json!({"input": "test"});
        let response = json!({"output": "result"});
        
        ai_cache_put(
            &conn,
            purpose,
            input_hash,
            "test_model",
            &request,
            &response,
            Some(1),
            &now,
        ).unwrap();
        
        // Should be retrievable now
        let entry = ai_cache_get(&conn, purpose, input_hash, &now).unwrap();
        assert!(entry.is_some());
        
        // Simulate expiration by using a future date
        let future = (Utc::now() + chrono::Duration::days(2)).to_rfc3339();
        let entry = ai_cache_get(&conn, purpose, input_hash, &future).unwrap();
        assert!(entry.is_none());
    }

    #[test]
    fn test_ai_cache_miss() {
        let conn = setup_test_db();
        let now = Utc::now().to_rfc3339();
        
        // Try to get non-existent entry
        let entry = ai_cache_get(&conn, "nonexistent", "hash", &now).unwrap();
        assert!(entry.is_none());
    }
}
