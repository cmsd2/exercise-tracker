# Garmin Connect Integration

## Overview

The application syncs activities from Garmin Connect, including activity metadata (type, duration, distance, heart rate) and detailed FIT file data (GPS tracks, laps, power, cadence). Activities are deduplicated by Garmin activity ID and can be re-processed when the FIT parser is updated.

All Garmin operations are fully async. The service layer (`ActivityService`) is accessed directly without a mutex — the underlying `SqlitePool` is thread-safe.

## Authentication

### Current Approach: Manual Token Extraction

The application uses a manual token extraction flow:

1. User navigates to [connect.garmin.com](https://connect.garmin.com) in a browser
2. Opens browser DevTools, goes to the Network tab
3. Performs any action that triggers an API call
4. Copies the Bearer token from the `Authorization` header
5. Pastes the token into the app's Garmin settings panel

The token is stored as a JSON-serialised `GarminTokens` struct in the `user_preferences` table under the key `garmin_tokens`:

```json
{
  "access_token": "eyJ...",
  "refresh_token": null,
  "expires_at": 1712345678
}
```

### Token Lifecycle

- **Storage**: SQLite `user_preferences` table (not a system keychain)
- **Expiry check**: `is_token_expired()` compares `expires_at` to current UTC timestamp
- **On 401 response**: Tokens are automatically cleared, user must re-authenticate
- **On expiry**: Tokens are cleared with a message to reconnect
- **Refresh**: Not implemented -- expired tokens require manual replacement

## Garmin Connect API

### Endpoints Used

| Endpoint | Method | Purpose |
|---|---|---|
| `https://connect.garmin.com/activitylist-service/activities/search/activities` | GET | List activities with date range |
| `https://connect.garmin.com/download-service/files/activity/{id}` | GET | Download FIT file binary |

### Request Headers

All requests include:

```
Authorization: Bearer {access_token}
DI-Backend: connectapi.garmin.com
```

The `DI-Backend` header is required by Garmin's API gateway.

### Activity List API

**Query parameters:**

| Parameter | Description |
|---|---|
| `startDate` | Start of date range (YYYY-MM-DD) |
| `endDate` | End of date range (YYYY-MM-DD) |
| `limit` | Page size (we use 100) |
| `start` | Offset for pagination (0, 100, 200, ...) |

**Response**: JSON array of activity objects. The client paginates until a batch returns fewer than `limit` results.

**Activity JSON structure** (relevant fields):

```json
{
  "activityId": 12345678,
  "activityName": "Morning Run",
  "activityType": { "typeKey": "running" },
  "startTimeLocal": "2026-04-01 07:30:00",
  "duration": 1800.0,
  "distance": 5000.0,
  "averageSpeed": 2.78,
  "maxHR": 175.0,
  "averageHR": 155.0,
  "description": null
}
```

Note: Garmin uses `averageHR` and `maxHR` (not camelCase `averageHr`), so the deserialiser uses `#[serde(alias = "averageHR")]`.

### FIT File Download

**Request**: `GET /download-service/files/activity/{activity_id}` with Bearer auth.

**Response**: Binary FIT file data. Parsed locally using the `fitparser` crate. See [FIT File Parsing](./fit-parsing.md) for details.

### Logging

All Garmin HTTP requests are logged via `tracing` at info/debug level, including:
- Request URL and query parameters
- Response HTTP status codes
- Downloaded byte counts for FIT files
- Error details with HTTP status codes on failure

Logs are written to `{app_data_dir}/logs/exercise-tracker.log`.

## Sync Flow

The `garmin_sync_activities` command orchestrates the full sync:

```
1. Load tokens from preferences
2. Check token expiry
3. Create HTTP client
4. Fetch all activities in date range (paginated)
5. Emit "Started" event to frontend
6. For each Garmin activity:
   a. Map activity type -> (ActivityType, Option<ActivitySubType>)
      - Skip unsupported types (yoga, strength, etc.)
   b. Check for existing import (source="garmin", source_id=activity_id)
      - Skip if already imported
   c. Parse date from start_time_local
   d. Compute pace = duration / distance
   e. Map HR zone from average heart rate
   f. Extract notes from activity_name or description
   g. Download FIT file
      - Parse binary -> FitDetail struct
      - Serialize to JSON string
      - On failure: log error, continue with fit_data=None
   h. Create activity via ActivityCommand::Create
   i. Emit progress event to frontend
7. Emit "Finished" event with counts
```

### Activity Type Mapping

Garmin's `type_key` string is mapped to our domain types:

| Garmin type_key | ActivityType | ActivitySubType |
|---|---|---|
| `running` | Run | -- |
| `treadmill_running` | Run | Treadmill |
| `trail_running` | Run | Trail |
| `track_running` | Run | Track |
| `cycling` | Cycle | -- |
| `indoor_cycling` | Cycle | Indoor |
| `road_biking` | Cycle | Road |
| `mountain_biking` | Cycle | Mountain |
| `lap_swimming` | Swim | Pool |
| `open_water_swimming` | Swim | Open Water |
| `rowing` | Row | -- |
| `indoor_rowing` | Row | Indoor Row |
| `walking` | Walk | -- |
| `casual_walking` | Walk | Casual |
| `hiking` | Hike | -- |

Unsupported types (yoga, strength training, etc.) return `None` and the activity is skipped with a "Skipped" progress event.

### Heart Rate Zone Mapping

Uses a fixed max HR of 190 bpm:

```
percentage = average_hr / 190 * 100
Zone 1: < 60%    (< 114 bpm)
Zone 2: 60-70%   (114-133 bpm)
Zone 3: 70-80%   (133-152 bpm)
Zone 4: 80-90%   (152-171 bpm)
Zone 5: >= 90%   (>= 171 bpm)
```

Activities with no heart rate data or avg HR below 60 bpm get no zone assigned.

### Pace Calculation

```
pace_s_per_m = duration_secs / distance_m
```

Returns `None` if distance is zero or either field is missing.

## Deduplication

Each Garmin activity is tagged with:
- `source = "garmin"`
- `source_id = "{activity_id}"` (Garmin's numeric ID as string)

A unique partial index on `(source, source_id)` prevents duplicate imports at the database level. Before importing, the sync also checks `activity_exists_by_source("garmin", source_id)` to skip already-imported activities early, avoiding unnecessary FIT downloads.

## Progress Events

During sync, the backend emits Tauri events (`garmin-sync-progress`) that the frontend listens to:

```typescript
type SyncProgress =
  | { kind: "Started", total: number }
  | { kind: "Activity", current: number, total: number }
  | { kind: "Skipped", current: number, total: number, reason: string }
  | { kind: "Finished", imported: number, skipped: number, errors: number }
```

The frontend renders a progress indicator showing "Importing X/Y" and a final summary.

## FIT Re-processing

When the FIT parser is improved, `CURRENT_FIT_VERSION` (currently `1`) is incremented in `commands/garmin.rs`. The `garmin_reprocess_fit` command:

1. Queries activities where `source = 'garmin' AND fit_version < CURRENT_FIT_VERSION`
2. For each: re-downloads FIT file from Garmin, re-parses with updated logic
3. Updates `fit_data` and `fit_version` columns
4. Returns count of updated activities

This allows iterating on the parser without losing data or requiring a full re-sync.

## Error Handling

The sync is designed for graceful degradation:

| Failure | Behaviour |
|---|---|
| Token expired | Clear tokens, return error asking user to reconnect |
| 401 from Garmin | Clear tokens, return HTTP error |
| Unsupported activity type | Skip, emit "Skipped" event, continue |
| Already imported | Skip, emit "Skipped" event, continue |
| Date parse failure | Use default date, continue |
| FIT download failure | Log error, store `fit_data: None, fit_version: 0`, continue |
| FIT parse failure | Log error, store `fit_data: None, fit_version: 0`, continue |
| DB insert failure | Increment error count, emit "Skipped" event, continue |
| Validation failure | Increment skip count, emit "Skipped" event, continue |

No single activity failure aborts the entire sync. All errors are logged to the application log file.

## CSP Configuration

The Tauri CSP allows outbound connections to Garmin:

```
connect-src 'self' https://*.garmin.com
```
