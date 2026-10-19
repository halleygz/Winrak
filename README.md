# Winrak · Window tracker

A private, local-first desktop activity dashboard built with **Tauri v2,
React 19, TypeScript, Vite 7, Tailwind CSS 4, Rust, and SQLite**. This repository
is the application root; do not create a nested `window-tracker/` directory.
No account, analytics, cloud sync, or network service is required.

## Install & run

Use Node.js **22.18+** (or current Node LTS), npm, and a current Rust stable
toolchain supporting edition 2024. Install the platform development dependencies
from [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/).

Ubuntu/Debian example (package names differ on other distributions):

```sh
sudo apt update
sudo apt install build-essential pkg-config libwebkit2gtk-4.1-dev \
  libgtk-3-dev libssl-dev librsvg2-dev libayatana-appindicator3-dev \
  libx11-dev libxcb1-dev libdbus-1-dev patchelf
npm install
npm run tauri dev
```

Windows requires MSVC C++ build tools and WebView2. macOS requires Xcode command
line tools. Native detection is provided by `active-win-pos-rs` 0.11 and can
require platform permissions/runtime utilities in addition to build packages.

```sh
npm run dev          # browser preview at http://127.0.0.1:1420
npm run build        # TypeScript validation + frontend output in dist/
npm run preview      # preview production frontend in a browser
npm run tauri dev    # Vite + native desktop application
npm run tauri build  # frontend + production desktop bundles
npm test             # formatting and date-range tests; no extra test dependency
```

A browser **cannot** read this app's SQLite database or track native focus. Its
preview clearly explains this and shows no fake activity. To use the dashboard,
launch the desktop application. Run commands from the repository root.

## Dashboard

- Live **last detected application**, kept separate from recorded totals.
- Today, rolling 7 days, rolling 30 days, and custom inclusive date ranges.
  Presets advance on local date rollover and window focus.
- Recorded duration, average per calendar day (including empty days), application
  count, saved segment count, and most-used app.
- Stacked activity graph: five leading applications plus all others. Each bar is
  keyboard-selectable and exposes exact durations. At most 31 bars: longer custom
  ranges are aggregated into contiguous, equally sized groups of days (last group
  may be shorter). This is not an hourly timeline.
- Duration-ranked application breakdown, eight apps per page; percentages are
  shares of **saved** time. Original app names are available on hover.
- Searchable session history, newest first, 15 rows per page. Expand an app name
  for its original value and stored timestamps. Clock times retain their recorded
  offset/local clock, rather than being shifted into today's timezone.
- Loading, no-data, invalid-date, import-failure, and query/tracking error states;
  refresh/retry actions; no synthetic statistics. Failed refreshes are labeled.
- Completed-session events trigger refreshes; focus events update the header.
  Queries also refresh on window focus and every 30 seconds as a fallback.

## Tracking semantics & limitations

**Native tracking behavior is preserved.** A background thread calls core
`poll()` immediately, then sleeps **one second after every poll**. Detection
and persistence occur under a mutex; event emission and sleeping do not. The
service alone owns tracking and database logic.

- A session is saved only when a **different app is successfully detected**.
  Its ongoing duration is **not** included in any dashboard total.
- There is **no idle timeout, idle exclusion, pause, startup autolaunch, or final
  shutdown flush**. Quitting loses the current unfinished session.
- Missing/failed detection leaves the existing session running, exactly as the
  original tracker did. The header can retain the last detected app. Time while
  locked, suspended, or detection is unavailable is not guaranteed to represent
  active work. Do not interpret the dashboard as a productivity score.
- App names are native reported names, not website categories. Display names may
  omit paths and `.exe`, but raw names remain distinct in storage and grouping.
- Durations come from monotonic elapsed time, floored to seconds in storage, not
  a subtraction of displayed wall-clock times. Native timestamps retain offsets.
- The existing logger's midnight split algorithm is retained: one row per local
  calendar-day segment, with a saturating final remainder. Crossing midnight
  increases the saved **segment** count; a final zero-second segment can exist.
- X11 detection normally needs an accessible desktop session. **Wayland support
  varies by compositor**, permissions, and native-library runtime helpers;
  universal Wayland support is not promised. On macOS, Accessibility permission
  may be required. Windows secure desktops and inaccessible/elevated windows can
  be undetectable. No detection is expected in headless/CI sessions.
- A failed save is visible as a sticky warning. The core may already have advanced
  to the next session, so a failed completed session may be lost; Retry clears the
  warning and retries initialization/import, **not that lost session**.
- SQLite is local and **not encrypted**. OS file permissions protect it. Avoid
  running the legacy logger concurrently if you want an unambiguous migration.

## Storage & schema

The shell opens **`app_data_dir()/winrak.db`**, using application identifier
`com.winrak.windowtracker`; it never falls back to a cwd database. The UI's
“Storage & tracking notes” displays the resolved database path.

Typical locations (exact OS settings may vary):

- Linux: `$XDG_DATA_HOME/com.winrak.windowtracker/winrak.db`, normally
  `~/.local/share/com.winrak.windowtracker/winrak.db`.
- macOS: `~/Library/Application Support/com.winrak.windowtracker/winrak.db`.
- Windows: `%APPDATA%\com.winrak.windowtracker\winrak.db`.

The core creates missing parent directories, uses a 5-second SQLite busy timeout,
foreign keys, and WAL mode. SQLite is bundled through `rusqlite` 0.32.

Migration `src-tauri/src/db/migrations/001_create_sessions.sql` initializes schema
version **1** transactionally, tracked with `PRAGMA user_version`. Unsupported
schema versions are rejected, not silently reset.

- `sessions`: `id`, `app_name`, `started_at`, `ended_at`, nonnegative integer
  `duration_seconds`, and local `date` (`YYYY-MM-DD`). Date/app and start-time
  indexes support range queries.
- `legacy_import_lines`: canonical source path, physical line number, original
  line text, and imported session foreign key. This preserves legitimate identical
  sessions while making a repeated import of the same source idempotent.

To back up, close Winrak first and copy the database (and any remaining `-wal` /
`-shm` sidecars). Never edit the live SQLite files manually. There is no retention
or deletion control in this release.

## Migrating the legacy `log.txt`

At startup the shell asks the core to import existing candidates in this order:

1. `app_data_dir()/log.txt`.
2. The current working directory's `log.txt`.
3. The working directory's parent's `log.txt` (covers Cargo running in
   `src-tauri/` while the original log is in the project root).
4. `resource_dir()/log.txt` **only if actually present** in a bundle.

Canonical aliases are deduplicated. This build does **not** embed the developer's
log or absolute project path into a bundle. The original file is never modified
or removed. Missing files are harmless; import errors are shown in the UI.

**For an installed/packaged application:** place the original `log.txt` alongside
`winrak.db`, then restart Winrak or open “Storage & tracking notes” and choose
**Retry tracking & import**. No file picker/plugin or arbitrary-path IPC import
is exposed. You can leave the file there; new appended lines are imported on
restart/retry without duplicating unchanged lines.

The core imports valid dated logger rows transactionally, skips malformed rows,
and preserves the original text for import identity. Historical timestamps have
no invented UTC offset. Duration values are retained rather than recomputed.
Because identity includes the **canonical path**, importing a copy at a different
path, or editing/reordering an already imported log, can create additional rows.
Keep one stable source. Historical data may need a custom date range to appear.

## IPC & feature configuration

All serialized models use camelCase. Registered commands:

- `get_sessions({ from?, to? })` → persisted calendar-day `Session[]`.
- `get_daily_totals({ from?, to? })` → per-date/per-app `DailyAppTotal[]`.
- `get_app_totals({ from?, to? })` → duration-ranked `AppTotal[]`.
- `get_dashboard({ from?, to? })` → sessions/daily/apps under one service mutex
  lock so the dashboard stays coherent if a poll completes during refresh.
- `get_current_app()` → last detected app name or null.
- `get_tracking_status()` → current app, tracking/import errors, database path.
- `retry_tracking()` → retry database initialization and idempotent imports.

Bounds are optional strict `YYYY-MM-DD`, inclusive; absent bounds are unbounded.
The core validates dates/ranges. Events: `session_completed` only after successful
persistence (payload is the core completed-session object), and `current_focus`
(string or null) after polling. The frontend uses the completion event as an
invalidation signal, not as a new row to fabricate locally.

Default Rust feature **`desktop`** enables Tauri, `tauri-build`, and
**`native-active-window`** (`active-win-pos-rs`). The binary requires `desktop`.
`--no-default-features` builds/tests the core without Tauri/native desktop
system dependencies and detects no focus. Production defaults preserve native
window detection. Tauri v2 main-window core capabilities and an explicit local
content security policy are configured; no shell/filesystem/network plugins are
exposed to the frontend.

## Project layout

```text
package.json / package-lock.json / index.html / vite.config.ts / tsconfig.json
public/
src/
  main.tsx / App.tsx / styles.css
  components/dashboard/{TodaySummary,AppUsageChart,SessionTable}.tsx
  components/ui/Panel.tsx
  hooks/useSessions.ts
  lib/{tauri,format-duration,date-range}.ts
  types/session.ts
src-tauri/
  Cargo.toml / Cargo.lock / build.rs / tauri.conf.json
  capabilities/default.json / icons/
  src/{main,lib}.rs
  src/commands/{mod,sessions}.rs
  src/platform/{mod,active_window}.rs
  src/tracker/{mod,session}.rs
  src/db/{mod,connection,repository}.rs
  src/db/migrations/001_create_sessions.sql
  src/services/{mod,tracking_service}.rs
tests/frontend.test.mjs
```

The Rust engine has moved into **`src-tauri/src/`**; root `src/` contains only
the frontend. The obsolete root Rust manifest, lockfile, and source files were
removed after comparing tracking behavior with the originals in Git history.
The original `log.txt` remains intact as the migration source. Run Rust checks
inside **`src-tauri`**.

## Verification

```sh
npm run build
npm test
cd src-tauri
cargo check --locked
cargo test --locked
cargo test --locked --no-default-features
cargo fmt -- --check
```

Integration checks passed: frontend production build, five frontend tests,
17 default-feature Rust tests (including the shell import test), 17 headless
core tests, and formatting. No remaining build/dependency blockers.

The scaffold was created manually in place to preserve legacy sources, following
[Tauri v2 manual setup](https://v2.tauri.app/start/create-project/#manual-setup-tauri-cli)
and [Vite configuration](https://v2.tauri.app/start/frontend/vite/) documentation.
Installed Tauri CLI **2.12.3** and `tauri init --help` were checked. Frontend assets
resolve from `../dist` relative to `src-tauri/`; development uses root npm commands.
A real GUI launch, OS permission prompts, and platform-specific native detection
were not verified in this build environment.
