# Application Architecture

## Overview

Exercise Tracker is a desktop application for logging and analysing cardio activities. It supports manual activity logging, Garmin Connect sync with FIT file parsing, and weekly progress visualisation.

**Tech stack:**

- **Frontend**: React 19, TypeScript, Vite, Zustand (state), Recharts (charts)
- **Backend**: Rust, Tauri v2, Tokio (async), reqwest (HTTP), fitparser (FIT parsing)
- **Core library**: Rust crate with SQLite (sqlx async), Serde, UUID, Chrono
- **Database**: SQLite (local file) via sqlx with managed migrations
- **Logging**: tracing + tracing-subscriber + tracing-appender (file-based)
- **External APIs**: Garmin Connect

## Project Structure

```
exercise-tracker/
├── Cargo.toml                     # Rust workspace root
├── package.json                   # Frontend dependencies
├── .env                           # DATABASE_URL for sqlx dev tooling
├── crates/
│   └── exercise-tracker-core/     # Domain logic, DB, commands
│       ├── migrations/
│       │   └── 20260405000001_initial.sql  # sqlx migration
│       └── src/
│           ├── model.rs           # Domain models (Activity, enums)
│           ├── commands.rs        # ActivityCommand / ActivityEffect
│           ├── service.rs         # Business logic & validation (async)
│           ├── db.rs              # Async SQLite via sqlx SqlitePool
│           ├── filter.rs          # Query filter struct
│           ├── summary.rs         # WeeklySummary model
│           └── error.rs           # CoreError enum
├── src-tauri/                     # Tauri app crate
│   ├── tauri.conf.json            # App config, CSP, permissions
│   └── src/
│       ├── lib.rs                 # Tauri setup, logging init, command registration
│       ├── state.rs               # Shared AppState (no Mutex — pool is thread-safe)
│       ├── error.rs               # IPC error type
│       ├── commands/
│       │   ├── activity.rs        # Activity IPC handlers (async)
│       │   ├── preferences.rs     # Preference IPC handlers (async)
│       │   └── garmin.rs          # Garmin sync IPC handlers (async)
│       └── garmin/
│           ├── types.rs           # Garmin API & FIT structs
│           ├── client.rs          # HTTP client for Garmin API
│           ├── mapping.rs         # Garmin type -> domain type
│           └── fit_parser.rs      # FIT binary -> structured JSON
├── src/                           # React frontend
│   ├── App.tsx                    # Root component with tab routing
│   ├── lib/
│   │   ├── tauri.ts               # Typed IPC wrappers & constants
│   │   ├── units.ts               # Distance/pace/duration formatting
│   │   └── autofill.ts            # Duration/distance/pace triad logic
│   ├── store/
│   │   ├── activityStore.ts       # Zustand store for activities
│   │   └── garminStore.ts         # Zustand store for Garmin state
│   └── components/
│       ├── TabBar.tsx
│       ├── LogActivity/LogActivityView.tsx
│       ├── History/HistoryView.tsx
│       ├── Progress/ProgressView.tsx
│       └── Garmin/GarminSyncView.tsx
└── docs/                          # This documentation
```

## Layered Architecture

The application has three layers with clear boundaries:

```
┌─────────────────────────────────────────┐
│  Frontend (React + TypeScript)          │
│  Components, stores, IPC wrappers       │
├─────────────────────────────────────────┤
│  Tauri IPC Layer (Rust)                 │
│  Command handlers, state management,    │
│  Garmin client, FIT parsing, logging    │
├─────────────────────────────────────────┤
│  Core Library (Rust crate)              │
│  Domain model, commands, async service, │
│  validation, database (sqlx)            │
└─────────────────────────────────────────┘
```

**Core crate** (`exercise-tracker-core`) is transport-agnostic. It knows nothing about Tauri, HTTP, or UI. It exposes `ActivityService` with a command-effect interface. All database and service methods are async.

**Tauri app** (`src-tauri`) bridges the frontend and core. It handles IPC serialisation, Garmin API communication, FIT file processing, logging setup, and application state.

**Frontend** (`src/`) is a React SPA that communicates with the backend exclusively through Tauri's `invoke()` IPC.

## Command-Effect Pattern

All mutations flow through a single method: `ActivityService::apply(command).await -> Result<ActivityEffect, CoreError>`.

```rust
// Commands (what to do)
enum ActivityCommand {
    Create { activity_type, date, duration_secs, ... },
    Update { id, activity_type?, date?, ... },
    Delete { id },
}

// Effects (what happened)
enum ActivityEffect {
    Created { id: Uuid },
    Updated { id: Uuid },
    Deleted { id: Uuid },
    ValidationError { reason: String },
}
```

This pattern provides:

- **Single validation point** -- all business rules live in `service.rs`
- **Testability** -- tests call `service.apply()` directly, no IPC needed
- **Transport independence** -- the same service works from Tauri IPC, tests, or a future CLI

Validation rules enforced during `apply()`:
- Duration and distance must be non-negative
- At least one of duration or distance is required

## State Management

### Backend

`AppState` holds an `ActivityService` directly. No `Mutex` is needed because the underlying `SqlitePool` from sqlx is `Send + Sync + Clone` (it uses an internal `Arc`).

```rust
pub struct AppState {
    pub service: ActivityService,
}
```

`ActivityService` is also `Clone`, wrapping the `Database` which holds the pool.

### Frontend

Two Zustand stores manage client-side state:

- **activityStore** -- cached activity list, weekly summaries, current filter, loading/error state. Methods auto-refetch after mutations.
- **garminStore** -- connection status, sync progress, last sync result.

## Tauri Command Naming

All Tauri commands use `#[tauri::command(rename_all = "snake_case")]` so that Rust parameter names (e.g. `activity_type`) match the JavaScript argument names exactly. Without this, Tauri 2 defaults to camelCase conversion.

## Logging

The application uses the `tracing` ecosystem for structured logging:

- **Output**: Daily rolling log files at `{app_data_dir}/logs/exercise-tracker.log`
- **Initialisation**: `init_logging()` in `lib.rs` during Tauri setup
- **Log levels** (defaults, configurable via `RUST_LOG` env var):
  - `exercise_tracker*` crates: `debug`
  - `sqlx`: `info` (shows connection lifecycle)
  - `reqwest`: `debug`
  - Everything else: `warn`
- **What is logged**:
  - All Tauri command invocations (entry, parameters, results, errors)
  - All Garmin HTTP requests (URL, parameters, response status, byte counts)
  - All database operations (method name, key parameters, row counts)
  - Service command dispatch
  - Application startup lifecycle

## Domain Model

The central `Activity` struct:

| Field | Type | Description |
|---|---|---|
| `id` | UUID | Unique identifier |
| `activity_type` | ActivityType | Run, Cycle, Swim, Row, Walk, Hike |
| `sub_type` | Option\<ActivitySubType\> | Treadmill, Trail, Indoor, etc. |
| `date` | NaiveDateTime | Local date and time |
| `duration_secs` | Option\<f64\> | Duration in seconds |
| `distance_m` | Option\<f64\> | Distance in metres |
| `pace_s_per_m` | Option\<f64\> | Seconds per metre |
| `hr_zone` | Option\<HrZone\> | Heart rate zone (1-5) |
| `notes` | Option\<String\> | User notes or Garmin activity name |
| `is_commute` | bool | Commute flag |
| `is_race` | bool | Race flag |
| `fit_data` | Option\<String\> | JSON-serialised FIT file detail |
| `fit_version` | i32 | FIT parser version that produced fit_data |
| `source` | Option\<String\> | Import source (e.g. "garmin") |
| `source_id` | Option\<String\> | Remote ID for deduplication |
| `created_at` | DateTime\<Utc\> | Creation timestamp |
| `updated_at` | DateTime\<Utc\> | Last modification timestamp |

Sub-types are a flat enum validated per parent type via `ActivitySubType::sub_types_for(activity_type)`:

| Activity Type | Valid Sub-types |
|---|---|
| Run | Treadmill, Trail, Track |
| Cycle | Indoor, Road, Mountain |
| Swim | Pool, Open Water |
| Row | Indoor Row |
| Walk | Casual |
| Hike | (none) |

## Unit Conventions

All values are stored in SI units internally:

| Measurement | Internal unit | Display unit |
|---|---|---|
| Distance | metres | km or miles |
| Duration | seconds | HH:MM:SS or MM:SS |
| Pace | seconds per metre | MM:SS per km or per mile |

Conversions happen at the frontend boundary in `lib/units.ts`.

## Security

Content Security Policy (from `tauri.conf.json`):

```
default-src 'self';
style-src 'self' 'unsafe-inline';
connect-src 'self' https://*.garmin.com;
```

- No inline scripts, only bundled modules
- Network requests restricted to Garmin domains
- Garmin tokens stored in SQLite preferences (not in a system keychain)

## Related Documentation

- [Database Design](./database.md) -- schema, migrations, query patterns
- [Garmin Integration](./garmin-integration.md) -- sync flow, API, auth
- [FIT File Parsing](./fit-parsing.md) -- FIT format, parsed fields, versioning
