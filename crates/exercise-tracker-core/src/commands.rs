use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::{ActivitySubType, ActivityType, HrZone};

#[derive(Debug, Clone)]
pub enum ActivityCommand {
    Create {
        activity_type: ActivityType,
        date: NaiveDateTime,
        duration_secs: Option<f64>,
        distance_m: Option<f64>,
        pace_s_per_m: Option<f64>,
        hr_zone: Option<HrZone>,
        notes: Option<String>,
        sub_type: Option<ActivitySubType>,
        is_commute: bool,
        is_race: bool,
        fit_data: Option<String>,
        fit_version: i32,
        source: Option<String>,
        source_id: Option<String>,
    },
    Update {
        id: Uuid,
        activity_type: Option<ActivityType>,
        date: Option<NaiveDateTime>,
        duration_secs: Option<Option<f64>>,
        distance_m: Option<Option<f64>>,
        pace_s_per_m: Option<Option<f64>>,
        hr_zone: Option<Option<HrZone>>,
        notes: Option<Option<String>>,
        sub_type: Option<Option<ActivitySubType>>,
        is_commute: Option<bool>,
        is_race: Option<bool>,
    },
    Delete {
        id: Uuid,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActivityEffect {
    Created { id: Uuid },
    Updated { id: Uuid },
    Deleted { id: Uuid },
    ValidationError { reason: String },
}
