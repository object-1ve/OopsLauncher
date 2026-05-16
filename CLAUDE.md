# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Dev Commands

| Command | Description |
|---|---|
| `npm run tdev` | Full Tauri dev mode (auto-starts Vite on port 5175, then Rust backend) |
| `npm run tbuild` | Production build (Vite + Rust, default bundling) |
| `npm run tbundle` | MSI-only production build |
| `npm run dev` | Vite frontend-only dev server (port 5175) |
| `npm run build` | Vite frontend-only production build to `dist/` |
| `npm run v` | Sync version across package.json, tauri.conf.json, and Cargo.toml |

## Architecture

**Tauri v2 desktop app** — Vue 3 frontend in a frameless WebView window, Rust backend for all system/database/fs operations. The frontend never accesses the filesystem or database directly.

### Communication Model

All frontend→backend calls go through `invoke()` from `@tauri-apps/api/core`. Commands are defined as `#[tauri::command]` functions in `src-tauri/src/commands/`, registered in `src-tauri/src/lib.rs` under the `invoke_handler`.

Backend→frontend push uses `app.emit()` in Rust, received via `listen()` from `@tauri-apps/api/event` in JS. Key events:
- `settings-changed` — cross-window settings sync
- `request-exit` — system tray quit
- `tauri://drag-drop` — native file drops

Tauri plugins (global-shortcut, clipboard-manager, dialog, autostart, notification, shell) expose their own JS APIs directly without `invoke()`.

### State Management

- **`src/stores/settingsStore.js`** — Pinia store for app settings (general/appearance/shortcuts). Deep-watches with 500ms debounce, auto-persists to `settings.json`. Has `isExternallyUpdating` lock to prevent cross-window save loops.
- **`src/composables/useFiles.js`** — Singleton composable (module-level `ref()` variables) for all file/category/search/classification/sort state. Not Pinia — importing `useFiles()` anywhere returns the same reactive state. Persists via `invoke()` calls to SQLite.

### Data Persistence (3 tiers)

1. **SQLite** (`oopslauncher.db` in Tauri app_data_dir) — WAL mode. `files` table, `categories` table, legacy `settings` table. Schema and migrations in `src-tauri/src/db.rs`.
2. **JSON files** (same dir) — `settings.json` (appearance/shortcuts/general settings), `launcher_state.json` (current category, sort/classify preferences).
3. **localStorage** — Fallback when Tauri APIs are unavailable (dev mode without Rust backend). Keys: `oopslauncher_categories`, `oopslauncher_files`, `oopslauncher_current_category`, etc.

### Routing

Two Vue Router routes:
- `/` → `BasicLayout` → `Home.vue` (main launcher)
- `/settings` → `Settings.vue` (opened as a **separate Tauri WebviewWindow**, not in-app navigation)

### Key Frontend Files

- **`src/composables/useFiles.js`** — Core business logic: file CRUD, category management, classification (`'none'` / `'type'`), sorting (name/openCount/created_at), search across all categories, drag-drop handling, deduplication by `normalizePathKey()`.
- **`src/composables/useContextMenu.js`** — Singleton context menu manager ensuring only one menu is open at a time.
- **`src/views/Home.vue`** — Main orchestrator: wires DropZone, ContextMenu, FileInfoDialog, and global search/keyboard shortcuts together.
- **`src/components/DropZone.vue`** — File grid with pinned section, grouped-by-type sections (when classifyMethod=`'type'`), and drop zone for adding files.
- **`src/components/SearchOverlay.vue`** — Modal search with keyboard nav, 15-result limit, right-click "定位到此文件" (locate to file — switches category and scrolls/highlights).

### Key Rust Files

- **`src-tauri/src/commands/file.rs`** — SQLite save/load for files, file info retrieval, open file/location/terminal/dialog.
- **`src-tauri/src/commands/category.rs`** — Category CRUD against SQLite.
- **`src-tauri/src/commands/settings.rs`** — JSON file persistence for launcher_state and settings, emits `settings-changed` event.
- **`src-tauri/src/db.rs`** — SQLite connection, schema creation, column-level migrations.
- **`src-tauri/src/icon.rs`** — Windows-specific: extracts file icons via `IShellItemImageFactory` COM API, returns base64 PNG.

### Deduplication

`useFiles.js` uses `normalizePathKey()` (lowercase + `\\` separator normalization) for cross-category uniqueness. `categoryHasSameFile()` prevents inserting the same file twice in one category. The "All Files" virtual category uses a Map to deduplicate by path.

### Windows-Specific Features

- Frameless window with custom title bar (`data-tauri-drag-region`)
- Taskbar hide/reveal (`set_skip_taskbar`)
- DWM window animation removal
- .lnk shortcut resolution
- "Open With" dialog via `open_with_dialog`
