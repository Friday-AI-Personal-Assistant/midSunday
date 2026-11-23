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
