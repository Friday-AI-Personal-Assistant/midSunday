# midSunday — Product Specification

**Status:** Initial draft; owner review required  
**Product descriptor:** AI Personal Companion  
**Platform priority:** Windows first

## 1. Purpose

midSunday is a desktop AI companion combining local AI assistance with a personal, configurable companion experience. It should be useful offline, respect user privacy, and avoid mandatory paid services.

## 2. Target users

Initial audience:
- People who want a private AI assistant on their own computer.
- Users who prefer offline/local AI or want control over their data.
- Users who want voice interaction and configurable assistant personalities.
- Users with modest hardware, provided performance can be validated.

The product should not assume that every user has a dedicated GPU, high-speed internet, or a paid AI subscription.

## 3. Core user outcomes

A user should eventually be able to:
1. Install and launch midSunday on a supported Windows computer.
2. Install or import a compatible local model.
3. Chat with the local assistant without an internet connection.
4. Save and manage conversations locally.
5. Choose whether memory is enabled and inspect/edit/delete stored memories.
6. Use supported local speech input and output.
7. Select or customise a personality.
8. Ask the assistant to perform supported desktop tasks through safe, permissioned tools.
9. Use online search when connected, understanding that it uses the internet.
10. Export or remove personal data.

## 4. Functional requirements

### 4.1 Application and onboarding
- Windows desktop app is the first delivery target.
- Onboarding explains local models, storage, offline limitations, and optional online services.
- Users can choose model and data storage locations where supported.
- The app must clearly distinguish installed assets from assets that still need downloading.

### 4.2 Local chat
- Support local model selection and loading.
- Stream generated responses.
- Allow stop, retry, and conversation management.
- Display model loading, failure, and resource states.
- Provide configurable context limits appropriate to the machine.
- Work offline once required model assets are installed.

### 4.3 Memory and history
- Store chat history locally.
- Treat chat history and long-term memory as separate concepts.
- Allow users to inspect, edit, delete, export, and disable memory.
- Explain what is stored and where.
- No cloud sync or remote memory transfer by default.

### 4.4 Voice
- Support local speech recognition and synthesis where compatible assets are installed.
- Provide clear microphone permission and active-microphone indicators.
- Begin with push-to-talk; hands-free activation is a later feature requiring privacy safeguards.
- Make supported languages and known limitations visible.
- Urdu, Pakistani accents, Roman Urdu, and Urdu-English code-switching are priority test cases.

### 4.5 Personalities and characters
- Provide configurable personality profiles.
- Initial concepts: best friend, mentor, professional assistant, girlfriend, boyfriend, and custom character.
- Separate persona configuration from core application code.
- Allow user control over tone, warmth, humour, formality, and response length.
- Character presentation should begin with a lightweight 2D approach as a proposal; 3D is a later evaluation.
- Characters must be transparent AI entities and follow documented safety boundaries.

### 4.6 Desktop assistance
- Expose only explicit, allow-listed actions.
- Require user confirmation for sensitive, destructive, external, or privacy-impacting actions.
- Do not allow arbitrary model-generated shell commands.
- Scope file access to locations explicitly granted by the user.
- Provide previews and recovery options for file modifications.

### 4.7 Online capabilities
- Web, Google, and YouTube search require internet.
- Optional cloud model providers/BYOK may be supported later.
- Clearly show when a request is being sent to an external provider.
- No paid provider or external API may be enabled by default without owner approval.

### 4.8 Settings and control
- Provide settings for model, language, voice, memory, privacy, storage, performance, and permissions.
- Provide controls to disable optional features.
- Make offline/online status visible.

## 5. Non-functional requirements

- **Privacy:** Local-first data handling and clear consent for external transfers.
- **Reliability:** Recover from model failures, interrupted downloads, and application restarts.
- **Performance:** Provide documented profiles based on benchmark results, not assumptions.
- **Accessibility:** Keyboard navigation, scalable UI, readable contrast, and accessible controls.
- **Maintainability:** Modular adapters for model and speech engines.
- **Security:** Typed and validated frontend/backend boundaries; least privilege for tools.
- **Transparency:** Explain model, voice, network, permission, and data-storage states.
- **Cost:** No mandatory subscription or hidden usage charges.
- **Licensing:** All distributed dependencies and assets must be cleared for the intended distribution model.

## 6. Hardware expectations

4 GB RAM is a constrained target to investigate, not a confirmed minimum. Test total application memory, model runtime, context cache, speech engines, and operating-system overhead together. Publish separate minimum and recommended configurations only after repeatable testing.

## 7. Out of scope for the first prototype

Unless separately approved, the first prototype will not include:
- Always-listening wake-word operation.
- Remote phone control or remote access.
- Cloud-hosted memory or mandatory account login.
- Paid search APIs or paid inference by default.
- Arbitrary shell/terminal execution.
- Full 3D animated characters.
- Linux/macOS production support.
- Automatic background recording or data collection.

## 8. Success criteria

The first meaningful prototype is successful when a user can:
- Launch the Windows app.
- Load a locally installed, licence-reviewed model.
- Chat while fully offline.
- Stop generation and recover from an error.
- Keep conversations locally.
- Confirm that no conversation is sent to a cloud provider by default.

Later release success criteria are defined by the relevant roadmap phase.

## 9. Open decisions

- Final desktop framework and app architecture.
- Default local model(s) and supported model catalogue.
- Minimum/recommended hardware.
- Urdu STT/TTS engine and quality threshold.
- Memory model: explicit-only initially vs optional assisted suggestions.
- Character visual style and asset pipeline.
- Product licence and commercial distribution model.
- Update and code-signing strategy.


## 10. Scalable hardware experience (owner-confirmed direction)

midSunday must serve a range of computers rather than being optimised only for 4 GB RAM. The intended direction is to scale local model capability, context length, speech quality, and concurrency upward with available resources.

Planning classes include 4 GB, 6 GB, 8 GB, 16 GB, 32 GB, and 64 GB+ systems. These are not certified minimum/recommended specifications and must not be used as compatibility promises before testing.

The application should:
- Detect available system resources and account for OS/background usage, not just installed RAM.
- Consider CPU, GPU/VRAM, storage, thermals, and power in addition to RAM.
- Provide understandable Automatic, Lightweight, Balanced, and High capability modes.
- Recommend suitable models while allowing manual choice.
- Explain model download size and expected resource needs before download.
- Avoid forcing powerful computers to use the smallest model.
- Avoid forcing constrained computers to load a model that will cause instability.
- Permit users on powerful machines to choose smaller/faster models when preferred.
- Schedule model, STT, and TTS workloads based on measured capability.

All profile mappings and model recommendations remain subject to repeatable benchmarks.
