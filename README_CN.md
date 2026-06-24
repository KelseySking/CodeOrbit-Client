# CodeOrbit Client

[English](README.md)

CodeOrbit Client 是
[CodeOrbit](https://github.com/KelseySking/CodeOrbit) 的轻量桌面管理界面。它基于
Tauri 构建，目标是为 CodeOrbit 提供一个简单、可视化、本地优先的桌面入口，用于管理和观察
CodeOrbit Runtime。

CodeOrbit 主项目负责接入 CLI hook 事件，归一化会话、审批和问答状态，并通过带 token
认证的 REST/WebSocket 接口把状态提供给多个展示端。本仓库就是面向桌面的一类展示/管理客户端。

## 当前状态

当前仓库是一个初始化阶段的 Tauri + Vue 客户端外壳。后续会围绕 CodeOrbit 的公开接口逐步补齐可视化管理能力，
UI 层不直接依赖 Runtime 内部实现。

## 技术栈

- [Tauri](https://tauri.app/) 2
- Vue 3
- TypeScript
- Vite
- Rust

感谢 Tauri 项目提供轻量、快速、跨平台的桌面应用框架，它非常适合 CodeOrbit Client 这种本地优先的管理工具。

## 开发调试

安装前端依赖：

```powershell
npm install
```

安装 Tauri CLI：

```powershell
cargo install tauri-cli --version "^2.0.0" --locked
```

启动开发环境：

```powershell
cargo tauri dev
```

Tauri 配置会通过 `npm run dev` 自动启动 Vite 开发服务器。

## 关联项目

想了解 Runtime 模型、API 合同、插件系统和展示端接入方式，请查看 CodeOrbit 主仓库：

- CodeOrbit：https://github.com/KelseySking/CodeOrbit
- API 和展示端文档：https://github.com/KelseySking/CodeOrbit/tree/main/docs

如果你正在开发新的展示端或管理界面，CodeOrbit 应该继续作为 Runtime 的状态源。客户端应该消费它公开的
REST/WebSocket 合同，而不是依赖 Runtime 内部模块。

## 许可证

请以本仓库的许可证为准。Runtime 相关内容可参考
[CodeOrbit 许可证](https://github.com/KelseySking/CodeOrbit/blob/main/LICENSE)。
