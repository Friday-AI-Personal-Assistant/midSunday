# midSunday — Project History

This document preserves the project's origin, discussions, decisions, and rationale so that future work does not depend on any one ChatGPT conversation.

## 1. Origin and identity

- The product name **midSunday** was selected by the owner on 2026-11-22.
- Product descriptor: **AI Personal Companion**.
- The project has its own repository, identity, interface, feature set, and roadmap.
- It is hosted in the Friday-AI-Personal-Assistant GitHub organisation for now; the owner may consider transferring it later.
- The product is not intended to be a clone of Stonic AI or FRIDAY. Existing products may inform research, but midSunday must have its own design and product decisions.

## 2. Product motivation

The owner explored the capabilities and pricing of Stonic AI as a reference point for the kind of desktop companion experience that may be possible. The goal is to create an original product with a clear identity and sustainable design, not to copy another product's branding or implementation.

The owner wants ChatGPT to act as the lead technical collaborator because the owner does not have coding experience. Explanations should be plain-language, and implementation work should be handled by the developer wherever connected tools allow.

## 3. Confirmed direction and priorities

### Product and platform
- Independent desktop AI companion.
- Windows first; Linux and macOS considered later.
- Companion characters and personalities are an important part of the product.
- Commercial sellability is a consideration from the beginning.

### Offline-first and privacy
- Core functions should work locally wherever practical.
- Local chat, voice, memory, and PC assistance are intended core areas.
- Online search and cloud-based features necessarily require internet.
- Optional cloud model providers / BYOK may be considered, but no cloud service should be mandatory.
- Private user data should remain local by default and should not be transmitted without informed user consent.

### Cost
- No hidden costs.
- No mandatory subscription.
- No surprise paid API usage or developer-funded cloud usage.
- Paid services, domains, certificates, assets, and other commitments require explicit owner approval in advance.

### Languages and voice
- Multilingual text and voice are desired.
- Urdu is a high priority, especially Pakistani Urdu, Roman Urdu, and Urdu-English mixed speech.
- Local/free speech recognition and speech synthesis are preferred, subject to quality, hardware, and licence validation.

### Personalities
The owner has identified these personality directions:
- Best friend.
- Girlfriend.
- Boyfriend.
- Mentor.
- Professional assistant.
- Custom/user-created character.

The girlfriend personality may be warm, playful, and flirtatious. Companion modes must remain transparent about being AI and must not use manipulation, coercion, isolation, or dependency pressure. Adult-oriented modes require appropriate age boundaries and safeguards.

### Hardware
- A usable experience on computers with 4 GB RAM is an aspiration.
- This is not yet a verified minimum specification.
- Higher-memory hardware should enable better models and richer features.
- Actual runtime memory and performance must be measured before publishing compatibility claims.

## 4. Technical ideas discussed (not final decisions)

These options were discussed as candidates and must be validated:
- Tauri 2 + Rust + React + TypeScript + Vite for the desktop application.
- llama.cpp and quantised GGUF models for local LLM inference.
- Compact Qwen-family models as candidates for constrained hardware.
- Vosk and/or whisper.cpp for speech recognition.
- Piper and/or Kokoro for speech synthesis.
- SQLite for local memory.
- Browser-opened search as a simple initial online-search path, avoiding a paid search API.

No candidate is approved solely because it appears in this history. Check current versions, performance, model quality, and licences before adopting.

## 5. Relationship with FRIDAY

FRIDAY is a separate personal AI assistant project in the same GitHub organisation. Its architecture and engineering patterns may be inspected for lessons and possible reuse. midSunday must not copy FRIDAY wholesale, merge the products, or make changes to FRIDAY without the owner's approval.

A dedicated reuse analysis must identify:
- Reusable general engineering patterns.
- Components that could be extracted cleanly.
- Components that should remain FRIDAY-specific.
- Licensing, security, and maintenance implications.
- Whether reuse is actually cheaper and safer than independent implementation.

## 6. Conversation and repository continuity

On 2026-11-23, the owner created a separate ChatGPT Project named **midSunday** and added relevant conversations there. The GitHub repository is the durable source of truth; chat history is not the only record.

Important decisions and changes must be transferred into repository documentation. If a conversation becomes unavailable, use the latest repository state, decision log, roadmap, and handoff notes to continue.

## 7. Confirmed vs proposed

This history deliberately distinguishes owner-confirmed requirements from developer proposals. Technical candidates, suggested release stages, and implementation details remain proposals until validated and accepted.

## 8. Timeline

- **2026-11-22:** Product name midSunday selected; independent repository and project direction discussed.
- **2026-11-23:** Separate ChatGPT Project created; complete product roadmap added to the GitHub repository; project foundation documentation started.
