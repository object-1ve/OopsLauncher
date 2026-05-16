---
name: tauri2-coding
description: Tauri v2 桌面应用开发规范，涵盖 Rust 后端与 Vue/JS 前端的架构模式、通信模型和平台特性
metadata:
  type: project
---

# Tauri v2 项目编码规范

遵循本项目的 Tauri v2 实际实现模式（OopsLauncher 代码库），所有新代码必须与现有架构一致。

## 通信模型

### 前端调用后端 (invoke)

使用 `@tauri-apps/api/core` 的 `invoke` 函数，命令名与 Rust 端 `#[tauri::command]` 函数名一致：

```js
import { invoke } from '@tauri-apps/api/core'

// 基本调用
await invoke('save_files_to_db', { files: allFiles })

// 无参数调用
const version = await invoke('get_app_version')
```

**参数名映射规则**：JS 端使用驼峰（camelCase）参数名会被 Tauri 自动映射为 Rust 端的蛇形（snake_case）。例如 `{ showSuccess: true }` 映射到 Rust 端的 `show_success: bool`。

### 后端推送事件到前端 (emit/listen)

Rust 端通过 `app.emit()` 发送事件：

```rust
use tauri::Emitter;
app.emit("event-name", payload)?;
// payload 可以是任意 serde::Serialize 类型
```

前端通过 `@tauri-apps/api/event` 的 `listen` 接收：

```js
import { listen } from '@tauri-apps/api/event'

await listen('settings-changed', (event) => {
  const payload = event.payload // 自动反序列化
})
```

**本项目已用事件**：
- `settings-changed` — 跨窗口设置同步（`settings.rs` 中 `save_settings_to_json` 发送，`settingsStore.js` 接收）
- `request-exit` — 系统托盘退出请求（`lib.rs` 中 tray menu 发送，`App.vue` 中接收并保存数据后调用 `exit_app`）
- `tauri://drag-drop` — Tauri 原生拖拽事件（`useFiles.js` 中监听，处理文件拖入）

## 命令注册模式

### 定义命令

每个命令文件放在 `src-tauri/src/commands/` 下，使用 `#[tauri::command]` 属性：

```rust
// commands/my_module.rs
#[tauri::command]
pub fn my_command(app: tauri::AppHandle, param: String) -> Result<(), String> {
    // 返回 Result<Ok, String> 错误会自动转为 JS Error
}
```

**参数规则**：
- 需要 `AppHandle` 时作为第一个参数
- 简单类型自动反序列化
- 复杂类型实现 `serde::Deserialize`

### 注册命令

在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册：

```rust
.invoke_handler(tauri::generate_handler![
    commands::my_module::my_command,
    commands::my_module::another_command,
])
```

**模块声明**：在 `lib.rs` 顶部 `pub mod commands;`，然后在 `commands/mod.rs` 中 `pub mod my_module;`。

## 插件使用模式

在 `Builder::default()` 的链式调用中通过 `.plugin()` 注册。本项目已用插件：

### 常用 Tauri v2 插件

| 插件 | npm 包 | Cargo 包 | 注册方式 |
|---|---|---|---|
| global-shortcut | `@tauri-apps/plugin-global-shortcut` | `tauri-plugin-global-shortcut` | `Builder::new().build()` |
| clipboard-manager | `@tauri-apps/plugin-clipboard-manager` | `tauri-plugin-clipboard-manager` | `::init()` |
| dialog | `@tauri-apps/plugin-dialog` | `tauri-plugin-dialog` | `::init()` |
| notification | `@tauri-apps/plugin-notification` | `tauri-plugin-notification` | `::init()` |
| autostart | `@tauri-apps/plugin-autostart` | `tauri-plugin-autostart` | `::init(MacosLauncher::LaunchAgent, args)` |
| single-instance | 无 | `tauri-plugin-single-instance` | `::init(callback)` |
| shell | `@tauri-apps/plugin-shell` | `tauri-plugin-shell` | `::init()` |

JS 端插件注册方式示例：

```js
import { register, unregister } from '@tauri-apps/plugin-global-shortcut'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
```

### 插件初始化顺序

在 `lib.rs` 中：`plugin()` 调用在 `invoke_handler` 之前。

### single-instance 回调

```rust
tauri_plugin_single_instance::init(|app, _args, _cwd| {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
})
```

## 窗口管理

### 窗口配置（tauri.conf.json）

```json
{
  "app": {
    "windows": [
      {
        "title": "OopsLauncher",
        "width": 1200,
        "height": 800,
        "center": true,
        "resizable": true,
        "decorations": false,
        "shadow": true,
        "visible": false
      }
    ]
  }
}
```

关键选项：
- `decorations: false` — 无边框窗口，配合自定义标题栏 + `data-tauri-drag-region`
- `visible: false` — 初始隐藏，加载完成后通过 `appWindow.show()` 控制显示
- 设置窗口通过 `new WebviewWindow('settings', { url: '/settings' })` 创建（独立窗口，非路由导航）

### JS 窗口 API

```js
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

const appWindow = getCurrentWebviewWindow()
const isVisible = await appWindow.isVisible()
await appWindow.show()
await appWindow.setFocus()
await appWindow.hide()
await appWindow.setOpacity(val) // 透明度
appWindow.label // 窗口标识 'main' | 'settings'
```

### 窗口事件

```rust
.on_window_event(|window, event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == "main" {
            let _ = window.hide();
            api.prevent_close(); // 主窗口隐藏而非退出
        }
    }
})
```

## 系统托盘

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

// 创建菜单项
let show_i = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

// 创建托盘
let _tray = TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu)
    .show_menu_on_left_click(false) // 左键点击不显示菜单
    .on_menu_event(|app, event| { /* 处理菜单事件 */ })
    .on_tray_icon_event(|tray, event| { /* 处理托盘图标事件 */ })
    .build(app)?;
```

左键单击托盘图标切换窗口显示/隐藏，菜单项控制"显示主窗口"/"退出"。

## 数据持久化

### SQLite (WAL 模式)

通过 `rusqlite::Connection` 操作，连接通过 `AppHandle.path().app_data_dir()` 获取数据库路径：

```rust
use tauri::Manager;
let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
// 确保目录存在
fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
let conn = Connection::open(dir.join("oopslauncher.db"))?;
```

**WAL 模式 + NORMAL 同步模式**：
```rust
conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
```

**事务使用**：
```rust
let tx = conn.transaction().map_err(|e| e.to_string())?;
// ... 执行操作 ...
tx.commit().map_err(|e| e.to_string())?;
```

**列级迁移**：通过 `PRAGMA table_info(table)` 获取现有列，逐列检查并 `ALTER TABLE ADD COLUMN`。

### JSON 文件

使用 `serde_json` 序列化/反序列化，通过 `app.path().app_data_dir()` 获取路径：

```rust
use tauri::Manager;
let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
let path = dir.join("settings.json");
let content = serde_json::to_string_pretty(&data)?;
fs::write(path, content)?;
```

### 前端 fallback

在非 Tauri 环境（dev 模式无 Rust 后端）使用 `localStorage`：
```js
const isTauri = () => !!window.__TAURI_INTERNALS__

if (isTauri()) {
  await invoke('save_xxx', { ... })
} else {
  localStorage.setItem('key', JSON.stringify(data))
}
```

## 前端架构模式

### Event Listener 去重

使用模块级布尔标志防止重复注册 Tauri 事件监听器：
```js
let tauriListenersSet = false;

const setup = async () => {
  if (tauriListenersSet) return;
  tauriListenersSet = true;
  await listen('event-name', handler);
};
```

### 保存防抖

数据保存带防抖（500ms）和锁机制防止并发：
```js
let saveTimeout = null;
const saveWithDebounce = (data) => {
  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(async () => {
    await invoke('save_cmd', data);
  }, 500);
};
```

### 跨窗口同步锁

使用 `isExternallyUpdating` 标志阻止外部更新触发循环保存：
```js
const isExternallyUpdating = ref(false)

// 监听外部变更时：
isExternallyUpdating.value = true
settings.value = mergeSettings(newSettings)
nextTick(() => { isExternallyUpdating.value = false })
```

### 保存队列

防止并发保存导致数据丢失（`useFiles.js` 模式）：
```js
let isSaving = false;
let savePending = false;

const save = async () => {
  if (isSaving) { savePending = true; return; }
  isSaving = true;
  try { /* invoke save */ } finally {
    isSaving = false;
    if (savePending) { savePending = false; await save(); }
  }
};
```

### 数据加载保护

通过 `hasLoaded` 标志防止在数据加载完成前触发保存致数据丢失：
```js
let hasLoaded = false;
// 仅在 hasLoaded 为 true 时才执行 saveFiles()
```

### 文件路径规范化去重

```js
const normalizePathKey = (rawPath) => {
  const path = String(rawPath || '').trim()
  if (!path) return ''
  const normalizedSlashes = path.replace(/\//g, '\\').replace(/\\+/g, '\\')
  return normalizedSlashes.toLowerCase()
}
```

## 平台特定代码

### Windows 条件编译

```rust
#[cfg(target_os = "windows")]
{
    // Windows-specific implementation
}

#[cfg(target_os = "macos")]
{
    // macOS-specific implementation
}

#[cfg(all(unix, not(target_os = "macos")))]
{
    // Linux-specific implementation
}
```

### Windows COM API 使用模式

```rust
#[cfg(target_os = "windows")]
use windows::{
    core::HSTRING,
    Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
};
```

模式：`CoInitializeEx` → 操作 → `CoUninitialize`，用 closure 封装返回值处理。

### Windows 特有功能

- **文件图标提取**：`IShellItemImageFactory` COM 接口，返回 base64 PNG
- **快捷方式解析**：`IShellLinkW` + `IPersistFile` COM 接口，解析 `.lnk` 文件
- **`\\?\` 前缀移除**：`canonicalize()` 返回的路径可能含 `\\?\`，需清理
- **打开方式对话框**：`rundll32.exe shell32.dll,OpenAs_RunDLL`
- **资源管理器定位**：`explorer.exe /select,<path>`
- **终端打开**：`cmd /c start powershell -NoExit -Command Set-Location`
- **任务栏隐藏**：通过 `invoke("set_skip_taskbar")` 调用 Rust 端 `set_skip_taskbar` 命令

### Cargo.toml 平台依赖

```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.52.0", features = [...] }
```

## 项目结构

```
src-tauri/
├── src/
│   ├── main.rs          # 入口（调用 lib::run）
│   ├── lib.rs           # Builder 配置、插件注册、托盘、事件处理
│   ├── db.rs            # SQLite 连接管理、建表、迁移
│   ├── models.rs        # serde 序列化结构体
│   ├── utils.rs         # 工具函数（路径解析、快捷方式解析）
│   ├── icon.rs          # Windows 文件图标提取
│   └── commands/
│       ├── mod.rs
│       ├── file.rs      # 文件 CRUD、打开/定位/终端
│       ├── category.rs  # 分类 CRUD
│       ├── settings.rs  # JSON 持久化、launcher state
│       ├── app.rs       # 应用命令（版本、窗口控制、退出）
│       └── notification.rs  # 系统通知
src/
├── App.vue              # 主入口：快捷键注册、启动逻辑、事件监听
├── main.js              # Vue 挂载、Pinia、ElementPlus
├── router/index.js      # Vue Router（/ 和 /settings 路由）
├── stores/
│   └── settingsStore.js # Pinia 设置状态管理
├── composables/
│   ├── useFiles.js      # 文件/分类/搜索核心逻辑
│   └── useContextMenu.js # 右键菜单管理
├── components/
│   ├── DropZone.vue     # 文件网格
│   ├── SearchOverlay.vue # 搜索遮罩层
│   ├── ContextMenu.vue  # 右键菜单
│   └── FileInfoDialog.vue # 文件详情弹窗
├── views/
│   ├── Home.vue         # 主页面
│   └── Settings.vue     # 设置页面
└── layout/
    └── basic.vue        # 基础布局
```

## 路由与窗口

- `/` → `BasicLayout > Home.vue`（主窗口，frameless）
- `/settings` → `Settings.vue`（独立 WebviewWindow，通过 Tauri API 创建，非前端路由跳转）
- 设置窗口通过 `new WebviewWindow('settings', { url: '/settings' })` 打开，这意味着设置页作为一个全新的 Tauri 窗口打开，拥有独立的 JS 上下文

## 性能与安全

- 避免在 Rust 命令中执行长时间同步操作（会阻塞 WebView 主线程）
- 前端 invoke 调用是异步的，但 Rust 端命令默认在后台线程执行
- 所有文件系统操作在 Rust 端完成，前端不直接访问文件系统
- 使用 Content Security Policy（`"csp": null` 表示不限制，生产环境应配置）

## 调试

- `tauri-plugin-log` 仅在 `debug_assertions` 下加载
- JS 端通过 `isTauri()` 检测环境，非 Tauri 环境使用 localStorage 降级
- Rust 端使用 `println!` 输出日志（不依赖 log crate 配置）
