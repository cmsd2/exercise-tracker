use std::path::PathBuf;
use std::process;

use chrono::NaiveDateTime;
use clap::Parser;

use exercise_tracker_core::db::Database;
use exercise_tracker_core::filter::ActivityFilter;
use exercise_tracker_core::model::ActivityType;

/// Export exercise-tracker activities as JSON.
#[derive(Parser)]
#[command(name = "exercise-tracker-cli")]
struct Cli {
    /// Path to the SQLite database file.
    /// Defaults to the Tauri app data directory.
    #[arg(long)]
    db_path: Option<PathBuf>,

    /// Filter by activity type (run, cycle, swim, row, walk, hike).
    #[arg(long = "type", value_parser = parse_activity_type)]
    activity_type: Option<ActivityType>,

    /// Filter activities from this date (ISO 8601, e.g. 2026-01-01T00:00:00).
    #[arg(long)]
    from: Option<NaiveDateTime>,

    /// Filter activities up to this date (ISO 8601).
    #[arg(long)]
    to: Option<NaiveDateTime>,

    /// Maximum number of activities to return.
    #[arg(long)]
    limit: Option<u32>,

    /// Include full fit_data JSON per activity (large, omitted by default).
    #[arg(long)]
    include_fit_data: bool,

    /// Pretty-print the JSON output.
    #[arg(long)]
    pretty: bool,
}

fn parse_activity_type(s: &str) -> Result<ActivityType, String> {
    match s {
        "run" => Ok(ActivityType::Run),
        "cycle" => Ok(ActivityType::Cycle),
        "swim" => Ok(ActivityType::Swim),
        "row" => Ok(ActivityType::Row),
        "walk" => Ok(ActivityType::Walk),
        "hike" => Ok(ActivityType::Hike),
        _ => Err(format!(
            "unknown activity type '{}' (expected: run, cycle, swim, row, walk, hike)",
            s
        )),
    }
}

fn default_db_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("com.cmsd2.exercise-tracker").join("exercise-tracker.db"))
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let db_path = cli
        .db_path
        .or_else(default_db_path)
        .unwrap_or_else(|| {
            eprintln!("error: could not determine default database path; use --db-path");
            process::exit(1);
        });

    if !db_path.exists() {
        eprintln!("error: database not found at {}", db_path.display());
        process::exit(1);
    }

    let url = format!("sqlite:{}?mode=ro", db_path.display());
    let db = match Database::open(&url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("error: failed to open database: {}", e);
            process::exit(1);
        }
    };

    let filter = ActivityFilter {
        activity_type: cli.activity_type,
        date_from: cli.from,
        date_to: cli.to,
        limit: cli.limit,
        ..Default::default()
    };

    let mut activities = match db.list_activities(&filter).await {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: failed to list activities: {}", e);
            process::exit(1);
        }
    };

    if cli.include_fit_data {
        for activity in &mut activities {
            match db.get_activity(&activity.id).await {
                Ok(full) => activity.fit_data = full.fit_data,
                Err(e) => {
                    eprintln!(
                        "warning: failed to fetch fit_data for {}: {}",
                        activity.id, e
                    );
                }
            }
        }
    }

    let json = if cli.pretty {
        serde_json::to_string_pretty(&activities)
    } else {
        serde_json::to_string(&activities)
    };

    match json {
        Ok(s) => println!("{}", s),
        Err(e) => {
            eprintln!("error: failed to serialize JSON: {}", e);
            process::exit(1);
        }
    }
}
