# midSunday — Offline-First Requirements

**Status:** Product requirement; implementation details remain subject to validation.

## Purpose
Offline operation is a defining product requirement. A user must be able to use the core companion without creating an account, connecting to a hosted service, or paying for inference.

## Offline capability levels
- **Offline core:** App launch, settings, installed local model chat, local conversation history, local memory controls, and installed local speech assets.
- **Offline after setup:** Model/voice downloads and licence metadata retrieval may require an initial connection. Once assets are installed and verified, the relevant feature must work offline.
- **Online-only optional features:** Web/Google/YouTube search, cloud inference, remote sync, and update checks.
- **Not promised offline:** Any feature whose provider or data source inherently requires network access.

## Requirements
1. The app must not require login or network access to launch or use installed local features.
2. Network calls must be attributable to a documented feature and visible to the user.
3. No chat, audio, file, or memory content may be transmitted externally by default.
4. Strict offline mode must block optional network adapters, not merely hide their UI.
5. Model and voice assets must include source, version, file size, checksum, and licence information.
6. Downloads must be resumable where practical and verified before use.
7. The app must clearly distinguish installed assets from assets requiring download.
8. Offline errors must explain which feature is unavailable and why.
9. Update checks, telemetry, crash reporting, analytics, and remote configuration must be documented and user-controllable; default to no content collection.
10. BYOK/cloud providers are optional integrations. Users must explicitly configure them and understand data transfer and provider charges.

## Offline acceptance tests
- Disconnect the network before launch; app opens.
- With a supported model already installed, a user can chat, stop generation, and revisit local history.
- Local memory can be inspected, edited, disabled, exported, and deleted.
- Installed local speech assets work without network access.
- Network monitoring confirms no unexpected outbound requests during local-only use.
- Online-only features fail gracefully and do not block offline core functions.

## Design constraints
- Keep network access behind explicit service adapters.
- Avoid hidden background requests and automatic cloud fallback.
- Never silently send a local prompt to a cloud provider because a local model fails.
- Do not make model downloads mandatory during every launch.
- Document any unavoidable online requirement before the user enables the relevant feature.

## Open questions
- Should strict offline mode be a first-run choice, a global switch, or both?
- Should optional update checks be disabled by default?
- What is the safest and simplest model download/verification process for non-technical users?
