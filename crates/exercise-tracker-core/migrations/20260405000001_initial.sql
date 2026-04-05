CREATE TABLE IF NOT EXISTS activities (
    id TEXT PRIMARY KEY NOT NULL,
    activity_type TEXT NOT NULL,
    date TEXT NOT NULL,
    duration_secs REAL,
    distance_m REAL,
    pace_s_per_m REAL,
    hr_zone INTEGER,
    notes TEXT,
    sub_type TEXT,
    is_commute INTEGER NOT NULL DEFAULT 0,
    is_race INTEGER NOT NULL DEFAULT 0,
    fit_data TEXT,
    fit_version INTEGER NOT NULL DEFAULT 0,
    source TEXT,
    source_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_activities_date ON activities(date);
CREATE INDEX IF NOT EXISTS idx_activities_type ON activities(activity_type);
CREATE UNIQUE INDEX IF NOT EXISTS idx_activities_source
    ON activities(source, source_id)
    WHERE source IS NOT NULL AND source_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS user_preferences (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
