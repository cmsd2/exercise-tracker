mod commands;
mod error;
pub mod garmin;
mod state;

use tauri::Manager;
use tracing::info;

use exercise_tracker_core::db::Database;
use exercise_tracker_core::service::ActivityService;
use state::AppState;

fn init_logging(app_dir: &std::path::Path) {
    let log_dir = app_dir.join("logs");
    std::fs::create_dir_all(&log_dir).expect("failed to create log directory");

    let file_appender = tracing_appender::rolling::daily(&log_dir, "exercise-tracker.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Leak the guard so it lives for the entire process
    std::mem::forget(_guard);

    use tracing_subscriber::fmt;
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Log our crates at debug, sqlx at info (shows queries), reqwest at debug, everything else at warn
        EnvFilter::new(
            "exercise_tracker=debug,exercise_tracker_core=debug,exercise_tracker_lib=debug,sqlx=info,reqwest=debug,hyper=info,warn",
        )
    });

    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");

            init_logging(&app_dir);
            info!("Application starting, data dir: {}", app_dir.display());

            let db_path = app_dir.join("exercise-tracker.db");
            let url = format!("sqlite:{}", db_path.display());
            info!("Opening database: {}", url);
            let db = tauri::async_runtime::block_on(Database::open(&url))
                .expect("failed to open database");
            tauri::async_runtime::block_on(db.migrate())
                .expect("failed to run database migrations");
            info!("Database migrations complete");
            let service = ActivityService::new(db);
            app.manage(AppState { service });
            info!("Application setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::activity::create_activity,
            commands::activity::update_activity,
            commands::activity::delete_activity,
            commands::activity::list_activities,
            commands::activity::get_activity,
            commands::activity::weekly_summary,
            commands::preferences::get_preference,
            commands::preferences::set_preference,
            commands::garmin::garmin_start_login,
            commands::garmin::garmin_check_auth,
            commands::garmin::garmin_store_tokens,
            commands::garmin::garmin_sync_activities,
            commands::garmin::garmin_disconnect,
            commands::garmin::garmin_reprocess_fit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
