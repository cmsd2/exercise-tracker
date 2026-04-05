use base64::Engine;
use serde::Deserialize;
use tracing::{debug, error, info, warn};

use super::types::GarminTokens;

const DI_TOKEN_URL: &str = "https://diauth.garmin.com/di-oauth2-service/oauth/token";
const DI_GRANT_TYPE: &str =
    "https://connectapi.garmin.com/di-oauth2-service/oauth/grant/service_ticket";
const SSO_SERVICE_URL: &str = "https://connect.garmin.com/app";

/// Client IDs to try, in order of preference (newest first).
const DI_CLIENT_IDS: &[&str] = &[
    "GARMIN_CONNECT_MOBILE_ANDROID_DI_2025Q2",
    "GARMIN_CONNECT_MOBILE_ANDROID_DI_2024Q4",
    "GARMIN_CONNECT_MOBILE_ANDROID_DI",
];

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
}

/// Exchange an SSO CAS service ticket for DI OAuth2 tokens.
///
/// Uses the same DI-OAuth2 flow as the Garmin Connect Android app:
/// POST to diauth.garmin.com with the service ticket and Basic auth.
pub async fn exchange_ticket(ticket: &str) -> Result<GarminTokens, String> {
    let client = reqwest::Client::builder()
        .user_agent("com.garmin.android.apps.connectmobile")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    for client_id in DI_CLIENT_IDS {
        info!(
            "oauth_exchange: trying client_id={}",
            client_id
        );

        let basic_auth = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:", client_id));

        let resp = client
            .post(DI_TOKEN_URL)
            .header("Authorization", format!("Basic {}", basic_auth))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("client_id", *client_id),
                ("service_ticket", ticket),
                ("grant_type", DI_GRANT_TYPE),
                ("service_url", SSO_SERVICE_URL),
            ])
            .send()
            .await
            .map_err(|e| format!("DI token request failed: {}", e))?;

        let status = resp.status();
        debug!("oauth_exchange: response status={} for client_id={}", status, client_id);

        if status.is_success() {
            let token_resp: TokenResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse token response: {}", e))?;

            info!("oauth_exchange: got DI OAuth2 tokens with client_id={}", client_id);

            let expires_at = token_resp
                .expires_in
                .map(|secs| chrono::Utc::now().timestamp() + secs);

            return Ok(GarminTokens {
                access_token: token_resp.access_token,
                refresh_token: token_resp.refresh_token,
                expires_at,
            });
        }

        let body = resp.text().await.unwrap_or_default();
        warn!(
            "oauth_exchange: client_id={} failed ({}): {}",
            client_id,
            status,
            &body[..body.len().min(200)]
        );
    }

    error!("oauth_exchange: all client IDs failed");
    Err("Failed to exchange ticket with any client ID".into())
}
