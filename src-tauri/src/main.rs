// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod ai_cache;
mod ai_client;
mod ai;
mod commands;
mod resume_generator;
mod logging;

use db::init_database;

#[tokio::main]
async fn main() {
    // Initialize logging first (before any other operations)
    logging::init_logging();
    logging::setup_panic_hook();
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
            commands::test_ai_connection,
            commands::check_local_provider_availability,
            commands::get_artifacts_for_application,
            commands::get_artifacts_for_job,
            commands::get_artifact,
            commands::update_artifact,
            commands::update_artifact_title,
            commands::save_resume,
            commands::save_cover_letter,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

