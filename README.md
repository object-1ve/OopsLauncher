# 🚀 OopsLauncher

OopsLauncher 是一个基于 [Tauri](https://tauri.app/) 和 [Vue 3](https://vuejs.org/) 构建的现代、轻量级桌面应用启动器和文件管理器。它旨在提供简洁、高效的文件组织和快速启动体验。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Tauri](https://img.shields.io/badge/Tauri-v2-orange)
![Vue](https://img.shields.io/badge/Vue-v3-green)

## ✨ 特性

*   **⚡ 极速启动**：基于 Rust 和 Tauri 构建，资源占用极低，启动速度飞快。
*   **📂 文件管理**：
    *   **拖拽添加**：直接将文件或应用拖入窗口即可添加。
    *   **分类管理**：支持“常用”、“编程”、“娱乐”等多个自定义分类，井井有条。
    *   **智能分组**：支持按“文件类型”自动分类并分组换行显示。
    *   **灵活排序**：支持按名称、打开次数、创建时间进行排序。
    *   **智能图标**：自动提取 .exe 和常用文件类型的系统图标。
*   **🖱️ 便捷交互**：
    *   **右键菜单**：支持右键删除、管理文件、管理分类（重命名、删除、排序）、复制到其他分类。
    *   **全局快捷键**：内置实用快捷键（如 `Alt+T` 快速复制当前时间）。
*   **🎨 现代界面**：
    *   自定义标题栏，无边框设计。
    *   响应式侧边栏，支持折叠/展开，分类名自动换行。
    *   固定两行高度的文件名区域，确保图标对齐整洁。
*   **💾 可靠持久化**：
    *   不仅业务数据（文件/分类）存入 SQLite，UI 状态（选中分类、排序方式、分类方式）也已接入数据库持久化，告别 localStorage 的不稳定性。

## 🛠️ 技术栈

*   **前端**：Vue 3, Element Plus, Vite
*   **后端**：Rust, Tauri v2
*   **数据库**：SQLite (rusqlite) - 全面覆盖业务数据与 UI 配置状态
*   **其他**：
    *   `tauri-plugin-global-shortcut`: 全局快捷键支持
    *   `tauri-plugin-clipboard-manager`: 剪贴板管理
    *   `tauri-plugin-notification`: 系统通知支持
    *   `tauri-plugin-autostart`: 开机自启管理
    *   `tauri-plugin-single-instance`: 单实例模式运行
    *   `tauri-plugin-dialog`: 文件/目录对话框支持

## 🚀 快速开始

### 前置要求

确保你的开发环境已安装：
*   [Node.js](https://nodejs.org/) (建议 v16+)
*   [Rust](https://www.rust-lang.org/) (最新稳定版)
*   C++ 生成工具 (Windows 上需安装 Visual Studio 生成工具)

### 安装依赖

```bash
# 安装前端依赖
npm install

# 安装 Rust 依赖（通常在首次运行时自动处理，也可手动）
cd src-tauri
cargo check
```

### 开发模式运行

```bash
npm run tauri dev
```

### 打包构建

```bash
npm run tauri build
```

构建产物将位于 `src-tauri/target/release/bundle` 目录下。

## ⌨️ 快捷键

| 快捷键 | 功能 | 描述 |
| :--- | :--- | :--- |
| `Alt + T` | 复制时间 | 将当前时间 (YYYY-MM-DD HH:mm:ss) 复制到剪贴板 |

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！如果你有好的想法或建议，请随时分享。

1.  Fork 本仓库
2.  创建你的特性分支 (`git checkout -b feature/AmazingFeature`)
3.  提交你的修改 (`git commit -m 'Add some AmazingFeature'`)
4.  推送到分支 (`git push origin feature/AmazingFeature`)
5.  开启一个 Pull Request

参考了 https://github.com/fanchenio/DawnLauncher 项目的设计和实现。感谢大佬
## 📄 许可证

本项目基于 [MIT 许可证](LICENSE) 开源。
