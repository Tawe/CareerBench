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
mod calendar;
mod reminders;
mod portfolio_export;
mod analytics;
mod email;
mod learning;
mod recruiter_crm;
mod companies;

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
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
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
            commands::get_cache_stats,
            commands::clear_cache_by_purpose,
            commands::clear_all_cache,
            commands::cleanup_expired_cache,
            commands::evict_cache_by_size,
            commands::evict_cache_by_count,
            commands::export_dashboard_data,
            commands::get_calendar_events,
            commands::get_events_for_date,
            commands::sync_interview_to_calendar,
            commands::create_reminder,
            commands::get_reminders,
            commands::get_due_reminders,
            commands::get_reminders_for_application,
            commands::mark_reminder_sent,
            commands::delete_reminder,
            commands::export_portfolio_html,
            commands::export_portfolio_markdown,
            commands::export_portfolio_text,
            commands::get_portfolio_for_application,
            commands::link_portfolio_to_application,
            commands::get_applications_for_portfolio,
            commands::get_conversion_rates,
            commands::get_time_in_stage,
            commands::get_channel_effectiveness,
            commands::get_analytics_insights,
            commands::save_email_account,
            commands::get_email_accounts,
            commands::delete_email_account,
            commands::get_email_threads_for_application,
            commands::link_email_thread_to_application,
            commands::get_email_messages_for_thread,
            commands::test_email_connection,
            commands::sync_email_account,
            commands::analyze_skill_gaps,
            commands::create_learning_plan,
            commands::get_learning_plans,
            commands::get_learning_tracks,
            commands::get_learning_tasks,
            commands::create_learning_track,
            commands::create_learning_task,
            commands::complete_learning_task,
            commands::add_learning_resource,
            commands::get_learning_resources,
            commands::delete_learning_plan,
            commands::update_learning_plan_status,
            commands::generate_learning_content,
            commands::create_recruiter_contact,
            commands::get_recruiter_contacts,
            commands::get_recruiter_contact,
            commands::update_recruiter_contact,
            commands::delete_recruiter_contact,
            commands::create_interaction,
            commands::get_interactions_for_contact,
            commands::get_interactions_for_application,
            commands::link_contact_to_application,
            commands::get_contacts_for_application,
            commands::get_applications_for_contact,
            commands::unlink_contact_from_application,
            commands::delete_interaction,
            commands::create_company,
            commands::get_companies,
            commands::get_companies_with_stats,
            commands::get_company,
            commands::update_company,
            commands::delete_company,
            commands::link_job_to_company,
            commands::link_application_to_company,
            commands::unlink_job_from_company,
            commands::unlink_application_from_company,
            commands::fetch_company_info_from_url,
            commands::clear_company_fetch_cache,
            commands::download_model,
            commands::cleanup_invalid_model_files,
            commands::clear_invalid_model_path,
            commands::find_model_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

