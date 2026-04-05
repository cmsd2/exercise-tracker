use chrono::NaiveDateTime;
use tauri::State;
use tracing::{debug, error, info};
use uuid::Uuid;

use exercise_tracker_core::commands::{ActivityCommand, ActivityEffect};
use exercise_tracker_core::filter::ActivityFilter;
use exercise_tracker_core::model::{Activity, ActivitySubType, ActivityType, HrZone};
use exercise_tracker_core::summary::WeeklySummary;

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn create_activity(
    state: State<'_, AppState>,
    activity_type: ActivityType,
    date: String,
    duration_secs: Option<f64>,
    distance_m: Option<f64>,
    pace_s_per_m: Option<f64>,
    hr_zone: Option<HrZone>,
    notes: Option<String>,
    sub_type: Option<ActivitySubType>,
    is_commute: Option<bool>,
    is_race: Option<bool>,
) -> Result<ActivityEffect, AppError> {
    info!(
        "create_activity: type={:?}, date={}, duration={:?}, distance={:?}",
        activity_type, date, duration_secs, distance_m
    );

    let date = NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M"))
        .map_err(|e| {
            error!("create_activity: invalid date format: {}", e);
            AppError {
                message: format!("Invalid date format: {}", e),
            }
        })?;

    let effect = state
        .service
        .apply(ActivityCommand::Create {
            activity_type,
            date,
            duration_secs,
            distance_m,
            pace_s_per_m,
            hr_zone,
            notes,
            sub_type,
            is_commute: is_commute.unwrap_or(false),
            is_race: is_race.unwrap_or(false),
            fit_data: None,
            fit_version: 0,
            source: None,
            source_id: None,
        })
        .await
        .map_err(|e| {
            error!("create_activity: {}", e);
            AppError::from(e)
        })?;
    debug!("create_activity: result={:?}", effect);
    Ok(effect)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_activity(
    state: State<'_, AppState>,
    id: String,
    activity_type: Option<ActivityType>,
    date: Option<String>,
    duration_secs: Option<Option<f64>>,
    distance_m: Option<Option<f64>>,
    pace_s_per_m: Option<Option<f64>>,
    hr_zone: Option<Option<HrZone>>,
    notes: Option<Option<String>>,
    sub_type: Option<Option<ActivitySubType>>,
    is_commute: Option<bool>,
    is_race: Option<bool>,
) -> Result<ActivityEffect, AppError> {
    info!("update_activity: id={}", id);

    let id = Uuid::parse_str(&id).map_err(|e| {
        error!("update_activity: invalid UUID: {}", e);
        AppError {
            message: format!("Invalid UUID: {}", e),
        }
    })?;

    let date = date
        .map(|d| {
            NaiveDateTime::parse_from_str(&d, "%Y-%m-%dT%H:%M:%S")
                .or_else(|_| NaiveDateTime::parse_from_str(&d, "%Y-%m-%dT%H:%M"))
                .map_err(|e| {
                    error!("update_activity: invalid date format: {}", e);
                    AppError {
                        message: format!("Invalid date format: {}", e),
                    }
                })
        })
        .transpose()?;

    let effect = state
        .service
        .apply(ActivityCommand::Update {
            id,
            activity_type,
            date,
            duration_secs,
            distance_m,
            pace_s_per_m,
            hr_zone,
            notes,
            sub_type,
            is_commute,
            is_race,
        })
        .await
        .map_err(|e| {
            error!("update_activity: {}", e);
            AppError::from(e)
        })?;
    debug!("update_activity: result={:?}", effect);
    Ok(effect)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_activity(
    state: State<'_, AppState>,
    id: String,
) -> Result<ActivityEffect, AppError> {
    info!("delete_activity: id={}", id);

    let id = Uuid::parse_str(&id).map_err(|e| {
        error!("delete_activity: invalid UUID: {}", e);
        AppError {
            message: format!("Invalid UUID: {}", e),
        }
    })?;

    let effect = state
        .service
        .apply(ActivityCommand::Delete { id })
        .await
        .map_err(|e| {
            error!("delete_activity: {}", e);
            AppError::from(e)
        })?;
    debug!("delete_activity: result={:?}", effect);
    Ok(effect)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_activities(
    state: State<'_, AppState>,
    filter: ActivityFilter,
) -> Result<Vec<Activity>, AppError> {
    debug!("list_activities: filter={:?}", filter);

    let activities = state.service.list_activities(&filter).await.map_err(|e| {
        error!("list_activities: {}", e);
        AppError::from(e)
    })?;
    debug!("list_activities: returned {} activities", activities.len());
    Ok(activities)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_activity(
    state: State<'_, AppState>,
    id: String,
) -> Result<Activity, AppError> {
    debug!("get_activity: id={}", id);

    let id = Uuid::parse_str(&id).map_err(|e| {
        error!("get_activity: invalid UUID: {}", e);
        AppError {
            message: format!("Invalid UUID: {}", e),
        }
    })?;

    let activity = state.service.get_activity(&id).await.map_err(|e| {
        error!("get_activity: {}", e);
        AppError::from(e)
    })?;
    Ok(activity)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn weekly_summary(
    state: State<'_, AppState>,
    activity_type: Option<ActivityType>,
) -> Result<Vec<WeeklySummary>, AppError> {
    debug!("weekly_summary: activity_type={:?}", activity_type);

    let summaries = state
        .service
        .weekly_summary(activity_type)
        .await
        .map_err(|e| {
            error!("weekly_summary: {}", e);
            AppError::from(e)
        })?;
    debug!("weekly_summary: returned {} weeks", summaries.len());
    Ok(summaries)
}
