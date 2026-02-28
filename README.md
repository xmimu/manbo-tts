# 曼波语音生成器 (Manbo TTS)

基于 [Tauri](https://tauri.app) + [Vue 3](https://vuejs.org) + [TypeScript](https://www.typescriptlang.org) 构建的桌面端语音生成应用。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## 功能特性

- 文本转语音（TTS）生成
- 跨平台桌面应用（Windows / macOS / Linux）
- 轻量、快速，基于 Rust 后端

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
