use chrono::NaiveDateTime;
use tauri::webview::WebviewWindowBuilder;
use tauri::{Emitter, Manager, State};
use tracing::{debug, error, info, warn};

use exercise_tracker_core::commands::{ActivityCommand, ActivityEffect};

use crate::error::AppError;
use crate::garmin::client::GarminClient;
use crate::garmin::fit_parser::parse_fit_data;
use crate::garmin::mapping::{compute_pace, map_activity_type, map_hr_zone};
use crate::garmin::oauth_exchange;
use crate::garmin::types::{GarminTokens, SyncProgress};
use crate::state::AppState;

const GARMIN_LOGIN_URL: &str = "https://sso.garmin.com/portal/sso/en-US/sign-in?clientId=GarminConnect&service=https%3A%2F%2Fconnect.garmin.com%2Fapp";

// The init script is minimal — the real work happens in Rust via on_navigation.
const GARMIN_LOGIN_INIT_SCRIPT: &str = "";

const GARMIN_TOKENS_KEY: &str = "garmin_tokens";
pub const CURRENT_FIT_VERSION: i32 = 1;

async fn get_stored_tokens(state: &State<'_, AppState>) -> Result<Option<GarminTokens>, AppError> {
    let json = state
        .service
        .get_preference(GARMIN_TOKENS_KEY)
        .await
        .map_err(|e| {
            error!("Failed to read Garmin tokens from DB: {}", e);
            AppError {
                message: format!("Failed to read tokens: {}", e),
            }
        })?;
    match json {
        Some(j) => {
            let tokens: GarminTokens = serde_json::from_str(&j).map_err(|e| {
                error!("Failed to parse stored Garmin tokens: {}", e);
                AppError {
                    message: format!("Failed to parse tokens: {}", e),
                }
            })?;
            Ok(Some(tokens))
        }
        None => Ok(None),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn garmin_start_login(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), AppError> {
    info!("garmin_start_login: opening login window");

    // If login window already exists, focus it
    if let Some(existing) = app.get_webview_window("garmin-login") {
        info!("garmin_start_login: login window already open, focusing");
        let _ = existing.set_focus();
        return Ok(());
    }

    let service = state.service.clone();
    let app_handle = app.clone();

    let login_window = WebviewWindowBuilder::new(&app, "garmin-login", tauri::WebviewUrl::External(
        GARMIN_LOGIN_URL.parse().map_err(|e| AppError {
            message: format!("Invalid login URL: {}", e),
        })?,
    ))
    .title("Connect to Garmin")
    .inner_size(480.0, 700.0)
    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15")
    .initialization_script(GARMIN_LOGIN_INIT_SCRIPT)
    .on_navigation(move |url| {
        // Intercept the SSO redirect back to connect.garmin.com with a CAS ticket
        if url.host_str() == Some("connect.garmin.com") {
            if let Some(ticket) = url
                .query_pairs()
                .find(|(k, _)| k == "ticket")
                .map(|(_, v)| v.into_owned())
            {
                info!("garmin_start_login: intercepted CAS ticket");
                let svc = service.clone();
                let handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    match oauth_exchange::exchange_ticket(&ticket).await {
                        Ok(tokens) => {
                            match serde_json::to_string(&tokens) {
                                Ok(json) => {
                                    if let Err(e) =
                                        svc.set_preference(GARMIN_TOKENS_KEY, &json).await
                                    {
                                        error!("garmin_start_login: failed to store tokens: {}", e);
                                    } else {
                                        info!("garmin_start_login: tokens stored successfully");
                                        let _ = handle.emit("garmin-auth-complete", ());
                                    }
                                }
                                Err(e) => {
                                    error!("garmin_start_login: serialize error: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("garmin_start_login: OAuth exchange failed: {}", e);
                            let _ = handle.emit("garmin-auth-error", e);
                        }
                    }
                    // Close the login window
                    if let Some(win) = handle.get_webview_window("garmin-login") {
                        let _ = win.close();
                    }
                });
                // Block navigation — we consume the ticket in Rust instead
                return false;
            }
        }
        true
    })
    .build()
    .map_err(|e| {
        error!("garmin_start_login: failed to create window: {}", e);
        AppError {
            message: format!("Failed to open login window: {}", e),
        }
    })?;

    let _ = login_window.set_focus();
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn garmin_check_auth(state: State<'_, AppState>) -> Result<bool, AppError> {
    debug!("garmin_check_auth");
    let tokens = get_stored_tokens(&state).await?;
    let authenticated = tokens.is_some();
    debug!("garmin_check_auth: authenticated={}", authenticated);
    Ok(authenticated)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn garmin_store_tokens(
    state: State<'_, AppState>,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<i64>,
) -> Result<(), AppError> {
    info!("garmin_store_tokens: storing new tokens");
    let tokens = GarminTokens {
        access_token,
        refresh_token,
        expires_at,
    };
    let json = serde_json::to_string(&tokens).map_err(|e| {
        error!("garmin_store_tokens: failed to serialize: {}", e);
        AppError {
            message: format!("Failed to serialize tokens: {}", e),
        }
    })?;
    state
        .service
        .set_preference(GARMIN_TOKENS_KEY, &json)
        .await
        .map_err(|e| {
            error!("garmin_store_tokens: failed to store: {}", e);
            AppError {
                message: format!("Failed to store tokens: {}", e),
            }
        })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn garmin_sync_activities(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
    app: tauri::AppHandle,
) -> Result<SyncProgress, AppError> {
    info!(
        "garmin_sync_activities: start_date={}, end_date={}",
        start_date, end_date
    );

    // Load tokens
    let tokens = get_stored_tokens(&state).await?;
    let tokens = tokens.ok_or_else(|| {
        warn!("garmin_sync_activities: no tokens stored");
        AppError {
            message: "Not connected to Garmin. Please authenticate first.".into(),
        }
    })?;

    let client = GarminClient::new(tokens).map_err(|e| {
        error!("garmin_sync_activities: failed to create client: {}", e);
        AppError {
            message: format!("Failed to create Garmin client: {}", e),
        }
    })?;

    if client.is_token_expired() {
        warn!("garmin_sync_activities: token expired, clearing");
        let _ = state.service.set_preference(GARMIN_TOKENS_KEY, "").await;
        return Err(AppError {
            message: "Garmin token has expired. Please reconnect.".into(),
        });
    }

    info!("garmin_sync_activities: fetching activities from Garmin API");
    let activities = client
        .fetch_activities(&start_date, &end_date)
        .await
        .map_err(|e| {
            let status = e.status().map(|s| s.as_u16());
            error!(
                "garmin_sync_activities: fetch failed, status={:?}, error={}",
                status, e
            );
            if status == Some(401) {
                warn!("garmin_sync_activities: 401 response, clearing tokens");
                let service = state.service.clone();
                tokio::spawn(async move {
                    let _ = service.set_preference(GARMIN_TOKENS_KEY, "").await;
                });
            }
            AppError {
                message: format!("Failed to fetch Garmin activities: {}", e),
            }
        })?;

    let total = activities.len();
    info!(
        "garmin_sync_activities: fetched {} activities from Garmin",
        total
    );
    let _ = app.emit("garmin-sync-progress", SyncProgress::Started { total });

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    for (i, garmin_activity) in activities.iter().enumerate() {
        let current = i + 1;

        // Map activity type; skip unsupported types
        let (activity_type, sub_type) =
            match map_activity_type(&garmin_activity.activity_type.type_key) {
                Some(t) => t,
                None => {
                    skipped += 1;
                    debug!(
                        "garmin_sync: skipping unsupported type: {}",
                        garmin_activity.activity_type.type_key
                    );
                    let _ = app.emit(
                        "garmin-sync-progress",
                        SyncProgress::Skipped {
                            current,
                            total,
                            reason: format!(
                                "Unsupported type: {}",
                                garmin_activity.activity_type.type_key
                            ),
                        },
                    );
                    continue;
                }
            };

        let source_id = garmin_activity.activity_id.to_string();

        // Check for duplicate
        match state
            .service
            .activity_exists_by_source("garmin", &source_id)
            .await
        {
            Ok(true) => {
                skipped += 1;
                debug!(
                    "garmin_sync: skipping duplicate activity_id={}",
                    source_id
                );
                let _ = app.emit(
                    "garmin-sync-progress",
                    SyncProgress::Skipped {
                        current,
                        total,
                        reason: "Already imported".into(),
                    },
                );
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                errors += 1;
                error!(
                    "garmin_sync: DB error checking source existence: {}",
                    e
                );
                let _ = app.emit(
                    "garmin-sync-progress",
                    SyncProgress::Skipped {
                        current,
                        total,
                        reason: format!("DB error: {}", e),
                    },
                );
                continue;
            }
        }

        // Parse date
        let date = garmin_activity
            .start_time_local
            .as_deref()
            .and_then(|s| {
                NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                    .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
                    .ok()
            })
            .unwrap_or_default();

        let pace_s_per_m = compute_pace(garmin_activity);
        let hr_zone = map_hr_zone(garmin_activity.average_hr);

        let notes = garmin_activity
            .activity_name
            .as_deref()
            .or(garmin_activity.description.as_deref())
            .map(|s| s.to_string());

        // Download and parse FIT file (graceful fallback on failure)
        debug!(
            "garmin_sync: downloading FIT file for activity_id={}",
            garmin_activity.activity_id
        );
        let (fit_data, fit_version) = match client
            .download_fit_file(garmin_activity.activity_id)
            .await
        {
            Ok(bytes) => {
                debug!(
                    "garmin_sync: downloaded {} bytes for activity_id={}",
                    bytes.len(),
                    garmin_activity.activity_id
                );
                match parse_fit_data(&bytes) {
                    Ok(detail) => match serde_json::to_string(&detail) {
                        Ok(json) => (Some(json), CURRENT_FIT_VERSION),
                        Err(e) => {
                            error!(
                                "garmin_sync: FIT serialize error for activity {}: {}",
                                garmin_activity.activity_id, e
                            );
                            (None, 0)
                        }
                    },
                    Err(e) => {
                        error!(
                            "garmin_sync: FIT parse error for activity {}: {}",
                            garmin_activity.activity_id, e
                        );
                        (None, 0)
                    }
                }
            }
            Err(e) => {
                error!(
                    "garmin_sync: FIT download error for activity {}: {}",
                    garmin_activity.activity_id, e
                );
                (None, 0)
            }
        };

        let cmd = ActivityCommand::Create {
            activity_type,
            date,
            duration_secs: garmin_activity.duration,
            distance_m: garmin_activity.distance,
            pace_s_per_m,
            hr_zone,
            notes,
            sub_type,
            is_commute: false,
            is_race: false,
            fit_data,
            fit_version,
            source: Some("garmin".into()),
            source_id: Some(source_id.clone()),
        };

        match state.service.apply(cmd).await {
            Ok(ActivityEffect::Created { id }) => {
                imported += 1;
                info!(
                    "garmin_sync: imported activity_id={} as id={}",
                    source_id, id
                );
                let _ = app.emit(
                    "garmin-sync-progress",
                    SyncProgress::Activity { current, total },
                );
            }
            Ok(ActivityEffect::ValidationError { reason }) => {
                skipped += 1;
                warn!(
                    "garmin_sync: validation error for activity_id={}: {}",
                    source_id, reason
                );
                let _ = app.emit(
                    "garmin-sync-progress",
                    SyncProgress::Skipped {
                        current,
                        total,
                        reason,
                    },
                );
            }
            Ok(_) => {
                errors += 1;
                error!(
                    "garmin_sync: unexpected effect for activity_id={}",
                    source_id
                );
            }
            Err(e) => {
                errors += 1;
                error!(
                    "garmin_sync: error creating activity_id={}: {}",
                    source_id, e
                );
                let _ = app.emit(
                    "garmin-sync-progress",
                    SyncProgress::Skipped {
                        current,
                        total,
                        reason: format!("Error: {}", e),
                    },
                );
            }
        }
    }

    let result = SyncProgress::Finished {
        imported,
        skipped,
        errors,
    };
    info!(
        "garmin_sync_activities: finished — imported={}, skipped={}, errors={}",
        imported, skipped, errors
    );
    let _ = app.emit("garmin-sync-progress", result.clone());
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn garmin_disconnect(state: State<'_, AppState>) -> Result<(), AppError> {
    info!("garmin_disconnect: clearing stored tokens");
    state
        .service
        .set_preference(GARMIN_TOKENS_KEY, "")
        .await
        .map_err(|e| {
            error!("garmin_disconnect: failed to clear tokens: {}", e);
            AppError {
                message: format!("Failed to clear tokens: {}", e),
            }
        })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn garmin_reprocess_fit(state: State<'_, AppState>) -> Result<u32, AppError> {
    info!("garmin_reprocess_fit: starting");

    // Load tokens
    let tokens = get_stored_tokens(&state).await?;
    let tokens = tokens.ok_or_else(|| {
        warn!("garmin_reprocess_fit: no tokens stored");
        AppError {
            message: "Not connected to Garmin. Please authenticate first.".into(),
        }
    })?;

    let client = GarminClient::new(tokens).map_err(|e| {
        error!("garmin_reprocess_fit: failed to create client: {}", e);
        AppError {
            message: format!("Failed to create Garmin client: {}", e),
        }
    })?;

    // Find activities needing re-processing
    let stale = state
        .service
        .list_stale_fit_activities(CURRENT_FIT_VERSION)
        .await
        .map_err(|e| {
            error!("garmin_reprocess_fit: failed to list stale: {}", e);
            AppError {
                message: format!("Failed to list stale activities: {}", e),
            }
        })?;
    info!(
        "garmin_reprocess_fit: found {} stale activities",
        stale.len()
    );

    let mut updated = 0u32;
    for (id, source_id) in &stale {
        let activity_id: u64 = match source_id.parse() {
            Ok(v) => v,
            Err(_) => {
                warn!(
                    "garmin_reprocess_fit: invalid source_id={} for id={}",
                    source_id, id
                );
                continue;
            }
        };

        debug!(
            "garmin_reprocess_fit: downloading FIT for activity_id={}",
            activity_id
        );
        let (fit_data, fit_version) = match client.download_fit_file(activity_id).await {
            Ok(bytes) => {
                debug!(
                    "garmin_reprocess_fit: downloaded {} bytes for activity_id={}",
                    bytes.len(),
                    activity_id
                );
                match parse_fit_data(&bytes) {
                    Ok(detail) => match serde_json::to_string(&detail) {
                        Ok(json) => (Some(json), CURRENT_FIT_VERSION),
                        Err(e) => {
                            error!(
                                "garmin_reprocess_fit: FIT serialize error for {}: {}",
                                activity_id, e
                            );
                            (None, 0)
                        }
                    },
                    Err(e) => {
                        error!(
                            "garmin_reprocess_fit: FIT parse error for {}: {}",
                            activity_id, e
                        );
                        (None, 0)
                    }
                }
            }
            Err(e) => {
                error!(
                    "garmin_reprocess_fit: FIT download error for {}: {}",
                    activity_id, e
                );
                (None, 0)
            }
        };

        match state
            .service
            .update_fit_data(id, fit_data.as_deref(), fit_version)
            .await
        {
            Ok(()) => {
                updated += 1;
                debug!("garmin_reprocess_fit: updated id={}", id);
            }
            Err(e) => {
                error!("garmin_reprocess_fit: update_fit_data error for {}: {}", id, e);
            }
        }
    }

    info!("garmin_reprocess_fit: finished, updated={}", updated);
    Ok(updated)
}
