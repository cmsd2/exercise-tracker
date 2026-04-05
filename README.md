# Exercise Tracker

A desktop app for logging and tracking cardio exercises with Garmin sync. Built with Tauri, React, and Rust.

## Features

- **Manual activity logging** — Run, cycle, swim, row, walk, and hike with duration, distance, pace, heart rate zones, and notes
- **Garmin integration** — OAuth2 login via embedded browser, automatic activity sync with FIT file parsing
- **History** — Browse and filter past activities by type and date
- **Progress charts** — Weekly summaries and trend lines for pace, distance, and duration over configurable time ranges
- **Activity subtypes** — Treadmill, trail, track, indoor cycling, virtual rides, open water swimming, and more
- **Offline-first** — All data stored locally in SQLite

## Tech Stack

| Layer    | Technology |
|----------|------------|
| Frontend | React 19, TypeScript, Vite, Zustand, Recharts |
| Backend  | Rust, Tauri 2, SQLx (SQLite), Reqwest, FitParser |
| Build    | GitHub Actions (macOS + Windows) |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 22+
- [Rust](https://rustup.rs/) stable
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform

### Development

```bash
npm install
npx tauri dev
```

This starts the Vite dev server (port 1420) and the Tauri app together with hot reload.

### Production Build

```bash
npx tauri build
```

Produces platform-specific installers (DMG on macOS, MSI/NSIS on Windows).

## Project Structure

```
src/                    # React frontend
  components/           # UI views (LogActivity, History, Progress, Garmin)
  stores/               # Zustand state (activities, Garmin auth)
src-tauri/src/          # Tauri Rust backend
  commands/             # IPC command handlers
  garmin/               # OAuth, API client, FIT parser, type mapping
crates/exercise-tracker-core/  # Shared Rust library (DB, models, services)
```

## Garmin Sync

1. Click **Garmin** tab and log in via the embedded browser
2. Select a date range and hit **Sync**
3. Activities are downloaded, FIT files parsed, and stored locally — duplicates are skipped automatically

## IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
