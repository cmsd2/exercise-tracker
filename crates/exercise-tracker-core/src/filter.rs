use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::model::{ActivitySubType, ActivityType};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivityFilter {
    pub activity_type: Option<ActivityType>,
    pub sub_type: Option<ActivitySubType>,
    pub date_from: Option<NaiveDateTime>,
    pub date_to: Option<NaiveDateTime>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
