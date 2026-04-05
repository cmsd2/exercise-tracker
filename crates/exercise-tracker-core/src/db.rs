use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::Row;
use tracing::debug;
use uuid::Uuid;

use crate::error::CoreError;
use crate::filter::ActivityFilter;
use crate::model::{Activity, ActivitySubType, ActivityType, HrZone};
use crate::summary::WeeklySummary;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn open(url: &str) -> Result<Self, CoreError> {
        let options: SqliteConnectOptions = url.parse::<SqliteConnectOptions>()?
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        Ok(Self { pool })
    }

    pub async fn open_in_memory() -> Result<Self, CoreError> {
        // Use max_connections(1) for in-memory so all queries share the same database
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), CoreError> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub async fn insert_activity(&self, activity: &Activity) -> Result<(), CoreError> {
        debug!(
            "db: insert_activity id={} type={:?} date={}",
            activity.id, activity.activity_type, activity.date
        );
        sqlx::query(
            "INSERT INTO activities (id, activity_type, date, duration_secs, distance_m, pace_s_per_m, hr_zone, notes, sub_type, is_commute, is_race, fit_data, fit_version, source, source_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(activity.id.to_string())
        .bind(activity.activity_type.as_str())
        .bind(activity.date.format("%Y-%m-%dT%H:%M:%S").to_string())
        .bind(activity.duration_secs)
        .bind(activity.distance_m)
        .bind(activity.pace_s_per_m)
        .bind(activity.hr_zone.map(|z| z.as_int()))
        .bind(activity.notes.as_deref())
        .bind(activity.sub_type.map(|s| s.as_str().to_string()))
        .bind(activity.is_commute as i32)
        .bind(activity.is_race as i32)
        .bind(activity.fit_data.as_deref())
        .bind(activity.fit_version)
        .bind(activity.source.as_deref())
        .bind(activity.source_id.as_deref())
        .bind(activity.created_at.to_rfc3339())
        .bind(activity.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_activity(&self, activity: &Activity) -> Result<(), CoreError> {
        debug!("db: update_activity id={}", activity.id);
        let result = sqlx::query(
            "UPDATE activities SET activity_type = ?, date = ?, duration_secs = ?, distance_m = ?, pace_s_per_m = ?, hr_zone = ?, notes = ?, sub_type = ?, is_commute = ?, is_race = ?, fit_data = ?, fit_version = ?, source = ?, source_id = ?, updated_at = ? WHERE id = ?",
        )
        .bind(activity.activity_type.as_str())
        .bind(activity.date.format("%Y-%m-%dT%H:%M:%S").to_string())
        .bind(activity.duration_secs)
        .bind(activity.distance_m)
        .bind(activity.pace_s_per_m)
        .bind(activity.hr_zone.map(|z| z.as_int()))
        .bind(activity.notes.as_deref())
        .bind(activity.sub_type.map(|s| s.as_str().to_string()))
        .bind(activity.is_commute as i32)
        .bind(activity.is_race as i32)
        .bind(activity.fit_data.as_deref())
        .bind(activity.fit_version)
        .bind(activity.source.as_deref())
        .bind(activity.source_id.as_deref())
        .bind(activity.updated_at.to_rfc3339())
        .bind(activity.id.to_string())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(activity.id.to_string()));
        }
        Ok(())
    }

    pub async fn delete_activity(&self, id: &Uuid) -> Result<(), CoreError> {
        debug!("db: delete_activity id={}", id);
        let result = sqlx::query("DELETE FROM activities WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub async fn get_activity(&self, id: &Uuid) -> Result<Activity, CoreError> {
        debug!("db: get_activity id={}", id);
        let row = sqlx::query(
            "SELECT id, activity_type, date, duration_secs, distance_m, pace_s_per_m, hr_zone, notes, sub_type, is_commute, is_race, fit_data, fit_version, source, source_id, created_at, updated_at FROM activities WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => Ok(row_to_activity(&row)),
            None => Err(CoreError::NotFound(id.to_string())),
        }
    }

    pub async fn list_activities(
        &self,
        filter: &ActivityFilter,
    ) -> Result<Vec<Activity>, CoreError> {
        debug!("db: list_activities filter={:?}", filter);
        let activity_type_str = filter.activity_type.map(|at| at.as_str().to_string());
        let sub_type_str = filter.sub_type.map(|st| st.as_str().to_string());
        let date_from_str = filter
            .date_from
            .map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string());
        let date_to_str = filter
            .date_to
            .map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string());
        let limit = filter.limit.map(|l| l as i64).unwrap_or(-1);
        let offset = filter.offset.map(|o| o as i64).unwrap_or(0);

        let rows = sqlx::query(
            "SELECT id, activity_type, date, duration_secs, distance_m, pace_s_per_m,
                    hr_zone, notes, sub_type, is_commute, is_race,
                    NULL AS fit_data,
                    fit_version, source, source_id, created_at, updated_at
             FROM activities
             WHERE (? IS NULL OR activity_type = ?)
               AND (? IS NULL OR sub_type = ?)
               AND (? IS NULL OR date >= ?)
               AND (? IS NULL OR date <= ?)
             ORDER BY date DESC
             LIMIT ?
             OFFSET ?",
        )
        .bind(activity_type_str.as_deref())
        .bind(activity_type_str.as_deref())
        .bind(sub_type_str.as_deref())
        .bind(sub_type_str.as_deref())
        .bind(date_from_str.as_deref())
        .bind(date_from_str.as_deref())
        .bind(date_to_str.as_deref())
        .bind(date_to_str.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        debug!("db: list_activities returned {} rows", rows.len());
        Ok(rows.iter().map(row_to_activity).collect())
    }

    pub async fn weekly_summary(
        &self,
        activity_type: Option<ActivityType>,
    ) -> Result<Vec<WeeklySummary>, CoreError> {
        debug!("db: weekly_summary activity_type={:?}", activity_type);
        let activity_type_str = activity_type.map(|at| at.as_str().to_string());

        let rows = sqlx::query(
            "SELECT
                date(date, 'weekday 0', '-6 days') as week_start,
                COUNT(*) as activity_count,
                COALESCE(SUM(distance_m), 0.0) as total_distance_m,
                COALESCE(SUM(duration_secs), 0.0) as total_duration_secs
             FROM activities
             WHERE (? IS NULL OR activity_type = ?)
             GROUP BY week_start
             ORDER BY week_start DESC
             LIMIT 12",
        )
        .bind(activity_type_str.as_deref())
        .bind(activity_type_str.as_deref())
        .fetch_all(&self.pool)
        .await?;

        let summaries = rows
            .iter()
            .map(|row| {
                let week_start_str: String = row.get("week_start");
                let activity_count: i32 = row.get("activity_count");
                let total_distance_m: f64 = row.get("total_distance_m");
                let total_duration_secs: f64 = row.get("total_duration_secs");

                WeeklySummary {
                    week_start: NaiveDate::parse_from_str(&week_start_str, "%Y-%m-%d")
                        .unwrap_or_default(),
                    activity_count: activity_count as u32,
                    total_distance_m,
                    total_duration_secs,
                    activity_type,
                }
            })
            .collect();

        Ok(summaries)
    }

    pub async fn get_preference(&self, key: &str) -> Result<Option<String>, CoreError> {
        debug!("db: get_preference key={}", key);
        let row: Option<(String,)> =
            sqlx::query_as("SELECT value FROM user_preferences WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn set_preference(&self, key: &str, value: &str) -> Result<(), CoreError> {
        debug!("db: set_preference key={}", key);
        sqlx::query(
            "INSERT INTO user_preferences (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn activity_exists_by_source(
        &self,
        source: &str,
        source_id: &str,
    ) -> Result<bool, CoreError> {
        debug!("db: activity_exists_by_source source={} source_id={}", source, source_id);
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM activities WHERE source = ? AND source_id = ?")
                .bind(source)
                .bind(source_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(count.0 > 0)
    }

    pub async fn update_fit_data(
        &self,
        id: &Uuid,
        fit_data: Option<&str>,
        fit_version: i32,
    ) -> Result<(), CoreError> {
        debug!("db: update_fit_data id={} fit_version={}", id, fit_version);
        let result =
            sqlx::query("UPDATE activities SET fit_data = ?, fit_version = ? WHERE id = ?")
                .bind(fit_data)
                .bind(fit_version)
                .bind(id.to_string())
                .execute(&self.pool)
                .await?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub async fn list_stale_fit_activities(
        &self,
        current_version: i32,
    ) -> Result<Vec<(Uuid, String)>, CoreError> {
        debug!("db: list_stale_fit_activities current_version={}", current_version);
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT id, source_id FROM activities WHERE source = 'garmin' AND fit_version < ?",
        )
        .bind(current_version)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id_str, source_id)| (Uuid::parse_str(&id_str).unwrap_or_default(), source_id))
            .collect())
    }
}

fn row_to_activity(row: &SqliteRow) -> Activity {
    let id_str: String = row.get("id");
    let type_str: String = row.get("activity_type");
    let date_str: String = row.get("date");
    let duration_secs: Option<f64> = row.get("duration_secs");
    let distance_m: Option<f64> = row.get("distance_m");
    let pace_s_per_m: Option<f64> = row.get("pace_s_per_m");
    let hr_zone_int: Option<i32> = row.get("hr_zone");
    let notes: Option<String> = row.get("notes");
    let sub_type_str: Option<String> = row.get("sub_type");
    let is_commute_int: i32 = row.get::<Option<i32>, _>("is_commute").unwrap_or(0);
    let is_race_int: i32 = row.get::<Option<i32>, _>("is_race").unwrap_or(0);
    let fit_data: Option<String> = row.get("fit_data");
    let fit_version: i32 = row.get::<Option<i32>, _>("fit_version").unwrap_or(0);
    let source: Option<String> = row.get("source");
    let source_id: Option<String> = row.get("source_id");
    let created_at_str: String = row.get("created_at");
    let updated_at_str: String = row.get("updated_at");

    Activity {
        id: Uuid::parse_str(&id_str).unwrap_or_default(),
        activity_type: ActivityType::from_str(&type_str).unwrap_or(ActivityType::Run),
        date: NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S").unwrap_or_default(),
        duration_secs,
        distance_m,
        pace_s_per_m,
        hr_zone: hr_zone_int.and_then(HrZone::from_int),
        notes,
        sub_type: sub_type_str.as_deref().and_then(ActivitySubType::from_str),
        is_commute: is_commute_int != 0,
        is_race: is_race_int != 0,
        fit_data,
        fit_version,
        source,
        source_id,
        created_at: DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_default(),
        updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_default(),
    }
}
