# midSunday — FRIDAY Reuse Analysis

**Review date:** 2026-11-23  
**Review type:** Read-only repository inspection  
**Repositories:** `Friday-AI-Personal-Assistant/FRIDAY` and `Friday-AI-Personal-Assistant/midSunday`

## Executive summary
FRIDAY is an early-stage Tauri 2 desktop scaffold with a React/TypeScript/Vite frontend and Rust backend. Its README explicitly states that AI, accounts, email integrations, persistent workspace storage, and job features are not active. It is useful as a reference for desktop project structure and selected security-minded patterns, but it is not a completed AI engine and should not be copied wholesale.

## Files inspected in FRIDAY
- `README.md`
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src-tauri/src/lib.rs`
- `src-tauri/src/main.rs`

This was a limited read-only review of these files, not a full source-code, security, or licence audit.

## Observed technical patterns
- Tauri 2 desktop shell.
- React 18, TypeScript, Vite, and Tauri JavaScript API.
- Rust native backend.
- NSIS and MSI Windows bundle targets.
- A restrictive-looking Content Security Policy in the Tauri configuration.
- Rust command registration through Tauri's invoke handler.
- Dependencies include `chacha20poly1305`, `rand`, and `keyring` with Windows-native support.
- README describes an encrypted-note test using a key stored in Windows Credential Manager.

## Reuse candidates
These patterns may be useful to study or independently implement:
- Tauri 2 + React/TypeScript/Vite integration and development workflow.
- Rust command boundary between UI and native capabilities.
- Windows installer configuration.
- Restrictive CSP as a starting point for a reviewed security policy.
- OS credential-store approach for optional secrets, after platform-specific review.
- Documentation style and separation of privacy/security principles.

## Important limitations
- FRIDAY's encrypted-note test is explicitly not a complete workspace/database, backup, or recovery system.
- The README says the feature has not received an independent security audit.
- AI inference, model management, STT/TTS, and a production memory system were not shown as implemented in the inspected files.
- The inspected package manifests do not establish that all dependencies are currently secure, maintained, or commercially suitable.
- A configured CSP or use of a credential store does not by itself prove the application is secure.

## Independence and reuse rules
1. Keep separate repositories, application identifiers, branding, product decisions, release processes, and issue tracking.
2. Do not modify FRIDAY as part of midSunday work without explicit approval.
3. Prefer documenting patterns and implementing a clean midSunday equivalent over copying code.
4. Before copying any source, inspect its exact licence, authorship, dependencies, and security implications.
5. Avoid shared runtime dependencies or release coupling unless a later architecture decision explicitly justifies them.
6. Revisit this analysis after a deeper, approved read-only code review.

## Recommended next FRIDAY audit
If the owner approves a deeper review, inspect:
- Full frontend component and state structure.
- Rust workspace commands and permission configuration.
- Tauri capabilities and plugin permissions.
- Encryption implementation and key lifecycle.
- Tests, CI workflows, dependency lockfiles, and security policies.
- Existing product architecture and roadmap documents.

## Conclusion
FRIDAY provides a useful architectural reference, especially for the Windows/Tauri/Rust foundation. It does not remove the need to design and validate midSunday's own offline inference, voice, memory, safety, and licensing architecture.


## 2026-11-23 — Expanded read-only source review

The following additional FRIDAY paths were inspected:
- Frontend: `src/App.tsx`, `src/main.tsx`, `src/styles.css`.
- Native permissions: `src-tauri/capabilities/default.json`.
- Workspace implementation: `src-tauri/src/workspace.rs`.
- Product/security docs: `docs/PRODUCT-PLAN.md`, `docs/LOCAL-WORKSPACE-DESIGN.md`, `docs/PRIVACY-AND-SECURITY.md`, `docs/DISTRIBUTION-PLAN.md`.
- CI: `.github/workflows/build-windows.yml`, `.github/workflows/windows-build.yml`, `.github/workflows/deploy-preview.yml`.
- Root `.gitignore` and directory listings.

### Additional observations

**Frontend**
- The current React app is a single main component with local page state and a small preview workspace interaction.
- It uses Tauri's `invoke` API to call three narrow workspace commands.
- The current UI is a product preview/scaffold, not a completed modular application architecture.
- A browser-preview detection branch exists; desktop-specific calls are guarded by checking for Tauri internals.

**Native backend and permissions**
- Tauri capabilities currently grant only `core:default` to the main window.
- Rust registers `initialize_workspace`, `save_private_note`, and `read_private_note`.
- The workspace code uses the Windows credential store via the `keyring` crate, OS randomness, and XChaCha20-Poly1305 for one fixed encrypted note.
- The note path is fixed within the application data workspace; the frontend is not given arbitrary filesystem or shell commands.
- Inline unit tests cover an encryption/decryption round trip and tamper rejection. They do not establish a complete workspace security audit.

**Documentation**
- FRIDAY's local workspace design provides useful principles: per-user app-data storage, OS-protected keys, explicit external data flows, least privilege, parameterised database access, and honest recovery limitations.
- Its product and distribution plans emphasise keeping account/licence services separate from private local workspaces. These principles are relevant as patterns, not requirements to copy verbatim.

**CI and repository hygiene**
- The repository has multiple Windows build workflow files plus a browser-preview deployment workflow.
- The inspected Windows workflow content appears to contain duplicated icon-generation steps; workflow consolidation/cleanup may be worth considering in FRIDAY's own roadmap, but midSunday must not change FRIDAY.
- The root listing did not show a package-lock.json or Cargo.lock. Reproducible dependency pinning should be reviewed before relying on these workflows for release builds.
- `.gitignore` excludes build outputs, environment files, common secret/key files, local databases, logs, and editor files.

### Reuse recommendations for midSunday

1. Reuse the *architectural idea* of a narrow Rust command boundary and minimal Tauri capabilities.
2. Use a dedicated model-process adapter and speech adapters; do not place inference logic in the React UI.
3. Keep a permission broker between model output and every desktop action.
4. Adopt per-user app-data storage and OS credential storage where appropriate, but design a separate midSunday data/key lifecycle.
5. Use FRIDAY's privacy and distribution documents as references, then write midSunday-specific policies.
6. Establish one clear CI pipeline per purpose and pinned dependency lockfiles from the beginning.
7. Keep all midSunday identifiers, package names, app data paths, keyring service names, signing identities, and release artifacts distinct from FRIDAY.

**Scope limitation:** This was a read-only review of the listed files and directory listings. No FRIDAY file was changed, no build was run, and no independent security review was performed.
