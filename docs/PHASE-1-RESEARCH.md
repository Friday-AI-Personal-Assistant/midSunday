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


## 10. Additional Urdu-specific model research

A second search identified more targeted Urdu candidates. These are particularly relevant because generic multilingual claims are not enough for the product's Pakistani Urdu requirement.

### Urdu Sukhan ASR — promising quality, currently non-commercial
Loudness.ai's Urdu Sukhan ASR v1 is a Whisper large-v3-turbo fine-tune targeting Urdu, English, and Urdu-English code-switching. Its published evaluation reports improvements over the Whisper baseline on several Urdu test sets. However, the repository explicitly lists **CC BY-NC 4.0 for the code and model** and says commercial use requires contacting the developer.

**Conclusion:** Do not plan to bundle this model in a commercial midSunday release under the currently published terms. It may be useful for research comparison only if its access and evaluation terms permit that use.

Source: https://github.com/Loudness-ai/urdu-sukhan-asr

### whisper-small-ur-en — compact bilingual candidate, licence and quality need verification
A Hugging Face model card for `ubaidngt/whisper-small-ur-en` reports a bilingual Urdu-English fine-tune of Whisper small and lists Apache-2.0. Its published evaluation reports Urdu WER 29.38% versus 43.77% for the baseline on the stated held-out test set, and English WER 8.50% versus 26.89%. The author notes that the model is trained mostly on read speech and that noisy or heavily accented conversational speech may perform worse.

**Conclusion:** Worth testing as a compact Urdu-English candidate. Verify the exact weights, base model terms, training data provenance, conversion/inference format, and independent performance on Pakistani conversational speech before any product decision.

Source: https://huggingface.co/ubaidngt/whisper-small-ur-en

### Smol Urdu TTS V1 — lightweight experimental candidate
ProximaAI's Smol Urdu TTS V1 is described as an experimental Urdu TTS model based on SmolLM2-135M. Its model page lists Apache-2.0, but warns that quality may not be perfect and improvements are ongoing.

**Conclusion:** This is a promising low-resource candidate for a prototype, not a release-ready voice. Test pronunciation, prosody, latency, memory, and whether its exact code/weights and base components can be redistributed commercially.

Source: https://huggingface.co/ProximaAI/smol-urdu-tts-v1

### DeepSpeak-v1 — experimental Urdu TTS candidate
The DeepSpeak-v1 model card describes an Urdu-only experimental TTS model with approximately 400M parameters, Apache-2.0 model licensing, and an explicit dependency on a separate semantic codec model whose licence must also be checked. The card says training is incomplete and that speaker-reference audio is required for best results.

**Conclusion:** Keep on the research list, but it is not a practical default for constrained hardware until inference performance, codec licensing, voice rights, and model quality are established.

Source: https://huggingface.co/mahwizzzz/deepspeak-v1

### Dia-1.6B-Urdu — not a 4 GB first choice
A separate Urdu TTS fine-tune is listed at approximately 1.6B parameters and reports training on a 24 GB VRAM GPU. It is not a sensible first candidate for a constrained 4 GB profile without substantial quantisation and inference testing. Its base model and asset terms also require review.

Source: https://huggingface.co/mahwizzzz/Dia-1.6B-Urdu

### Commercial dataset warning
Mozilla Data Collective now lists large Pakistan-recorded Urdu TTS corpora with commercial licensing options, but some listed packages are tens of gigabytes and priced in the tens of thousands of US dollars. These are not compatible with our no-unapproved-cost principle and must not be considered without explicit owner approval.

Example: https://mozilladatacollective.com/datasets/cmw574b6z00xi06mo4dchu497

## 11. Updated Urdu voice strategy

- **STT:** Compare Whisper multilingual baseline with `whisper-small-ur-en`; keep Urdu Sukhan ASR out of commercial bundling unless the rights holder grants suitable commercial terms.
- **TTS:** Test Smol Urdu TTS V1 as the first lightweight Urdu candidate; evaluate DeepSpeak-v1 as experimental; keep Kokoro for English voice evaluation.
- **Quality:** Test Urdu script, Roman Urdu input normalisation, Pakistani regional accents, names, everyday conversational speech, and Urdu-English code-switching.
- **Cost/licensing:** Prefer candidates whose exact weights, code, and data rights allow commercial redistribution without a paid licence. Do not purchase datasets or voices without explicit approval.
- **Reality check:** No Urdu voice candidate is approved or proven on 4 GB hardware at this checkpoint.


## 12. Local runtime comparison — llama.cpp and Ollama (2026-11-23)

### llama.cpp
The upstream repository is MIT-licensed and provides local inference across CPU and several acceleration backends, quantisation options, a library/API, and an OpenAI-compatible server interface. This makes it a strong candidate for direct integration into a desktop application. It gives the engineering team control over model loading, runtime settings, packaging, and resource management.

Sources:
- https://github.com/ggml-org/llama.cpp
- https://github.com/ggml-org/llama.cpp/blob/master/LICENSE
- https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md

### Ollama
Ollama's software repository is MIT-licensed. Its local model-running workflow is free to use and local inference runs on the user's own hardware; its current pricing page separately lists paid plans/usage for cloud-hosted models. Therefore, do not confuse free local use with cloud inference being free. Ollama's repository identifies llama.cpp as a supported backend.

Sources:
- https://github.com/ollama/ollama
- https://github.com/ollama/ollama/blob/main/LICENSE
- https://ollama.com/pricing

### What this means for midSunday
- **Development convenience:** Ollama is useful for quickly downloading, switching, and testing local models through a local API.
- **Potential product integration:** llama.cpp is the leading candidate for a directly controlled, packaged local runtime, subject to Windows integration, process isolation, updates, and dependency review.
- **No need to ship both:** We should avoid adding two runtimes to the installer unless testing demonstrates a concrete benefit.
- **Cloud boundary:** Ollama cloud features are optional and may incur usage charges. They are not part of the no-cost offline core and must never be invoked silently.
- **Licensing:** Both runtime repositories are MIT-licensed at the time of this research. Model weights, quantised conversions, and other bundled assets keep their own licence obligations.

### Small-model task suitability
A compact model running through llama.cpp may be sufficient for bounded tasks such as short text classification, extracting structured fields, tagging and organising notes, simple rewriting, concise summaries, and routing a request to a small set of predefined intents. It is not yet proven to be sufficient for high-quality long-form companion dialogue, nuanced Urdu-English conversation, complex reasoning, or reliable multi-step planning.

For safety-critical or deterministic operations, use ordinary application code and strict schemas rather than asking a small model to make security decisions. The model may propose an intent; the Rust permission broker must validate it against an allow-list and request confirmation for sensitive actions.

### Additional compact model candidate: Qwen3.5-0.8B
The official Qwen3.5-0.8B model card lists Apache-2.0 and positions the model at approximately 0.9B parameters. The llama.cpp project currently documents a GGUF quick-start reference for a Qwen3.5-0.8B GGUF build. This makes it a useful additional compact candidate alongside Qwen3-0.6B, but does not establish Pakistani Urdu quality, 4 GB compatibility, or suitability as the main companion model.

Sources:
- https://huggingface.co/Qwen/Qwen3.5-0.8B
- https://github.com/ggml-org/llama.cpp

### Updated practical test plan
1. Compare the same exact GGUF model and quantisation through llama.cpp and Ollama on Windows.
2. Record installer/runtime footprint, model download size, load time, time to first token, tokens per second, peak RAM, idle RAM, and CPU/GPU use.
3. Use a fixed task set: classification, structured extraction, note tagging, short summarisation, rewriting, English chat, Urdu script, Roman Urdu, and Urdu-English switching.
4. Compare Qwen3-0.6B and Qwen3.5-0.8B, plus one slightly larger compact model if hardware allows.
5. Test cold start, repeated requests, context growth, cancellation, crash recovery, and fully offline operation after model installation.
6. Record hardware, exact model repository/revision, quantisation, runtime version, configuration, and file hashes.

**Status:** Desk research only. No model has been run, no performance benchmark has been completed, and no runtime/model is approved for release.


## 13. Hardware strategy — scale upward, not just downward (2026-11-23)

The product owner clarified that 4 GB is only one target, not the design ceiling. midSunday should provide the best practical experience that each computer can support. Users with more memory and capable CPUs/GPUs should be able to use stronger local models, longer context, richer speech and more concurrent workloads. Users with constrained hardware should retain a useful lighter option where feasible.

Planning classes now include 4 GB, 6 GB, 8 GB, 16 GB, 32 GB, and 64 GB+. These are not minimum/recommended specifications and do not imply that RAM alone determines capability.

The evaluation must:
- Compare more than one model size and quantisation, not merely find the smallest model that launches.
- Measure available RAM and total app-plus-model memory under realistic Windows load.
- Record CPU, GPU/VRAM, storage, thermal and power behaviour.
- Test different context lengths and quality/speed trade-offs.
- Evaluate voice workloads both sequentially and concurrently with chat.
- Provide model choice and clear resource estimates rather than hard-coding one model per RAM tier.
- Ensure high-memory users are not artificially capped at the smallest model.
- Ensure a smaller model remains selectable on powerful hardware for speed or privacy/resource preference.

Potential user-facing presets are Automatic, Lightweight, Balanced, and High capability. These labels and their internal resource rules remain proposals until tested.

**Research consequence:** The model benchmark matrix must include a compact baseline and at least one more capable model, then scale upward as hardware permits. A 4 GB test alone cannot satisfy Phase 1's performance research.

## 14. Hardware auto-detection and adaptive configuration — OpenJarvis reference (2026-11-23)

OpenJarvis has a documented **Hardware Auto-Detection** workflow. During `jarvis init`, it collects a hardware profile and uses separate `recommend_engine()` and `recommend_model()` functions to select initial defaults, then writes those defaults to `config.toml`. Its documented profile includes platform, CPU brand/count, total RAM, and optional GPU vendor/name/VRAM/compute capability/count. Users can override the generated configuration.

The current documented detection methods include NVIDIA `nvidia-smi`, AMD `rocm-smi`, Apple `system_profiler`, and platform-specific CPU/RAM queries. Its published model-selection heuristic uses reported GPU VRAM when available, otherwise system RAM with a fixed memory reserve. This is a useful architectural pattern: **detect → recommend engine → recommend model → save configurable defaults**.

However, the current documented hardware profile does not include storage capacity/free space, storage type/speed, or disk health. The documented setup workflow is primarily initial configuration; it does not establish a continuously adapting desktop resource manager. Its memory formula is a heuristic, not proof that a model will fit or perform acceptably on every system. The code and guide also show platform-specific detection differences, so we must not assume every metric is equally available on Windows.

Sources:
- https://github.com/open-jarvis/OpenJarvis/blob/main/docs/architecture/engine.md
- https://github.com/open-jarvis/OpenJarvis/blob/main/src/openjarvis/core/config.py
- https://open-jarvis.github.io/OpenJarvis/getting-started/installation/
- https://open-jarvis.github.io/OpenJarvis/getting-started/configuration/

### Proposed midSunday direction

Study the pattern, but design our own Windows-first **Hardware Intelligence & Adaptive Resource Manager (HIARM)**. It should cover:
- CPU identity, core topology and relevant capabilities.
- Installed and currently available RAM.
- GPU adapters, dedicated/shared memory, driver and verified acceleration support.
- OS/build/architecture.
- Storage volume capacity and free space before model downloads; storage performance only where reliable.
- Live resource pressure and midSunday process usage.
- Model/runtime/context/speech recommendations informed by measured benchmarks.
- Safe scheduling of LLM, STT and TTS workloads, with user override and clear explanations.

The subsystem should be local-only, work without an account or network, and never silently upload hardware information, download a large model, switch a user's model, enable cloud inference, alter system settings, or terminate unrelated processes.

**Status:** Research/design requirement only. No Windows detection API, Rust crate, runtime, or adaptive policy is approved. Detailed requirements and acceptance criteria are in [HARDWARE-INTELLIGENCE.md](HARDWARE-INTELLIGENCE.md).

## 15. Windows hardware-detection API and Rust-library shortlist (2026-11-24)

The initial Windows-first API review is now documented in detail in [HARDWARE-INTELLIGENCE.md](HARDWARE-INTELLIGENCE.md#12-windows-detection-research--api-and-rust-library-shortlist-2026-11-24).

### Preliminary findings
- `GlobalMemoryStatusEx` provides installed and currently available physical memory; the values are live/volatile and should be re-queried before large model operations.
- `GetLogicalProcessorInformationEx` provides processor-core, package, cache and NUMA topology relationships; logical processor count must not be mistaken for physical core count.
- DXGI can enumerate adapters and report adapter identity plus dedicated/shared memory capacity. `IDXGIAdapter3::QueryVideoMemoryInfo` provides a process-specific OS GPU-memory budget and usage, not a global free-VRAM counter.
- `GetDiskFreeSpaceExW` can check free space at the selected model-library destination. `GetDriveTypeW` distinguishes fixed/removable/network volumes; storage seek-penalty information is a more advanced optional query.
- Microsoft's Rust `windows` crate provides typed bindings to Win32 and COM APIs.
- `sysinfo` supports Windows and provides a useful cross-platform baseline for CPU, memory, process and disk information. Its current docs require retaining and refreshing a `System` instance; its latest documented version has Rust MSRV 1.95, which must be checked against the project's eventual pinned toolchain.
- NVIDIA `nvml-wrapper` is a possible optional diagnostics adapter, not a cross-vendor or mandatory dependency.

### Important rejected shortcut
Do not rely on WMI `Win32_VideoController.AdapterRAM` as authoritative VRAM. The field is 32-bit and Microsoft's documentation warns that values can be inaccurate on non-WDDM-compatible hardware. Use DXGI for generic adapter inventory and mark missing/uncertain readings honestly.

### Preliminary implementation direction
Prototype `sysinfo` plus narrowly-featured `windows` APIs first. Keep vendor-specific telemetry optional. Validate actual acceleration through the selected inference runtime and a controlled model-load test; a detected GPU does not prove the runtime can use it.

**Status:** Desk research only. No crate has been added, no detector code has been written, and no Windows test machine has been inspected. Exact crate versions, licences, MSRV, feature flags, and transitive dependencies must be checked before adoption.


## 16. Preliminary local model ladder — Qwen3.5 and Gemma 4 candidates (2026-11-24)

**Status:** Research shortlist only. No model is selected, downloaded, run, or approved for release. Parameter count and quantised file size are not RAM requirements.

### 16.1 Why the ladder is not a fixed RAM chart

The hardware manager should recommend from measured combinations of:
- Model architecture and exact checkpoint.
- Quantisation and conversion provenance.
- Runtime build and acceleration backend.
- Context length and KV-cache configuration.
- GPU offload and current process GPU budget.
- Available system memory, not only installed RAM.
- Concurrent UI, STT, TTS, and other workloads.
- Required quality for English, Pakistani Urdu, Roman Urdu, and Urdu-English code-switching.

A long advertised context window is not a sensible default. Start with short, measured context profiles and increase only when memory and quality tests justify it.

### 16.2 Primary family to evaluate: Qwen3.5

Qwen3.5 is a useful initial family because the official model cards list Apache-2.0 and broad multilingual support, and llama.cpp/ggml-org publish GGUF paths for multiple sizes. Its official model card reports 201 languages/dialects, but that does not establish strong Pakistani Urdu or Roman Urdu performance.

| Candidate | Example Q4 GGUF file size found | Intended evaluation role | Preliminary caution |
|---|---:|---|---|
| Qwen3.5-0.8B | ~563 MB | Very constrained fallback; bounded tasks; compare as basic chat | Official card positions this parameter scale mainly for prototyping/task-specific use; benchmark results are substantially below 2B on several general benchmarks. |
| Qwen3.5-2B | ~1.31 GB | Smallest serious conversational candidate to test | Promising size/quality balance; still needs Urdu and companion-dialogue evaluation. |
| Qwen3.5-4B | ~3.03 GB | Main balanced candidate for capable mainstream PCs | Weight file alone is already substantial; context/runtime overhead matters. |
| Qwen3.5-9B | ~5.78–6.17 GB depending on quant source | Higher-quality candidate for 16 GB and stronger systems | Quantisation provenance and exact build differ; 9B weights alone can exceed the practical memory budget of lower tiers. |
| Qwen3.5-27B | ~18 GB at Q4_K_M | High-capability CPU/GPU-offload candidate for 32 GB+ / 64 GB+ testing | Very large memory footprint; 32 GB is not a guarantee, especially with long context or other workloads. |

File sizes are observed published GGUF artifacts, not memory benchmarks. The 9B range reflects different quantization packages; use one exact, provenance-verified build in comparisons. A vision projector, if bundled, adds further storage and runtime cost; text-only companion tests should not download it unnecessarily.

### 16.3 Alternate family: Gemma 4 edge models

Gemma 4 deserves a comparison because Google describes E2B/E4B as edge-oriented, multilingual models and official model cards list Apache-2.0. Current ggml-org GGUF artifacts are available for llama.cpp.

| Candidate | Observed Q4 GGUF weight file | Notes |
|---|---:|---|
| Gemma 4 E2B IT | ~2.84 GB | The name refers to effective parameters; the GGUF package reports roughly 5B total parameters. Do not estimate memory from the "E2B" label alone. |
| Gemma 4 E4B IT | ~4.59 GB | Roughly 8B total parameters in the GGUF package; candidate for 16 GB class comparisons and stronger machines. |

Gemma 4 supports multilingual tasks and audio input in E2B/E4B, but its language claims are not a substitute for Pakistani Urdu tests. Compare its actual quality, memory, and speed with Qwen at similar resource budgets. Gemma 4's GGUF architecture is supported by current llama.cpp material, but pin and test the exact runtime revision before adopting it.

### 16.4 What the published benchmark evidence tells us — and does not tell us

Qwen's official Qwen3.5-2B model card compares 2B with 0.8B and reports a clear gap on some general evaluations (for example, MMLU-Pro and instruction-following tests). This supports treating 0.8B as a constrained baseline rather than assuming it can provide the same companion quality as a larger model.

These are vendor-published benchmark results. They are not independent midSunday tests, do not measure Pakistani Urdu, and do not predict performance on a particular Windows PC.

### 16.5 Preliminary evaluation ladder (not RAM promises)

- **Constrained / 4 GB target:** Qwen3.5-0.8B as the tiny baseline; Qwen3.5-2B Q4 as a stretch candidate. Do not promise either until the full Windows app + model + context is measured.
- **6–8 GB planning classes:** Compare Qwen3.5-2B and 4B, with Gemma 4 E2B as an alternative. The 4B/E2B options may be constrained by Windows overhead and context.
- **16 GB planning class:** Compare Qwen3.5-4B and 9B, Gemma 4 E2B/E4B, and smaller quantisations for speed.
- **32 GB planning class:** Compare Qwen3.5-9B and 27B with modest context and different CPU/GPU offload settings.
- **64 GB+ planning class:** Include Qwen3.5-27B and larger current candidates as appropriate; do not assume RAM alone is enough—GPU VRAM, CPU bandwidth, storage, and thermal limits remain relevant.

These are test buckets to organise the benchmark matrix, not automatic mappings or compatibility claims. A high-end user must still be able to choose a smaller model for speed.

### 16.6 Required midSunday benchmark suite

Use one fixed set of prompts and scoring rubrics across models:
1. Natural English companion conversation and emotional nuance.
2. Pakistani Urdu in Urdu script.
3. Roman Urdu typed naturally.
4. Urdu-English code-switching within the same message.
5. Correct understanding of Pakistani names, places, idioms, and everyday phrasing.
6. Personality consistency over a multi-turn conversation.
7. Instruction following, structured extraction, summarisation, and rewriting.
8. Reasoning and multi-step planning, with hallucination/error checks.
9. Safe tool-intent proposals (the app, not the model, validates all actions).
10. Refusal and boundary behaviour for unsafe or manipulative requests.

Measure exact model revision, quantisation, context, runtime commit, CPU/GPU offload, cold load time, first-token latency, tokens/second, peak system RAM, GPU memory budget/usage, disk size, stability, and quality scores. Test short, medium, and longer context separately. Include concurrent STT/TTS tests only after single-model baselines are established.

### 16.7 Licensing and provenance gate

- Official Qwen3.5 and Gemma 4 model cards currently list Apache-2.0; preserve the exact source model card and licence in the model catalogue.
- A GGUF conversion or third-party quantization must record the original model revision, converter/runtime commit, quantizer, quantisation method, file hashes, and applicable notices.
- Do not assume that an unlabelled community GGUF repository has complete provenance merely because the source model is Apache-2.0.
- Keep model weights separate from optional vision/audio projectors and MTP/draft assets. Do not download or bundle components the current feature does not need.
- Re-check current terms and all dependencies at the time of distribution.

### 16.8 Sources

- Qwen3.5-0.8B official model card: https://huggingface.co/Qwen/Qwen3.5-0.8B
- Qwen3.5-2B official model card and benchmark comparison: https://huggingface.co/Qwen/Qwen3.5-2B
- Qwen3.5-4B official model card: https://huggingface.co/Qwen/Qwen3.5-4B
- Qwen3.5-9B official model card: https://huggingface.co/Qwen/Qwen3.5-9B
- Qwen3.5-27B official model card: https://huggingface.co/Qwen/Qwen3.5-27B
- ggml-org Qwen3.5-0.8B GGUF: https://huggingface.co/ggml-org/Qwen3.5-0.8B-GGUF
- ggml-org Qwen3.5-4B GGUF and quant sizes: https://huggingface.co/ggml-org/Qwen3.5-4B-GGUF
- Qwen3.5-2B GGUF size listing: https://huggingface.co/gguf/Qwen3.5-2B-GGUF
- Qwen3.5-9B GGUF quantization/provenance example: https://huggingface.co/SoAIHQ/Qwen3.5-9B-GGUF
- Qwen3.5-27B GGUF sizes: https://huggingface.co/gguf/Qwen3.5-27B-GGUF
- Gemma 4 E2B official model card: https://huggingface.co/google/gemma-4-E2B-it
- Gemma 4 E4B official model card: https://huggingface.co/google/gemma-4-E4B-it
- Gemma 4 GGUF collection: https://huggingface.co/collections/ggml-org/gemma-4
- Gemma 4 with llama.cpp: https://github.com/google-gemma/docs/blob/main/docs/deployment/llamacpp.md
- llama.cpp current model support/docs: https://github.com/ggml-org/llama.cpp

**Conclusion:** Qwen3.5 should be the first model family in the benchmark matrix, with Gemma 4 E2B/E4B as meaningful alternatives. The 2B and 4B Qwen candidates look especially useful to compare for the low-to-mainstream range; 9B and 27B give us a path to test more capable systems. Nothing is selected until real tests—especially Pakistani Urdu and companion quality—support it.

