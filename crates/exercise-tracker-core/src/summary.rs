use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::model::ActivityType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklySummary {
    pub week_start: NaiveDate,
    pub activity_count: u32,
    pub total_distance_m: f64,
    pub total_duration_secs: f64,
    pub activity_type: Option<ActivityType>,
}
