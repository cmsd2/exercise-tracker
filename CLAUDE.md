# Exercise Tracker

## Architectural Decision Records

Before implementing anything, search the ADRs using the `search_adrs` and `list_adrs` MCP tools to check for relevant decisions. Follow any applicable ADRs.

## FIT Data Processing

When modifying the FIT JSON data extraction or parsing logic in `src-tauri/src/garmin/fit_parser.rs`, you **must** increment `CURRENT_FIT_VERSION` in `src-tauri/src/commands/garmin.rs`. This version is stored alongside each activity's parsed FIT data. The `garmin_reprocess_fit` command uses it to find activities with stale FIT data (version < current) and re-downloads and re-parses their FIT files.

## CLI Tool

Use the `exercise-tracker-cli` crate (`crates/exercise-tracker-cli`) to inspect the database from the terminal. It opens the SQLite database and prints activities as JSON.

```bash
# All activities (compact JSON)
cargo run -p exercise-tracker-cli

# Pretty-print, filter by type, limit results
cargo run -p exercise-tracker-cli -- --pretty --type run --limit 5

# Include full fit_data blobs (large)
cargo run -p exercise-tracker-cli -- --pretty --include-fit-data --limit 1

# Custom database path
cargo run -p exercise-tracker-cli -- --db-path /path/to/exercise-tracker.db
```

The default database path is the Tauri app data directory (`~/Library/Application Support/com.cmsd2.exercise-tracker/exercise-tracker.db` on macOS).
