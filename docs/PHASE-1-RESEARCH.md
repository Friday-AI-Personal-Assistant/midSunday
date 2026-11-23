# Phase 1 — Initial Feasibility Research

**Research checkpoint:** 2026-11-23  
**Status:** In progress — desk research only; no local benchmark has been run.  
**Purpose:** Record evidence and identify what must be tested before the technology stack is approved.

## Executive summary

The initial research supports keeping Tauri 2, llama.cpp, compact Qwen models, whisper.cpp, Kokoro, and SQLite on the candidate list. It does **not** justify locking the stack or claiming Urdu voice support, 4 GB compatibility, or production readiness.

The most important early finding is that Pakistani Urdu speech is a distinct validation challenge. A model or engine advertising multilingual support is not proof of acceptable Pakistani Urdu, Roman Urdu, or Urdu-English code-switching quality.

## 1. Desktop framework

### Tauri 2 — leading candidate, not yet approved
FRIDAY already uses Tauri 2 with a React/TypeScript/Vite frontend and Rust backend. Its repository offers a useful reference for the Windows desktop shell, but FRIDAY's AI and major assistant features are not implemented.

Official Tauri documentation confirms Windows development requires Microsoft C++ Build Tools and WebView2. On Windows 10 (1803+) and Windows 11, WebView2 is distributed with the OS. Tauri's default installer can download a WebView2 bootstrapper; an offline installer option adds approximately 127 MB to the installer. This is relevant to our offline installation promise: app operation can be offline, but first-time installation requirements must be clearly handled.

Sources:
- https://v2.tauri.app/start/prerequisites/
- https://v2.tauri.app/distribute/windows-installer/
- https://github.com/tauri-apps/tauri

### Electron — comparison still required
Electron remains a legitimate alternative and has mature process, security, and distribution documentation. We have not yet run a fair packaged-size or memory comparison. Do not reject it solely on general claims about bundle size.

Source: https://www.electronjs.org/docs/latest/

**Current view:** Continue with Tauri as the leading candidate because it aligns with FRIDAY's existing engineering reference and Rust native-backend direction. Final approval requires a small Windows prototype and comparison of packaged size, idle memory, accessibility, update strategy, and developer/maintenance complexity.

## 2. Local language model runtime

### llama.cpp — strong candidate
The upstream project is an actively developed C/C++ inference runtime focused on local inference across varied hardware. It supports CPU and multiple acceleration backends, quantisation options, and an MIT-licensed runtime repository.

Runtime licensing does not grant rights to every model distributed in GGUF format. Each model's exact weights, conversion, quantisation source, model card, and terms must be reviewed independently.

Sources:
- https://github.com/ggml-org/llama.cpp
- https://github.com/ggml-org/llama.cpp/blob/master/LICENSE

### Qwen3-0.6B — candidate for constrained hardware testing
The official Qwen3-0.6B model card lists Apache 2.0 and describes support for 100+ languages. It is a 0.6-billion-parameter model with a listed context length of 32,768 tokens.

This does not prove that it is useful for natural companion conversations, Pakistani Urdu, tool use, or 4 GB computers. A small parameter count and model file do not equal total runtime RAM. Test the exact instruction/chat checkpoint and quantised GGUF build.

Source:
- https://huggingface.co/Qwen/Qwen3-0.6B

**Next model comparisons:** include at least one compact model above 0.6B if hardware permits, and evaluate conversational quality, role/personality consistency, English, Urdu script, Roman Urdu, mixed-language messages, hallucinations, and response latency.

## 3. Speech recognition (STT)

### whisper.cpp — leading multilingual candidate for evaluation
whisper.cpp provides a C/C++ implementation of Whisper, supports CPU-only inference, Windows, quantisation, and multiple acceleration backends. Its documentation lists model memory estimates that show why model selection matters: the tiny model is listed around 273 MB runtime memory, base around 388 MB, small around 852 MB, medium around 2.1 GB, and large around 3.9 GB. These are upstream estimates, not midSunday benchmarks.

Whisper's official model card documents multilingual speech recognition but warns that performance varies by language and accent and that low-resource languages can have lower accuracy. Urdu and Pakistani accents therefore require a dedicated evaluation set.

Sources:
- https://github.com/ggml-org/whisper.cpp
- https://github.com/ggml-org/whisper.cpp/blob/master/models/README.md
- https://github.com/openai/whisper/blob/main/model-card.md

### Vosk — useful lightweight comparison, but no Urdu model found in its published list
Vosk's official model page describes small models around 50 MB on disk and roughly 300 MB runtime memory as a typical class. The currently published language list includes English and several other languages, but the reviewed list did not show an Urdu model. It should not be selected as the Urdu solution without finding and validating a suitable model.

Source: https://alphacephei.com/vosk/models

**Current view:** Evaluate whisper.cpp first for Urdu and multilingual STT. Vosk may still be useful for a lightweight English option if its accuracy, maintenance, and exact model licence meet requirements.

## 4. Speech synthesis (TTS)

### Kokoro 82M — promising English candidate, not an Urdu solution yet
The official Hugging Face model card lists Apache 2.0 weights and describes an 82-million-parameter model. Its published voice/language catalogue must be checked for the exact version being considered. The reviewed model is positioned as multilingual in later releases, but the sources reviewed do not establish a Pakistani Urdu voice.

Source:
- https://huggingface.co/hexgrad/Kokoro-82M

### Piper — treat runtime and each voice separately
The current OHF-Voice Piper repository states that the engine is GPL-3.0-or-later. Its licensing guidance is unusually important: Piper does not grant a blanket licence over voice models; rights depend on each voice's training data and training lineage. Some common voice families have non-commercial or otherwise restrictive ancestry. A voice's own model card and upstream training chain must be reviewed before redistribution.

Sources:
- https://github.com/OHF-Voice/piper1-gpl
- https://github.com/OHF-Voice/piper1-gpl/blob/main/docs/LICENSING.md
- https://github.com/OHF-Voice/piper1-gpl/blob/main/docs/COMMERCIAL_USE.md

**Current view:** Kokoro is an English TTS candidate. Piper is not automatically rejected, but its GPL runtime and voice-by-voice rights need careful evaluation. Neither is yet confirmed for Pakistani Urdu. We may need a separate Urdu TTS candidate or a future custom voice project, subject to rights and costs.

## 5. Local database

SQLite's core library is dedicated to the public domain and can be used and distributed commercially. It is a strong candidate for local conversations, settings, and user-controlled memory. Any third-party extensions or encryption add-ons require separate review.

Sources:
- https://www.sqlite.org/copyright.html
- https://www.sqlite.org/about.html

Database encryption and key management remain separate architecture decisions; SQLite being public domain does not provide encryption automatically.

## 6. Preliminary candidate status

| Area | Candidate | Preliminary status | Main blocker |
|---|---|---|---|
| Desktop | Tauri 2 + Rust + React/TS | Leading candidate | Windows prototype and comparative measurements |
| Alternative desktop | Electron | Comparison candidate | Fair size, RAM, accessibility, and maintenance comparison |
| LLM runtime | llama.cpp | Leading candidate | Integration prototype and bundled dependency audit |
| Small LLM | Qwen3-0.6B | Test candidate | Real conversation, Urdu, quality, and RAM benchmarks |
| STT | whisper.cpp | Leading candidate for evaluation | Pakistani Urdu accuracy and constrained-hardware tests |
| STT | Vosk | Secondary candidate | No Urdu model found in reviewed official catalogue |
| TTS | Kokoro 82M | English candidate | Voice quality, CPU/RAM, and language coverage |
| TTS | Piper | Conditional candidate | GPL runtime and exact voice dataset/training-chain rights |
| Database | SQLite | Strong candidate | Data protection, encryption, migration, and backup design |

Nothing in this table is final approval to bundle or distribute.

## 7. Required practical tests

Desk research is not enough. The next research work must produce repeatable tests:

1. Build a minimal Tauri and Electron shell on Windows and compare installer size, idle memory, launch time, and accessibility.
2. Run exact candidate GGUF models with documented quantisation and context settings on representative 4 GB, 8 GB, and 16 GB machines where available.
3. Measure model load time, time to first token, tokens per second, peak memory, and stability.
4. Compare Whisper model sizes on CPU-only hardware with English, Urdu script, Pakistani Urdu accents, and code-switching samples.
5. Evaluate Kokoro and any alternative TTS voices for intelligibility, naturalness, CPU usage, and rights.
6. Identify at least one viable Pakistani Urdu TTS candidate or explicitly document the gap.
7. Inspect the complete licence and dependency tree for any component intended for bundling.
8. Record exact hardware, versions, hashes, settings, and test results.

Use permissioned test recordings; do not retain personal audio unnecessarily.

## 8. Current conclusions

- Tauri 2 remains the leading desktop-shell candidate, not a final decision.
- llama.cpp remains the leading local LLM runtime candidate.
- Qwen3-0.6B is worth testing, but must not be advertised as the final model or a proven 4 GB solution.
- whisper.cpp is the strongest initial STT candidate from this desk review; Urdu quality remains unproven.
- Vosk's currently published catalogue does not provide an obvious Urdu model.
- Kokoro is worth evaluating for English TTS; Urdu TTS remains an open research gap.
- Piper licensing is more complicated than simply calling the engine "free"; exact voice lineage matters.
- SQLite is a strong local storage candidate; encryption/key handling still needs design.

## 9. Next actions
- Complete a fair desktop-framework comparison.
- Inspect FRIDAY's remaining source, tests, CI, and Tauri capabilities read-only.
- Build or obtain repeatable benchmark harnesses for model and speech candidates.
- Research Pakistani Urdu TTS options and exact voice licences.
- Update the architecture, decision log, and licence register only when evidence supports a decision.
- Do not begin production application implementation until the Phase 1 exit gate is satisfied.
