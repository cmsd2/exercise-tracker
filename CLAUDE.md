# Exercise Tracker

## Architectural Decision Records

Before implementing anything, search the ADRs using the `search_adrs` and `list_adrs` MCP tools to check for relevant decisions. Follow any applicable ADRs.

## FIT Data Processing

When modifying the FIT JSON data extraction or parsing logic in `src-tauri/src/garmin/fit_parser.rs`, you **must** increment `CURRENT_FIT_VERSION` in `src-tauri/src/commands/garmin.rs`. This version is stored alongside each activity's parsed FIT data. The `garmin_reprocess_fit` command uses it to find activities with stale FIT data (version < current) and re-downloads and re-parses their FIT files.
