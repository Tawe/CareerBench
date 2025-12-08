//! Script to populate the CareerBench database with test data
//! 
//! Usage:
//!   cargo run --bin populate_test_data
//! 
//! Or compile and run:
//!   cargo build --bin populate_test_data
//!   ./target/debug/populate_test_data

use rusqlite::Connection;
use std::path::PathBuf;
use chrono::Utc;

fn get_db_path() -> PathBuf {
    // When run from src-tauri directory, current_dir is already src-tauri
    // So we just need .careerbench/careerbench.db
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    
    // Check if we're in src-tauri (has Cargo.toml) or project root
    if current_dir.join("Cargo.toml").exists() && current_dir.join("src").exists() {
        // We're in src-tauri directory
        current_dir.join(".careerbench").join("careerbench.db")
    } else {
        // We're in project root, need to go to src-tauri
        current_dir.join("src-tauri").join(".careerbench").join("careerbench.db")
    }
}

fn clear_database(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    println!("Clearing existing data...");
    
    // Delete in reverse order of dependencies
    conn.execute("DELETE FROM artifacts", [])?;
    conn.execute("DELETE FROM application_events", [])?;
    conn.execute("DELETE FROM applications", [])?;
    conn.execute("DELETE FROM jobs", [])?;
    conn.execute("DELETE FROM portfolio_items", [])?;
    conn.execute("DELETE FROM certifications", [])?;
    conn.execute("DELETE FROM education", [])?;
    conn.execute("DELETE FROM skills", [])?;
    conn.execute("DELETE FROM experience", [])?;
    conn.execute("DELETE FROM user_profile", [])?;
    conn.execute("DELETE FROM ai_cache", [])?;
    
    println!("Database cleared.");
    Ok(())
}

fn populate_test_data(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().to_rfc3339();
    
    println!("Populating test data...");
    
    // 1. User Profile
    println!("  - Creating user profile...");
    conn.execute(
        "INSERT INTO user_profile (id, full_name, headline, location, summary, current_role_title, current_company, seniority, open_to_roles, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            1,
            "Alex Johnson",
            "Senior Full-Stack Engineer | React, Node.js, TypeScript, AWS",
            "San Francisco, CA",
            "Experienced full-stack engineer with 8+ years building scalable web applications. Passionate about clean code, test-driven development, and mentoring junior developers. Led multiple successful product launches and have a track record of improving system performance by 40%+.",
            "Senior Software Engineer",
            "TechCorp Inc.",
            "Senior",
            "Senior Engineer, Tech Lead, Engineering Manager",
            now,
            now
        ],
    )?;

    // 2. Experience
    println!("  - Adding experience entries...");
    let experiences = vec![
        (
            "TechCorp Inc.",
            "Senior Software Engineer",
            "San Francisco, CA",
            "2021-01",
            "",
            1, // is_current
            "Leading development of customer-facing web applications using React and Node.js",
            "• Architected and implemented microservices architecture, reducing API response time by 45%\n• Mentored 3 junior engineers, improving team velocity by 30%\n• Led migration from legacy codebase to TypeScript, reducing bugs by 60%",
            "React, TypeScript, Node.js, AWS, Docker, Kubernetes"
        ),
        (
            "StartupXYZ",
            "Full-Stack Engineer",
            "Remote",
            "2019-06",
            "2020-12",
            0,
            "Built and maintained core product features for B2B SaaS platform",
            "• Developed real-time collaboration features using WebSockets\n• Implemented CI/CD pipeline reducing deployment time from 2 hours to 15 minutes\n• Designed and built RESTful APIs serving 1M+ requests per day",
            "React, Python, Django, PostgreSQL, Redis, AWS"
        ),
        (
            "BigTech Co.",
            "Software Engineer",
            "Seattle, WA",
            "2017-08",
            "2019-05",
            0,
            "Developed internal tools and services for engineering teams",
            "• Built automated testing framework used by 50+ engineers\n• Optimized database queries, improving page load times by 35%\n• Contributed to open-source projects with 10K+ GitHub stars",
            "Java, Spring Boot, MySQL, Docker, Jenkins"
        ),
    ];

    for (company, title, location, start_date, end_date, is_current, description, achievements, tech_stack) in experiences {
        conn.execute(
            "INSERT INTO experience (user_profile_id, company, title, location, start_date, end_date, is_current, description, achievements, tech_stack, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                1,
                company,
                title,
                location,
                start_date,
                end_date,
                is_current,
                description,
                achievements,
                tech_stack,
                now,
                now
            ],
        )?;
    }

    // 3. Skills
    println!("  - Adding skills...");
    let skills = vec![
        ("React", "Technical", 5, "Core", 6.0),
        ("TypeScript", "Technical", 5, "Core", 5.0),
        ("Node.js", "Technical", 4, "Core", 5.0),
        ("AWS", "Technical", 4, "Core", 4.0),
        ("Docker", "Tool", 4, "Supporting", 3.0),
        ("Kubernetes", "Tool", 3, "Supporting", 2.0),
        ("Leadership", "Soft", 4, "Core", 4.0),
        ("Mentoring", "Soft", 4, "Supporting", 3.0),
        ("Python", "Technical", 3, "Supporting", 2.0),
        ("PostgreSQL", "Technical", 4, "Supporting", 3.0),
    ];

    for (name, category, rating, priority, years) in skills {
        conn.execute(
            "INSERT INTO skills (user_profile_id, name, category, self_rating, priority, years_experience) VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![1, name, category, rating, priority, years],
        )?;
    }

    // 4. Education
    println!("  - Adding education...");
    conn.execute(
        "INSERT INTO education (user_profile_id, institution, degree, field_of_study, start_date, end_date, grade, description) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            1,
            "University of California, Berkeley",
            "Bachelor of Science",
            "Computer Science",
            "2013-09",
            "2017-05",
            "3.8 GPA",
            "Focused on software engineering and distributed systems. Senior project: Built a distributed task queue system."
        ],
    )?;

    // 5. Certifications
    println!("  - Adding certifications...");
    let certifications = vec![
        ("AWS Certified Solutions Architect", "Amazon Web Services", "2022-03", "", "AWS-12345", "https://aws.amazon.com/certification"),
        ("Kubernetes Administrator (CKA)", "Cloud Native Computing Foundation", "2021-11", "2024-11", "CKA-67890", ""),
    ];

    for (name, org, issue_date, exp_date, cred_id, cred_url) in certifications {
        conn.execute(
            "INSERT INTO certifications (user_profile_id, name, issuing_organization, issue_date, expiration_date, credential_id, credential_url) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![1, name, org, issue_date, exp_date, cred_id, cred_url],
        )?;
    }

    // 6. Portfolio Items
    println!("  - Adding portfolio items...");
    let portfolio_items = vec![
        (
            "Open Source Task Queue",
            "https://github.com/alex/task-queue",
            "Distributed task queue system built with Go and Redis",
            "Creator & Maintainer",
            "Go, Redis, Docker",
            1
        ),
        (
            "React Component Library",
            "https://github.com/alex/ui-components",
            "Reusable React component library with 500+ stars",
            "Creator",
            "React, TypeScript, Storybook",
            1
        ),
    ];

    for (title, url, description, role, tech_stack, highlighted) in portfolio_items {
        conn.execute(
            "INSERT INTO portfolio_items (user_profile_id, title, url, description, role, tech_stack, highlighted) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![1, title, url, description, role, tech_stack, highlighted],
        )?;
    }

    // 7. Jobs
    println!("  - Adding job postings...");
    let jobs = vec![
        (
            "Senior Full-Stack Engineer",
            "InnovateTech",
            "San Francisco, CA",
            "LinkedIn",
            "https://linkedin.com/jobs/view/12345",
            r#"We're looking for a Senior Full-Stack Engineer to join our growing team. You'll work on building scalable web applications using React, Node.js, and AWS.

Requirements:
- 5+ years of experience with React and Node.js
- Strong TypeScript skills
- Experience with AWS cloud services
- Experience with microservices architecture
- Strong communication and leadership skills

Nice to have:
- Kubernetes experience
- Experience with GraphQL
- Previous startup experience"#,
            "Senior",
            "React, Node.js, TypeScript, AWS, Microservices"
        ),
        (
            "Tech Lead - Platform Engineering",
            "ScaleUp Inc.",
            "Remote",
            "Company Website",
            "https://scaleup.com/careers/tech-lead",
            r#"Join our platform engineering team as a Tech Lead. You'll be responsible for architecting and building our core platform infrastructure.

Key Responsibilities:
- Lead a team of 4-6 engineers
- Design and implement scalable platform services
- Mentor junior engineers
- Collaborate with product teams on technical strategy

Requirements:
- 7+ years of software engineering experience
- 2+ years of experience leading engineering teams
- Strong backend engineering skills (Node.js, Python, or Go)
- Experience with distributed systems
- Excellent communication skills"#,
            "Lead",
            "Node.js, Python, Go, Kubernetes, AWS, Leadership"
        ),
        (
            "Senior Frontend Engineer",
            "DesignFirst",
            "New York, NY",
            "Indeed",
            "https://indeed.com/viewjob?jk=abc123",
            r#"DesignFirst is looking for a Senior Frontend Engineer to help build beautiful, performant user interfaces.

What you'll do:
- Build responsive web applications using React
- Optimize performance and user experience
- Collaborate with designers and product managers
- Contribute to technical architecture decisions

Requirements:
- 4+ years of React experience
- Strong TypeScript skills
- Experience with modern CSS and design systems
- Portfolio demonstrating strong UI/UX skills"#,
            "Senior",
            "React, TypeScript, CSS, Design Systems"
        ),
    ];

    for (title, company, location, source, url, description, seniority, tags) in jobs {
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, seniority, domain_tags, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                title,
                company,
                location,
                source,
                url,
                description,
                seniority,
                tags,
                1,
                now,
                now
            ],
        )?;
    }

    // 8. Applications
    println!("  - Adding applications...");
    let job_ids: Vec<i64> = conn.prepare("SELECT id FROM jobs ORDER BY id")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    if !job_ids.is_empty() {
        // Application 1: Applied
        conn.execute(
            "INSERT INTO applications (job_id, status, channel, priority, date_saved, date_applied, last_activity_date, next_action_date, next_action_note, notes_summary, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                job_ids[0],
                "applied",
                "LinkedIn",
                "high",
                now,
                now,
                now,
                "",
                "",
                "Applied through LinkedIn. Waiting for response.",
                now,
                now
            ],
        )?;

        // Application 2: Interview
        conn.execute(
            "INSERT INTO applications (job_id, status, channel, priority, date_saved, date_applied, last_activity_date, next_action_date, next_action_note, notes_summary, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                job_ids[1],
                "interview",
                "Company Website",
                "high",
                now,
                now,
                now,
                "2024-12-15",
                "Prepare for technical interview - review system design",
                "Passed phone screen. Technical interview scheduled for Dec 15.",
                now,
                now
            ],
        )?;
        let app_id_2: i64 = conn.last_insert_rowid(); // Get ID after insertion

        // Application 3: Saved
        conn.execute(
            "INSERT INTO applications (job_id, status, channel, priority, date_saved, date_applied, last_activity_date, next_action_date, next_action_note, notes_summary, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                job_ids[2],
                "saved",
                "Indeed",
                "medium",
                now,
                "",
                now,
                "",
                "",
                "Interesting role, need to tailor resume before applying.",
                now,
                now
            ],
        )?;

        // 9. Application Events
        println!("  - Adding application events...");
        let events = vec![
            ("status_change", "saved", "applied", "Applied for position"),
            ("status_change", "applied", "interview", "Phone screen scheduled"),
            ("note", "", "", "Phone screen went well, technical interview next"),
        ];

        for (event_type, from_status, to_status, details) in events {
            conn.execute(
                "INSERT INTO application_events (application_id, event_type, event_date, from_status, to_status, title, details, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    app_id_2,
                    event_type,
                    now,
                    from_status,
                    to_status,
                    if event_type == "status_change" { "Status Changed" } else { "Note Added" },
                    details,
                    now
                ],
            )?;
        }
    }

    println!("Test data populated successfully!");
    Ok(())
}

fn init_database_if_needed(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    // Check if migrations table exists
    let migrations_exist: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='migrations'",
            [],
            |row| Ok(row.get::<_, i64>(0)? > 0),
        )
        .unwrap_or(false);
    
    if !migrations_exist {
        println!("Initializing database schema...");
        
        // Create migrations table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Run basic schema creation (simplified version)
        // This is a minimal schema - the full migrations are in db.rs
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
                open_to_roles INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS experience (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_profile_id INTEGER NOT NULL,
                company TEXT NOT NULL,
                title TEXT NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT,
                current INTEGER DEFAULT 0,
                description TEXT,
                location TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_profile_id) REFERENCES user_profile(id)
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS skills (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_profile_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                category TEXT,
                proficiency TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_profile_id) REFERENCES user_profile(id)
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS education (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_profile_id INTEGER NOT NULL,
                institution TEXT NOT NULL,
                degree TEXT,
                field_of_study TEXT,
                start_date TEXT,
                end_date TEXT,
                gpa TEXT,
                honors TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_profile_id) REFERENCES user_profile(id)
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS certifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_profile_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                issuer TEXT,
                issue_date TEXT,
                expiry_date TEXT,
                credential_id TEXT,
                credential_url TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_profile_id) REFERENCES user_profile(id)
            )",
            [],
        )?;
        
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
                version INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (application_id) REFERENCES applications(id),
                FOREIGN KEY (job_id) REFERENCES jobs(id)
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cache_key TEXT NOT NULL UNIQUE,
                cache_value TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT
            )",
            [],
        )?;
        
        println!("Database schema initialized.");
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_db_path();
    
    // Create directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let conn = Connection::open(&db_path)?;
    
    // Initialize database schema if needed
    init_database_if_needed(&conn)?;
    
    // Clear existing data
    clear_database(&conn)?;
    
    // Populate with test data
    populate_test_data(&conn)?;
    
    println!("\n✅ Test data population complete!");
    println!("Database location: {}", db_path.display());
    
    Ok(())
}

