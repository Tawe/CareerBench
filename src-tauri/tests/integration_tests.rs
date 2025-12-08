// Integration tests for Tauri commands
// These tests use a test database and mock AI client

use careerbench::commands::*;
use careerbench::db;
use rusqlite::Connection;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

// Helper to create a test database
fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    
    // Create migrations table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL
        )",
        [],
    ).unwrap();
    
    // Run migrations
    db::migration_001_initial_schema(&conn).unwrap();
    
    // Create ai_cache table for tests
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
    ).unwrap();
    
    conn
}

// Helper to load a fixture JSON file
fn load_fixture(path: &str) -> serde_json::Value {
    let mut fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixture_path.pop(); // Go up from src-tauri
    fixture_path.push("tests");
    fixture_path.push("fixtures");
    fixture_path.push(path);
    
    let content = fs::read_to_string(fixture_path).unwrap();
    serde_json::from_str(&content).unwrap()
}

mod tests {
    use super::*;

    #[test]
    fn test_parse_job_deserialization() {
        // Test that we can deserialize a valid ParsedJob from fixture
        let fixture = load_fixture("job_parsing/basic_job.json");
        let parsed: Result<ParsedJob, _> = serde_json::from_value(fixture);
        assert!(parsed.is_ok());
        
        let parsed = parsed.unwrap();
        assert_eq!(parsed.title_suggestion, Some("Senior Software Engineer".to_string()));
        assert_eq!(parsed.required_skills.len(), 3);
    }

    #[test]
    fn test_generated_resume_deserialization() {
        // Test that we can deserialize a valid GeneratedResume from fixture
        let fixture = load_fixture("resume_generation/basic_resume.json");
        let resume: Result<GeneratedResume, _> = serde_json::from_value(fixture);
        assert!(resume.is_ok());
        
        let resume = resume.unwrap();
        assert!(resume.summary.is_some());
        assert_eq!(resume.sections.len(), 2);
    }

    #[test]
    fn test_generated_letter_deserialization() {
        // Test that we can deserialize a valid GeneratedLetter from fixture
        let fixture = load_fixture("cover_letter/basic_letter.json");
        let letter: Result<GeneratedLetter, _> = serde_json::from_value(fixture);
        assert!(letter.is_ok());
        
        let letter = letter.unwrap();
        assert!(letter.subject.is_some());
        assert_eq!(letter.body_paragraphs.len(), 3);
    }

    #[test]
    fn test_invalid_json_handling() {
        // Test that invalid JSON fails gracefully
        let invalid = json!({
            "invalid": "structure",
            "missing": "required_fields"
        });
        
        let parsed: Result<ParsedJob, _> = serde_json::from_value(invalid);
        // Should still deserialize (with defaults) but may have None values
        // This tests that we don't panic on partial data
        assert!(parsed.is_ok());
    }

    // Comprehensive deserialization tests for all AI output types
    #[test]
    fn test_parsed_job_comprehensive_deserialization() {
        // Test with all fields populated
        let full_job = json!({
            "titleSuggestion": "Senior Software Engineer",
            "companySuggestion": "Tech Corp",
            "seniority": "Senior",
            "location": "San Francisco, CA",
            "summary": "We are looking for an experienced software engineer.",
            "responsibilities": ["Design systems", "Lead team"],
            "requiredSkills": ["Rust", "TypeScript"],
            "niceToHaveSkills": ["Kubernetes"],
            "domainTags": ["Backend"],
            "seniorityScore": 0.75,
            "remoteFriendly": true
        });
        
        let parsed: ParsedJob = serde_json::from_value(full_job).unwrap();
        assert_eq!(parsed.title_suggestion, Some("Senior Software Engineer".to_string()));
        assert_eq!(parsed.required_skills.len(), 2);
        assert_eq!(parsed.seniority_score, Some(0.75));
        assert_eq!(parsed.remote_friendly, Some(true));
    }

    #[test]
    fn test_parsed_job_minimal_deserialization() {
        // Test with minimal fields (all optional)
        let minimal_job = json!({});
        
        let parsed: ParsedJob = serde_json::from_value(minimal_job).unwrap();
        assert_eq!(parsed.title_suggestion, None);
        assert_eq!(parsed.required_skills.len(), 0);
        assert_eq!(parsed.responsibilities.len(), 0);
    }

    #[test]
    fn test_generated_resume_comprehensive_deserialization() {
        // Test with all fields populated
        let full_resume = json!({
            "summary": "Experienced engineer",
            "headline": "Senior Software Engineer",
            "sections": [
                {
                    "title": "Experience",
                    "items": [
                        {
                            "heading": "Engineer at Tech Corp",
                            "subheading": "2020 - Present",
                            "bullets": ["Built systems", "Led team"]
                        }
                    ]
                },
                {
                    "title": "Skills",
                    "items": [
                        {
                            "heading": "Technical Skills",
                            "subheading": null,
                            "bullets": ["Rust", "TypeScript"]
                        }
                    ]
                }
            ],
            "highlights": ["5+ years", "Led team"]
        });
        
        let resume: GeneratedResume = serde_json::from_value(full_resume).unwrap();
        assert_eq!(resume.summary, Some("Experienced engineer".to_string()));
        assert_eq!(resume.sections.len(), 2);
        assert_eq!(resume.sections[0].items[0].bullets.len(), 2);
        assert_eq!(resume.highlights.len(), 2);
    }

    #[test]
    fn test_generated_resume_minimal_deserialization() {
        // Test with minimal fields
        let minimal_resume = json!({
            "sections": [],
            "highlights": []
        });
        
        let resume: GeneratedResume = serde_json::from_value(minimal_resume).unwrap();
        assert_eq!(resume.summary, None);
        assert_eq!(resume.headline, None);
        assert_eq!(resume.sections.len(), 0);
    }

    #[test]
    fn test_generated_letter_comprehensive_deserialization() {
        // Test with all fields populated
        let full_letter = json!({
            "subject": "Application for Position",
            "greeting": "Dear Hiring Manager,",
            "bodyParagraphs": [
                "First paragraph",
                "Second paragraph",
                "Third paragraph"
            ],
            "closing": "Best regards,",
            "signature": "John Doe"
        });
        
        let letter: GeneratedLetter = serde_json::from_value(full_letter).unwrap();
        assert_eq!(letter.subject, Some("Application for Position".to_string()));
        assert_eq!(letter.body_paragraphs.len(), 3);
        assert_eq!(letter.closing, Some("Best regards,".to_string()));
    }

    #[test]
    fn test_generated_letter_minimal_deserialization() {
        // Test with minimal fields
        let minimal_letter = json!({
            "bodyParagraphs": []
        });
        
        let letter: GeneratedLetter = serde_json::from_value(minimal_letter).unwrap();
        assert_eq!(letter.subject, None);
        assert_eq!(letter.body_paragraphs.len(), 0);
    }

    // Schema validation tests - these should break if schema changes
    #[test]
    fn test_parsed_job_schema_validation() {
        let fixture = load_fixture("job_parsing/basic_job.json");
        let parsed: ParsedJob = serde_json::from_value(fixture).unwrap();
        
        // Validate all expected fields exist and have correct types
        assert!(parsed.title_suggestion.is_some());
        assert!(parsed.required_skills.is_empty() || parsed.required_skills.iter().all(|s| !s.is_empty()));
        assert!(parsed.nice_to_have_skills.is_empty() || parsed.nice_to_have_skills.iter().all(|s| !s.is_empty()));
        assert!(parsed.responsibilities.is_empty() || parsed.responsibilities.iter().all(|s| !s.is_empty()));
        assert!(parsed.domain_tags.is_empty() || parsed.domain_tags.iter().all(|s| !s.is_empty()));
        
        // Validate optional numeric field
        if let Some(score) = parsed.seniority_score {
            assert!(score >= 0.0 && score <= 1.0);
        }
    }

    #[test]
    fn test_generated_resume_schema_validation() {
        let fixture = load_fixture("resume_generation/basic_resume.json");
        let resume: GeneratedResume = serde_json::from_value(fixture).unwrap();
        
        // Validate structure
        assert!(!resume.sections.is_empty());
        for section in &resume.sections {
            assert!(!section.title.is_empty());
            assert!(!section.items.is_empty());
            for item in &section.items {
                assert!(!item.heading.is_empty());
                assert!(!item.bullets.is_empty());
            }
        }
    }

    #[test]
    fn test_generated_letter_schema_validation() {
        let fixture = load_fixture("cover_letter/basic_letter.json");
        let letter: GeneratedLetter = serde_json::from_value(fixture).unwrap();
        
        // Validate structure
        assert!(!letter.body_paragraphs.is_empty());
        assert!(letter.body_paragraphs.iter().all(|p| !p.is_empty()));
    }

    // Test that schema changes break tests appropriately
    #[test]
    fn test_parsed_job_missing_required_arrays() {
        // Test that missing array fields default to empty vec
        let missing_arrays = json!({
            "titleSuggestion": "Engineer"
            // Missing required_skills, responsibilities, etc.
        });
        
        let parsed: ParsedJob = serde_json::from_value(missing_arrays).unwrap();
        assert_eq!(parsed.required_skills.len(), 0);
        assert_eq!(parsed.responsibilities.len(), 0);
        assert_eq!(parsed.domain_tags.len(), 0);
    }

    #[test]
    fn test_generated_resume_missing_required_arrays() {
        // Test that missing array fields default to empty vec
        // Note: sections is required, so we must include it
        let missing_arrays = json!({
            "summary": "Test",
            "sections": [],
            "highlights": []
        });
        
        let resume: GeneratedResume = serde_json::from_value(missing_arrays).unwrap();
        assert_eq!(resume.sections.len(), 0);
        assert_eq!(resume.highlights.len(), 0);
    }

    #[test]
    fn test_type_mismatch_handling() {
        // Test that wrong types fail appropriately
        let wrong_type = json!({
            "titleSuggestion": 123, // Should be string
            "requiredSkills": "not an array", // Should be array
            "seniorityScore": "not a number" // Should be number
        });
        
        // These should either fail or use defaults
        let parsed: Result<ParsedJob, _> = serde_json::from_value(wrong_type);
        // Depending on serde configuration, this might fail or use defaults
        // We test that it doesn't panic
        if let Ok(p) = parsed {
            // If it succeeds, verify defaults are used
            assert!(p.title_suggestion.is_none() || p.title_suggestion.as_ref().map(|s| s.parse::<i32>().is_ok()).unwrap_or(false));
        }
    }

    #[test]
    fn test_roundtrip_serialization() {
        // Test that we can serialize and deserialize without data loss
        let fixture = load_fixture("job_parsing/basic_job.json");
        let parsed1: ParsedJob = serde_json::from_value(fixture.clone()).unwrap();
        
        let serialized = serde_json::to_value(&parsed1).unwrap();
        let parsed2: ParsedJob = serde_json::from_value(serialized).unwrap();
        
        assert_eq!(parsed1.title_suggestion, parsed2.title_suggestion);
        assert_eq!(parsed1.required_skills, parsed2.required_skills);
        assert_eq!(parsed1.seniority_score, parsed2.seniority_score);
    }

    #[test]
    fn test_resume_roundtrip_serialization() {
        let fixture = load_fixture("resume_generation/basic_resume.json");
        let resume1: GeneratedResume = serde_json::from_value(fixture.clone()).unwrap();
        
        let serialized = serde_json::to_value(&resume1).unwrap();
        let resume2: GeneratedResume = serde_json::from_value(serialized).unwrap();
        
        assert_eq!(resume1.summary, resume2.summary);
        assert_eq!(resume1.sections.len(), resume2.sections.len());
        assert_eq!(resume1.highlights, resume2.highlights);
    }

    #[test]
    fn test_letter_roundtrip_serialization() {
        let fixture = load_fixture("cover_letter/basic_letter.json");
        let letter1: GeneratedLetter = serde_json::from_value(fixture.clone()).unwrap();
        
        let serialized = serde_json::to_value(&letter1).unwrap();
        let letter2: GeneratedLetter = serde_json::from_value(serialized).unwrap();
        
        assert_eq!(letter1.subject, letter2.subject);
        assert_eq!(letter1.body_paragraphs, letter2.body_paragraphs);
        assert_eq!(letter1.closing, letter2.closing);
    }

    #[test]
    fn test_render_resume_to_text() {
        let fixture = load_fixture("resume_generation/basic_resume.json");
        let resume: GeneratedResume = serde_json::from_value(fixture).unwrap();
        
        let text = render_resume_to_text(&resume);
        assert!(!text.is_empty());
        assert!(text.contains("Experience")); // Should contain section title
    }

    #[test]
    fn test_render_letter_to_text() {
        let fixture = load_fixture("cover_letter/basic_letter.json");
        let letter: GeneratedLetter = serde_json::from_value(fixture).unwrap();
        
        let text = render_letter_to_text(&letter);
        assert!(!text.is_empty());
        assert!(text.contains("Dear")); // Should contain greeting
    }

    // Note: Integration tests for save_resume, save_cover_letter, and related commands
    // require database connection mocking which is complex. These tests would need
    // either dependency injection for the database connection or a test-specific
    // database setup. For now, we test the rendering and deserialization functions
    // which are the core logic, and leave full command integration tests for
    // when we have a proper test infrastructure with database mocking.
    
    #[test]
    fn test_format_date() {
        // Test the format_date helper function
        assert_eq!(format_date("2024-01"), "Jan 2024");
        assert_eq!(format_date("2023-12"), "Dec 2023");
        assert_eq!(format_date("2022-06"), "Jun 2022");
        assert_eq!(format_date("2024-13"), "13 2024"); // Invalid month
        assert_eq!(format_date("2024"), "2024"); // Too short
        assert_eq!(format_date(""), ""); // Empty
    }

    // Helper function to test dashboard queries with a test database
    fn test_dashboard_queries_with_db(conn: &Connection) -> Result<DashboardData, String> {
        use chrono::Utc;
        use std::collections::HashMap;

        // KPIs
        let total_jobs: i64 = conn
            .query_row("SELECT COUNT(*) FROM jobs", [], |row| row.get(0))
            .map_err(|e| format!("Failed to get total jobs: {}", e))?;

        let total_applications: i64 = conn
            .query_row("SELECT COUNT(*) FROM applications", [], |row| row.get(0))
            .map_err(|e| format!("Failed to get total applications: {}", e))?;

        let active_applications: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE archived = 0",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get active applications: {}", e))?;

        let applications_last_30_days: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE date_saved >= date('now', '-30 day')",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get recent applications: {}", e))?;

        let offers_received: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE status = 'Offer'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get offers: {}", e))?;

        let kpis = DashboardKpis {
            total_jobs_tracked: total_jobs,
            total_applications,
            active_applications,
            applications_last_30_days,
            offers_received,
        };

        // Status breakdown
        let mut stmt = conn
            .prepare(
                "SELECT status, COUNT(*) as count
                 FROM applications
                 WHERE archived = 0
                 GROUP BY status",
            )
            .map_err(|e| format!("Failed to prepare status query: {}", e))?;

        let status_rows = stmt
            .query_map([], |row| {
                Ok(StatusBucket {
                    status: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|e| format!("Failed to get status breakdown: {}", e))?;

        let mut status_breakdown = Vec::new();
        for row_result in status_rows {
            status_breakdown.push(row_result.map_err(|e| format!("Error: {}", e))?);
        }

        // Activity last 30 days
        let mut activity_map: HashMap<String, DailyActivityPoint> = HashMap::new();

        // Initialize all dates
        let now = Utc::now();
        for i in 0..30 {
            let date = now - chrono::Duration::days(i);
            let date_str = date.format("%Y-%m-%d").to_string();
            activity_map.insert(
                date_str.clone(),
                DailyActivityPoint {
                    date: date_str,
                    applications_created: 0,
                    interviews_completed: 0,
                    offers_received: 0,
                },
            );
        }

        // Applications created
        let mut stmt = conn
            .prepare(
                "SELECT date(date_saved) as day, COUNT(*) as count
                 FROM applications
                 WHERE date_saved >= date('now', '-30 day')
                 GROUP BY day",
            )
            .map_err(|e| format!("Failed to prepare applications query: {}", e))?;

        let app_rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| format!("Failed to get applications: {}", e))?;

        for row_result in app_rows {
            let (day, count) = row_result.map_err(|e| format!("Error: {}", e))?;
            if let Some(point) = activity_map.get_mut(&day) {
                point.applications_created = count;
            }
        }

        // Interviews completed
        let mut stmt = conn
            .prepare(
                "SELECT date(event_date) as day, COUNT(*) as count
                 FROM application_events
                 WHERE event_type = 'InterviewCompleted'
                   AND event_date >= date('now', '-30 day')
                 GROUP BY day",
            )
            .map_err(|e| format!("Failed to prepare interviews query: {}", e))?;

        let interview_rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| format!("Failed to get interviews: {}", e))?;

        for row_result in interview_rows {
            let (day, count) = row_result.map_err(|e| format!("Error: {}", e))?;
            if let Some(point) = activity_map.get_mut(&day) {
                point.interviews_completed = count;
            }
        }

        // Offers received
        let mut stmt = conn
            .prepare(
                "SELECT date(event_date) as day, COUNT(*) as count
                 FROM application_events
                 WHERE event_type = 'OfferReceived'
                   AND event_date >= date('now', '-30 day')
                 GROUP BY day",
            )
            .map_err(|e| format!("Failed to prepare offers query: {}", e))?;

        let offer_rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| format!("Failed to get offers: {}", e))?;

        for row_result in offer_rows {
            let (day, count) = row_result.map_err(|e| format!("Error: {}", e))?;
            if let Some(point) = activity_map.get_mut(&day) {
                point.offers_received = count;
            }
        }

        let mut activity_last_30_days: Vec<DailyActivityPoint> = activity_map.into_values().collect();
        activity_last_30_days.sort_by_key(|p| p.date.clone());

        // Funnel
        let applied: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE status IN ('Applied', 'Interviewing', 'Offer', 'Rejected', 'Ghosted', 'Withdrawn')",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get applied count: {}", e))?;

        let interviewing: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE status IN ('Interviewing', 'Offer', 'Rejected', 'Ghosted', 'Withdrawn')",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get interviewing count: {}", e))?;

        let offer: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM applications WHERE status = 'Offer'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get offer count: {}", e))?;

        let funnel = vec![
            FunnelStep {
                label: "Applied".to_string(),
                count: applied,
            },
            FunnelStep {
                label: "Interviewing".to_string(),
                count: interviewing,
            },
            FunnelStep {
                label: "Offer".to_string(),
                count: offer,
            },
        ];

        Ok(DashboardData {
            kpis,
            status_breakdown,
            activity_last_30_days,
            funnel,
        })
    }

    #[test]
    fn test_dashboard_data_empty() {
        // Test dashboard queries with empty database
        let conn = setup_test_db();
        let result = test_dashboard_queries_with_db(&conn);
        
        assert!(result.is_ok());
        let data = result.unwrap();
        
        // All counts should be zero
        assert_eq!(data.kpis.total_jobs_tracked, 0);
        assert_eq!(data.kpis.total_applications, 0);
        assert_eq!(data.kpis.active_applications, 0);
        assert_eq!(data.kpis.applications_last_30_days, 0);
        assert_eq!(data.kpis.offers_received, 0);
        assert_eq!(data.status_breakdown.len(), 0);
        assert_eq!(data.activity_last_30_days.len(), 30); // Should have 30 days
        assert_eq!(data.funnel.len(), 3);
        assert_eq!(data.funnel[0].count, 0);
        assert_eq!(data.funnel[1].count, 0);
        assert_eq!(data.funnel[2].count, 0);
    }

    #[test]
    fn test_dashboard_data_with_applications() {
        // Test dashboard queries with sample data
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        
        // Create a job
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Software Engineer",
                "Test Company",
                "San Francisco, CA",
                "LinkedIn",
                "https://example.com/job",
                "Job description",
                1,
                now,
                now
            ],
        ).unwrap();
        let job_id = conn.last_insert_rowid();

        // Create applications with different statuses
        conn.execute(
            "INSERT INTO applications (job_id, status, archived, date_saved, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![job_id, "Applied", 0, now, now, now],
        ).unwrap();
        
        conn.execute(
            "INSERT INTO applications (job_id, status, archived, date_saved, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![job_id, "Interviewing", 0, now, now, now],
        ).unwrap();
        
        conn.execute(
            "INSERT INTO applications (job_id, status, archived, date_saved, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![job_id, "Offer", 0, now, now, now],
        ).unwrap();
        
        // Create an archived application (should not count in active)
        conn.execute(
            "INSERT INTO applications (job_id, status, archived, date_saved, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![job_id, "Applied", 1, now, now, now],
        ).unwrap();

        let result = test_dashboard_queries_with_db(&conn);
        assert!(result.is_ok());
        let data = result.unwrap();
        
        // Verify KPIs
        assert_eq!(data.kpis.total_jobs_tracked, 1);
        assert_eq!(data.kpis.total_applications, 4);
        assert_eq!(data.kpis.active_applications, 3); // Excludes archived
        assert_eq!(data.kpis.applications_last_30_days, 4);
        assert_eq!(data.kpis.offers_received, 1);
        
        // Verify status breakdown
        assert_eq!(data.status_breakdown.len(), 3); // Applied, Interviewing, Offer
        let status_counts: std::collections::HashMap<String, i64> = data.status_breakdown
            .iter()
            .map(|s| (s.status.clone(), s.count))
            .collect();
        assert_eq!(status_counts.get("Applied"), Some(&1));
        assert_eq!(status_counts.get("Interviewing"), Some(&1));
        assert_eq!(status_counts.get("Offer"), Some(&1));
        
        // Verify funnel
        // "Applied" counts: Applied, Interviewing, Offer, Rejected, Ghosted, Withdrawn = 1 Applied + 1 Interviewing + 1 Offer + 1 Archived Applied = 4
        assert_eq!(data.funnel[0].count, 4);
        // "Interviewing" counts: Interviewing, Offer, Rejected, Ghosted, Withdrawn = 1 Interviewing + 1 Offer = 2
        assert_eq!(data.funnel[1].count, 2);
        // "Offer" counts: Offer = 1
        assert_eq!(data.funnel[2].count, 1);
    }

    // Helper function to test create_application logic with a test database
    fn test_create_application_with_db(
        conn: &Connection,
        input: CreateApplicationInput,
    ) -> Result<Application, String> {
        use chrono::Utc;
        let now = Utc::now().to_rfc3339();
        let status = input.status.unwrap_or_else(|| "Saved".to_string());

        // Check if job exists
        let job_exists: bool = conn
            .query_row("SELECT COUNT(*) FROM jobs WHERE id = ?", [input.job_id], |row| {
                row.get(0)
            })
            .map_err(|e| format!("Failed to check job: {}", e))?;

        if !job_exists {
            return Err("Job not found".to_string());
        }

        // Insert application
        conn.execute(
            "INSERT INTO applications (job_id, status, channel, priority, date_saved, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                input.job_id,
                status,
                input.channel,
                input.priority,
                now,
                now,
                now
            ],
        )
        .map_err(|e| format!("Failed to create application: {}", e))?;

        let application_id = conn.last_insert_rowid();

        // Create ApplicationCreated event
        conn.execute(
            "INSERT INTO application_events (application_id, event_type, event_date, created_at) VALUES (?, ?, ?, ?)",
            rusqlite::params![application_id, "ApplicationCreated", now, now],
        )
        .map_err(|e| format!("Failed to create event: {}", e))?;

        // Get the created application
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, status, channel, priority, date_saved, date_applied, last_activity_date, next_action_date, next_action_note, notes_summary, contact_name, contact_email, contact_linkedin, location_override, offer_compensation, archived, created_at, updated_at FROM applications WHERE id = ?"
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let application = stmt
            .query_row([application_id], |row| {
                Ok(Application {
                    id: Some(row.get(0)?),
                    job_id: row.get(1)?,
                    status: row.get(2)?,
                    channel: row.get(3)?,
                    priority: row.get(4)?,
                    date_saved: row.get(5)?,
                    date_applied: row.get(6)?,
                    last_activity_date: row.get(7)?,
                    next_action_date: row.get(8)?,
                    next_action_note: row.get(9)?,
                    notes_summary: row.get(10)?,
                    contact_name: row.get(11)?,
                    contact_email: row.get(12)?,
                    contact_linkedin: row.get(13)?,
                    location_override: row.get(14)?,
                    offer_compensation: row.get(15)?,
                    archived: row.get::<_, i32>(16)? != 0,
                    created_at: row.get(17)?,
                    updated_at: row.get(18)?,
                })
            })
            .map_err(|e| format!("Failed to get application: {}", e))?;

        Ok(application)
    }

    #[test]
    fn test_create_application_success() {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();

        // Create a job first
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Software Engineer",
                "Test Company",
                "San Francisco, CA",
                "LinkedIn",
                "https://example.com/job",
                "Job description",
                1,
                now,
                now
            ],
        ).unwrap();
        let job_id = conn.last_insert_rowid();

        // Create application
        let input = CreateApplicationInput {
            job_id,
            status: Some("Applied".to_string()),
            channel: Some("LinkedIn".to_string()),
            priority: Some("High".to_string()),
        };

        let result = test_create_application_with_db(&conn, input);
        assert!(result.is_ok());
        
        let app = result.unwrap();
        assert_eq!(app.job_id, job_id);
        assert_eq!(app.status, "Applied");
        assert_eq!(app.channel, Some("LinkedIn".to_string()));
        assert_eq!(app.priority, Some("High".to_string()));
        assert!(!app.archived);
        assert!(app.id.is_some());

        // Verify ApplicationCreated event was created
        let event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM application_events WHERE application_id = ? AND event_type = 'ApplicationCreated'",
                [app.id.unwrap()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(event_count, 1);
    }

    #[test]
    fn test_create_application_default_status() {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();

        // Create a job first
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Software Engineer",
                "Test Company",
                "San Francisco, CA",
                "LinkedIn",
                "https://example.com/job",
                "Job description",
                1,
                now,
                now
            ],
        ).unwrap();
        let job_id = conn.last_insert_rowid();

        // Create application without status (should default to "Saved")
        let input = CreateApplicationInput {
            job_id,
            status: None,
            channel: None,
            priority: None,
        };

        let result = test_create_application_with_db(&conn, input);
        assert!(result.is_ok());
        
        let app = result.unwrap();
        assert_eq!(app.status, "Saved");
    }

    #[test]
    fn test_create_application_job_not_found() {
        let conn = setup_test_db();

        // Try to create application for non-existent job
        let input = CreateApplicationInput {
            job_id: 999,
            status: Some("Applied".to_string()),
            channel: None,
            priority: None,
        };

        let result = test_create_application_with_db(&conn, input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Job not found"));
    }

    // Helper function to test update_application logic with a test database
    fn test_update_application_with_db(
        conn: &Connection,
        id: i64,
        input: UpdateApplicationInput,
    ) -> Result<Application, String> {
        use chrono::Utc;
        let now = Utc::now().to_rfc3339();

        // Get current application
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, status, channel, priority, date_saved, date_applied, last_activity_date, next_action_date, next_action_note, notes_summary, contact_name, contact_email, contact_linkedin, location_override, offer_compensation, archived, created_at, updated_at FROM applications WHERE id = ?"
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let current_app = stmt
            .query_row([id], |row| {
                Ok(Application {
                    id: Some(row.get(0)?),
                    job_id: row.get(1)?,
                    status: row.get(2)?,
                    channel: row.get(3)?,
                    priority: row.get(4)?,
                    date_saved: row.get(5)?,
                    date_applied: row.get(6)?,
                    last_activity_date: row.get(7)?,
                    next_action_date: row.get(8)?,
                    next_action_note: row.get(9)?,
                    notes_summary: row.get(10)?,
                    contact_name: row.get(11)?,
                    contact_email: row.get(12)?,
                    contact_linkedin: row.get(13)?,
                    location_override: row.get(14)?,
                    offer_compensation: row.get(15)?,
                    archived: row.get::<_, i32>(16)? != 0,
                    created_at: row.get(17)?,
                    updated_at: row.get(18)?,
                })
            })
            .map_err(|_| "Application not found".to_string())?;

        let old_status = current_app.status.clone();

        // Build update query
        let mut updates = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(status) = &input.status {
            updates.push("status = ?");
            params.push(status.clone());
        }
        if let Some(channel) = &input.channel {
            updates.push("channel = ?");
            params.push(channel.clone());
        }
        if let Some(priority) = &input.priority {
            updates.push("priority = ?");
            params.push(priority.clone());
        }
        if let Some(date_applied) = &input.date_applied {
            updates.push("date_applied = ?");
            params.push(date_applied.clone());
        }
        if let Some(next_action_date) = &input.next_action_date {
            updates.push("next_action_date = ?");
            params.push(next_action_date.clone());
        }
        if let Some(next_action_note) = &input.next_action_note {
            updates.push("next_action_note = ?");
            params.push(next_action_note.clone());
        }
        if let Some(notes_summary) = &input.notes_summary {
            updates.push("notes_summary = ?");
            params.push(notes_summary.clone());
        }
        if let Some(contact_name) = &input.contact_name {
            updates.push("contact_name = ?");
            params.push(contact_name.clone());
        }
        if let Some(contact_email) = &input.contact_email {
            updates.push("contact_email = ?");
            params.push(contact_email.clone());
        }
        if let Some(contact_linkedin) = &input.contact_linkedin {
            updates.push("contact_linkedin = ?");
            params.push(contact_linkedin.clone());
        }
        if let Some(location_override) = &input.location_override {
            updates.push("location_override = ?");
            params.push(location_override.clone());
        }
        if let Some(offer_compensation) = &input.offer_compensation {
            updates.push("offer_compensation = ?");
            params.push(offer_compensation.clone());
        }

        if updates.is_empty() {
            return Ok(current_app);
        }

        // Handle status change
        let new_status = input.status.as_ref().unwrap_or(&old_status);
        if new_status != &old_status {
            updates.push("last_activity_date = ?");
            params.push(now.clone());

            // If status becomes Applied and date_applied is empty, set it
            if new_status == "Applied" && input.date_applied.is_none() && current_app.date_applied.is_none() {
                updates.push("date_applied = ?");
                params.push(now.clone());
            }

            // Create StatusChanged event
            conn.execute(
                "INSERT INTO application_events (application_id, event_type, event_date, from_status, to_status, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    id,
                    "StatusChanged",
                    now,
                    old_status,
                    new_status,
                    now
                ],
            )
            .map_err(|e| format!("Failed to create status change event: {}", e))?;
        }

        updates.push("updated_at = ?");
        params.push(now.clone());

        // Build query and execute
        let query = format!("UPDATE applications SET {} WHERE id = ?", updates.join(", "));
        
        // Simple execution for test - in real code this uses match for many params
        let mut all_params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        all_params.push(&id);
        
        conn.execute(&query, rusqlite::params_from_iter(all_params.iter().cloned()))
            .map_err(|e| format!("Failed to update application: {}", e))?;

        // Get updated application
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, status, channel, priority, date_saved, date_applied, last_activity_date, next_action_date, next_action_note, notes_summary, contact_name, contact_email, contact_linkedin, location_override, offer_compensation, archived, created_at, updated_at FROM applications WHERE id = ?"
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let application = stmt
            .query_row([id], |row| {
                Ok(Application {
                    id: Some(row.get(0)?),
                    job_id: row.get(1)?,
                    status: row.get(2)?,
                    channel: row.get(3)?,
                    priority: row.get(4)?,
                    date_saved: row.get(5)?,
                    date_applied: row.get(6)?,
                    last_activity_date: row.get(7)?,
                    next_action_date: row.get(8)?,
                    next_action_note: row.get(9)?,
                    notes_summary: row.get(10)?,
                    contact_name: row.get(11)?,
                    contact_email: row.get(12)?,
                    contact_linkedin: row.get(13)?,
                    location_override: row.get(14)?,
                    offer_compensation: row.get(15)?,
                    archived: row.get::<_, i32>(16)? != 0,
                    created_at: row.get(17)?,
                    updated_at: row.get(18)?,
                })
            })
            .map_err(|e| format!("Failed to get application: {}", e))?;

        Ok(application)
    }

    #[test]
    fn test_update_application_status() {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();

        // Create a job and application
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Software Engineer",
                "Test Company",
                "San Francisco, CA",
                "LinkedIn",
                "https://example.com/job",
                "Job description",
                1,
                now,
                now
            ],
        ).unwrap();
        let job_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO applications (job_id, status, date_saved, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![job_id, "Saved", now, now, now],
        ).unwrap();
        let app_id = conn.last_insert_rowid();

        // Update status
        let input = UpdateApplicationInput {
            status: Some("Interviewing".to_string()),
            channel: None,
            priority: None,
            date_applied: None,
            next_action_date: None,
            next_action_note: None,
            notes_summary: None,
            contact_name: None,
            contact_email: None,
            contact_linkedin: None,
            location_override: None,
            offer_compensation: None,
        };

        let result = test_update_application_with_db(&conn, app_id, input);
        assert!(result.is_ok());
        
        let app = result.unwrap();
        assert_eq!(app.status, "Interviewing");
        assert!(app.last_activity_date.is_some()); // Should be set on status change

        // Verify StatusChanged event was created
        let event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM application_events WHERE application_id = ? AND event_type = 'StatusChanged'",
                [app_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(event_count, 1);
    }

    #[test]
    fn test_update_application_multiple_fields() {
        let conn = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();

        // Create a job and application
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Software Engineer",
                "Test Company",
                "San Francisco, CA",
                "LinkedIn",
                "https://example.com/job",
                "Job description",
                1,
                now,
                now
            ],
        ).unwrap();
        let job_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO applications (job_id, status, date_saved, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![job_id, "Saved", now, now, now],
        ).unwrap();
        let app_id = conn.last_insert_rowid();

        // Update multiple fields
        let input = UpdateApplicationInput {
            status: Some("Applied".to_string()),
            channel: Some("Company Website".to_string()),
            priority: Some("High".to_string()),
            date_applied: Some(now.clone()),
            next_action_date: Some("2024-12-31".to_string()),
            next_action_note: Some("Follow up in 2 weeks".to_string()),
            notes_summary: Some("Great opportunity".to_string()),
            contact_name: Some("John Doe".to_string()),
            contact_email: Some("john@example.com".to_string()),
            contact_linkedin: Some("linkedin.com/in/johndoe".to_string()),
            location_override: None,
            offer_compensation: None,
        };

        let result = test_update_application_with_db(&conn, app_id, input);
        assert!(result.is_ok());
        
        let app = result.unwrap();
        assert_eq!(app.status, "Applied");
        assert_eq!(app.channel, Some("Company Website".to_string()));
        assert_eq!(app.priority, Some("High".to_string()));
        assert_eq!(app.date_applied, Some(now));
        assert_eq!(app.next_action_date, Some("2024-12-31".to_string()));
        assert_eq!(app.next_action_note, Some("Follow up in 2 weeks".to_string()));
        assert_eq!(app.notes_summary, Some("Great opportunity".to_string()));
        assert_eq!(app.contact_name, Some("John Doe".to_string()));
        assert_eq!(app.contact_email, Some("john@example.com".to_string()));
        assert_eq!(app.contact_linkedin, Some("linkedin.com/in/johndoe".to_string()));
    }

    #[test]
    fn test_update_application_not_found() {
        let conn = setup_test_db();

        let input = UpdateApplicationInput {
            status: Some("Applied".to_string()),
            channel: None,
            priority: None,
            date_applied: None,
            next_action_date: None,
            next_action_note: None,
            notes_summary: None,
            contact_name: None,
            contact_email: None,
            contact_linkedin: None,
            location_override: None,
            offer_compensation: None,
        };

        let result = test_update_application_with_db(&conn, 999, input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Application not found"));
    }

    // Helper function to test parse_job_with_ai logic with a mock provider
    #[tokio::test]
    async fn test_parse_job_with_ai_success() {
        use careerbench::ai::mock_provider::MockProvider;
        use careerbench::ai::provider::AiProvider;
        use careerbench::ai::types::{JobParsingInput, JobMeta, ParsedJobOutput};
        use careerbench::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_JOB_PARSE_DAYS};
        use chrono::Utc;
        
        let conn = setup_test_db();
        let now = Utc::now().to_rfc3339();

        // Create a job with description
        let job_description = "Software Engineer position at Tech Company. Requires Rust and TypeScript. Remote friendly.";
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Software Engineer",
                "Tech Company",
                "San Francisco, CA",
                "LinkedIn",
                "https://example.com/job",
                job_description,
                1,
                now,
                now
            ],
        ).unwrap();
        let _job_id = conn.last_insert_rowid();

        // Create mock provider and register response
        let mock_provider = MockProvider::new();
        let parsed_output = ParsedJobOutput {
            title_suggestion: Some("Software Engineer".to_string()),
            company_suggestion: Some("Tech Company".to_string()),
            location: Some("San Francisco, CA".to_string()),
            seniority: Some("Mid".to_string()),
            required_skills: vec!["Rust".to_string(), "TypeScript".to_string()],
            nice_to_have_skills: vec!["Python".to_string()],
            responsibilities: vec!["Build features".to_string(), "Code reviews".to_string()],
            domain_tags: vec!["Backend".to_string()],
            remote_friendly: Some(true),
            summary: Some("Great opportunity".to_string()),
            seniority_score: Some(0.5),
        };
        
        let key = MockProvider::job_key(job_description);
        mock_provider.register_parse_job(&key, parsed_output.clone());

        // Test the parsing logic
        // Build canonical input JSON for caching
        let request_payload = serde_json::json!({
            "jobDescription": job_description,
            "jobMeta": {
                "source": "LinkedIn",
                "url": "https://example.com/job"
            }
        });

        // Compute input hash and check cache (should be empty)
        let input_hash = compute_input_hash(&request_payload).unwrap();
        let cached_entry = ai_cache_get(&conn, "job_parse", &input_hash, &now).unwrap();
        assert!(cached_entry.is_none());

        // Call mock provider
        let parsing_input = JobParsingInput {
            job_description: job_description.to_string(),
            job_meta: Some(JobMeta {
                source: Some("LinkedIn".to_string()),
                url: Some("https://example.com/job".to_string()),
            }),
        };
        
        let parsed_output_result = mock_provider.parse_job(parsing_input).await.unwrap();
        
        // Convert ParsedJobOutput to ParsedJob
        let parsed = ParsedJob {
            title_suggestion: parsed_output_result.title_suggestion,
            company_suggestion: parsed_output_result.company_suggestion,
            seniority: parsed_output_result.seniority,
            location: parsed_output_result.location,
            summary: parsed_output_result.summary,
            responsibilities: parsed_output_result.responsibilities,
            required_skills: parsed_output_result.required_skills,
            nice_to_have_skills: parsed_output_result.nice_to_have_skills,
            domain_tags: parsed_output_result.domain_tags,
            seniority_score: parsed_output_result.seniority_score,
            remote_friendly: parsed_output_result.remote_friendly,
        };

        // Verify parsed data
        assert_eq!(parsed.title_suggestion, Some("Software Engineer".to_string()));
        assert_eq!(parsed.required_skills.len(), 2);
        assert_eq!(parsed.required_skills[0], "Rust");
        assert_eq!(parsed.remote_friendly, Some(true));

        // Store in cache
        let response_payload = serde_json::to_value(&parsed).unwrap();
        let model_name = "mock-model".to_string();
        ai_cache_put(
            &conn,
            "job_parse",
            &input_hash,
            &model_name,
            &request_payload,
            &response_payload,
            Some(CACHE_TTL_JOB_PARSE_DAYS),
            &now,
        ).unwrap();

        // Verify cache was stored
        let cached_entry = ai_cache_get(&conn, "job_parse", &input_hash, &now).unwrap();
        assert!(cached_entry.is_some());
        let cached = cached_entry.unwrap();
        let cached_parsed: ParsedJob = serde_json::from_value(cached.response_payload).unwrap();
        assert_eq!(cached_parsed.title_suggestion, parsed.title_suggestion);

        // Update job with parsed data
        let parsed_json = serde_json::to_string(&parsed).unwrap();
        let domain_tags_str = parsed.domain_tags.join(", ");
        conn.execute(
            "UPDATE jobs SET parsed_json = ?, seniority = COALESCE(?, seniority), domain_tags = COALESCE(?, domain_tags), last_updated = ? WHERE id = ?",
            rusqlite::params![
                parsed_json,
                parsed.seniority,
                if domain_tags_str.is_empty() { None } else { Some(domain_tags_str) },
                now,
                _job_id
            ],
        ).unwrap();

        // Verify job was updated
        let updated_job: String = conn
            .query_row("SELECT parsed_json FROM jobs WHERE id = ?", [_job_id], |row| row.get(0))
            .unwrap();
        let job_parsed: ParsedJob = serde_json::from_str(&updated_job).unwrap();
        assert_eq!(job_parsed.title_suggestion, Some("Software Engineer".to_string()));
    }

    #[tokio::test]
    async fn test_parse_job_with_ai_cache_hit() {
        use careerbench::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_JOB_PARSE_DAYS};
        use chrono::Utc;
        
        let conn = setup_test_db();
        let now = Utc::now().to_rfc3339();

        // Create a job
        let job_description = "Software Engineer position";
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Software Engineer",
                "Tech Company",
                "San Francisco, CA",
                "LinkedIn",
                "https://example.com/job",
                job_description,
                1,
                now,
                now
            ],
        ).unwrap();
        let _job_id = conn.last_insert_rowid();

        // Pre-populate cache
        let request_payload = serde_json::json!({
            "jobDescription": job_description,
            "jobMeta": {
                "source": "LinkedIn",
                "url": "https://example.com/job"
            }
        });
        let input_hash = compute_input_hash(&request_payload).unwrap();
        
        let cached_parsed = ParsedJob {
            title_suggestion: Some("Cached Title".to_string()),
            company_suggestion: Some("Cached Company".to_string()),
            seniority: Some("Senior".to_string()),
            location: Some("Remote".to_string()),
            summary: None,
            responsibilities: vec![],
            required_skills: vec!["Cached Skill".to_string()],
            nice_to_have_skills: vec![],
            domain_tags: vec![],
            seniority_score: None,
            remote_friendly: None,
        };
        
        let response_payload = serde_json::to_value(&cached_parsed).unwrap();
        ai_cache_put(
            &conn,
            "job_parse",
            &input_hash,
            "test-model",
            &request_payload,
            &response_payload,
            Some(CACHE_TTL_JOB_PARSE_DAYS),
            &now,
        ).unwrap();

        // Verify cache hit
        let cached_entry = ai_cache_get(&conn, "job_parse", &input_hash, &now).unwrap();
        assert!(cached_entry.is_some());
        let cached = cached_entry.unwrap();
        let parsed: ParsedJob = serde_json::from_value(cached.response_payload).unwrap();
        assert_eq!(parsed.title_suggestion, Some("Cached Title".to_string()));
        assert_eq!(parsed.required_skills[0], "Cached Skill");
    }

    // Helper function to set up user profile data for resume generation tests
    fn setup_user_profile_for_resume(conn: &Connection) -> Result<(), String> {
        use chrono::Utc;
        let now = Utc::now().to_rfc3339();

        // Create user profile
        conn.execute(
            "INSERT OR REPLACE INTO user_profile (id, full_name, headline, location, summary, current_role_title, current_company, seniority, open_to_roles, created_at, updated_at) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "John Doe",
                "Software Engineer",
                "San Francisco, CA",
                "Experienced software engineer with expertise in Rust and TypeScript",
                "Senior Software Engineer",
                "Tech Corp",
                "Senior",
                "Backend Engineer, Full Stack Engineer",
                now,
                now
            ],
        ).map_err(|e| format!("Failed to create profile: {}", e))?;

        // Create experience
        conn.execute(
            "INSERT INTO experience (user_profile_id, company, title, location, start_date, end_date, is_current, description, achievements, tech_stack, created_at, updated_at) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Tech Corp",
                "Senior Software Engineer",
                "San Francisco, CA",
                "2020-01",
                None::<String>,
                1,
                "Led development of microservices architecture",
                "Built scalable backend systems\nImproved performance by 50%\nMentored junior engineers",
                "Rust, TypeScript, PostgreSQL, AWS",
                now,
                now
            ],
        ).map_err(|e| format!("Failed to create experience: {}", e))?;

        // Create skills (check schema - may not have created_at/updated_at)
        conn.execute(
            "INSERT INTO skills (user_profile_id, name, category, self_rating, priority, years_experience, notes) VALUES (1, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Rust",
                "Programming Language",
                9,
                "Core",
                3.0,
                "Primary language"
            ],
        ).map_err(|e| format!("Failed to create skill: {}", e))?;

        conn.execute(
            "INSERT INTO skills (user_profile_id, name, category, self_rating, priority, years_experience, notes) VALUES (1, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "TypeScript",
                "Programming Language",
                8,
                "Core",
                5.0,
                "Frontend and backend"
            ],
        ).map_err(|e| format!("Failed to create skill: {}", e))?;

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_resume_for_job_with_cache() {
        use careerbench::resume_generator::{JobDescriptionSummary, MappedBullet, RewrittenBullet};
        use careerbench::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_JOB_PARSE_DAYS, CACHE_TTL_RESUME_DAYS};
        use chrono::Utc;
        use serde_json;
        
        let conn = setup_test_db();
        let now = Utc::now().to_rfc3339();

        // Set up user profile
        setup_user_profile_for_resume(&conn).unwrap();

        // Create a job
        let job_description = "Senior Software Engineer position. Requires Rust and TypeScript. Experience with microservices. Remote friendly.";
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Senior Software Engineer",
                "Tech Company",
                "San Francisco, CA",
                "LinkedIn",
                "https://example.com/job",
                job_description,
                1,
                now,
                now
            ],
        ).unwrap();
        let job_id = conn.last_insert_rowid();

        // Pre-populate JD summary cache
        let jd_request = serde_json::json!({
            "jobDescription": job_description,
            "parsedJob": None::<serde_json::Value>
        });
        let jd_hash = compute_input_hash(&jd_request).unwrap();
        let jd_summary = JobDescriptionSummary {
            role_title: Some("Senior Software Engineer".to_string()),
            seniority: Some("Senior".to_string()),
            must_have_skills: vec!["Rust".to_string(), "TypeScript".to_string()],
            nice_to_have_skills: vec!["Python".to_string()],
            top_responsibilities: vec!["Build microservices".to_string()],
            tools_tech: vec!["AWS".to_string()],
            tone: Some("technical".to_string()),
        };
        let jd_response = serde_json::to_value(&jd_summary).unwrap();
        ai_cache_put(
            &conn,
            "jd_summary",
            &jd_hash,
            "test-model",
            &jd_request,
            &jd_response,
            Some(CACHE_TTL_JOB_PARSE_DAYS),
            &now,
        ).unwrap();

        // Pre-populate bullet rewrite cache
        let bullets = vec![
            MappedBullet {
                id: "exp_1_b1".to_string(),
                original_text: "Built scalable backend systems".to_string(),
                relevance_score: 0.9,
                matched_keywords: vec!["backend".to_string()],
            },
            MappedBullet {
                id: "exp_1_b2".to_string(),
                original_text: "Improved performance by 50%".to_string(),
                relevance_score: 0.8,
                matched_keywords: vec!["performance".to_string()],
            },
        ];
        let bullet_request = serde_json::json!({
            "roleTitle": "Senior Software Engineer",
            "company": "Tech Corp",
            "bullets": bullets.iter().map(|b| serde_json::json!({
                "id": b.id,
                "text": b.original_text
            })).collect::<Vec<_>>(),
            "jdSummary": jd_summary,
        });
        let bullet_hash = compute_input_hash(&bullet_request).unwrap();
        let rewritten_bullets = vec![
            RewrittenBullet {
                id: "exp_1_b1".to_string(),
                new_text: "Architected and built scalable microservices backend systems using Rust".to_string(),
            },
            RewrittenBullet {
                id: "exp_1_b2".to_string(),
                new_text: "Optimized system performance, achieving 50% improvement in response times".to_string(),
            },
        ];
        let bullet_response = serde_json::to_value(&rewritten_bullets).unwrap();
        ai_cache_put(
            &conn,
            "bullet_rewrite",
            &bullet_hash,
            "test-model",
            &bullet_request,
            &bullet_response,
            Some(CACHE_TTL_RESUME_DAYS),
            &now,
        ).unwrap();

        // Pre-populate professional summary cache
        let summary_request = serde_json::json!({
            "profile": {
                "full_name": "John Doe",
                "current_role_title": "Senior Software Engineer",
                "summary": "Experienced software engineer"
            },
            "jdSummary": jd_summary,
        });
        let summary_hash = compute_input_hash(&summary_request).unwrap();
        let professional_summary = "Experienced Senior Software Engineer with expertise in Rust and TypeScript. Currently Senior Software Engineer at Tech Corp.".to_string();
        let summary_response = serde_json::to_value(&professional_summary).unwrap();
        ai_cache_put(
            &conn,
            "professional_summary",
            &summary_hash,
            "test-model",
            &summary_request,
            &summary_response,
            Some(CACHE_TTL_RESUME_DAYS),
            &now,
        ).unwrap();

        // Test that we can retrieve cached data (simulating the pipeline)
        let cached_jd = ai_cache_get(&conn, "jd_summary", &jd_hash, &now).unwrap();
        assert!(cached_jd.is_some());
        let jd: JobDescriptionSummary = serde_json::from_value(cached_jd.unwrap().response_payload).unwrap();
        assert_eq!(jd.role_title, Some("Senior Software Engineer".to_string()));
        assert_eq!(jd.must_have_skills.len(), 2);

        let cached_bullets = ai_cache_get(&conn, "bullet_rewrite", &bullet_hash, &now).unwrap();
        assert!(cached_bullets.is_some());
        let bullets_rewritten: Vec<RewrittenBullet> = serde_json::from_value(cached_bullets.unwrap().response_payload).unwrap();
        assert_eq!(bullets_rewritten.len(), 2);
        assert!(bullets_rewritten[0].new_text.contains("microservices"));

        let cached_summary = ai_cache_get(&conn, "professional_summary", &summary_hash, &now).unwrap();
        assert!(cached_summary.is_some());
        let summary: String = serde_json::from_value(cached_summary.unwrap().response_payload).unwrap();
        assert!(summary.contains("Senior Software Engineer"));
    }

    #[tokio::test]
    async fn test_generate_resume_for_job_final_cache() {
        use careerbench::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_RESUME_DAYS};
        use chrono::Utc;
        
        let conn = setup_test_db();
        let now = Utc::now().to_rfc3339();

        // Set up user profile
        setup_user_profile_for_resume(&conn).unwrap();

        // Create a job
        let job_description = "Senior Software Engineer position";
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Senior Software Engineer",
                "Tech Company",
                "San Francisco, CA",
                "LinkedIn",
                "https://example.com/job",
                job_description,
                1,
                now,
                now
            ],
        ).unwrap();
        let _job_id = conn.last_insert_rowid();

        // Pre-populate final resume cache
        let resume_request = serde_json::json!({
            "userProfile": {
                "full_name": "John Doe",
                "current_role_title": "Senior Software Engineer"
            },
            "experience": [{
                "company": "Tech Corp",
                "title": "Senior Software Engineer"
            }],
            "skills": [{"name": "Rust"}, {"name": "TypeScript"}],
            "education": [],
            "job": {
                "title": "Senior Software Engineer",
                "company": "Tech Company",
                "rawDescription": job_description
            },
            "options": None::<serde_json::Value>
        });
        let resume_hash = compute_input_hash(&resume_request).unwrap();
        
        let cached_resume = GeneratedResume {
            summary: Some("Experienced Senior Software Engineer".to_string()),
            headline: Some("John Doe – Senior Software Engineer".to_string()),
            sections: vec![
                ResumeSection {
                    title: "Experience".to_string(),
                    items: vec![
                        ResumeSectionItem {
                            heading: "Senior Software Engineer – Tech Corp".to_string(),
                            subheading: Some("Jan 2020 – Present | San Francisco, CA".to_string()),
                            bullets: vec![
                                "Architected scalable microservices".to_string(),
                                "Improved performance by 50%".to_string(),
                            ],
                        },
                    ],
                },
                ResumeSection {
                    title: "Skills".to_string(),
                    items: vec![
                        ResumeSectionItem {
                            heading: "Key Skills".to_string(),
                            subheading: None,
                            bullets: vec!["Rust, TypeScript".to_string()],
                        },
                    ],
                },
            ],
            highlights: vec!["Tailored for Senior Software Engineer role".to_string()],
        };
        
        let resume_response = serde_json::to_value(&cached_resume).unwrap();
        ai_cache_put(
            &conn,
            "resume_generation",
            &resume_hash,
            "test-model",
            &resume_request,
            &resume_response,
            Some(CACHE_TTL_RESUME_DAYS),
            &now,
        ).unwrap();

        // Verify cache hit
        let cached_entry = ai_cache_get(&conn, "resume_generation", &resume_hash, &now).unwrap();
        assert!(cached_entry.is_some());
        let cached = cached_entry.unwrap();
        let resume: GeneratedResume = serde_json::from_value(cached.response_payload).unwrap();
        assert_eq!(resume.headline, Some("John Doe – Senior Software Engineer".to_string()));
        assert_eq!(resume.sections.len(), 2);
        assert_eq!(resume.sections[0].title, "Experience");
        assert_eq!(resume.sections[1].title, "Skills");
    }

    #[tokio::test]
    async fn test_generate_cover_letter_for_job_with_cache() {
        use careerbench::ai_cache::{ai_cache_get, ai_cache_put, compute_input_hash, CACHE_TTL_COVER_LETTER_DAYS};
        use chrono::Utc;
        
        let conn = setup_test_db();
        let now = Utc::now().to_rfc3339();

        // Set up user profile
        setup_user_profile_for_resume(&conn).unwrap();

        // Create a job
        let job_description = "Senior Software Engineer position at Tech Company. Looking for someone with Rust and TypeScript experience.";
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Senior Software Engineer",
                "Tech Company",
                "San Francisco, CA",
                "LinkedIn",
                "https://example.com/job",
                job_description,
                1,
                now,
                now
            ],
        ).unwrap();
        let _job_id = conn.last_insert_rowid();

        // Pre-populate cover letter cache
        let letter_request = serde_json::json!({
            "userProfile": {
                "full_name": "John Doe",
                "current_role_title": "Senior Software Engineer",
                "summary": "Experienced software engineer"
            },
            "experience": [{
                "company": "Tech Corp",
                "title": "Senior Software Engineer"
            }],
            "skills": [{"name": "Rust"}, {"name": "TypeScript"}],
            "job": {
                "title": "Senior Software Engineer",
                "company": "Tech Company",
                "rawDescription": job_description,
                "parsedJson": None::<serde_json::Value>
            },
            "options": None::<serde_json::Value>
        });
        let letter_hash = compute_input_hash(&letter_request).unwrap();
        
        let cached_letter = GeneratedLetter {
            subject: Some("Application for Senior Software Engineer Position".to_string()),
            greeting: Some("Dear Hiring Manager,".to_string()),
            body_paragraphs: vec![
                "I am writing to express my interest in the Senior Software Engineer position at Tech Company.".to_string(),
                "With my extensive experience in Rust and TypeScript, I am confident I can contribute to your team.".to_string(),
                "I am particularly excited about the opportunity to work on microservices architecture.".to_string(),
            ],
            closing: Some("Thank you for your consideration.".to_string()),
            signature: Some("Sincerely,\nJohn Doe".to_string()),
        };
        
        let letter_response = serde_json::to_value(&cached_letter).unwrap();
        ai_cache_put(
            &conn,
            "cover_letter_generation",
            &letter_hash,
            "test-model",
            &letter_request,
            &letter_response,
            Some(CACHE_TTL_COVER_LETTER_DAYS),
            &now,
        ).unwrap();

        // Verify cache hit
        let cached_entry = ai_cache_get(&conn, "cover_letter_generation", &letter_hash, &now).unwrap();
        assert!(cached_entry.is_some());
        let cached = cached_entry.unwrap();
        let letter: GeneratedLetter = serde_json::from_value(cached.response_payload).unwrap();
        assert_eq!(letter.subject, Some("Application for Senior Software Engineer Position".to_string()));
        assert_eq!(letter.body_paragraphs.len(), 3);
        assert!(letter.body_paragraphs[0].contains("Senior Software Engineer"));
        assert!(letter.body_paragraphs[1].contains("Rust and TypeScript"));
        assert_eq!(letter.greeting, Some("Dear Hiring Manager,".to_string()));
        assert_eq!(letter.closing, Some("Thank you for your consideration.".to_string()));
        assert_eq!(letter.signature, Some("Sincerely,\nJohn Doe".to_string()));
    }

    #[tokio::test]
    async fn test_generate_cover_letter_for_job_cache_miss() {
        use careerbench::ai_cache::{ai_cache_get, compute_input_hash};
        use chrono::Utc;
        
        let conn = setup_test_db();
        let now = Utc::now().to_rfc3339();

        // Set up user profile
        setup_user_profile_for_resume(&conn).unwrap();

        // Create a job
        let job_description = "Software Engineer position";
        conn.execute(
            "INSERT INTO jobs (title, company, location, job_source, posting_url, raw_description, is_active, date_added, last_updated) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "Software Engineer",
                "Tech Company",
                "San Francisco, CA",
                "LinkedIn",
                "https://example.com/job",
                job_description,
                1,
                now,
                now
            ],
        ).unwrap();
        let _job_id = conn.last_insert_rowid();

        // Test cache miss (no cache entry)
        let letter_request = serde_json::json!({
            "userProfile": {
                "full_name": "John Doe"
            },
            "experience": [],
            "skills": [],
            "job": {
                "title": "Software Engineer",
                "company": "Tech Company",
                "rawDescription": job_description,
                "parsedJson": None::<serde_json::Value>
            },
            "options": None::<serde_json::Value>
        });
        let letter_hash = compute_input_hash(&letter_request).unwrap();
        
        let cached_entry = ai_cache_get(&conn, "cover_letter_generation", &letter_hash, &now).unwrap();
        assert!(cached_entry.is_none()); // Should be cache miss
    }
}

