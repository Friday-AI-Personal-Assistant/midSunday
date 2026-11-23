# midSunday — Hardware Compatibility and Benchmark Policy

**Status:** Targets for investigation; no minimum or recommended specification is yet certified.

## Principle
Do not equate model file size with runtime memory use. Total footprint includes the desktop app, inference runtime, model weights, context/KV cache, audio buffers, speech engines, operating system, and other running applications.

## Initial profiles to investigate

| System RAM | Intended profile | Status |
|---|---|---|
| 4 GB | Constrained/basic text experience; very small model and limited context | Experimental target |
| 8 GB | More practical local chat and selected voice workflows | Candidate target |
| 16 GB | Better model choices, longer context, more comfortable multitasking | Candidate target |
| 32 GB+ | Higher-capability local models and advanced workflows | Candidate target |

These are planning profiles, not compatibility promises. CPU, storage, operating system, GPU/VRAM, and background applications materially affect results.

## Benchmark matrix
For every candidate machine and model, record:
- Exact CPU, RAM, GPU/VRAM, storage type, OS build, and power mode.
- App and runtime versions; model name, revision, quantisation, and context size.
- Cold start and model load time.
- Time to first token and sustained tokens/second.
- Peak and steady-state RAM; GPU memory where applicable.
- CPU utilisation, temperature/throttling where measurable, and battery impact on laptops.
- Speech recognition and synthesis memory/latency when enabled.
- Responsiveness while the model is generating.
- Stability over a representative long session.

## Resource management requirements
- Load only the selected model by default.
- Unload or pause inactive engines when appropriate.
- Use bounded context and expose a simple context/performance setting.
- Avoid running large STT, TTS, and LLM workloads concurrently on constrained systems.
- Provide clear low-memory warnings and graceful recovery.
- Allow users to choose model storage location where feasible.
- Never automatically download a large model without clear size and consent.

## Release claims
A hardware profile becomes supported only after repeatable tests on representative machines. Publish exact test conditions and known limitations. A successful test on one machine does not guarantee every machine with the same RAM will perform identically.

## 4 GB acceptance threshold (to be defined)
Before claiming a useful 4 GB experience, define and test:
- Maximum acceptable app-plus-model memory footprint.
- Minimum response speed and maximum first-response delay.
- Minimum useful context size.
- Whether voice must be disabled or unloaded during chat.
- Behaviour when other common desktop applications are open.

## Open questions
- Which Windows versions and CPU architectures will be supported first?
- What is the oldest practical CPU instruction set for selected runtimes?
- Is CPU-only inference the baseline, with GPU acceleration optional?
- What performance threshold makes a model usable rather than merely launchable?


## Scalable experience profiles — expanded direction (2026-11-23)

The product is **not** a 4 GB-only application. The goal is to scale capability upward so users with stronger machines receive a meaningfully better local experience. These are planning profiles, not guaranteed specifications or fixed model assignments.

| Installed system RAM | Intended experience direction | Candidate capabilities to evaluate |
|---|---|---|
| 4 GB | Constrained/basic | Very compact quantised model, short context, text-first use, sequential loading, optional voice only if measurements allow |
| 6 GB | Entry/balanced | More headroom than 4 GB, compact model choices, moderate context, carefully scheduled voice workloads |
| 8 GB | Comfortable | Stronger compact models, larger context, more practical voice use and multitasking |
| 16 GB | Advanced | Larger and more capable local models, longer context, improved voice quality, more concurrent services |
| 32 GB | High capability | Substantially larger models and richer context, higher-quality speech pipelines, more simultaneous workloads where CPU/GPU permit |
| 64 GB+ | Enthusiast / workstation | Evaluate larger local models, advanced acceleration and high-fidelity voice workflows; no fixed promise until tested |

### Resource-aware behaviour
- Detect available memory at runtime; installed RAM alone is not enough because Windows and other applications consume memory.
- Consider CPU architecture/performance, GPU and dedicated/shared VRAM, storage capacity/speed, and power/thermal limits.
- Offer clear presets such as **Automatic**, **Lightweight**, **Balanced**, and **High capability**, with an advanced manual override.
- Recommend models based on the machine's measured/available resources and the user's quality/speed preference.
- Do not silently download or switch to a large model. Show model size, required storage estimate, expected resource class, and ask for consent.
- Prefer one active LLM at a time. Schedule or unload STT/TTS workloads according to available resources; allow capable systems to use more concurrency when benchmarks support it.
- Make it possible to choose a smaller/faster model even on powerful hardware.
- Do not treat system RAM as equivalent to GPU VRAM or assume all model layers can be offloaded.

### Benchmark requirement
Test representative machines in each available class, including 6 GB where practical. Record both idle and available memory before loading, total application-plus-model peak RAM, VRAM, latency, throughput, thermal behaviour, and voice concurrency. Test under realistic Windows background load. A RAM tier is never a guarantee by itself.
