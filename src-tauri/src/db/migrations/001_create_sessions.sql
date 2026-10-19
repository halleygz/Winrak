CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY,
    app_name TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL CHECK (duration_seconds >= 0),
    date TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS sessions_date_app ON sessions(date, app_name);
CREATE INDEX IF NOT EXISTS sessions_started_at ON sessions(started_at, id);
-- Identity is per source line, not per session value: identical real rows survive.
-- Preserve the original text as well, including historical midnight/DST ambiguity.
CREATE TABLE IF NOT EXISTS legacy_import_lines (
    source_path TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    raw_line TEXT NOT NULL,
    session_id INTEGER NOT NULL REFERENCES sessions(id),
    PRIMARY KEY (source_path, line_number, raw_line)
);
PRAGMA user_version = 1;
