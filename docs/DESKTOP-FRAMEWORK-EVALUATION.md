# midSunday — Windows Desktop Framework Evaluation

**Date:** 2026-11-24  
**Status:** Desk evaluation complete; Tauri 2 selected as the prototype direction only. Production stack approval remains pending prototype measurements and owner review.  
**Scope:** Windows-first desktop shell for a local-first AI companion.

## 1. Executive recommendation

Use **Tauri 2 + Rust + React + TypeScript + Vite** for the first Windows desktop-shell prototype.

This is a provisional engineering recommendation, not a final production lock. It is supported by:
- midSunday's need for a small native Rust core to own hardware detection, local storage, model lifecycle, and permission-checked desktop actions;
- Tauri's explicit Rust core + operating-system WebView process model;
- granular Tauri capabilities for limiting what the frontend can invoke;
- FRIDAY's existing Tauri 2 / React / TypeScript / Rust implementation as a read-only engineering reference;
- avoiding a second bundled Chromium + Node.js runtime where Windows WebView2 is already available on supported Windows versions.

Electron remains a valid fallback if the prototype reveals material issues with WebView2 rendering, accessibility, native integration, packaging, or maintenance.

## 2. Comparison against midSunday requirements

| Requirement | Tauri 2 | Electron | midSunday assessment |
|---|---|---|---|
| Native backend | Rust core | Node.js main process; native addons possible | Tauri aligns directly with the proposed Rust hardware and permission layers |
| UI technology | Web frontend using OS WebView | Web frontend using bundled Chromium | Both support React/TypeScript |
| Windows rendering | Microsoft Edge WebView2 | Chromium bundled with Electron | Electron is more consistent across machines; Tauri relies on installed/system WebView2 |
| Installer/runtime | Small app bundle; WebView2 is system dependency or installer-managed | Ships Electron runtime, Chromium and Node.js with app | Tauri likely reduces app-specific package payload; measure real installer |
| Idle RAM | Uses WebView2 process; actual cost depends on app and OS | Chromium renderer plus Electron main/utility processes | No universal winner can be claimed; benchmark both if prototype concerns arise |
| Rust integration | Native first-class core | Usually a separate native addon/sidecar or service boundary | Tauri is simpler for the planned Rust subsystem |
| Security model | Explicit capabilities/permissions for frontend commands; Rust core boundary | Strong security possible with sandbox, context isolation, preload bridge, IPC validation and fuses | Both require disciplined design; neither makes unsafe app logic safe automatically |
| Offline operation | App can run locally; WebView2 installation/update handling matters | Runtime is bundled; app can run offline after installation | Both can support offline use |
| Updates | Tauri updater can be configured; signing and update infrastructure need a later decision | Electron has an updater ecosystem; signing and update infrastructure need a later decision | Do not choose an update provider or incur costs yet |
| Accessibility | Depends on WebView2 and frontend implementation | Depends on bundled Chromium and frontend implementation | Must test keyboard, screen reader, scaling, IME, and Urdu shaping |
| Maintenance | Rust + frontend ecosystem; system WebView maintained by Windows | Electron + Node + Chromium update cadence | Both require security updates; compare release burden in prototype |
| Cross-platform path | Windows, macOS, Linux; webview varies by OS | Windows, macOS, Linux with bundled Chromium | Both support later desktop expansion |

## 3. Important technical facts

### 3.1 Tauri 2 on Windows

- Tauri uses Microsoft Edge WebView2 to render the frontend.
- Windows 10 version 1803 and later, and Windows 11, include WebView2 as part of the OS distribution.
- Windows development requires Microsoft C++ Build Tools, WebView2, Rust, and the frontend toolchain. These are developer prerequisites, not all end-user installation requirements.
- Tauri's default Windows installer can download the WebView2 bootstrapper when needed. This keeps the app installer smaller but requires internet during installation if WebView2 is absent.
- Tauri documents an offline WebView2 installer option adding approximately 127 MB. A fixed WebView2 runtime adds approximately 180 MB and transfers runtime update responsibility to the application distributor.
- Therefore, an offline-first *application* does not automatically mean a small, fully offline *first-time installer*. We must explain the requirement and choose packaging after testing.

### 3.2 Tauri security boundary

Tauri capabilities define which permissions are granted to which windows/webviews. The frontend should receive only narrow, typed commands needed for its UI. Sensitive actions, model access, filesystem access, memory operations, and hardware management remain in Rust modules with explicit validation. Capabilities are not a substitute for application-level authorization or user confirmation.

### 3.3 Electron on Windows

- Electron packages the application with its Electron runtime, Chromium and Node.js.
- Electron's security guidance requires keeping Node integration away from remote content, enabling context isolation and renderer sandboxing, using a restrictive Content Security Policy, validating IPC senders, limiting navigation/window creation, and keeping Electron current.
- These controls make secure Electron applications possible, but they create a security configuration and maintenance checklist that must be continuously enforced.
- Electron's consistent bundled Chromium is a real advantage when identical rendering behaviour across machines is more important than the package footprint or direct Rust integration.

## 4. Performance and package-size caution

Do not use framework marketing claims as midSunday performance guarantees.

A third-party October 2026 same-machine comparison reported a much smaller installer for its Tauri test app than its Electron test app, while idle memory was much closer (142 MB vs 149 MB in that particular test). These figures are from one small sample app and one Windows 11 machine, not a benchmark of midSunday. They demonstrate why installer size and runtime memory must be treated as separate measurements.

Our own prototype should measure:
- installer and installed size, with WebView2 already present and absent;
- cold startup and warm startup;
- idle RAM after a defined settling period;
- CPU use at idle and during UI interaction;
- memory with chat history, streaming, settings, and character UI open;
- keyboard navigation, screen-reader compatibility, text scaling, and Urdu/English rendering;
- Windows update/WebView2 availability behaviour;
- clean install, upgrade, uninstall, and recovery;
- crash behaviour and logs.

Do not compare raw process totals without documenting which WebView2/Electron child processes and shared OS components are counted.

## 5. Proposed prototype scope

The first prototype should be intentionally small:
1. One native application window.
2. React + TypeScript + Vite frontend.
3. Rust core with one harmless health/status command.
4. A typed, narrow frontend-to-Rust command boundary.
5. A minimal capability allow-list.
6. Local settings placeholder only; no personal chat data yet.
7. A hardware inventory screen stub, not a complete detector.
8. Build and run on a supported Windows machine.
9. Record installer size, installed size, startup, idle memory, CPU, and basic accessibility observations.

No LLM, speech model, model download, cloud API, telemetry, paid service, auto-updater, or sensitive PC action is needed for this prototype.

## 6. Prototype acceptance criteria

- Clean checkout can install dependencies and build with documented steps.
- App launches on the selected Windows test machine.
- Rust core and frontend communicate only through the defined command boundary.
- Frontend cannot invoke arbitrary shell commands or unrestricted filesystem operations.
- No network request is made by the prototype at runtime unless explicitly required and documented.
- App can launch and display its shell without an internet connection once prerequisites are installed.
- Startup, idle resource use, package size, and errors are recorded with test conditions.
- Keyboard navigation, scaling, and basic Urdu/English text rendering are checked.
- No FRIDAY files are modified; reference material is read-only.
- No paid service or asset is introduced.

## 7. Decision status and fallback conditions

**Prototype direction:** Tauri 2 + Rust + React + TypeScript + Vite.

**Not yet approved:** Production framework lock, minimum supported Windows version, installer WebView2 mode, update mechanism, code-signing purchase, or release packaging.

Reconsider Electron if the prototype identifies a concrete Tauri/WebView2 blocker, such as unacceptable accessibility/rendering inconsistency, critical integration limitation, or maintenance risk. Do not switch solely because of generic internet comparisons.

## 8. Sources

Official:
- Tauri Windows prerequisites: https://v2.tauri.app/start/prerequisites/
- Tauri Windows installer and WebView2 modes: https://v2.tauri.app/distribute/windows-installer/
- Tauri process model: https://v2.tauri.app/concept/process-model/
- Tauri capabilities: https://v2.tauri.app/security/capabilities/
- Electron process model: https://www.electronjs.org/docs/latest/tutorial/process-model
- Electron security checklist: https://www.electronjs.org/docs/latest/tutorial/security/
- Electron distribution overview: https://www.electronjs.org/docs/latest/tutorial/distribution-overview/

Supplementary third-party measurement (directional only):
- Dawloom, Tauri vs Electron same-machine comparison, published 23 October 2026: https://dawloom.com/blog/tauri-vs-electron-vs-wpf-benchmark/

**Conclusion:** Tauri is the best-aligned first prototype direction for midSunday's Rust-centric, Windows-first, offline-capable architecture. The actual application build and a measured Windows prototype—not desk research—must validate this recommendation.
