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
}

