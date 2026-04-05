use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActivityType {
    Run,
    Cycle,
    Swim,
    Row,
    Walk,
    Hike,
}

impl ActivityType {
    pub fn all() -> &'static [ActivityType] {
        &[
            Self::Run,
            Self::Cycle,
            Self::Swim,
            Self::Row,
            Self::Walk,
            Self::Hike,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Cycle => "cycle",
            Self::Swim => "swim",
            Self::Row => "row",
            Self::Walk => "walk",
            Self::Hike => "hike",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "run" => Some(Self::Run),
            "cycle" => Some(Self::Cycle),
            "swim" => Some(Self::Swim),
            "row" => Some(Self::Row),
            "walk" => Some(Self::Walk),
            "hike" => Some(Self::Hike),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HrZone {
    Zone1 = 1,
    Zone2 = 2,
    Zone3 = 3,
    Zone4 = 4,
    Zone5 = 5,
}

impl HrZone {
    pub fn from_int(n: i32) -> Option<Self> {
        match n {
            1 => Some(Self::Zone1),
            2 => Some(Self::Zone2),
            3 => Some(Self::Zone3),
            4 => Some(Self::Zone4),
            5 => Some(Self::Zone5),
            _ => None,
        }
    }

    pub fn as_int(&self) -> i32 {
        *self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActivitySubType {
    // Run
    Treadmill,
    Trail,
    Track,
    // Cycle
    Indoor,
    Road,
    Mountain,
    // Swim
    Pool,
    OpenWater,
    // Row
    IndoorRow,
    // Walk
    Casual,
}

impl ActivitySubType {
    pub fn sub_types_for(activity_type: ActivityType) -> &'static [ActivitySubType] {
        match activity_type {
            ActivityType::Run => &[Self::Treadmill, Self::Trail, Self::Track],
            ActivityType::Cycle => &[Self::Indoor, Self::Road, Self::Mountain],
            ActivityType::Swim => &[Self::Pool, Self::OpenWater],
            ActivityType::Row => &[Self::IndoorRow],
            ActivityType::Walk => &[Self::Casual],
            ActivityType::Hike => &[],
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Treadmill => "treadmill",
            Self::Trail => "trail",
            Self::Track => "track",
            Self::Indoor => "indoor",
            Self::Road => "road",
            Self::Mountain => "mountain",
            Self::Pool => "pool",
            Self::OpenWater => "open-water",
            Self::IndoorRow => "indoor-row",
            Self::Casual => "casual",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "treadmill" => Some(Self::Treadmill),
            "trail" => Some(Self::Trail),
            "track" => Some(Self::Track),
            "indoor" => Some(Self::Indoor),
            "road" => Some(Self::Road),
            "mountain" => Some(Self::Mountain),
            "pool" => Some(Self::Pool),
            "open-water" => Some(Self::OpenWater),
            "indoor-row" => Some(Self::IndoorRow),
            "casual" => Some(Self::Casual),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Treadmill => "Treadmill",
            Self::Trail => "Trail",
            Self::Track => "Track",
            Self::Indoor => "Indoor",
            Self::Road => "Road",
            Self::Mountain => "Mountain",
            Self::Pool => "Pool",
            Self::OpenWater => "Open Water",
            Self::IndoorRow => "Indoor",
            Self::Casual => "Casual",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: Uuid,
    pub activity_type: ActivityType,
    pub date: NaiveDateTime,
    pub duration_secs: Option<f64>,
    pub distance_m: Option<f64>,
    pub pace_s_per_m: Option<f64>,
    pub hr_zone: Option<HrZone>,
    pub notes: Option<String>,
    pub sub_type: Option<ActivitySubType>,
    pub is_commute: bool,
    pub is_race: bool,
    pub fit_data: Option<String>,
    pub fit_version: i32,
    pub source: Option<String>,
    pub source_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
