# 曼波语音生成器 (Manbo TTS)

基于 [Tauri](https://tauri.app) + [Vue 3](https://vuejs.org) + [TypeScript](https://www.typescriptlang.org) 构建的桌面端语音生成应用。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![GitHub Release](https://img.shields.io/github/v/release/milorapart/manbo-tts?include_prereleases)](https://github.com/milorapart/manbo-tts/releases/latest)

## 下载安装

> 无需配置开发环境，直接下载预构建安装包即可使用。

前往 **[GitHub Releases](https://github.com/xmimu/manbo-tts/releases/latest)** 页面下载 Windows 安装包：

- `.msi`（推荐）
- `.exe`（NSIS 安装程序）

下载后双击安装包按提示完成安装即可。

## 截图预览

![应用截图](manbo-tts-screen.png)

## 功能特性

- 文本转语音（TTS）生成
- Windows 桌面应用
- 轻量、快速，基于 Rust 后端

## API 说明

本应用的语音生成功能由 **MiloraAPI** 提供支持。

应用在后端通过调用 MiloraAPI 的语音合成接口，将用户输入的文本发送至服务端进行处理，并将返回的音频数据在客户端播放或保存。使用前请确保已获取有效的 MiloraAPI 访问凭证，并在应用设置中正确配置。

> 详细的接口文档请参阅 MiloraAPI 官方说明。

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3 + TypeScript |
| 构建工具 | Vite |
| UI 组件库 | Element Plus |
| 桌面框架 | Tauri 2 |
| 后端语言 | Rust |

## 开发环境要求

- [Node.js](https://nodejs.org) >= 18
- [pnpm](https://pnpm.io) >= 8
- [Rust](https://www.rust-lang.org) >= 1.77（含 `cargo`）
- Tauri 系统依赖（见 [官方文档](https://tauri.app/start/prerequisites/)）

## 快速开始

```bash
# 安装依赖
pnpm install

# 启动开发模式（同时启动前端与 Tauri 窗口）
pnpm tauri dev

# 构建生产包
pnpm tauri build
```

## 项目结构

```
rhythm-editor/
├── src/              # Vue 前端源码
├── src-tauri/        # Rust 后端源码
│   ├── src/
│   │   ├── main.rs   # 程序入口
│   │   └── lib.rs    # 核心逻辑
│   └── Cargo.toml
├── public/           # 静态资源
├── index.html
└── vite.config.ts
```

## 推荐开发工具

- [VS Code](https://code.visualstudio.com/)
- [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 许可证

本项目基于 [MIT 许可证](LICENSE) 开源。
