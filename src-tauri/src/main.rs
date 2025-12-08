// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod ai_cache;
mod ai_client;
mod ai;
mod commands;
mod resume_generator;
mod logging;
mod errors;
mod error_logging;
mod encryption;
mod secure_storage;
mod data_export;
mod data_deletion;
mod local_storage;
mod profile_import;
mod job_scraper;

use db::init_database;

#[tokio::main]
async fn main() {
    // Initialize logging first (before any other operations)
    logging::init_logging();
    logging::setup_panic_hook();
    error_logging::init_error_metrics();
    log::info!("CareerBench starting up...");
    
    // Initialize database
    if let Err(e) = init_database() {
        log::error!("Failed to initialize database: {}", e);
        eprintln!("Failed to initialize database: {}", e);
    } else {
        log::info!("Database initialized successfully");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_dashboard_data,
            commands::get_user_profile_data,
            commands::save_user_profile_data,
            commands::create_job,
            commands::update_job,
            commands::get_job_list,
            commands::get_job_detail,
            commands::parse_job_with_ai,
            commands::create_application,
            commands::update_application,
            commands::get_applications,
            commands::get_application_detail,
            commands::add_application_event,
            commands::archive_application,
            commands::generate_resume_for_job,
            commands::generate_cover_letter_for_job,
            commands::ai_resume_suggestions,
            commands::ai_cover_letter,
            commands::ai_skill_suggestions,
            commands::get_ai_settings,
            commands::save_ai_settings,
            commands::rotate_api_key,
            commands::get_api_key_metadata,
            commands::check_api_key_rotation_needed,
            commands::test_ai_connection,
            commands::check_local_provider_availability,
            commands::get_artifacts_for_application,
            commands::get_artifacts_for_job,
            commands::get_artifact,
            commands::update_artifact,
            commands::update_artifact_title,
            commands::save_resume,
            commands::save_cover_letter,
            commands::generate_profile_summary,
            commands::extract_skills_from_experience,
            commands::rewrite_portfolio_description,
            commands::export_all_data,
            commands::delete_job,
            commands::delete_application,
            commands::delete_artifact,
            commands::delete_profile_section,
            commands::delete_all_user_data,
            commands::get_deletion_summary,
            commands::get_storage_info,
            commands::verify_local_storage,
            commands::get_storage_size,
            commands::extract_resume_text,
            commands::extract_profile_from_resume,
            commands::scrape_job_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

