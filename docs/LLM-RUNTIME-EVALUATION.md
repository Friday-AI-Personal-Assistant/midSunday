# Local LLM Runtime Evaluation — llama.cpp vs Ollama

**Research date:** 2026-11-24  
**Status:** Desk evaluation complete; no runtime selected for production and no runtime integrated.  
**Scope:** Local inference for the midSunday desktop application on Windows, Linux, and macOS.  
**Decision rule:** Local-first, offline-capable, zero mandatory service cost, commercially reviewable distribution, replaceable adapter, and honest hardware support.

## 1. Executive recommendation

Use **llama.cpp as the primary runtime candidate** for the first midSunday integration prototype.

For the first prototype, prefer launching a pinned, locally built `llama-server` as a managed child process and communicating with it over loopback. This is a deliberately reversible integration boundary: it avoids immediately writing unsafe or fragile C/C++ FFI bindings into the Rust application, while keeping the runtime under midSunday's control.

Keep the direct `llama` library C API as a later option if measurements show that a managed local server adds unacceptable overhead or complicates lifecycle management.

Keep **Ollama as an optional external comparison tool**, not a required midSunday dependency. It is useful for quick manual model testing and its local HTTP API is convenient, but requiring users to install and run a separate general-purpose model manager would add another installation, update, service-lifecycle, and support dependency to midSunday.

This is a prototype direction, not final production approval. Exact build configuration, model compatibility, resource use, commercial redistribution, and all transitive licences still require validation.

## 2. What each project provides

### llama.cpp

The upstream project describes its main product as the `llama` inference library and also provides command-line tools and `llama-server`. Its build documentation covers CPU builds and multiple acceleration backends, with platform-specific instructions. The server can listen on a loopback address and expose HTTP endpoints.

Relevant official sources:
- https://github.com/ggml-org/llama.cpp
- https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md
- https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md
- https://github.com/ggml-org/llama.cpp/blob/master/include/llama.h
- https://github.com/ggml-org/llama.cpp/blob/master/LICENSE

### Ollama

Ollama provides an installable application/CLI, a local server, model download and management commands, and HTTP APIs. Its documented local API normally uses `http://localhost:11434`; local requests do not need an API key. Its documentation also describes cloud model access, which is separate from local inference and must never be enabled by midSunday without an explicit user choice.

Relevant official sources:
- https://github.com/ollama/ollama
- https://github.com/ollama/ollama/blob/main/LICENSE
- https://github.com/ollama/ollama/blob/main/docs/api/introduction.mdx
- https://github.com/ollama/ollama/blob/main/docs/quickstart.mdx
- https://github.com/ollama/ollama/blob/main/docs/development.md

## 3. Comparison against midSunday requirements

| Requirement | llama.cpp | Ollama | Assessment |
|---|---|---|---|
| Local inference without an account | Yes | Yes | Both can satisfy core offline inference after required assets are installed |
| Offline use after setup | Yes, with local model/runtime assets | Yes, with local model/runtime assets | Both require a clear first-download/import process |
| CPU inference | Yes | Yes | Both candidates |
| Windows, Linux, macOS | Documented build/run support | Official installation paths for all three | Both candidates; midSunday must test its exact builds |
| Hardware acceleration choices | Many documented backends; build configuration matters | Runtime manages supported execution paths | llama.cpp offers more direct backend/build control; validate actual devices |
| GGUF workflow | Native focus | Supports GGUF workflows through Ollama model packaging/import | Both useful for current candidate ladder |
| Runtime integration | Library API or managed server | Local HTTP server/API | Both possible; server boundary reduces initial coupling |
| Model download/catalogue | Runtime tools and external sources; app can own catalogue | Integrated model library and pull workflow | Ollama is more convenient; midSunday should own its own vetted catalogue and consent flow |
| Process lifecycle control | Directly managed by midSunday if launched as child | Separate Ollama application/service lifecycle | llama.cpp better fit for a self-contained desktop app |
| Runtime footprint and startup | Must measure exact build/configuration | Must measure service + runner + model | No benchmark evidence yet; do not assume |
| Commercial redistribution | Upstream llama.cpp repository is MIT; audit backend and bundled dependencies | Upstream Ollama repository is MIT; audit packaged native payloads and dependencies | Neither repository's top-level licence replaces a complete distribution audit |
| Long-term support burden | midSunday owns more build/backend integration work | Users may need a separate Ollama install/update path if not bundled | Trade-off: control vs convenience |

## 4. Proposed integration shape

### Prototype A — managed llama-server process

1. midSunday's Rust backend validates the selected runtime binary and model file paths.
2. Rust launches the pinned `llama-server` executable as a child process with a narrowly defined argument list.
3. The server binds only to loopback (initial target: `127.0.0.1`), never all network interfaces.
4. The backend checks readiness, then sends generation requests through a typed local adapter.
5. The UI receives streamed output through the existing Tauri command/event boundary.
6. Stop, unload, application exit, crashes, and orphan-process cleanup receive explicit tests.
7. Logs must not record prompts, private conversation text, model secrets, or unnecessary hardware identifiers.

The model path, context size, thread count, GPU layers, and other resource settings must come from validated app configuration, not arbitrary model-generated shell text. No model-generated command is ever executed.

### Prototype B — direct library API (later comparison)

If Prototype A works but shows measurable overhead or lifecycle limitations, evaluate linking the `llama` C API behind a small Rust FFI wrapper. Keep unsafe code isolated, document ownership/threading rules, and test crashes and cancellation carefully. Do not begin here before a working baseline exists.

### Ollama comparison mode

Developers may install Ollama separately on their own test machine and point a temporary benchmark client at its documented local API. This is only a test convenience. The midSunday application must not require Ollama to be installed, start it silently, use its cloud features, or download a model without the user's approval.

## 5. Privacy, security, and offline requirements

- Bind local inference endpoints to loopback only; never expose an unauthenticated inference server to the LAN or internet.
- Treat a local port as a security boundary: validate the expected process/endpoint and avoid trusting an unrelated service merely because a port responds.
- Keep model and runtime downloads explicit, with source, exact revision, size, hash, licence, destination, and user confirmation.
- Do not auto-download, auto-update, or silently switch models.
- No telemetry or hardware inventory upload by default.
- Do not send prompts, chat history, memory, or audio to cloud services in the local mode.
- If optional cloud inference is added later, show a clear provider, destination, data disclosure, and cost/credential warning before enabling it.
- Use a vetted model catalogue. Runtime support for a file format does not approve the model's licence, provenance, safety, or language quality.
- Do not let the LLM choose executable paths, arbitrary flags, shell commands, or permission decisions.

## 6. Licensing and distribution gate

The upstream repositories identify llama.cpp and Ollama as MIT-licensed projects. This is encouraging but is not sufficient by itself for commercial redistribution.

Before bundling any runtime:
- Pin an exact upstream commit/release and preserve its licence and notices.
- Generate a complete dependency/software bill of materials for the exact platform build.
- Review each acceleration backend and linked/bundled dependency separately.
- Review compiler/runtime redistributables and platform-specific packaging obligations.
- Check whether optional CUDA, Vulkan, Metal, OpenCL, OpenVINO, or other components introduce separate dependencies or terms.
- Record hashes and provenance for every distributed binary.
- Review every model's original weights licence, conversion source, quantisation, and notices independently of the runtime.
- Re-run the review whenever the runtime revision or build flags change.

No runtime or model is cleared for commercial distribution by this desk evaluation.

## 7. Benchmark plan

Compare llama.cpp and Ollama with the same model file, quantisation, prompt set, context size, hardware, and comparable generation settings wherever the APIs permit.

Record:
- OS, CPU model/topology, RAM installed and available, GPU adapters, driver, and GPU memory budget.
- Exact runtime release/commit, build flags, acceleration backend, and binary hash.
- Exact model repository/revision, GGUF file, quantisation, and SHA-256.
- Context length, batch settings, CPU threads, GPU offload, and prompt settings.
- Cold runtime start, model load time, time to first token, generation tokens/second, and peak process/system RAM.
- GPU usage/memory where reliably available.
- Idle and active overhead, cancellation latency, repeated-request stability, and clean shutdown.
- Output quality using the frozen midSunday dataset: English, Urdu script, Roman Urdu, code-switching, persona consistency, reasoning, factuality, and safety.
- Behaviour with constrained available RAM and CPU-only inference.

Run the same tests on Windows first, then Linux and macOS. GitHub-hosted CI proves that the desktop source can compile/package on those runners; it does not substitute for real-device inference benchmarks.

## 8. Prototype acceptance criteria

The first runtime prototype is successful only when:
- The app can detect and report the pinned runtime binary version.
- It can start and stop a local runtime without requiring a separate user-installed service.
- The runtime binds only to loopback.
- It can load one explicitly selected, locally available test model.
- It can stream a response into the chat UI and stop generation reliably.
- It can report useful load/runtime errors without crashing the desktop shell.
- It works with network disconnected after the runtime and model are present.
- It does not send prompts or system details outside the device.
- The process is cleaned up on normal exit and recoverable after a forced exit.
- All bundled runtime/dependency licences are reviewed before distribution.

## 9. Current decision record

**Decision:** Select llama.cpp as the first runtime integration prototype candidate, using a managed `llama-server` child process and loopback HTTP adapter. Keep Ollama as an optional external benchmark comparator only.

**Confidence:** Moderate for architectural fit; low for measured performance and production distribution readiness.

**Not decided:** Exact llama.cpp revision, build flags, backend per OS, model, quantisation, context defaults, installer bundling strategy, or direct-library vs server architecture for final release.

**Next actions:**
1. Create a runtime adapter interface in the Rust backend without adding model/runtime binaries yet.
2. Design process lifecycle and loopback security tests.
3. Pin and build a CPU-only llama.cpp test binary in a separate CI experiment or documented developer test.
4. Obtain a real Windows test machine for inference measurements before selecting defaults.
5. Add acceleration backends only after CPU baseline and exact hardware tests.
6. Review exact dependency licences and generate platform-specific SBOMs before any public distribution.

## 10. Research references

- llama.cpp build and backend documentation: https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md
- llama.cpp library and C API: https://github.com/ggml-org/llama.cpp/blob/master/include/llama.h
- llama-server documentation: https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md
- llama.cpp licence: https://github.com/ggml-org/llama.cpp/blob/master/LICENSE
- Ollama project and licence: https://github.com/ollama/ollama
- Ollama local API introduction: https://github.com/ollama/ollama/blob/main/docs/api/introduction.mdx
- Ollama quickstart: https://github.com/ollama/ollama/blob/main/docs/quickstart.mdx
- Ollama development/build notes: https://github.com/ollama/ollama/blob/main/docs/development.md
