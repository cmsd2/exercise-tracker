use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarminTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GarminActivity {
    pub activity_id: u64,
    pub activity_name: Option<String>,
    pub activity_type: GarminActivityType,
    pub start_time_local: Option<String>,
    pub duration: Option<f64>,
    pub distance: Option<f64>,
    pub average_speed: Option<f64>,
    #[serde(alias = "maxHR")]
    pub max_hr: Option<f64>,
    #[serde(alias = "averageHR")]
    pub average_hr: Option<f64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GarminActivityType {
    pub type_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SyncProgress {
    Started { total: usize },
    Activity { current: usize, total: usize },
    Skipped { current: usize, total: usize, reason: String },
    Updating { current: usize, total: usize },
    Finished { imported: usize, skipped: usize, errors: usize, updated: usize },
}

// ── FIT file parsed structures ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitDetail {
    pub session: Option<FitSession>,
    pub laps: Vec<FitLap>,
    pub records: Vec<FitRecord>,
    pub device_info: Vec<FitDeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitSession {
    pub sport: Option<String>,
    pub sub_sport: Option<String>,
    pub total_elapsed_time: Option<f64>,
    pub total_timer_time: Option<f64>,
    pub total_distance: Option<f64>,
    pub total_calories: Option<u32>,
    pub avg_heart_rate: Option<u8>,
    pub max_heart_rate: Option<u8>,
    pub avg_cadence: Option<u8>,
    pub max_cadence: Option<u8>,
    pub avg_power: Option<u16>,
    pub max_power: Option<u16>,
    pub total_ascent: Option<u16>,
    pub total_descent: Option<u16>,
    pub avg_speed: Option<f64>,
    pub max_speed: Option<f64>,
    pub avg_temperature: Option<i8>,
    pub training_stress_score: Option<f64>,
    pub intensity_factor: Option<f64>,
    pub threshold_power: Option<u16>,
    pub normalized_power: Option<u16>,
    pub swim_stroke: Option<String>,
    pub pool_length: Option<f64>,
    pub num_laps: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitLap {
    pub start_time: Option<String>,
    pub total_elapsed_time: Option<f64>,
    pub total_timer_time: Option<f64>,
    pub total_distance: Option<f64>,
    pub total_calories: Option<u32>,
    pub avg_heart_rate: Option<u8>,
    pub max_heart_rate: Option<u8>,
    pub avg_cadence: Option<u8>,
    pub max_cadence: Option<u8>,
    pub avg_power: Option<u16>,
    pub max_power: Option<u16>,
    pub avg_speed: Option<f64>,
    pub max_speed: Option<f64>,
    pub total_ascent: Option<u16>,
    pub total_descent: Option<u16>,
    pub intensity: Option<String>,
    pub lap_trigger: Option<String>,
    pub swim_stroke: Option<String>,
    pub num_lengths: Option<u16>,
    pub avg_stroke_count: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitRecord {
    pub timestamp: Option<String>,
    pub position_lat: Option<f64>,
    pub position_long: Option<f64>,
    pub altitude: Option<f64>,
    pub heart_rate: Option<u8>,
    pub cadence: Option<u8>,
    pub distance: Option<f64>,
    pub speed: Option<f64>,
    pub power: Option<u16>,
    pub temperature: Option<i8>,
    pub enhanced_altitude: Option<f64>,
    pub enhanced_speed: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitDeviceInfo {
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<u32>,
    pub software_version: Option<f64>,
    pub device_index: Option<u8>,
    pub device_type: Option<String>,
    pub battery_voltage: Option<f64>,
    pub battery_status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens_serialization_round_trip() {
        let tokens = GarminTokens {
            access_token: "abc123".into(),
            refresh_token: Some("refresh456".into()),
            expires_at: Some(1712345678),
        };
        let json = serde_json::to_string(&tokens).unwrap();
        let parsed: GarminTokens = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.access_token, "abc123");
        assert_eq!(parsed.refresh_token, Some("refresh456".into()));
        assert_eq!(parsed.expires_at, Some(1712345678));
    }

    #[test]
    fn test_tokens_without_optionals() {
        let tokens = GarminTokens {
            access_token: "token".into(),
            refresh_token: None,
            expires_at: None,
        };
        let json = serde_json::to_string(&tokens).unwrap();
        let parsed: GarminTokens = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.access_token, "token");
        assert_eq!(parsed.refresh_token, None);
        assert_eq!(parsed.expires_at, None);
    }

    #[test]
    fn test_garmin_activity_deserialization() {
        let json = r#"{
            "activityId": 12345678,
            "activityName": "Morning Run",
            "activityType": {"typeKey": "running"},
            "startTimeLocal": "2026-04-01 07:30:00",
            "duration": 1800.0,
            "distance": 5000.0,
            "averageSpeed": 2.78,
            "maxHR": 175.0,
            "averageHR": 155.0,
            "description": null
        }"#;
        let activity: GarminActivity = serde_json::from_str(json).unwrap();
        assert_eq!(activity.activity_id, 12345678);
        assert_eq!(activity.activity_name, Some("Morning Run".into()));
        assert_eq!(activity.activity_type.type_key, "running");
        assert_eq!(activity.duration, Some(1800.0));
        assert_eq!(activity.distance, Some(5000.0));
        assert_eq!(activity.average_hr, Some(155.0));
    }
}
