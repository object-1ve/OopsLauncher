# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Dev Commands

| Command | Description |
|---|---|
| `npm run tdev` | Full Tauri dev mode (Vite on port 5175 + Rust backend) |
| `npm run tbuild` | Production build (Vite + Rust, default bundling) |
| `npm run tbundle` | MSI-only production build |
| `npm run dev` | Vite frontend-only dev server (port 5175) — Rust backend unavailable |
| `npm run build` | Vite frontend-only production build to `dist/` |
| `npm run v` | Sync version across package.json, tauri.conf.json, and Cargo.toml |
| `cargo check` | Rust type-check (in `src-tauri/`) |
| `cargo clippy` | Rust lint (in `src-tauri/`) |

No test framework is configured for either frontend or Rust.

## Architecture

**Tauri v2 desktop app** — Vue 3 + Element Plus frontend in a frameless WebView window, Rust backend for all system/DB/fs operations. Frontend never accesses filesystem or DB directly.

### Communication Model

**Frontend→Backend**: `invoke()` from `@tauri-apps/api/core` → `#[tauri::command]` in Rust.
**Backend→Frontend**: `app.emit()` in Rust → `listen()` from `@tauri-apps/api/event` in JS.

Key events:
- `settings-changed` — cross-window settings sync (sent from `save_settings_to_json`)
- `request-exit` — system tray quit → frontend saves data → calls `exit_app`
- `tauri://drag-drop` — native file drops (handled in `useFiles.js`)

Tauri plugins (global-shortcut, clipboard-manager, dialog, autostart, notification, shell) expose their own JS APIs directly — no `invoke()` needed.

### Field Naming Convention

JS frontend uses **camelCase** (`displayName`, `openCount`, `isPinned`), Rust backend uses **snake_case** (`display_name`, `open_count`, `is_pinned`). The conversion is **manual** in `saveFiles()` / `loadFiles()` — there is no auto-mapping layer:

```js
// JS → Rust (save)
const fileToSave = {
  display_name: String(file.displayName || ''),
  open_count: Number(file.openCount || 0),
  is_pinned: !!file.isPinned
}

// Rust → JS (load)
const { open_count, display_name, is_pinned } = file
const fileWithFormattedFields = {
  openCount: open_count || 0,
  displayName: display_name || '',
  isPinned: !!is_pinned
}
```

### Rust Backend Structure

```
src-tauri/src/
├── main.rs              # Entry, calls app_lib::run()
├── lib.rs               # Builder assembly: plugins, invoke_handler, tray, window events
├── db.rs                # SQLite connection, schema creation, column-level migrations
├── models.rs            # Serde structs: FileInfo, Category, LauncherState
├── utils.rs             # Path utilities + .lnk shortcut resolution
├── icon.rs              # Windows IShellItemImageFactory COM → base64 PNG
└── commands/
    ├── mod.rs           # Re-exports all submodules
    ├── file.rs          # File CRUD, get_file_info, open_path/location/terminal/dialog
    ├── category.rs      # Category CRUD
    ├── settings.rs      # JSON persistence + settings-changed event emit
    ├── app.rs           # get_app_version, set_skip_taskbar, exit_app, window animation
    └── notification.rs  # send_notification_custom via tauri-plugin-notification
```

All commands return `Result<_, String>`. Errors are string-based, no custom error type.

### Frontend Structure

```
src/
├── main.js               # Vue app mount, Pinia, Element Plus, icon registration
├── App.vue               # Entry: shortcut registration, autostart, window show/hide, events
├── router/index.js       # Vue Router: / → Home, /settings → Settings (separate window)
├── stores/
│   └── settingsStore.js  # Pinia: deep-watch with 500ms debounce → settings.json
├── composables/
│   ├── useFiles.js       # Singleton: files/categories/search/sort/classify state + CRUD
│   └── useContextMenu.js # Singleton: ensures one context menu open at a time
├── views/
│   ├── Home.vue          # Main layout orchestrator
│   └── Settings.vue      # Settings page (opened as separate WebviewWindow)
├── layout/
│   └── basic.vue         # Sidebar + main content layout
├── components/
│   ├── DropZone.vue      # File grid: pinned, grouped-by-type, drop zone
│   ├── SearchOverlay.vue # Modal search, 15-result limit, keyboard nav, "locate file"
│   ├── ContextMenu.vue   # Right-click menu (file/category operations)
│   └── FileInfoDialog.vue # File details dialog
└── styles/
    └── main.css          # Global styles + Element Plus overrides
```

### State Management (3 patterns)

1. **Pinia store** (`settingsStore.js`) — App settings (general/appearance/shortcuts). Deep-watch with 500ms debounce auto-persists to `settings.json`. Has `isExternallyUpdating` lock to prevent cross-window save loops.

2. **Singleton composable** (`useFiles.js`) — Module-level `ref()` variables. Not Pinia — importing `useFiles()` anywhere returns the same reactive state. Uses save queue pattern to prevent concurrent saves:
   ```js
   let isSaving = false;
   let savePending = false;
   // If save is called while already saving, queues one retry
   ```

3. **Singleton composable** (`useContextMenu.js`) — Module-level `activeMenuClose` reference. Ensures only one context menu is open.

### Data Persistence (3 tiers)

1. **SQLite** (`oopslauncher.db` in Tauri `app_data_dir`) — WAL mode. Tables: `files`, `categories`, legacy `settings`. Column-level migrations via `PRAGMA table_info()`.
2. **JSON files** (same dir) — `settings.json` (appearance/shortcuts/general), `launcher_state.json` (current category, sort/classify preferences).
3. **localStorage** — Fallback when Tauri APIs unavailable. Keys: `oopslauncher_categories`, `oopslauncher_files`, `oopslauncher_current_category`, etc.

### Deduplication

`normalizePathKey()` normalizes paths to lowercase + backslash separators for consistent comparison. `categoryHasSameFile()` prevents inserting the same file path twice within one category. "All Files" virtual category uses a `Map` keyed by `normalizePathKey(file.path)`.

### Settings Structure

```
general:   { autoStart, autoStartMinimized, minimizeToTray, hideTaskbar, language }
appearance: { theme, transparency, itemLayout, iconSize, showFileName, css: { primaryColor, ... } }
shortcuts:  { showHide, copyTime, testNotification, notificationIcon }
```

### Windows-Specific Features

- Frameless window with `data-tauri-drag-region` custom title bar
- Taskbar hide/reveal via `set_skip_taskbar` command
- DWM window transitions removal (`DwmSetWindowAttribute`)
- `.lnk` shortcut resolution via `IShellLinkW` COM interface
- File icon extraction via `IShellItemImageFactory` COM interface (128px → 48px fallback for transparency)
- "Open With" dialog via `rundll32.exe shell32.dll,OpenAs_RunDLL`
- File explorer location: `explorer.exe /select,<path>`
- Terminal open: `cmd /c start powershell -NoExit -Command Set-Location '<path>'`

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **OopsLauncher** (812 symbols, 1133 relationships, 20 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/OopsLauncher/context` | Codebase overview, check index freshness |
| `gitnexus://repo/OopsLauncher/clusters` | All functional areas |
| `gitnexus://repo/OopsLauncher/processes` | All execution flows |
| `gitnexus://repo/OopsLauncher/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
