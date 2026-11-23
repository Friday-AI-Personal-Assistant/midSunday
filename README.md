# midSunday

**AI Personal Companion**

midSunday is an independent, local-first desktop AI companion designed to combine useful assistance with a personal, customisable companion experience.

> Project status: **Planning / foundation**  
> Initial target: **Windows**  
> Core direction: **Offline-capable, privacy-first, no mandatory subscription**

## What we are building

The long-term goal is a desktop companion that can provide:
- Local AI chat using models installed on the user's computer.
- User-controlled local memory and conversation history.
- Local voice input and speech output where supported.
- Customisable personalities and companion characters.
- Carefully permissioned desktop assistance.
- Optional internet-based search and optional cloud model providers.

The product must remain useful without a cloud account. Online features must be clearly identified. No paid service, subscription, hosted backend, domain, certificate, or paid asset may be adopted without explicit owner approval.

## Product principles

1. Local-first by default; no silent transfer of conversations, audio, files, or memory.
2. No hidden costs or mandatory subscriptions.
3. Honest, benchmark-backed hardware and language claims.
4. User control over memory, privacy, permissions, and companion behaviour.
5. Safe, allow-listed computer actions; never arbitrary model-generated shell execution.
6. Commercial licence review for every bundled dependency, model, voice, and creative asset.
7. Independent identity and codebase from FRIDAY.

## Roadmap and project records

- [Complete product roadmap](ROADMAP.md)
- [Project history](docs/PROJECT-HISTORY.md)
- [Product specification](docs/PRODUCT-SPECIFICATION.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Decision log](docs/DECISIONS.md)
- [Risks and open questions](docs/RISKS-AND-OPEN-QUESTIONS.md)

The roadmap is the master sequence of work. These documents provide the detailed requirements and decision history. Update them as the project changes so development can recover without relying on chat history.

## Current status

The repository currently contains planning documentation. **No application code, AI runtime, model, voice engine, installer, or production feature is implemented yet.**

The next work is to complete the product foundation, inspect FRIDAY for reusable engineering patterns, and validate technical choices before committing to implementation.

## Development approach

Development proceeds in small, testable milestones. A task is complete only when its acceptance criteria, tests, documentation, privacy review, and applicable licence review are satisfied.

No paid usage or external service is to be enabled without explicit approval.

## Repository

Private repository: [Friday-AI-Personal-Assistant/midSunday](https://github.com/Friday-AI-Personal-Assistant/midSunday)
