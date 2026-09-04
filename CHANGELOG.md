# Changelog

## 0.4.30

- Fix missing Windows MSI in GitHub Release (recursive asset globs, fail loudly on unmatched files)
- Sync versions to 0.4.30 across package.json / tauri.conf.json / Cargo.toml / Cargo.lock

## 0.4.29

- Auto-update: check on startup, prompt dialog, click to download and install (Windows)
- Publish latest.json updater manifest with signed MSI
- NOTE: Windows MSI asset missing in this release due to workflow artifact layout (fixed in 0.4.30)

## 0.4.28

- Release workflow triggers only on v-prefixed tags

## 0.4.27

- Fix type annotation for HashSet in scan_start_menu_programs

## 0.4.26

- Add macOS ARM64 build to release workflow

## 0.4.25

- Version bump only, no functional changes

## 0.4.24

- Release build for macOS and Windows
- Fix macOS Rust test failure (empty path edge case)

## 0.4.23

- Support macOS DMG build
- Add CI automated testing
- Add Rust unit tests

## 0.4.22

- Add macOS DMG build support
- Add CI test step with Rust unit tests
- Fix hotkey show/hide settings window sync
- Fix minimum window size issue
