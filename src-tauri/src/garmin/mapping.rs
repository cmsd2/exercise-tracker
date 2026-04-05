use exercise_tracker_core::model::{ActivitySubType, ActivityType, HrZone};

use super::types::GarminActivity;

/// Maps a Garmin activity type_key to our (ActivityType, Option<ActivitySubType>).
/// Returns None for unsupported types (they should be skipped).
pub fn map_activity_type(
    type_key: &str,
) -> Option<(ActivityType, Option<ActivitySubType>)> {
    match type_key {
        "running" => Some((ActivityType::Run, None)),
        "treadmill_running" => Some((ActivityType::Run, Some(ActivitySubType::Treadmill))),
        "trail_running" => Some((ActivityType::Run, Some(ActivitySubType::Trail))),
        "track_running" => Some((ActivityType::Run, Some(ActivitySubType::Track))),
        "cycling" => Some((ActivityType::Cycle, None)),
        "indoor_cycling" => Some((ActivityType::Cycle, Some(ActivitySubType::Indoor))),
        "road_biking" => Some((ActivityType::Cycle, Some(ActivitySubType::Road))),
        "mountain_biking" => Some((ActivityType::Cycle, Some(ActivitySubType::Mountain))),
        "virtual_ride" => Some((ActivityType::Cycle, Some(ActivitySubType::Indoor))),
        "lap_swimming" => Some((ActivityType::Swim, Some(ActivitySubType::Pool))),
        "open_water_swimming" => Some((ActivityType::Swim, Some(ActivitySubType::OpenWater))),
        "indoor_rowing" => Some((ActivityType::Row, Some(ActivitySubType::IndoorRow))),
        "rowing" => Some((ActivityType::Row, None)),
        "walking" => Some((ActivityType::Walk, None)),
        "casual_walking" => Some((ActivityType::Walk, Some(ActivitySubType::Casual))),
        "hiking" => Some((ActivityType::Hike, None)),
        _ => None,
    }
}

/// Maps average heart rate to an HR zone using standard 5-zone model.
pub fn map_hr_zone(avg_hr: Option<f64>) -> Option<HrZone> {
    let hr = avg_hr?;
    if hr < 60.0 {
        return None;
    }
    let pct = hr / 190.0 * 100.0;
    Some(if pct < 60.0 {
        HrZone::Zone1
    } else if pct < 70.0 {
        HrZone::Zone2
    } else if pct < 80.0 {
        HrZone::Zone3
    } else if pct < 90.0 {
        HrZone::Zone4
    } else {
        HrZone::Zone5
    })
}

/// Compute pace in seconds per metre from Garmin's distance (m) and duration (s).
pub fn compute_pace(activity: &GarminActivity) -> Option<f64> {
    let distance = activity.distance?;
    let duration = activity.duration?;
    if distance > 0.0 {
        Some(duration / distance)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_activity_types() {
        assert_eq!(
            map_activity_type("running"),
            Some((ActivityType::Run, None))
        );
        assert_eq!(
            map_activity_type("treadmill_running"),
            Some((ActivityType::Run, Some(ActivitySubType::Treadmill)))
        );
        assert_eq!(
            map_activity_type("trail_running"),
            Some((ActivityType::Run, Some(ActivitySubType::Trail)))
        );
        assert_eq!(
            map_activity_type("cycling"),
            Some((ActivityType::Cycle, None))
        );
        assert_eq!(
            map_activity_type("indoor_cycling"),
            Some((ActivityType::Cycle, Some(ActivitySubType::Indoor)))
        );
        assert_eq!(
            map_activity_type("mountain_biking"),
            Some((ActivityType::Cycle, Some(ActivitySubType::Mountain)))
        );
        assert_eq!(
            map_activity_type("lap_swimming"),
            Some((ActivityType::Swim, Some(ActivitySubType::Pool)))
        );
        assert_eq!(
            map_activity_type("rowing"),
            Some((ActivityType::Row, None))
        );
        assert_eq!(
            map_activity_type("indoor_rowing"),
            Some((ActivityType::Row, Some(ActivitySubType::IndoorRow)))
        );
        assert_eq!(
            map_activity_type("walking"),
            Some((ActivityType::Walk, None))
        );
        assert_eq!(
            map_activity_type("casual_walking"),
            Some((ActivityType::Walk, Some(ActivitySubType::Casual)))
        );
        assert_eq!(
            map_activity_type("hiking"),
            Some((ActivityType::Hike, None))
        );
        assert_eq!(map_activity_type("yoga"), None);
        assert_eq!(map_activity_type("strength_training"), None);
    }

    #[test]
    fn test_map_hr_zone() {
        assert_eq!(map_hr_zone(None), None);
        assert_eq!(map_hr_zone(Some(50.0)), None);
        assert_eq!(map_hr_zone(Some(100.0)), Some(HrZone::Zone1));
        assert_eq!(map_hr_zone(Some(120.0)), Some(HrZone::Zone2));
        assert_eq!(map_hr_zone(Some(140.0)), Some(HrZone::Zone3));
        assert_eq!(map_hr_zone(Some(160.0)), Some(HrZone::Zone4));
        assert_eq!(map_hr_zone(Some(180.0)), Some(HrZone::Zone5));
    }

    #[test]
    fn test_compute_pace() {
        let activity = GarminActivity {
            activity_id: 1,
            activity_name: None,
            activity_type: super::super::types::GarminActivityType {
                type_key: "running".into(),
            },
            start_time_local: None,
            duration: Some(1800.0),
            distance: Some(5000.0),
            average_speed: None,
            max_hr: None,
            average_hr: None,
            description: None,
        };
        let pace = compute_pace(&activity).unwrap();
        assert!((pace - 0.36).abs() < 0.001);

        let no_dist = GarminActivity {
            distance: None,
            ..activity.clone()
        };
        assert_eq!(compute_pace(&no_dist), None);

        let zero_dist = GarminActivity {
            distance: Some(0.0),
            ..activity.clone()
        };
        assert_eq!(compute_pace(&zero_dist), None);
    }
}
