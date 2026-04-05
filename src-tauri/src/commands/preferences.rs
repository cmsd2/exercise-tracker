use tauri::State;
use tracing::{debug, error};

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_preference(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<String>, AppError> {
    debug!("get_preference: key={}", key);
    let value = state.service.get_preference(&key).await.map_err(|e| {
        error!("get_preference: {}", e);
        AppError::from(e)
    })?;
    Ok(value)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_preference(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    debug!("set_preference: key={}", key);
    state
        .service
        .set_preference(&key, &value)
        .await
        .map_err(|e| {
            error!("set_preference: {}", e);
            AppError::from(e)
        })?;
    Ok(())
}
