# midSunday desktop shell prototype

This is the first technical shell prototype, not the production application.

## Scope

- Tauri 2 desktop window.
- React + TypeScript + Vite frontend.
- Rust core with one harmless status command.
- Narrow default Tauri capability (core:default).
- No model runtime, model download, speech engine, cloud API, telemetry, local conversation storage, or privileged PC actions.

## Prerequisites

For Windows development, install:
- Node.js LTS and npm.
- Rust using rustup with the MSVC toolchain.
- Microsoft C++ Build Tools with the Desktop development with C++ workload.
- Microsoft Edge WebView2 Runtime.

WebView2 is included with supported modern Windows installations (Windows 10 version 1803 and later, and Windows 11). If it is absent, the eventual installer must handle that dependency deliberately. See the desktop framework evaluation in ../../docs/DESKTOP-FRAMEWORK-EVALUATION.md.

## Run in development

From this directory:

```powershell
npm install
npm run tauri dev
```

## Build a Windows installer

From this directory:

```powershell
npm install
npm run tauri build
```

The NSIS installer output is generated under src-tauri/target/release/bundle/nsis/.

## Current limitations

- This prototype has not yet been built or run on a Windows machine.
- The UI is a technical placeholder, not an approved product design.
- The Rust status command is the only backend command.
- No performance, accessibility, installer, or offline-install claim has been validated.
- Do not add unrestricted filesystem, shell, network, model, microphone, or desktop-control permissions without a reviewed design and explicit need.

See [WINDOWS-BUILD.md](WINDOWS-BUILD.md) for the manual GitHub-hosted Windows build workflow and artifact retrieval steps.
