# midSunday — Decision Log

Record important decisions with date, status, rationale, and consequences. A superseding decision should not erase the historical record.

Statuses: **Confirmed**, **Proposed**, **Under review**, **Superseded**.

## D-001 — Product name

- **Date:** 2026-11-22
- **Status:** Confirmed
- **Decision:** The product name is **midSunday** (exact casing).
- **Rationale:** Selected by the product owner.
- **Consequences:** Use this spelling consistently in product documentation and repository naming. Trademark clearance is still outstanding.

## D-002 — Product identity

- **Date:** 2026-11-22
- **Status:** Confirmed
- **Decision:** midSunday is an independent AI Personal Companion, not a clone or sub-product of FRIDAY or Stonic AI.
- **Consequences:** Maintain a separate repository, identity, interface, feature roadmap, and release process.

## D-003 — Repository location

- **Date:** 2026-11-22
- **Status:** Confirmed
- **Decision:** Create a separate midSunday repository inside the Friday-AI-Personal-Assistant organisation, with the possibility of transferring it later.
- **Consequences:** Current repository is Friday-AI-Personal-Assistant/midSunday.

## D-004 — Windows-first delivery

- **Date:** 2026-11-22
- **Status:** Confirmed
- **Decision:** Windows is the first target platform; Linux and macOS are later targets.
- **Consequences:** Initial prototypes, installers, and compatibility tests prioritise Windows.

## D-005 — Offline-first product direction

- **Date:** 2026-11-22
- **Status:** Confirmed
- **Decision:** Core chat, voice, memory, and PC assistance should work locally/offline wherever technically practical. Internet search and cloud services require internet.
- **Consequences:** Local engines and adapters are prioritised; network boundaries must be explicit.

## D-006 — No mandatory paid services

- **Date:** 2026-11-22
- **Status:** Confirmed
- **Decision:** No hidden costs, mandatory subscriptions, surprise API charges, or unapproved paid services.
- **Consequences:** Obtain explicit approval before adopting paid infrastructure, APIs, domains, certificates, or assets.

## D-007 — 4 GB hardware target

- **Date:** 2026-11-22
- **Status:** Confirmed as a target; feasibility unverified
- **Decision:** Investigate a useful basic experience on 4 GB RAM machines.
- **Consequences:** Do not publish 4 GB as a supported minimum until benchmarked.

## D-008 — Urdu language priority

- **Date:** 2026-11-22
- **Status:** Confirmed
- **Decision:** Urdu is a high-priority language, including Pakistani Urdu, Roman Urdu, and Urdu-English mixed speech.
- **Consequences:** Include these cases in STT, TTS, and model evaluations.

## D-009 — Personality options

- **Date:** 2026-11-22
- **Status:** Confirmed as product directions
- **Decision:** Explore best friend, girlfriend, boyfriend, mentor, professional assistant, and custom personalities.
- **Consequences:** Build configurable persona profiles with clear AI identity and healthy boundaries.

## D-010 — Proposed technical stack

- **Date:** 2026-11-23
- **Status:** Proposed, not approved
- **Decision:** Evaluate Tauri 2, Rust, React, TypeScript, and Vite as the Windows desktop foundation.
- **Rationale:** Lightweight native shell with a modern UI and a Rust backend.
- **Consequences:** Benchmark and review ecosystem/licensing before locking the stack.

## D-011 — Proposed local AI and voice candidates

- **Date:** 2026-11-23
- **Status:** Proposed, not approved
- **Decision:** Evaluate llama.cpp/GGUF, Vosk or whisper.cpp, Piper or Kokoro, and SQLite.
- **Consequences:** Validate language quality, runtime footprint, maintenance, security, and commercial redistribution rights before selection.

## D-012 — Roadmap as continuity source

- **Date:** 2026-11-23
- **Status:** Confirmed
- **Decision:** Maintain a complete roadmap and repository documentation so work can continue if a conversation is lost or corrupted.
- **Consequences:** Update project records at each meaningful milestone and include handoff notes.


## D-013 — Phase 1 research checkpoint

- **Date:** 2026-11-23
- **Status:** Under review; no technology approval
- **Decision:** Record initial feasibility findings in `docs/PHASE-1-RESEARCH.md` and keep all engines/models as candidates until practical tests and exact licence reviews are complete.
- **Rationale:** Official documentation supports several promising candidates, but Urdu language quality, constrained-hardware performance, and redistribution rights are not established by desk research alone.
- **Consequences:** Do not lock the stack, claim 4 GB compatibility, or bundle a model/voice until Phase 1 acceptance evidence exists.


## D-014 — Local inference runtime comparison

- **Date:** 2026-11-23
- **Status:** Under review
- **Decision:** Keep llama.cpp as the leading candidate for direct midSunday runtime integration; evaluate Ollama as a local development and model-testing convenience.
- **Rationale:** Both projects publish MIT licences for their software repositories. llama.cpp offers direct control and a server/library integration path; Ollama offers a convenient local model-management workflow. Ollama cloud inference is a separate optional paid service and is not required for local use.
- **Consequences:** Do not bundle both by default. Compare them on Windows with the same model before final selection. Review each model and quantised file separately for commercial redistribution rights. No runtime is approved yet.
