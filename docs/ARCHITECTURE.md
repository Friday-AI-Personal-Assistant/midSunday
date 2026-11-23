# midSunday — Architecture (Proposed)

**Status:** Preliminary architecture proposal; subject to feasibility research and owner approval.

## 1. Architecture goals

- Keep core functionality local and offline-capable.
- Separate UI, application logic, model inference, speech, storage, and tool permissions.
- Make model and voice engines replaceable.
- Minimise resource use on modest hardware.
- Keep network access explicit and optional.
- Avoid unnecessary services and paid infrastructure.

## 2. Proposed high-level stack

| Layer | Candidate | Status |
|---|---|---|
| Desktop shell | Tauri 2 | Proposed |
| Native/backend logic | Rust | Proposed |
| UI | React + TypeScript + Vite | Proposed |
| Local LLM runtime | llama.cpp | Candidate |
| Model format | Quantised GGUF | Candidate |
| Speech recognition | Vosk / whisper.cpp | Candidates |
| Speech synthesis | Piper / Kokoro | Candidates |
| Local database | SQLite | Candidate |

No dependency is final until current licensing, maintenance, security, and performance are reviewed.

## 3. Logical components

### Desktop shell
Owns the native window, application lifecycle, OS integration, permissions, and packaging.

### User interface
Owns chat presentation, onboarding, settings, character presentation, and user confirmation screens. It must not directly execute privileged OS operations.

### Application coordinator
Coordinates conversations, model selection, context preparation, cancellation, and tool requests.

### Model adapter
Defines a stable interface for loading/unloading a model and generating responses. The first implementation may use one local runtime, but the app should not bind product logic to a single model vendor.

### Speech adapters
Separate interfaces for speech-to-text and text-to-speech. Audio capture must be permissioned and visibly indicated.

### Local data layer
Stores settings, conversations, memory, and metadata. SQLite is the leading candidate. Data schema migrations and export/restore must be planned from the beginning.

### Tool/permission broker
The only route from model intent to desktop actions. It validates action names and arguments, checks permissions, requests confirmation when required, and records an appropriate local audit entry.

### Online service adapters
Optional adapters for web search and cloud inference. They must be disabled or unavailable when offline and must disclose what data is sent externally.

## 4. Trust boundaries

- Treat model output as untrusted.
- Treat webpage, document, and imported content as untrusted.
- The UI cannot bypass backend permissions.
- Model adapters cannot directly invoke arbitrary OS commands.
- The tool broker exposes only documented actions.
- External providers receive no conversation or memory unless the user has clearly enabled and approved that flow.
- Secrets and API keys must never be stored in source code or logs.

## 5. Resource management

- Load only the selected model.
- Unload unused engines when appropriate.
- Avoid running multiple heavy model/STT/TTS engines simultaneously on constrained devices.
- Bound context size and provide clear controls.
- Make model load/unload state visible.
- Provide graceful low-memory failure and recovery.
- Measure actual process and system memory rather than estimating from model file size.

## 6. Data organisation (conceptual)

Potential local data groups:
- Application settings.
- Conversation metadata and messages.
- User-approved memory records.
- Model and voice asset metadata.
- Tool permission settings and action history.
- Schema migration and backup metadata.

Exact schema, encryption requirements, storage paths, and backup behaviour remain open until the threat model and platform research are complete.

## 7. Offline and network policy

The app should support a strict offline mode. Local chat, local history, local memory, and installed local speech assets should not require network access. Network-dependent features must use explicit adapters and visible UI status.

Do not add background network calls, analytics, telemetry, or update checks without documenting their purpose and user controls.

## 8. Build and test strategy

- Keep frontend and Rust backend contracts typed and validated.
- Unit-test domain logic and permission decisions.
- Integration-test model and speech adapters.
- Test network-disabled operation.
- Test failure recovery and interrupted operations.
- Run performance tests on documented hardware profiles.
- Maintain dependency and licence inventories.

## 9. Open architecture decisions

- Final Tauri configuration and Rust module boundaries.
- Model process strategy: embedded library vs managed subprocess.
- Model download and update mechanism.
- SQLite schema and encryption/key management.
- Audio capture pipeline and device abstraction.
- Windows notification and startup integration.
- Secure update distribution and code signing.
- Linux/macOS abstraction boundaries.

These decisions must be resolved with prototypes and evidence, not assumed.
