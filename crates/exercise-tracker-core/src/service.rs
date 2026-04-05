use chrono::Utc;
use tracing::debug;
use uuid::Uuid;

use crate::commands::{ActivityCommand, ActivityEffect};
use crate::db::Database;
use crate::error::CoreError;
use crate::filter::ActivityFilter;
use crate::model::{Activity, ActivityType};
use crate::summary::WeeklySummary;

#[derive(Clone)]
pub struct ActivityService {
    db: Database,
}

impl ActivityService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn apply(&self, cmd: ActivityCommand) -> Result<ActivityEffect, CoreError> {
        debug!("service: apply command={:?}", std::mem::discriminant(&cmd));
        match cmd {
            ActivityCommand::Create {
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
                fit_data,
                fit_version,
                source,
                source_id,
            } => {
                if let Some(d) = duration_secs {
                    if d < 0.0 {
                        return Ok(ActivityEffect::ValidationError {
                            reason: "Duration must not be negative".into(),
                        });
                    }
                }
                if let Some(d) = distance_m {
                    if d < 0.0 {
                        return Ok(ActivityEffect::ValidationError {
                            reason: "Distance must not be negative".into(),
                        });
                    }
                }
                if let Some(p) = pace_s_per_m {
                    if p < 0.0 {
                        return Ok(ActivityEffect::ValidationError {
                            reason: "Pace must not be negative".into(),
                        });
                    }
                }
                if duration_secs.is_none() && distance_m.is_none() {
                    return Ok(ActivityEffect::ValidationError {
                        reason: "At least one of duration or distance is required".into(),
                    });
                }

                let now = Utc::now();
                let id = Uuid::new_v4();
                let activity = Activity {
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
                    fit_data,
                    fit_version,
                    source,
                    source_id,
                    created_at: now,
                    updated_at: now,
                };

                self.db.insert_activity(&activity).await?;
                Ok(ActivityEffect::Created { id })
            }

            ActivityCommand::Update {
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
            } => {
                let mut activity = self.db.get_activity(&id).await?;

                if let Some(at) = activity_type {
                    activity.activity_type = at;
                }
                if let Some(d) = date {
                    activity.date = d;
                }
                if let Some(d) = duration_secs {
                    activity.duration_secs = d;
                }
                if let Some(d) = distance_m {
                    activity.distance_m = d;
                }
                if let Some(p) = pace_s_per_m {
                    activity.pace_s_per_m = p;
                }
                if let Some(z) = hr_zone {
                    activity.hr_zone = z;
                }
                if let Some(n) = notes {
                    activity.notes = n;
                }
                if let Some(st) = sub_type {
                    activity.sub_type = st;
                }
                if let Some(c) = is_commute {
                    activity.is_commute = c;
                }
                if let Some(r) = is_race {
                    activity.is_race = r;
                }

                if let Some(d) = activity.duration_secs {
                    if d < 0.0 {
                        return Ok(ActivityEffect::ValidationError {
                            reason: "Duration must not be negative".into(),
                        });
                    }
                }
                if let Some(d) = activity.distance_m {
                    if d < 0.0 {
                        return Ok(ActivityEffect::ValidationError {
                            reason: "Distance must not be negative".into(),
                        });
                    }
                }
                if let Some(p) = activity.pace_s_per_m {
                    if p < 0.0 {
                        return Ok(ActivityEffect::ValidationError {
                            reason: "Pace must not be negative".into(),
                        });
                    }
                }
                if activity.duration_secs.is_none() && activity.distance_m.is_none() {
                    return Ok(ActivityEffect::ValidationError {
                        reason: "At least one of duration or distance is required".into(),
                    });
                }

                activity.updated_at = Utc::now();
                self.db.update_activity(&activity).await?;
                Ok(ActivityEffect::Updated { id })
            }

            ActivityCommand::Delete { id } => {
                self.db.delete_activity(&id).await?;
                Ok(ActivityEffect::Deleted { id })
            }
        }
    }

    pub async fn list_activities(
        &self,
        filter: &ActivityFilter,
    ) -> Result<Vec<Activity>, CoreError> {
        self.db.list_activities(filter).await
    }

    pub async fn get_activity(&self, id: &Uuid) -> Result<Activity, CoreError> {
        self.db.get_activity(id).await
    }

    pub async fn weekly_summary(
        &self,
        activity_type: Option<ActivityType>,
    ) -> Result<Vec<WeeklySummary>, CoreError> {
        self.db.weekly_summary(activity_type).await
    }

    pub async fn activity_exists_by_source(
        &self,
        source: &str,
        source_id: &str,
    ) -> Result<bool, CoreError> {
        self.db.activity_exists_by_source(source, source_id).await
    }

    pub async fn get_preference(&self, key: &str) -> Result<Option<String>, CoreError> {
        self.db.get_preference(key).await
    }

    pub async fn set_preference(&self, key: &str, value: &str) -> Result<(), CoreError> {
        self.db.set_preference(key, value).await
    }

    pub async fn update_fit_data(
        &self,
        id: &Uuid,
        fit_data: Option<&str>,
        fit_version: i32,
    ) -> Result<(), CoreError> {
        self.db.update_fit_data(id, fit_data, fit_version).await
    }

    pub async fn list_stale_fit_activities(
        &self,
        current_version: i32,
    ) -> Result<Vec<(Uuid, String)>, CoreError> {
        self.db.list_stale_fit_activities(current_version).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::model::HrZone;

    async fn test_service() -> ActivityService {
        let db = Database::open_in_memory().await.unwrap();
        db.migrate().await.unwrap();
        ActivityService::new(db)
    }

    fn sample_date() -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 4, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap()
    }

    #[tokio::test]
    async fn test_create_and_get() {
        let svc = test_service().await;
        let effect = svc
            .apply(ActivityCommand::Create {
                activity_type: ActivityType::Run,
                date: sample_date(),
                duration_secs: Some(1800.0),
                distance_m: Some(5000.0),
                pace_s_per_m: Some(0.36),
                hr_zone: Some(HrZone::Zone3),
                notes: Some("Morning run".into()),
                sub_type: None,
                is_commute: false,
                is_race: false,
                fit_data: None,
                fit_version: 0,
                source: None,
                source_id: None,
            })
            .await
            .unwrap();

        let id = match effect {
            ActivityEffect::Created { id } => id,
            other => panic!("Expected Created, got {:?}", other),
        };

        let activity = svc.get_activity(&id).await.unwrap();
        assert_eq!(activity.activity_type, ActivityType::Run);
        assert_eq!(activity.duration_secs, Some(1800.0));
        assert_eq!(activity.distance_m, Some(5000.0));
        assert_eq!(activity.notes, Some("Morning run".into()));
    }

    #[tokio::test]
    async fn test_update() {
        let svc = test_service().await;
        let id = match svc
            .apply(ActivityCommand::Create {
                activity_type: ActivityType::Run,
                date: sample_date(),
                duration_secs: Some(1800.0),
                distance_m: Some(5000.0),
                pace_s_per_m: None,
                hr_zone: None,
                notes: None,
                sub_type: None,
                is_commute: false,
                is_race: false,
                fit_data: None,
                fit_version: 0,
                source: None,
                source_id: None,
            })
            .await
            .unwrap()
        {
            ActivityEffect::Created { id } => id,
            _ => panic!("Expected Created"),
        };

        let effect = svc
            .apply(ActivityCommand::Update {
                id,
                activity_type: Some(ActivityType::Cycle),
                date: None,
                duration_secs: None,
                distance_m: Some(Some(10000.0)),
                pace_s_per_m: None,
                hr_zone: Some(Some(HrZone::Zone4)),
                notes: Some(Some("Updated notes".into())),
                sub_type: None,
                is_commute: None,
                is_race: None,
            })
            .await
            .unwrap();

        assert!(matches!(effect, ActivityEffect::Updated { .. }));

        let activity = svc.get_activity(&id).await.unwrap();
        assert_eq!(activity.activity_type, ActivityType::Cycle);
        assert_eq!(activity.distance_m, Some(10000.0));
        assert_eq!(activity.hr_zone, Some(HrZone::Zone4));
        assert_eq!(activity.notes, Some("Updated notes".into()));
        // duration unchanged
        assert_eq!(activity.duration_secs, Some(1800.0));
    }

    #[tokio::test]
    async fn test_delete() {
        let svc = test_service().await;
        let id = match svc
            .apply(ActivityCommand::Create {
                activity_type: ActivityType::Swim,
                date: sample_date(),
                duration_secs: Some(3600.0),
                distance_m: None,
                pace_s_per_m: None,
                hr_zone: None,
                notes: None,
                sub_type: None,
                is_commute: false,
                is_race: false,
                fit_data: None,
                fit_version: 0,
                source: None,
                source_id: None,
            })
            .await
            .unwrap()
        {
            ActivityEffect::Created { id } => id,
            _ => panic!("Expected Created"),
        };

        let effect = svc.apply(ActivityCommand::Delete { id }).await.unwrap();
        assert!(matches!(effect, ActivityEffect::Deleted { .. }));

        let result = svc.get_activity(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let svc = test_service().await;
        let result = svc
            .apply(ActivityCommand::Delete {
                id: Uuid::new_v4(),
            })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_negative_duration() {
        let svc = test_service().await;
        let effect = svc
            .apply(ActivityCommand::Create {
                activity_type: ActivityType::Run,
                date: sample_date(),
                duration_secs: Some(-100.0),
                distance_m: Some(5000.0),
                pace_s_per_m: None,
                hr_zone: None,
                notes: None,
                sub_type: None,
                is_commute: false,
                is_race: false,
                fit_data: None,
                fit_version: 0,
                source: None,
                source_id: None,
            })
            .await
            .unwrap();

        assert!(matches!(effect, ActivityEffect::ValidationError { .. }));
    }

    #[tokio::test]
    async fn test_validation_negative_distance() {
        let svc = test_service().await;
        let effect = svc
            .apply(ActivityCommand::Create {
                activity_type: ActivityType::Run,
                date: sample_date(),
                duration_secs: Some(1800.0),
                distance_m: Some(-5000.0),
                pace_s_per_m: None,
                hr_zone: None,
                notes: None,
                sub_type: None,
                is_commute: false,
                is_race: false,
                fit_data: None,
                fit_version: 0,
                source: None,
                source_id: None,
            })
            .await
            .unwrap();

        assert!(matches!(effect, ActivityEffect::ValidationError { .. }));
    }

    #[tokio::test]
    async fn test_validation_no_duration_or_distance() {
        let svc = test_service().await;
        let effect = svc
            .apply(ActivityCommand::Create {
                activity_type: ActivityType::Run,
                date: sample_date(),
                duration_secs: None,
                distance_m: None,
                pace_s_per_m: None,
                hr_zone: None,
                notes: None,
                sub_type: None,
                is_commute: false,
                is_race: false,
                fit_data: None,
                fit_version: 0,
                source: None,
                source_id: None,
            })
            .await
            .unwrap();

        assert!(matches!(effect, ActivityEffect::ValidationError { .. }));
    }

    #[tokio::test]
    async fn test_list_with_type_filter() {
        let svc = test_service().await;

        svc.apply(ActivityCommand::Create {
            activity_type: ActivityType::Run,
            date: sample_date(),
            duration_secs: Some(1800.0),
            distance_m: Some(5000.0),
            pace_s_per_m: None,
            hr_zone: None,
            notes: None,
            sub_type: None,
            is_commute: false,
            is_race: false,
            fit_data: None,
            fit_version: 0,
            source: None,
            source_id: None,
        })
        .await
        .unwrap();

        svc.apply(ActivityCommand::Create {
            activity_type: ActivityType::Cycle,
            date: sample_date(),
            duration_secs: Some(3600.0),
            distance_m: Some(20000.0),
            pace_s_per_m: None,
            hr_zone: None,
            notes: None,
            sub_type: None,
            is_commute: false,
            is_race: false,
            fit_data: None,
            fit_version: 0,
            source: None,
            source_id: None,
        })
        .await
        .unwrap();

        let all = svc
            .list_activities(&ActivityFilter::default())
            .await
            .unwrap();
        assert_eq!(all.len(), 2);

        let runs = svc
            .list_activities(&ActivityFilter {
                activity_type: Some(ActivityType::Run),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].activity_type, ActivityType::Run);
    }

    #[tokio::test]
    async fn test_weekly_summary() {
        let svc = test_service().await;

        svc.apply(ActivityCommand::Create {
            activity_type: ActivityType::Run,
            date: sample_date(),
            duration_secs: Some(1800.0),
            distance_m: Some(5000.0),
            pace_s_per_m: None,
            hr_zone: None,
            notes: None,
            sub_type: None,
            is_commute: false,
            is_race: false,
            fit_data: None,
            fit_version: 0,
            source: None,
            source_id: None,
        })
        .await
        .unwrap();

        svc.apply(ActivityCommand::Create {
            activity_type: ActivityType::Run,
            date: sample_date(),
            duration_secs: Some(2400.0),
            distance_m: Some(7000.0),
            pace_s_per_m: None,
            hr_zone: None,
            notes: None,
            sub_type: None,
            is_commute: false,
            is_race: false,
            fit_data: None,
            fit_version: 0,
            source: None,
            source_id: None,
        })
        .await
        .unwrap();

        let summaries = svc
            .weekly_summary(Some(ActivityType::Run))
            .await
            .unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].activity_count, 2);
        assert!((summaries[0].total_distance_m - 12000.0).abs() < 0.01);
        assert!((summaries[0].total_duration_secs - 4200.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_preferences() {
        let svc = test_service().await;

        assert_eq!(svc.get_preference("theme").await.unwrap(), None);

        svc.set_preference("theme", "dark").await.unwrap();
        assert_eq!(
            svc.get_preference("theme").await.unwrap(),
            Some("dark".into())
        );

        svc.set_preference("theme", "light").await.unwrap();
        assert_eq!(
            svc.get_preference("theme").await.unwrap(),
            Some("light".into())
        );
    }

    #[tokio::test]
    async fn test_create_with_source() {
        let svc = test_service().await;
        let effect = svc
            .apply(ActivityCommand::Create {
                activity_type: ActivityType::Run,
                date: sample_date(),
                duration_secs: Some(1800.0),
                distance_m: Some(5000.0),
                pace_s_per_m: None,
                hr_zone: None,
                notes: None,
                sub_type: None,
                is_commute: false,
                is_race: false,
                fit_data: None,
                fit_version: 0,
                source: Some("garmin".into()),
                source_id: Some("12345".into()),
            })
            .await
            .unwrap();

        let id = match effect {
            ActivityEffect::Created { id } => id,
            other => panic!("Expected Created, got {:?}", other),
        };

        let activity = svc.get_activity(&id).await.unwrap();
        assert_eq!(activity.source, Some("garmin".into()));
        assert_eq!(activity.source_id, Some("12345".into()));
    }

    #[tokio::test]
    async fn test_activity_exists_by_source() {
        let svc = test_service().await;

        assert!(!svc
            .activity_exists_by_source("garmin", "99999")
            .await
            .unwrap());

        svc.apply(ActivityCommand::Create {
            activity_type: ActivityType::Cycle,
            date: sample_date(),
            duration_secs: Some(3600.0),
            distance_m: Some(20000.0),
            pace_s_per_m: None,
            hr_zone: None,
            notes: None,
            sub_type: None,
            is_commute: false,
            is_race: false,
            fit_data: None,
            fit_version: 0,
            source: Some("garmin".into()),
            source_id: Some("99999".into()),
        })
        .await
        .unwrap();

        assert!(svc
            .activity_exists_by_source("garmin", "99999")
            .await
            .unwrap());
        assert!(!svc
            .activity_exists_by_source("garmin", "00000")
            .await
            .unwrap());
        assert!(!svc
            .activity_exists_by_source("strava", "99999")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_source_dedup_unique_constraint() {
        let svc = test_service().await;

        // First insert should succeed
        let effect = svc
            .apply(ActivityCommand::Create {
                activity_type: ActivityType::Run,
                date: sample_date(),
                duration_secs: Some(1800.0),
                distance_m: Some(5000.0),
                pace_s_per_m: None,
                hr_zone: None,
                notes: None,
                sub_type: None,
                is_commute: false,
                is_race: false,
                fit_data: None,
                fit_version: 0,
                source: Some("garmin".into()),
                source_id: Some("dup123".into()),
            })
            .await
            .unwrap();
        assert!(matches!(effect, ActivityEffect::Created { .. }));

        // Duplicate source+source_id should fail
        let result = svc
            .apply(ActivityCommand::Create {
                activity_type: ActivityType::Run,
                date: sample_date(),
                duration_secs: Some(2400.0),
                distance_m: Some(7000.0),
                pace_s_per_m: None,
                hr_zone: None,
                notes: None,
                sub_type: None,
                is_commute: false,
                is_race: false,
                fit_data: None,
                fit_version: 0,
                source: Some("garmin".into()),
                source_id: Some("dup123".into()),
            })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_migration_runs_twice() {
        // Ensure migration is idempotent
        let db = Database::open_in_memory().await.unwrap();
        db.migrate().await.unwrap();
        db.migrate().await.unwrap(); // second call should be a no-op
        let svc = ActivityService::new(db);
        // Should still work fine
        let activities = svc
            .list_activities(&ActivityFilter::default())
            .await
            .unwrap();
        assert_eq!(activities.len(), 0);
    }

    #[tokio::test]
    async fn test_create_with_sub_type_and_flags() {
        use crate::model::ActivitySubType;

        let svc = test_service().await;
        let effect = svc
            .apply(ActivityCommand::Create {
                activity_type: ActivityType::Run,
                date: sample_date(),
                duration_secs: Some(1800.0),
                distance_m: Some(5000.0),
                pace_s_per_m: None,
                hr_zone: None,
                notes: None,
                sub_type: Some(ActivitySubType::Treadmill),
                is_commute: false,
                is_race: true,
                fit_data: Some(
                    r#"{"session":null,"laps":[],"records":[],"device_info":null}"#.into(),
                ),
                fit_version: 1,
                source: None,
                source_id: None,
            })
            .await
            .unwrap();

        let id = match effect {
            ActivityEffect::Created { id } => id,
            other => panic!("Expected Created, got {:?}", other),
        };

        let activity = svc.get_activity(&id).await.unwrap();
        assert_eq!(activity.sub_type, Some(ActivitySubType::Treadmill));
        assert!(!activity.is_commute);
        assert!(activity.is_race);
        assert!(activity.fit_data.is_some());
        assert_eq!(activity.fit_version, 1);
    }

    #[tokio::test]
    async fn test_update_sub_type_and_flags() {
        use crate::model::ActivitySubType;

        let svc = test_service().await;
        let id = match svc
            .apply(ActivityCommand::Create {
                activity_type: ActivityType::Cycle,
                date: sample_date(),
                duration_secs: Some(3600.0),
                distance_m: Some(20000.0),
                pace_s_per_m: None,
                hr_zone: None,
                notes: None,
                sub_type: None,
                is_commute: false,
                is_race: false,
                fit_data: None,
                fit_version: 0,
                source: None,
                source_id: None,
            })
            .await
            .unwrap()
        {
            ActivityEffect::Created { id } => id,
            _ => panic!("Expected Created"),
        };

        svc.apply(ActivityCommand::Update {
            id,
            activity_type: None,
            date: None,
            duration_secs: None,
            distance_m: None,
            pace_s_per_m: None,
            hr_zone: None,
            notes: None,
            sub_type: Some(Some(ActivitySubType::Indoor)),
            is_commute: Some(true),
            is_race: None,
        })
        .await
        .unwrap();

        let activity = svc.get_activity(&id).await.unwrap();
        assert_eq!(activity.sub_type, Some(ActivitySubType::Indoor));
        assert!(activity.is_commute);
        assert!(!activity.is_race);
    }

    #[tokio::test]
    async fn test_filter_by_sub_type() {
        use crate::model::ActivitySubType;

        let svc = test_service().await;

        svc.apply(ActivityCommand::Create {
            activity_type: ActivityType::Run,
            date: sample_date(),
            duration_secs: Some(1800.0),
            distance_m: Some(5000.0),
            pace_s_per_m: None,
            hr_zone: None,
            notes: None,
            sub_type: Some(ActivitySubType::Treadmill),
            is_commute: false,
            is_race: false,
            fit_data: None,
            fit_version: 0,
            source: None,
            source_id: None,
        })
        .await
        .unwrap();

        svc.apply(ActivityCommand::Create {
            activity_type: ActivityType::Run,
            date: sample_date(),
            duration_secs: Some(2400.0),
            distance_m: Some(7000.0),
            pace_s_per_m: None,
            hr_zone: None,
            notes: None,
            sub_type: Some(ActivitySubType::Trail),
            is_commute: false,
            is_race: false,
            fit_data: None,
            fit_version: 0,
            source: None,
            source_id: None,
        })
        .await
        .unwrap();

        let treadmill = svc
            .list_activities(&ActivityFilter {
                sub_type: Some(ActivitySubType::Treadmill),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(treadmill.len(), 1);

        let all_runs = svc
            .list_activities(&ActivityFilter {
                activity_type: Some(ActivityType::Run),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(all_runs.len(), 2);
    }
}
