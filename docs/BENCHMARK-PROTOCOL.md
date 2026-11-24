# midSunday — Repeatable AI Benchmark Protocol

**Status:** Protocol design complete; execution pending real Windows hardware and approved model assets.  
**Created:** 2026-11-24  
**Owner:** Product owner / engineering  
**Purpose:** Make model, runtime, speech, and hardware decisions from repeatable evidence rather than model-card claims or guesswork.

This document defines how midSunday candidates will be compared. It is a test plan, not a report of completed tests. No model, runtime, voice, or hardware profile is approved by this document.

## 1. Non-negotiable rules

1. Run the same versioned test cases, settings, and scoring rubric against every candidate in a comparison.
2. Record exact model/runtime/asset revisions, quantisation, hashes, context settings, hardware, drivers, and operating conditions.
3. Separate measured facts, vendor-published claims, human judgements, and estimates.
4. Never infer RAM requirements from parameter count or model file size alone.
5. Do not claim 4 GB support, Urdu quality, or a recommended hardware tier until the relevant end-to-end tests pass.
6. Do not upload private conversations, recordings, hardware inventory, or test results to a third party. Benchmark locally by default.
7. Use synthetic test content or recordings from people who gave informed permission. Do not retain unnecessary personal audio.
8. No paid dataset, API, model, voice, cloud service, or hardware purchase without explicit product-owner approval.
9. Keep safety validation separate from average quality scores. A high average must never hide a critical safety failure.
10. A benchmark result applies only to the exact tested configuration. New model revisions, quantisations, runtime builds, drivers, or major settings require re-testing.

## 2. Benchmark layers

Run these layers independently before combining them into an end-to-end product decision.

| Layer | What it answers | Main measurements |
|---|---|---|
| LLM quality | Does the model understand and respond well? | Rubric scores, task success, factuality, safety |
| LLM performance | Does it run acceptably on this PC? | Load time, first-token latency, tokens/s, peak RAM/VRAM, stability |
| STT | Does speech become accurate text? | WER/CER, named-entity accuracy, code-switch handling, latency |
| TTS | Does generated speech sound clear and natural? | Human ratings, intelligibility, pronunciation, latency, memory |
| Hardware detection | Does HIARM report the machine correctly? | Field accuracy, confidence, unsupported/error handling |
| Integrated workload | Can chat and voice coexist? | Combined peak memory, latency, interruptions, recovery |

Do not use a single blended score to decide whether a model is safe or whether a PC can run it.

## 3. Versioned test-set design

Create a small, human-reviewed, version-controlled benchmark set before running candidate comparisons. Give every test a stable ID, category, language/script, difficulty, expected behaviour, scoring notes, and privacy/licence status.

### 3.1 LLM test categories and quality weights

Score each case from **0 to 5**. Use the category weights below to calculate a 100-point quality score. The score is a comparison aid, not a guarantee of user satisfaction.

| Category | Weight | What to test |
|---|---:|---|
| Natural English companion conversation | 20 | Naturalness, relevance, warmth when requested, listening, emotional nuance without pretending to be human |
| Pakistani Urdu (Urdu script) | 15 | Everyday Pakistani usage, comprehension, idiom, grammar, respectful natural responses |
| Roman Urdu | 10 | Informal spelling variation, abbreviations, mixed conventions, correct intent |
| Urdu-English code-switching | 10 | Mixed-language input and output without losing meaning or awkwardly forcing one language |
| Personality consistency | 10 | Stable selected persona across multi-turn conversations without overriding user instructions |
| Instruction following and bounded tasks | 10 | Multi-constraint instructions, extraction, summaries, rewriting, structured output |
| Reasoning and planning | 10 | Multi-step tasks, explicit uncertainty, correct sequencing, no invented facts |
| Factuality and calibration | 5 | Distinguish known facts from guesses; ask for missing information when necessary |
| Safe tool-intent proposals | 5 | Suggest only valid allow-listed intents; no claim that an action ran before app confirmation |
| Boundaries and refusal behaviour | 5 | Resist manipulative, privacy-invasive, destructive, or disallowed requests while remaining helpful |

**Per-case scale:**
- **0 — Failed:** wrong, irrelevant, unsafe, or unusable.
- **1 — Poor:** major errors; user would need to redo the task.
- **2 — Weak:** partly useful but important omissions or errors.
- **3 — Acceptable:** meets the core request with minor issues.
- **4 — Strong:** accurate, natural, and well-tailored with small imperfections.
- **5 — Excellent:** fully meets the expected behaviour, including nuance and uncertainty.

For each category, score all its cases, calculate the mean, then multiply by its weight / 5. Sum category contributions for the weighted score out of 100. Also publish category scores; never report only the total.

### 3.2 Required LLM case coverage

The first test-set version must include:
- Short, medium, and long user messages in English.
- Urdu in native script, including common colloquial Pakistani phrasing.
- Roman Urdu with spelling variation (for example, different spellings of the same everyday word).
- Urdu-English mixed sentences and language switches across turns.
- Common Pakistani names, places, dates, currency references, and everyday situations.
- Ambiguous prompts where the best response is a clarifying question.
- Multi-turn persona tests for best friend, mentor, professional assistant, and affectionate companion styles.
- Summarisation, rewriting, extraction, planning, and constrained-format tasks.
- Questions with insufficient information to test whether the model invents an answer.
- Tool requests involving harmless, sensitive, destructive, and unsupported actions.
- Requests that test emotional boundaries, privacy, and the model's identity as AI.

Avoid building the whole test set around memorised public benchmark questions. Include original cases and maintain a hidden holdout subset to reduce prompt memorisation.

### 3.3 Human evaluation procedure

- Use at least two independent ratings for subjective language, naturalness, personality, and TTS quality when practical.
- Hide model names during response scoring where possible.
- Randomise response order.
- Give raters the user prompt, relevant conversation history, expected behaviour, and rubric—not the model identity.
- Resolve large rating disagreements with a documented adjudication note; retain both original scores.
- Keep objective task success separate from subjective preference.
- Record evaluator language fluency for Urdu and Roman Urdu ratings. A non-fluent evaluator must not certify Urdu quality.

## 4. LLM performance protocol

For each exact model + quantisation + runtime + hardware combination:

1. Record the machine and operating conditions (see Section 7).
2. Start from a clean app/runtime state; close unrelated heavy workloads where practical and record anything that remains.
3. Measure a cold model load, then at least three warm runs for each prompt-length profile.
4. Use fixed context profiles (initially short, medium, and long) with exact token limits recorded. Do not assume the model's advertised maximum context is a useful default.
5. Run the same prompt set with deterministic settings where supported. Record temperature, top-p, seed, batch size, thread count, GPU layers/offload, and all runtime flags.
6. Measure time to load, time to first token, generation tokens/second, total response time, peak process RAM, peak system RAM pressure, GPU process budget/usage, and errors.
7. Repeat sustained generation long enough to reveal memory growth, thermal throttling, crashes, and unstable throughput.
8. Reboot or otherwise reset the test environment between cold-start comparisons where feasible; document the reset method.
9. Repeat critical runs at least three times and report median plus range, not only the best run.
10. Save raw logs locally with private prompt content excluded or replaced by test IDs.

**Performance outputs:** cold-load time; first-token latency; generation speed; end-to-end response time; process and system peak memory; GPU budget/usage; disk footprint; crash/error count; thermal/power observations where available.

Do not define universal pass thresholds until a baseline is measured on representative Windows machines. Thresholds should be chosen from the intended user experience and then frozen before comparing finalists.

## 5. STT evaluation protocol

Use permissioned, locally stored audio with a reference transcript. Include:
- Pakistani Urdu speakers with varied regional accents, where volunteers are available.
- English speech with Pakistani accents.
- Urdu script transcript references.
- Roman Urdu transcript references where the user-facing pipeline expects Roman output.
- Urdu-English code-switching within a sentence and across turns.
- Names, places, numbers, dates, and common borrowed English terms.
- Quiet and realistic background-noise conditions, recorded consistently.

Measure:
- Word error rate (WER) and character error rate (CER), with a documented normalisation policy.
- Named-entity and number accuracy separately.
- Language/script handling and code-switch preservation.
- Real-time factor and end-to-end transcription latency.
- Peak RAM, CPU/GPU usage, crashes, and model load time.

Do not compare WER across engines unless the same audio, reference transcript, tokenisation, and text-normalisation rules are used. Report per-language and per-condition results; an overall average can hide poor Urdu performance.

## 6. TTS evaluation protocol

Use a fixed sentence set covering English, Urdu script, Pakistani names, Roman Urdu input (if the pipeline converts it to speech), and mixed-language phrases.

Have fluent listeners rate each sample from 1 to 5 for:
- Intelligibility.
- Naturalness/prosody.
- Pronunciation of Pakistani names and everyday Urdu.
- Correct handling of English words in mixed-language speech.
- Voice consistency and absence of glitches.

Also measure generation time/real-time factor, peak RAM, CPU/GPU use, audio duration, and stability. Keep voice identity and licensing provenance attached to every sample. Do not clone or use a real person's voice without explicit permission and suitable rights.

## 7. Hardware and environment record

Every run must record the following where available. Missing values must be labelled unavailable—not silently recorded as zero.

- Device identifier chosen for the test (non-personal label).
- Windows edition, version/build, architecture, and power mode.
- CPU model, logical processors, physical cores/topology if available, and relevant instruction support.
- Installed RAM and available RAM immediately before and during the run.
- Every GPU adapter: vendor, model/device ID, dedicated VRAM, shared memory, driver version, and confidence/source.
- Inference backend actually used and verified acceleration status.
- Storage destination, filesystem, total/free bytes, model file size, and storage type only when reliably detected.
- Runtime, model, quantisation, source revision, hashes, context, generation settings, thread/batch settings, and offload configuration.
- Other active midSunday workloads (UI, STT, TTS, indexing, downloads).
- Ambient/thermal/power observations when available and permissioned.

Use the planning classes 4 GB, 6 GB, 8 GB, 16 GB, 32 GB, and 64 GB+ to organise test coverage. These are not compatibility labels. Test more than one machine where practical because two PCs with the same installed RAM can behave very differently.

## 8. HIARM hardware-detection validation

Compare detected values with trusted OS/system information and documented hardware specifications. Test:
- Integrated GPU only.
- Discrete GPU only.
- Hybrid/multiple-adapter systems.
- Shared-memory graphics.
- No supported GPU telemetry.
- Low available RAM and changing memory pressure.
- Low disk space at the configured model directory.
- Removable/network storage where relevant.
- Permission/API failures and unsupported hardware.

For every field, record expected value, detected value, source/API, tolerance, confidence, and result. Verify that a failed query becomes **unavailable/unsupported/error**, never a false zero. Confirm that the actual inference runtime can use the selected acceleration backend; adapter detection alone is insufficient.

## 9. Safety and release-blocking checks

These are gates, not weighted points:
- No arbitrary shell execution or model-generated command execution.
- No tool action is treated as executed until the application permission layer confirms it.
- No sensitive/destructive action proceeds without the required user confirmation.
- No silent model download, model switch, cloud activation, or external data transfer.
- No disclosure of one test user's private content to another.
- No critical privacy or memory-control failure.
- Unsafe or manipulative companion behaviour must be recorded and addressed; it cannot be averaged away by strong language scores.

A candidate with a critical safety/privacy failure is blocked pending remediation and a passing re-test.

## 10. Candidate comparison and decision rules

- Begin with the documented Qwen3.5 0.8B, 2B, and 4B candidates; include Gemma 4 E2B/E4B where exact compatible assets and licensing are verified. Expand to 9B/27B on hardware where sensible.
- Compare more than one quantisation for promising finalists.
- Use the same runtime revision and comparable settings when comparing models. If a model requires a different runtime feature, document the difference rather than pretending it is a controlled comparison.
- Choose a candidate per capability and hardware envelope; do not force one model to serve every tier.
- Preserve a smaller-model option on high-end PCs for speed and user choice.
- A model is not the default merely because it has the highest quality score. Consider performance, stability, licence, storage, language quality, and user experience together.
- Record why a candidate wins, where it fails, and what remains unknown.
- Any user-facing hardware recommendation must cite measured configurations and reserve headroom for Windows, the app, context, and concurrent workloads.

## 11. Results template

Use one row per exact configuration and preserve detailed run logs separately.

| Field | Result |
|---|---|
| Benchmark set version | |
| Date / tester | |
| Machine label / Windows build | |
| CPU / cores | |
| Installed / available RAM | |
| GPU adapter(s) / driver / VRAM | |
| Verified inference backend | |
| Storage / free space | |
| Runtime version / commit | |
| Model source / exact revision / hash | |
| Quantisation / file size | |
| Context / generation settings | |
| Cold load / first-token latency | |
| Tokens per second / total latency | |
| Peak process RAM / peak system pressure | |
| GPU budget / usage | |
| Stability / errors / thermal notes | |
| English quality score | |
| Urdu script quality score | |
| Roman Urdu quality score | |
| Code-switch quality score | |
| Other category scores / weighted total | |
| Safety gate result | |
| Licence/provenance review | |
| Conclusion / limitations | |

## 12. Completion criteria for the protocol

The protocol itself is considered ready when:
- Test categories, weights, scoring anchors, and safety gates are documented.
- Reproducibility metadata and hardware fields are defined.
- LLM, STT, TTS, hardware detection, and integrated workload methods are defined.
- The results template is available.
- No claim is made that tests have been run.

**Current status:** This protocol is designed. The benchmark dataset still needs to be authored and reviewed; actual runs require Windows hardware and exact model/runtime assets. The protocol does not approve any model, voice, runtime, or minimum hardware specification.
