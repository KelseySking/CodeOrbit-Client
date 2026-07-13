# CodeOrbit Client / Companion

[English](README.md)

本仓库是 [CodeOrbit](https://github.com/KelseySking/CodeOrbit) 的 **Companion 展示端**（Tauri 2 + Vue 3）：
连接电脑上已运行的 Runtime，在手机形态 UI 上处理**权限审批 / 问答**并查看**会话**。
不在手机上运行 Runtime，也不替代桌面 HUD。

CodeOrbit 主项目负责 CLI hook、会话归一化与 token 认证的 REST/WebSocket；本客户端只消费公开合同。

## 当前状态（MVP）

- 三 Tab：**待处理 · 会话 · 连接**
- 手填 Runtime URL + Token（系统凭据存储；勿把 Runtime 裸暴露公网）
- 权限：允许 / 总是允许 / 拒绝；问答：`answer-current` / 关闭
- 会话列表与详情消息；可选移除会话
- WebSocket `/api/events` 触发 REST 刷新；失败时顶栏手动刷新仍可用
- 视觉：奶白主题（`design-source/` 对齐 Open Design）

**主路径不含**：本地 Runtime 启停、Hook/Assets 修复、激活终端。

### 已知限制

| 项 | 说明 |
|---|---|
| Android 真机构建 | 需本机 `ANDROID_HOME` / SDK；当前以桌面 `tauri dev` + 窄屏布局验证 |
| Token | 走 OS keyring；浏览器纯 `npm run dev` 时 keyring/invoke 受限，优先 `cargo tauri dev` |
| 实时 | WS 断线指数退避重连；无独立轮询（可手动刷新） |
| 联调前提 | 局域网可达 Runtime（默认示意 `http://<PC-IP>:32145`）+ 有效 Token |

### 联调

```powershell
npm install
cargo tauri dev
# 连接页填写 Runtime 地址与 Token → 保存并连接
```

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
