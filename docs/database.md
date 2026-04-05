# Database Design

## Overview

The application uses SQLite via `sqlx` (async), stored as a single file at `{app_data_dir}/exercise-tracker.db`. The database is opened and migrated during Tauri app startup. The `Database` struct wraps a `SqlitePool`, making it `Clone`, `Send`, and `Sync` without requiring a `Mutex`.

## Schema

### Activities Table

```sql
CREATE TABLE activities (
    id TEXT PRIMARY KEY NOT NULL,                 -- UUID as string
    activity_type TEXT NOT NULL,                   -- "run", "cycle", "swim", etc.
    date TEXT NOT NULL,                            -- ISO 8601 local: "2026-04-05T09:30:00"
    duration_secs REAL,                           -- seconds (nullable)
    distance_m REAL,                              -- metres (nullable)
    pace_s_per_m REAL,                            -- seconds per metre (nullable)
    hr_zone INTEGER,                              -- 1-5 or NULL
    notes TEXT,                                   -- free text
    sub_type TEXT,                                -- "treadmill", "trail", "indoor", etc.
    is_commute INTEGER NOT NULL DEFAULT 0,        -- boolean as 0/1
    is_race INTEGER NOT NULL DEFAULT 0,           -- boolean as 0/1
    fit_data TEXT,                                -- JSON blob of parsed FIT data
    fit_version INTEGER NOT NULL DEFAULT 0,       -- FIT parser version
    source TEXT,                                  -- "garmin" or NULL for manual
    source_id TEXT,                               -- remote activity ID
    created_at TEXT NOT NULL,                     -- RFC 3339 UTC
    updated_at TEXT NOT NULL                      -- RFC 3339 UTC
);

CREATE INDEX idx_activities_date ON activities(date);
CREATE INDEX idx_activities_type ON activities(activity_type);
CREATE UNIQUE INDEX idx_activities_source
    ON activities(source, source_id)
    WHERE source IS NOT NULL AND source_id IS NOT NULL;
```

### User Preferences Table

```sql
CREATE TABLE user_preferences (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
```

Used for storing application settings and Garmin auth tokens (as JSON strings).

Known keys:
- `garmin_tokens` -- serialised `GarminTokens` JSON

## Migration System

Migrations are managed by **sqlx's built-in migration framework** using timestamped SQL files in `crates/exercise-tracker-core/migrations/`. sqlx tracks migration state in its own `_sqlx_migrations` table.

### Migration Files

| File | Description |
|---|---|
| `20260405000001_initial.sql` | Full schema: activities table, indices, preferences table |

The initial migration uses `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` so it's safe for both new databases and existing ones that already have the schema.

On startup, `Database::migrate()` runs:

```rust
sqlx::migrate!("./migrations").run(&self.pool).await?;
```

### Adding New Migrations

To add a new migration:

```bash
# Create a new migration file
sqlx migrate add <description> --source crates/exercise-tracker-core/migrations

# Apply it to the dev database
sqlx migrate run --source crates/exercise-tracker-core/migrations --database-url sqlite:dev.db
```

## Database Layer

### Connection Management

The `Database` struct wraps a `SqlitePool`:

```rust
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}
```

- **Production**: `Database::open("sqlite:{path}")` -- pool with up to 5 connections
- **Tests**: `Database::open_in_memory()` -- pool with 1 connection (so all queries share the same in-memory database)

All methods are `async fn` and use the pool directly.

## Query Patterns

### Listing Activities

The `list_activities` query uses static SQL with the `? IS NULL OR col = ?` pattern to handle optional filters without dynamic query building:

```sql
SELECT id, activity_type, date, duration_secs, distance_m, pace_s_per_m,
       hr_zone, notes, sub_type, is_commute, is_race,
       NULL AS fit_data,  -- excluded for performance
       fit_version, source, source_id, created_at, updated_at
FROM activities
WHERE (? IS NULL OR activity_type = ?)
  AND (? IS NULL OR sub_type = ?)
  AND (? IS NULL OR date >= ?)
  AND (? IS NULL OR date <= ?)
ORDER BY date DESC
LIMIT ?
OFFSET ?
```

Each optional filter binds the same value twice (once for the `IS NULL` check, once for the comparison). `LIMIT` defaults to `-1` (SQLite: no limit), `OFFSET` defaults to `0`.

`fit_data` is explicitly returned as `NULL` in list queries to avoid loading potentially large JSON blobs (GPS tracks, lap data). FIT data is only loaded via `get_activity` which fetches a single row.

### Deduplication Check

```sql
SELECT COUNT(*) FROM activities WHERE source = ? AND source_id = ?
```

Called before each Garmin import to skip already-imported activities.

### Weekly Summary

```sql
SELECT
    date(date, 'weekday 0', '-6 days') AS week_start,
    COUNT(*) AS activity_count,
    COALESCE(SUM(distance_m), 0.0) AS total_distance_m,
    COALESCE(SUM(duration_secs), 0.0) AS total_duration_secs
FROM activities
WHERE (? IS NULL OR activity_type = ?)
GROUP BY week_start
ORDER BY week_start DESC
LIMIT 12
```

Uses SQLite date functions to group by ISO week (Monday start). The optional activity type filter uses the same `? IS NULL OR col = ?` pattern.

### Stale FIT Activities

```sql
SELECT id, source_id FROM activities
WHERE source = 'garmin' AND fit_version < ?
```

Identifies activities whose FIT data was produced by an older parser version and may benefit from re-processing.

### Preferences

```sql
-- Get
SELECT value FROM user_preferences WHERE key = ?

-- Set (upsert)
INSERT INTO user_preferences (key, value) VALUES (?, ?)
ON CONFLICT(key) DO UPDATE SET value = excluded.value
```

## Data Type Conventions

| Rust type | SQLite type | Notes |
|---|---|---|
| `Uuid` | TEXT | Stored as hyphenated string |
| `ActivityType` | TEXT | Lowercase: "run", "cycle", etc. |
| `ActivitySubType` | TEXT | Kebab-case: "open-water", "indoor-row" |
| `NaiveDateTime` | TEXT | Format: `%Y-%m-%dT%H:%M:%S` |
| `DateTime<Utc>` | TEXT | RFC 3339: `2026-04-05T09:30:00+00:00` |
| `HrZone` | INTEGER | 1-5 |
| `bool` | INTEGER | 0 or 1 |
| `Option<T>` | nullable | SQL NULL when None |

## Row Mapping

`row_to_activity()` maps SQLite columns by name using `sqlx::Row::get()`:

| Column | Rust field |
|---|---|
| `id` | `id: Uuid` (parsed from string) |
| `activity_type` | `activity_type: ActivityType` (parsed from string) |
| `date` | `date: NaiveDateTime` (parsed from `%Y-%m-%dT%H:%M:%S`) |
| `duration_secs` | `duration_secs: Option<f64>` |
| `distance_m` | `distance_m: Option<f64>` |
| `pace_s_per_m` | `pace_s_per_m: Option<f64>` |
| `hr_zone` | `hr_zone: Option<HrZone>` (from integer) |
| `notes` | `notes: Option<String>` |
| `sub_type` | `sub_type: Option<ActivitySubType>` (parsed from string) |
| `is_commute` | `is_commute: bool` (from 0/1 integer) |
| `is_race` | `is_race: bool` (from 0/1 integer) |
| `fit_data` | `fit_data: Option<String>` |
| `fit_version` | `fit_version: i32` |
| `source` | `source: Option<String>` |
| `source_id` | `source_id: Option<String>` |
| `created_at` | `created_at: DateTime<Utc>` (parsed from RFC 3339) |
| `updated_at` | `updated_at: DateTime<Utc>` (parsed from RFC 3339) |

## Dev Setup

```bash
# Install sqlx CLI
cargo install sqlx-cli --features sqlite

# Create dev database and run migrations
sqlx database create --database-url sqlite:dev.db
sqlx migrate run --source crates/exercise-tracker-core/migrations --database-url sqlite:dev.db
```

The `.env` file at the workspace root contains `DATABASE_URL=sqlite:dev.db` for sqlx CLI convenience.

## Performance Considerations

- **fit_data excluded from lists**: The `list_activities` query returns `NULL AS fit_data` instead of the actual column to avoid transferring megabytes of GPS/lap data when browsing history.
- **Indices on date and type**: The most common filter patterns are by activity type and date range.
- **Partial unique index on source**: Only enforced when both `source` and `source_id` are non-null, so manual activities (with null source) are unaffected.
- **Connection pool**: sqlx's `SqlitePool` manages a pool of connections (up to 5 in production), enabling concurrent async reads without blocking.
- **Static SQL**: All queries use static SQL (no dynamic query building), which is compatible with sqlx's compile-time query checking if enabled in the future.
