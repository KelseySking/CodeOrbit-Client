# CodeOrbit Client

[简体中文](README_CN.md)

CodeOrbit Client is a lightweight desktop management UI for
[CodeOrbit](https://github.com/KelseySking/CodeOrbit). It is built as a Tauri
application and is intended to provide a simple visual entry point for managing
and observing the CodeOrbit runtime from the desktop.

CodeOrbit itself is the central runtime: it receives CLI hook events, normalizes
sessions, approvals, and questions, then exposes token-authenticated REST and
WebSocket APIs for display clients. This repository is one of those clients.

## Status

This repository is currently an initial Tauri + Vue client shell. The goal is to
grow it into a practical visual management surface for CodeOrbit without mixing
runtime internals into the UI layer.

## Tech Stack

- [Tauri](https://tauri.app/) 2
- Vue 3
- TypeScript
- Vite
- Rust

Thanks to the Tauri project for providing a small, fast, cross-platform desktop
application framework that fits CodeOrbit Client's local-first workflow.

## Development

Install frontend dependencies:

```powershell
npm install
```

Install the Tauri CLI:

```powershell
cargo install tauri-cli --version "^2.0.0" --locked
```

Start the development app:

```powershell
cargo tauri dev
```

The Tauri config runs the Vite dev server automatically through `npm run dev`.

## Related Project

To understand the runtime model, API contract, plugin system, and display-client
integration patterns, read the main CodeOrbit repository:

- CodeOrbit: https://github.com/KelseySking/CodeOrbit
- API and display-client docs: https://github.com/KelseySking/CodeOrbit/tree/main/docs

If you are building a new display or management UI, CodeOrbit should remain the
runtime source of truth. Client applications should consume its public
REST/WebSocket contract instead of depending on internal runtime modules.

## License

Follow the license of this repository. For runtime details, see the
[CodeOrbit license](https://github.com/KelseySking/CodeOrbit/blob/main/LICENSE).
