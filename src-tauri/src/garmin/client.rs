use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use tracing::{debug, info};

use super::types::{GarminActivity, GarminTokens};

const ACTIVITIES_URL: &str =
    "https://connect.garmin.com/activitylist-service/activities/search/activities";
const FIT_DOWNLOAD_URL: &str = "https://connect.garmin.com/download-service/files/activity";

pub struct GarminClient {
    http: reqwest::Client,
    tokens: GarminTokens,
}

impl GarminClient {
    pub fn new(tokens: GarminTokens) -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert("DI-Backend", HeaderValue::from_static("connectapi.garmin.com"));
                headers
            })
            .build()?;
        Ok(Self { http, tokens })
    }

    pub fn is_token_expired(&self) -> bool {
        match self.tokens.expires_at {
            Some(expires_at) => Utc::now().timestamp() >= expires_at,
            None => false,
        }
    }

    pub async fn fetch_activities(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<GarminActivity>, reqwest::Error> {
        let mut all_activities = Vec::new();
        let mut start: usize = 0;
        let limit: usize = 100;

        loop {
            info!(
                "Garmin API: GET {} startDate={} endDate={} start={} limit={}",
                ACTIVITIES_URL, start_date, end_date, start, limit
            );

            let resp = self
                .http
                .get(ACTIVITIES_URL)
                .header(
                    AUTHORIZATION,
                    format!("Bearer {}", self.tokens.access_token),
                )
                .query(&[
                    ("startDate", start_date),
                    ("endDate", end_date),
                    ("limit", &limit.to_string()),
                    ("start", &start.to_string()),
                ])
                .send()
                .await?;

            let status = resp.status();
            info!("Garmin API: response status={}", status);

            let resp = resp.error_for_status()?;
            let batch: Vec<GarminActivity> = resp.json().await?;
            let batch_len = batch.len();
            debug!(
                "Garmin API: received {} activities in batch (offset={})",
                batch_len, start
            );
            all_activities.extend(batch);

            if batch_len < limit {
                break;
            }
            start += limit;
        }

        info!(
            "Garmin API: fetched {} total activities",
            all_activities.len()
        );
        Ok(all_activities)
    }

    pub async fn download_fit_file(&self, activity_id: u64) -> Result<Vec<u8>, reqwest::Error> {
        let url = format!("{}/{}", FIT_DOWNLOAD_URL, activity_id);
        debug!("Garmin API: GET {}", url);

        let resp = self
            .http
            .get(&url)
            .header(
                AUTHORIZATION,
                format!("Bearer {}", self.tokens.access_token),
            )
            .send()
            .await?;

        let status = resp.status();
        debug!("Garmin API: FIT download status={} for activity_id={}", status, activity_id);

        let bytes = resp.error_for_status()?.bytes().await?;
        debug!(
            "Garmin API: downloaded {} bytes for activity_id={}",
            bytes.len(),
            activity_id
        );
        Ok(bytes.to_vec())
    }
}
