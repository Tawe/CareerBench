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

    // Run migration 003 - Database indexes
    let migration_name = "003_database_indexes";
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
    let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
    
    if count == 0 {
        println!("Running migration: {}", migration_name);
        migration_003_database_indexes(conn)?;
        conn.execute(
            "INSERT INTO migrations (name, applied_at) VALUES (?, datetime('now'))",
            [migration_name],
        )?;
    }

    // Run migration 004 - Reminders
    let migration_name = "004_reminders";
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
    let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
    
    if count == 0 {
        println!("Running migration: {}", migration_name);
        migration_004_reminders(conn)?;
        conn.execute(
            "INSERT INTO migrations (name, applied_at) VALUES (?, datetime('now'))",
            [migration_name],
        )?;
    }

    // Run migration 005 - Portfolio Application Links
    let migration_name = "005_portfolio_application_links";
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
    let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
    
    if count == 0 {
        println!("Running migration: {}", migration_name);
        migration_005_portfolio_application_links(conn)?;
        conn.execute(
            "INSERT INTO migrations (name, applied_at) VALUES (?, datetime('now'))",
            [migration_name],
        )?;
    }

    // Run migration 006 - Email Integration
    let migration_name = "006_email_integration";
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
    let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
    
    if count == 0 {
        println!("Running migration: {}", migration_name);
        migration_006_email_integration(conn)?;
        conn.execute(
            "INSERT INTO migrations (name, applied_at) VALUES (?, datetime('now'))",
            [migration_name],
        )?;
    }

    // Run migration 007 - Learning Plans
    let migration_name = "007_learning_plans";
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
    let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
    
    if count == 0 {
        println!("Running migration: {}", migration_name);
        migration_007_learning_plans(conn)?;
        conn.execute(
            "INSERT INTO migrations (name, applied_at) VALUES (?, datetime('now'))",
            [migration_name],
        )?;
    }

    // Run migration 008 - Recruiter CRM
    let migration_name = "008_recruiter_crm";
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
    let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
    
    if count == 0 {
        println!("Running migration: {}", migration_name);
        migration_008_recruiter_crm(conn)?;
        conn.execute(
            "INSERT INTO migrations (name, applied_at) VALUES (?, datetime('now'))",
            [migration_name],
        )?;
    }

    // Run migration 009 - Dashboard Query Optimization Indexes
    let migration_name = "009_dashboard_optimization";
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
    let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
    
    if count == 0 {
        println!("Running migration: {}", migration_name);
        migration_009_dashboard_optimization(conn)?;
        conn.execute(
            "INSERT INTO migrations (name, applied_at) VALUES (?, datetime('now'))",
            [migration_name],
        )?;
    }

    // Run migration 010 - Companies
    let migration_name = "010_companies";
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
    let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
    
    if count == 0 {
        println!("Running migration: {}", migration_name);
        migration_010_companies(conn)?;
        conn.execute(
            "INSERT INTO migrations (name, applied_at) VALUES (?, datetime('now'))",
            [migration_name],
        )?;
    }

    // Run migration 011 - Companies mission/vision/values
    let migration_name = "011_companies_mission_vision_values";
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
    let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
    
    if count == 0 {
        println!("Running migration: {}", migration_name);
        migration_011_companies_mission_vision_values(conn)?;
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

fn migration_003_database_indexes(conn: &Connection) -> Result<()> {
    // Indexes for jobs table - common queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_jobs_is_active_date_added 
         ON jobs (is_active, date_added DESC)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_jobs_company 
         ON jobs (company)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_jobs_job_source 
         ON jobs (job_source)",
        [],
    )?;

    // Indexes for applications table - common queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_applications_job_id 
         ON applications (job_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_applications_status_archived 
         ON applications (status, archived)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_applications_date_saved 
         ON applications (date_saved DESC)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_applications_last_activity 
         ON applications (last_activity_date DESC)",
        [],
    )?;

    // Indexes for application_events table
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_application_events_application_id 
         ON application_events (application_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_application_events_event_date 
         ON application_events (event_date DESC)",
        [],
    )?;

    // Composite index for dashboard activity queries (optimizes UNION ALL query)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_application_events_type_date 
         ON application_events (event_type, event_date DESC)",
        [],
    )?;

    // Indexes for artifacts table
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_artifacts_application_id 
         ON artifacts (application_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_artifacts_job_id 
         ON artifacts (job_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_artifacts_type 
         ON artifacts (type)",
        [],
    )?;

    // Indexes for experience table
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_experience_user_profile_id 
         ON experience (user_profile_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_experience_is_current 
         ON experience (is_current, start_date DESC)",
        [],
    )?;

    // Indexes for skills table
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_skills_user_profile_id 
         ON skills (user_profile_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_skills_category 
         ON skills (category)",
        [],
    )?;

    // Index for ai_cache expiration cleanup
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ai_cache_expires_at 
         ON ai_cache (expires_at) WHERE expires_at IS NOT NULL",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ai_cache_created_at 
         ON ai_cache (created_at ASC)",
        [],
    )?;

    Ok(())
}

pub fn migration_004_reminders(conn: &Connection) -> Result<()> {
    // Reminders table for interview and event notifications
    conn.execute(
        "CREATE TABLE IF NOT EXISTS reminders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            application_id INTEGER,
            event_id INTEGER,
            reminder_type TEXT NOT NULL,
            reminder_date TEXT NOT NULL,
            message TEXT,
            is_sent INTEGER DEFAULT 0,
            sent_at TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (application_id) REFERENCES applications(id),
            FOREIGN KEY (event_id) REFERENCES application_events(id)
        )",
        [],
    )?;

    // Index for querying reminders by date
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reminders_reminder_date 
         ON reminders (reminder_date, is_sent)",
        [],
    )?;

    // Index for application reminders
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reminders_application_id 
         ON reminders (application_id)",
        [],
    )?;

    Ok(())
}

pub fn migration_005_portfolio_application_links(conn: &Connection) -> Result<()> {
    // Junction table to link portfolio items to applications
    conn.execute(
        "CREATE TABLE IF NOT EXISTS application_portfolio_links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            application_id INTEGER NOT NULL,
            portfolio_item_id INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (application_id) REFERENCES applications(id) ON DELETE CASCADE,
            FOREIGN KEY (portfolio_item_id) REFERENCES portfolio_items(id) ON DELETE CASCADE,
            UNIQUE(application_id, portfolio_item_id)
        )",
        [],
    )?;

    // Index for querying portfolio items by application
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_app_portfolio_links_application_id 
         ON application_portfolio_links (application_id)",
        [],
    )?;

    // Index for querying applications by portfolio item
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_app_portfolio_links_portfolio_id 
         ON application_portfolio_links (portfolio_item_id)",
        [],
    )?;

    Ok(())
}

pub fn migration_006_email_integration(conn: &Connection) -> Result<()> {
    // Email accounts table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email_address TEXT NOT NULL UNIQUE,
            provider TEXT NOT NULL,
            imap_server TEXT,
            imap_port INTEGER,
            smtp_server TEXT,
            smtp_port INTEGER,
            use_ssl INTEGER DEFAULT 1,
            is_active INTEGER DEFAULT 1,
            last_sync_at TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Email threads table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_threads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            application_id INTEGER,
            thread_id TEXT NOT NULL,
            subject TEXT,
            participants TEXT,
            last_message_date TEXT,
            message_count INTEGER DEFAULT 1,
            is_archived INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (application_id) REFERENCES applications(id) ON DELETE SET NULL
        )",
        [],
    )?;

    // Email messages table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            thread_id INTEGER NOT NULL,
            email_account_id INTEGER NOT NULL,
            message_id TEXT NOT NULL UNIQUE,
            from_address TEXT,
            to_address TEXT,
            subject TEXT,
            body_text TEXT,
            body_html TEXT,
            received_date TEXT NOT NULL,
            is_read INTEGER DEFAULT 0,
            is_application_event INTEGER DEFAULT 0,
            event_type TEXT,
            extracted_data TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (thread_id) REFERENCES email_threads(id) ON DELETE CASCADE,
            FOREIGN KEY (email_account_id) REFERENCES email_accounts(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_email_threads_application_id 
         ON email_threads (application_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_email_threads_thread_id 
         ON email_threads (thread_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_email_messages_thread_id 
         ON email_messages (thread_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_email_messages_message_id 
         ON email_messages (message_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_email_messages_received_date 
         ON email_messages (received_date DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_email_messages_is_application_event 
         ON email_messages (is_application_event, event_type)",
        [],
    )?;

    Ok(())
}

pub fn migration_007_learning_plans(conn: &Connection) -> Result<()> {
    // Learning plans table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS learning_plans (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            target_job_id INTEGER,
            skill_gaps TEXT,
            estimated_duration_days INTEGER,
            status TEXT DEFAULT 'active',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (target_job_id) REFERENCES jobs(id) ON DELETE SET NULL
        )",
        [],
    )?;

    // Learning tracks table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS learning_tracks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            learning_plan_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            skill_focus TEXT,
            order_index INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (learning_plan_id) REFERENCES learning_plans(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Learning tasks table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS learning_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            learning_track_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            task_type TEXT DEFAULT 'learning',
            resource_url TEXT,
            estimated_hours INTEGER,
            completed INTEGER DEFAULT 0,
            completed_at TEXT,
            due_date TEXT,
            order_index INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (learning_track_id) REFERENCES learning_tracks(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Learning resources table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS learning_resources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            learning_task_id INTEGER,
            title TEXT NOT NULL,
            url TEXT,
            resource_type TEXT DEFAULT 'link',
            description TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (learning_task_id) REFERENCES learning_tasks(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_learning_plans_target_job_id 
         ON learning_plans (target_job_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_learning_plans_status 
         ON learning_plans (status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_learning_tracks_plan_id 
         ON learning_tracks (learning_plan_id, order_index)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_learning_tasks_track_id 
         ON learning_tasks (learning_track_id, order_index)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_learning_tasks_completed 
         ON learning_tasks (completed, due_date)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_learning_resources_task_id 
         ON learning_resources (learning_task_id)",
        [],
    )?;

    Ok(())
}

pub fn migration_008_recruiter_crm(conn: &Connection) -> Result<()> {
    // Recruiter contacts table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS recruiter_contacts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT,
            phone TEXT,
            linkedin_url TEXT,
            company TEXT,
            title TEXT,
            notes TEXT,
            relationship_strength TEXT DEFAULT 'neutral',
            last_contact_date TEXT,
            tags TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Recruiter interactions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS recruiter_interactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contact_id INTEGER NOT NULL,
            interaction_type TEXT NOT NULL,
            interaction_date TEXT NOT NULL,
            subject TEXT,
            notes TEXT,
            linked_application_id INTEGER,
            linked_job_id INTEGER,
            outcome TEXT,
            follow_up_date TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (contact_id) REFERENCES recruiter_contacts(id) ON DELETE CASCADE,
            FOREIGN KEY (linked_application_id) REFERENCES applications(id) ON DELETE SET NULL,
            FOREIGN KEY (linked_job_id) REFERENCES jobs(id) ON DELETE SET NULL
        )",
        [],
    )?;

    // Contact-application relationships table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS contact_application_links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contact_id INTEGER NOT NULL,
            application_id INTEGER NOT NULL,
            role TEXT,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (contact_id) REFERENCES recruiter_contacts(id) ON DELETE CASCADE,
            FOREIGN KEY (application_id) REFERENCES applications(id) ON DELETE CASCADE,
            UNIQUE(contact_id, application_id)
        )",
        [],
    )?;

    // Indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_recruiter_contacts_company 
         ON recruiter_contacts (company)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_recruiter_contacts_last_contact 
         ON recruiter_contacts (last_contact_date DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_recruiter_interactions_contact_id 
         ON recruiter_interactions (contact_id, interaction_date DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_recruiter_interactions_application_id 
         ON recruiter_interactions (linked_application_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_contact_application_links_contact_id 
         ON contact_application_links (contact_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_contact_application_links_application_id 
         ON contact_application_links (application_id)",
        [],
    )?;

    Ok(())
}

pub fn migration_009_dashboard_optimization(conn: &Connection) -> Result<()> {
    // Composite index for dashboard activity queries
    // Optimizes the UNION ALL query that filters by event_type and event_date
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_application_events_type_date 
         ON application_events (event_type, event_date DESC)",
        [],
    )?;

    // Composite index for applications date range queries
    // Optimizes queries that filter by date_saved with status
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_applications_date_saved_status 
         ON applications (date_saved DESC, status)",
        [],
    )?;

    Ok(())
}

pub fn migration_010_companies(conn: &Connection) -> Result<()> {
    // Companies table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS companies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            website TEXT,
            industry TEXT,
            company_size TEXT,
            location TEXT,
            description TEXT,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Add company_id to jobs table
    conn.execute(
        "ALTER TABLE jobs ADD COLUMN company_id INTEGER REFERENCES companies(id) ON DELETE SET NULL",
        [],
    ).ok(); // Ignore error if column already exists

    // Add company_id to applications table (for direct access, though it can also be accessed via jobs)
    conn.execute(
        "ALTER TABLE applications ADD COLUMN company_id INTEGER REFERENCES companies(id) ON DELETE SET NULL",
        [],
    ).ok(); // Ignore error if column already exists

    // Indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_companies_name 
         ON companies (name)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_jobs_company_id 
         ON jobs (company_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_applications_company_id 
         ON applications (company_id)",
        [],
    )?;

    Ok(())
}

pub fn migration_011_companies_mission_vision_values(conn: &Connection) -> Result<()> {
    // Add mission, vision, and values columns to companies table
    conn.execute(
        "ALTER TABLE companies ADD COLUMN mission TEXT",
        [],
    ).ok(); // Ignore error if column already exists

    conn.execute(
        "ALTER TABLE companies ADD COLUMN vision TEXT",
        [],
    ).ok(); // Ignore error if column already exists

    conn.execute(
        "ALTER TABLE companies ADD COLUMN \"values\" TEXT",
        [],
    ).ok(); // Ignore error if column already exists

    Ok(())
}

