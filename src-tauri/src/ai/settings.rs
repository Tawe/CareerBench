use serde::{Deserialize, Serialize};
use crate::db::get_connection;
use crate::secure_storage::{store_secret, get_secret, remove_secret};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub mode: AiMode,
    pub cloud_provider: Option<CloudProvider>,
    pub api_key: Option<String>, // Encrypted when stored in database
    pub model_name: Option<String>,
    pub local_model_path: Option<String>, // Path to local GGUF model file
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum AiMode {
    Local,
    Cloud,
    Hybrid,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "anthropic")]
    Anthropic, // Future support
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            mode: AiMode::Cloud, // Default to Cloud since Local requires model download
            cloud_provider: None,
            api_key: None,
            model_name: None,
            local_model_path: None,
        }
    }
}

/// Load AI settings from database
pub fn load_ai_settings() -> Result<AiSettings, String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    
    // Check if settings table exists, if not create it
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='ai_settings'",
            [],
            |row| Ok(row.get::<_, i64>(0)? > 0),
        )
        .unwrap_or(false);
    
    if !table_exists {
        // Create settings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                mode TEXT NOT NULL DEFAULT 'Cloud',
                cloud_provider TEXT,
                api_key TEXT,
                model_name TEXT,
                local_model_path TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )
        .map_err(|e| format!("Failed to create settings table: {}", e))?;
        
        // Insert default settings
        let default = AiSettings::default();
        conn.execute(
            "INSERT INTO ai_settings (id, mode) VALUES (1, ?)",
            [serde_json::to_string(&default.mode).unwrap_or_else(|_| "Local".to_string())],
        )
        .map_err(|e| format!("Failed to insert default settings: {}", e))?;
        
        return Ok(default);
    }
    
    // Check if local_model_path column exists, if not add it
    let column_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('ai_settings') WHERE name='local_model_path'",
            [],
            |row| Ok(row.get::<_, i64>(0)? > 0),
        )
        .unwrap_or(false);
    
    if !column_exists {
        conn.execute(
            "ALTER TABLE ai_settings ADD COLUMN local_model_path TEXT",
            [],
        )
        .map_err(|e| format!("Failed to add local_model_path column: {}", e))?;
    }
    
    // Load settings
    let mut stmt = conn
        .prepare("SELECT mode, cloud_provider, api_key, model_name, local_model_path FROM ai_settings WHERE id = 1")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;
    
    let settings_result = stmt.query_row([], |row| {
        let mode_str: String = row.get(0)?;
        let cloud_provider_str: Option<String> = row.get(1)?;
        let api_key_encrypted: Option<String> = row.get(2)?;
        let model_name: Option<String> = row.get(3)?;
        let local_model_path: Option<String> = row.get(4)?;
        
        // Try to get API key from secure storage first, then fall back to database
        let api_key = if let Ok(Some(secret)) = get_secret("ai_api_key") {
            Some(secret)
        } else {
            // Fallback to database (for backward compatibility)
            api_key_encrypted.and_then(|encrypted| {
                // Try to decrypt (for old encrypted values)
                crate::encryption::decrypt(&encrypted).ok()
            })
        };
        
        let mode = serde_json::from_str::<AiMode>(&format!("\"{}\"", mode_str))
            .unwrap_or(AiMode::Cloud);
        
        let cloud_provider = cloud_provider_str.and_then(|s| {
            serde_json::from_str::<CloudProvider>(&format!("\"{}\"", s)).ok()
        });
        
        Ok(AiSettings {
            mode,
            cloud_provider,
            api_key,
            model_name,
            local_model_path,
        })
    });
    
    match settings_result {
        Ok(settings) => Ok(settings),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AiSettings::default()),
        Err(e) => Err(format!("Failed to load settings: {}", e)),
    }
}

/// Save AI settings to database
pub fn save_ai_settings(settings: &AiSettings) -> Result<(), String> {
    let conn = get_connection().map_err(|e| format!("DB error: {}", e))?;
    let now = chrono::Utc::now().to_rfc3339();
    
    let mode_str = serde_json::to_string(&settings.mode)
        .map_err(|e| format!("Failed to serialize mode: {}", e))?;
    let cloud_provider_str = settings.cloud_provider.as_ref()
        .and_then(|p| serde_json::to_string(p).ok());
    
    // Store API key in secure storage (OS keychain when available)
    if let Some(api_key) = &settings.api_key {
        store_secret("ai_api_key", api_key)
            .map_err(|e| format!("Failed to store API key in secure storage: {}", e))?;
    } else {
        // Remove API key from secure storage if it's being cleared
        let _ = remove_secret("ai_api_key");
    }
    
    // Store a placeholder in database (for backward compatibility and to indicate key exists)
    // The actual key is stored in secure storage
    let api_key_placeholder = if settings.api_key.is_some() {
        Some("***stored_in_secure_storage***".to_string())
    } else {
        None
    };
    
    conn.execute(
        "INSERT INTO ai_settings (id, mode, cloud_provider, api_key, model_name, local_model_path, updated_at)
         VALUES (1, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            mode = excluded.mode,
            cloud_provider = excluded.cloud_provider,
            api_key = excluded.api_key,
            model_name = excluded.model_name,
            local_model_path = excluded.local_model_path,
            updated_at = excluded.updated_at",
        rusqlite::params![
            mode_str.trim_matches('"'),
            cloud_provider_str.as_ref().map(|s| s.trim_matches('"')),
            api_key_placeholder,
            settings.model_name,
            settings.local_model_path,
            now
        ],
    )
    .map_err(|e| format!("Failed to save settings: {}", e))?;
    
    Ok(())
}

