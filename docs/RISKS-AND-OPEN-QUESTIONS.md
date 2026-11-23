# midSunday — Risks and Open Questions

This is a living register. Review it during planning, architecture changes, and release readiness. Risks should have an owner, mitigation, and status as the project matures.

## 1. Current risks

| ID | Risk | Impact | Mitigation / next action | Status |
|---|---|---|---|---|
| R-001 | 4 GB RAM target may not support the desired model and voice combination | High | Benchmark complete workflows on real low-memory hardware; define a constrained profile and avoid overpromising | Open |
| R-002 | Urdu STT/TTS quality may be inadequate, especially Pakistani accents and code-switching | High | Build a representative test set and compare engines before selecting | Open |
| R-003 | Model-weight licences may differ from code/runtime licences | High | Review each model's exact licence and redistribution terms | Open |
| R-004 | Voice model and voice asset licences may restrict redistribution or commercial use | High | Review runtime and each individual voice separately | Open |
| R-005 | Optional cloud features could create unexpected costs or privacy exposure | High | Keep disabled by default, disclose data flows and provider pricing, require explicit user setup | Open |
| R-006 | Desktop tool execution could be abused through prompt injection or malicious content | Critical | Use a strict allow-list, validate arguments, isolate permissions, and require confirmation | Open |
| R-007 | Product scope may become too large before a usable prototype exists | Medium | Build vertical slices; prioritise offline local chat first | Open |
| R-008 | Code signing and secure update distribution may introduce costs | Medium | Research options early; disclose all costs before selection | Open |
| R-009 | Product name may conflict with existing trademarks | Medium | Conduct trademark clearance before public commercial launch | Open |
| R-010 | Reusing FRIDAY code could create coupling or disrupt its roadmap | Medium | Perform a read-only reuse audit first; do not change FRIDAY without approval | Open |

## 2. Open product questions

- What exact Windows versions should the first release support?
- Should the first-run experience download one recommended model or let users choose from reviewed options?
- What is the acceptable first-run download size?
- Should memory begin as explicit user-saved facts only, with assisted suggestions added later?
- What exact visual style should the 2D companion use?
- How should users create, import, and share custom personalities?
- What are the minimum quality thresholds for Urdu transcription and speech?
- What is the intended commercial model, if the product is eventually sold?
- What update and signing approach is affordable without introducing mandatory recurring costs?

## 3. Technical validation questions

- Is Tauri 2 the best fit after comparing packaging, accessibility, native integrations, and update options?
- Should local model inference run in-process or as a managed subprocess?
- Which compact model provides the best balance of quality, Urdu ability, licence, and memory use?
- Which STT engine best handles Pakistani Urdu and Urdu-English code-switching offline?
- Which TTS engine has a redistributable Urdu voice of acceptable quality?
- What is the full memory footprint with UI, model, context cache, and speech engines?
- What data protection is appropriate for local memory and backups?
- How should optional BYOK credentials be stored on Windows and later on Linux/macOS?

## 4. Cost and licence gates

Before any dependency or service is adopted, record:
- Exact product, package, model, voice, or asset name and version.
- Source and date reviewed.
- Licence and commercial-use permission.
- Redistribution/bundling conditions.
- Attribution or notice requirements.
- Runtime/service costs and usage limits.
- Whether the owner approved any paid commitment.

## 5. Review policy

- Review critical risks before any PC-control feature is enabled.
- Review licence risks before bundling or distributing assets.
- Review cost risks before connecting paid or metered services.
- Close a risk only with documented evidence.
- Add new risks when tests reveal unexpected limitations.
