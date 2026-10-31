# Changelog

## Unreleased

- 导入快捷方式或刷新开始菜单时保留 `.lnk` 的命令行参数，启动时原样传给目标程序；修复腾讯应用宝及安卓应用快捷方式因参数丢失而无法启动的问题
- 同一 exe 配不同参数视为不同启动项：开始菜单去重、前端保存去重与数据库唯一键均纳入 `args`，两个 Androws 快捷方式不再互相覆盖
- 「编辑信息」新增「启动参数」字段，可手动修改可执行文件/快捷方式的启动参数；开始菜单中的程序也可编辑，保存后立即生效
- 开始菜单中的程序可直接拖到自定义分类：拖拽执行复制（保留原开始菜单条目、路径与启动参数），自定义分类之间的拖拽仍为移动并保留打开次数

## 0.4.41

- 常规设置新增「启动后隐藏主界面」：点击图标启动后自动隐藏主窗口，可用显示/隐藏快捷键再次唤出
- 图标可直接拖到左侧分类完成移动，保留打开次数、最近打开时间、备注与置顶状态；此前只能复制，次数会归零。特殊分类是聚合视图，不接受移动
- 图标拖拽改用指针事件实现：Tauri 的原生拖放（外部文件拖入所需）会禁用 WebView2 的 HTML5 拖放，旧实现只显示禁止光标、无法拖动

## 0.4.39

- Autostart uses a "at logon" scheduled task (`schtasks /XML`, task name `OopsLauncher`) instead of `HKCU\...\Run`: at logon explorer runs Run items one by one and waits for each to become ready, so an entry queued behind `SecurityHealthSystray.exe` / `RtkAudUService64.exe` can wait tens of seconds before it is even started. The legacy Run value is removed on the first launch
- Main window is created hidden (`visible: false`) and shown once the frontend is ready — no more blank window before content and no white flash when autostarting minimized
- Drop the render-blocking Google Fonts stylesheet (fetched through the system proxy; a cache miss or a not-yet-ready proxy stalls first paint)

## 0.4.38

- File explorer icons: request them in batches off the UI thread (`get_file_icons` / `list_directory` are now async commands) — a 30-file folder went from ~366 ms to ~73 ms cold and ~1 ms on revisit
- Cache icons by directory/extension in the backend and by generic key/path in the frontend; the cache survives directory switches, and folder icons render straight from the shared cache so entering a new folder no longer flashes a placeholder first
- Load icons only after scrolling stops (120 ms idle) and only for the visible items (max 80 per batch); list view requests 32 px icons and grid 64 px instead of 128 px everywhere
- Fix icon lookup for paths containing forward slashes (e.g. `C:/Windows` typed in the address bar), which silently produced no icon before
- Toolchain: `LazyLock` requires Rust 1.80; add `parking_lot` for the icon cache

## 0.4.36

- Update notice: prompt at most once per version — the notified version is persisted (`localStorage.oopslauncher_update_notified_version`) and skipped on later startups; a newer release prompts again
- Manual "Check for updates" in Settings is not limited by that record and always reports its result

## 0.4.35

- File explorer: rename files and folders in place — right-click menu item or F2 on the selected item (real disk rename, inline edit with the base name preselected)
- Reject invalid names before touching the disk (illegal characters, trailing dot/space, Windows reserved device names) and surface conflicts as `「name」已存在`
- Add Rust unit tests for name validation and real file renaming

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
