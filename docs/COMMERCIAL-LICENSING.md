# midSunday — Commercial Licensing Policy

**Status:** Mandatory review policy; no final bundled AI or voice stack has been approved.

## Core rule
"Open source" or "free to download" does not automatically mean an asset can be bundled, redistributed, modified, or used commercially. Review the exact version and exact asset, not just the project name.

## Review every component
The inventory must cover:
- Application frameworks, libraries, and Rust crates.
- Model inference runtimes and native binaries.
- Model weights, adapters, tokenisers, and configuration files.
- Speech recognition models.
- TTS runtimes and each individual voice.
- Character art, animation, icons, fonts, sounds, and music.
- Installer, updater, signing, and distribution tools.
- Optional APIs, hosted services, and integrations.

## Required record per item
Record:
1. Exact name, version/revision, source URL, and retrieval date.
2. Copyright holder and licence text or model terms.
3. Commercial-use permission.
4. Redistribution and bundling permission.
5. Modification and derivative-work conditions.
6. Attribution, notice, source-offer, or share-alike obligations.
7. Use restrictions, acceptable-use terms, and territory limitations.
8. Whether the licence is for code, weights, voice, or another separate asset.
9. Any runtime, hosting, per-use, or subscription costs.
10. Review outcome, reviewer, and approval date.

## Release gates
- Do not bundle an item with unknown or incompatible terms.
- Keep required notices and third-party attributions in the repository and installer.
- Recheck licences when changing versions or assets.
- Do not assume a model's runtime licence covers its weights.
- Do not assume a TTS engine licence covers its voice models.
- Seek legal review for ambiguous or commercially significant restrictions.

## Cost policy
No paid service, API, domain, certificate, hosted backend, or paid asset may be adopted without explicit owner approval. Any recurring or usage-based charge must be disclosed before activation. Free tiers must be assessed for limits, expiry, and commercial-use restrictions.

## Candidate status
llama.cpp/GGUF models, Vosk, whisper.cpp, Piper, Kokoro, SQLite, and other candidates remain unapproved until their exact versions and assets are reviewed. This document is a policy, not a legal opinion or a statement that any candidate is cleared.

## Open questions
- What distribution licence should apply to midSunday's own code?
- Will the product be sold, offered free, or use an optional paid model?
- Which exact models and voices can be redistributed commercially?
- What notices and source-offer obligations apply to the final dependency set?
