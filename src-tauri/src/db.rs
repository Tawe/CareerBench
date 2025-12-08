use rusqlite::{Connection, Result};
use std::path::PathBuf;

/// Get the app data directory (where database, logs, and models are stored)
/// 
/// This function returns a local directory path where all user data is stored.
/// In development, this is `.careerbench` in the current directory.
/// In production (when running as a Tauri app), this should use Tauri's app data directory.
/// 
/// **Local-First Storage**: All user data (database, logs, secure storage, models)
/// is stored locally on the user's device. No data is sent to external servers
/// except for AI API calls (when using cloud AI providers), and even then,
/// only the prompts are sent, not user data.
pub fn get_app_data_dir() -> PathBuf {
    // In development: use local .careerbench directory
    // In production: should use tauri::path::app_data_dir() when available
    // For now, we use a local path to ensure all data is stored locally
    
    let app_data_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".careerbench");
    std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");
    app_data_dir
}

pub fn get_db_path() -> PathBuf {
    get_app_data_dir().join("careerbench.db")
}

pub fn get_connection() -> Result<Connection> {
    let db_path = get_db_path();
    Connection::open(db_path)
}

pub fn init_database() -> Result<()> {
    let conn = get_connection()?;
    
    // Create migrations table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;

    // Run migrations
    run_migrations(&conn)?;
    
    Ok(())
}

fn run_migrations(conn: &Connection) -> Result<()> {
    // Run migration 001
    let migration_name = "001_initial_schema";
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
    let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
    
    if count == 0 {
        println!("Running migration: {}", migration_name);
        migration_001_initial_schema(conn)?;
        conn.execute(
            "INSERT INTO migrations (name, applied_at) VALUES (?, datetime('now'))",
            [migration_name],
        )?;
    }

    // Run migration 002
    let migration_name = "002_ai_cache";
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
    let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
    
    if count == 0 {
        println!("Running migration: {}", migration_name);
        migration_002_ai_cache(conn)?;
        conn.execute(
            "INSERT INTO migrations (name, applied_at) VALUES (?, datetime('now'))",
            [migration_name],
        )?;
    }

    Ok(())
}

pub fn migration_001_initial_schema(conn: &Connection) -> Result<()> {
            // User profile
            conn.execute(
                "CREATE TABLE IF NOT EXISTS user_profile (
                    id INTEGER PRIMARY KEY,
                    full_name TEXT NOT NULL,
                    headline TEXT,
                    location TEXT,
                    summary TEXT,
                    current_role_title TEXT,
                    current_company TEXT,
                    seniority TEXT,
                    open_to_roles TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
                [],
            )?;

            // Experience
            conn.execute(
                "CREATE TABLE IF NOT EXISTS experience (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_profile_id INTEGER NOT NULL,
                    company TEXT NOT NULL,
                    title TEXT NOT NULL,
                    location TEXT,
                    start_date TEXT,
                    end_date TEXT,
                    is_current INTEGER DEFAULT 0,
                    description TEXT,
                    achievements TEXT,
                    tech_stack TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    FOREIGN KEY (user_profile_id) REFERENCES user_profile(id)
                )",
                [],
            )?;

            // Skills
            conn.execute(
                "CREATE TABLE IF NOT EXISTS skills (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_profile_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    category TEXT,
                    self_rating INTEGER,
                    priority TEXT,
                    years_experience REAL,
                    notes TEXT,
                    FOREIGN KEY (user_profile_id) REFERENCES user_profile(id)
                )",
                [],
            )?;

            // Education
            conn.execute(
                "CREATE TABLE IF NOT EXISTS education (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_profile_id INTEGER NOT NULL,
                    institution TEXT NOT NULL,
                    degree TEXT,
                    field_of_study TEXT,
                    start_date TEXT,
                    end_date TEXT,
                    grade TEXT,
                    description TEXT,
                    FOREIGN KEY (user_profile_id) REFERENCES user_profile(id)
                )",
                [],
            )?;

            // Certifications
            conn.execute(
                "CREATE TABLE IF NOT EXISTS certifications (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_profile_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    issuing_organization TEXT,
                    issue_date TEXT,
                    expiration_date TEXT,
                    credential_id TEXT,
                    credential_url TEXT,
                    FOREIGN KEY (user_profile_id) REFERENCES user_profile(id)
                )",
                [],
            )?;

            // Portfolio items
            conn.execute(
                "CREATE TABLE IF NOT EXISTS portfolio_items (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_profile_id INTEGER NOT NULL,
                    title TEXT NOT NULL,
                    url TEXT,
                    description TEXT,
                    role TEXT,
                    tech_stack TEXT,
                    highlighted INTEGER DEFAULT 0,
                    FOREIGN KEY (user_profile_id) REFERENCES user_profile(id)
                )",
                [],
            )?;

            // Jobs
            conn.execute(
                "CREATE TABLE IF NOT EXISTS jobs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT,
                    company TEXT,
                    location TEXT,
                    job_source TEXT,
                    posting_url TEXT,
                    raw_description TEXT,
                    parsed_json TEXT,
                    seniority TEXT,
                    domain_tags TEXT,
                    is_active INTEGER DEFAULT 1,
                    date_added TEXT NOT NULL,
                    last_updated TEXT NOT NULL
                )",
                [],
            )?;

            // Applications
            conn.execute(
                "CREATE TABLE IF NOT EXISTS applications (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    job_id INTEGER NOT NULL,
                    status TEXT NOT NULL,
                    channel TEXT,
                    priority TEXT,
                    date_saved TEXT NOT NULL,
                    date_applied TEXT,
                    last_activity_date TEXT,
                    next_action_date TEXT,
                    next_action_note TEXT,
                    notes_summary TEXT,
                    contact_name TEXT,
                    contact_email TEXT,
                    contact_linkedin TEXT,
                    location_override TEXT,
                    offer_compensation TEXT,
                    archived INTEGER DEFAULT 0,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    FOREIGN KEY (job_id) REFERENCES jobs(id)
                )",
                [],
            )?;

            // Application events
            conn.execute(
                "CREATE TABLE IF NOT EXISTS application_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    application_id INTEGER NOT NULL,
                    event_type TEXT NOT NULL,
                    event_date TEXT NOT NULL,
                    from_status TEXT,
                    to_status TEXT,
                    title TEXT,
                    details TEXT,
                    created_at TEXT NOT NULL,
                    FOREIGN KEY (application_id) REFERENCES applications(id)
                )",
                [],
            )?;

            // Artifacts
            conn.execute(
                "CREATE TABLE IF NOT EXISTS artifacts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    application_id INTEGER,
                    job_id INTEGER,
                    type TEXT NOT NULL,
                    title TEXT NOT NULL,
                    content TEXT,
                    format TEXT,
                    ai_payload TEXT,
                    ai_model TEXT,
                    source TEXT,
                    version INTEGER,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    FOREIGN KEY (application_id) REFERENCES applications(id),
                    FOREIGN KEY (job_id) REFERENCES jobs(id)
                )",
                [],
            )?;

            Ok(())
}

fn migration_002_ai_cache(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_cache (
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
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ai_cache_purpose_input_hash 
         ON ai_cache (purpose, input_hash)",
        [],
    )?;

    Ok(())
}

