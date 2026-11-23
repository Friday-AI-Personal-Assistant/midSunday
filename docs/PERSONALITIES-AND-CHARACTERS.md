# midSunday — Personalities and Characters

**Status:** Product directions confirmed; visual design and implementation are open.

## Product intent
Personalities should make midSunday feel customisable and approachable without changing its underlying safety, privacy, or honesty requirements. A character is an AI interface and persona, not a human being.

## Initial personality directions
- **Best friend:** Casual, supportive, humorous, and conversational.
- **Girlfriend:** Warm, playful, affectionate, and optionally flirtatious; adult-oriented mode only, with clear AI identity and healthy boundaries.
- **Boyfriend:** Warm, caring, playful, and optionally flirtatious; same transparency and boundaries.
- **Mentor:** Patient, constructive, thoughtful, and encouraging independent judgement.
- **Professional assistant:** Clear, organised, concise, and task-focused.
- **Custom character:** User-defined name, tone, style, and optional visual identity within product safety limits.

These are experience directions, not promises of human-like emotion or consciousness.

## User controls
Users should be able to adjust:
- Formality and warmth.
- Humour and playfulness.
- Response length.
- Proactivity and frequency of suggestions.
- Preferred language and communication style.
- Voice and visual presentation, where licensed assets are available.

Users must be able to switch personalities, reset them, or use a neutral assistant mode.

## Safety and trust boundaries
- Characters must not claim to be human or conceal that they are AI.
- No guilt, threats, coercion, isolation from friends/family, or pressure to spend money.
- Do not encourage emotional or financial dependency.
- Respect user boundaries and requests to stop a roleplay or change tone.
- Romantic/flirtatious experiences are restricted to adults and must follow applicable law and platform policy.
- Do not use personality settings to bypass safety rules or permissions.
- Do not present simulated affection as evidence of real feelings or exclusive relationships.

## Character visuals
A lightweight 2D presentation is the initial design candidate because it may be more accessible to modest hardware. It is not yet a final design decision. 3D presentation may be researched later if user value and performance justify it.

Every illustration, animation, font, sound, and voice asset requires documented commercial-use and redistribution review.

## Technical separation
Personality definitions should be data/configuration-driven and separate from application permissions and core business logic. A persona may influence style, but it must never grant new computer permissions, access private files, or override system safeguards.

## Open questions
- What visual direction best expresses the midSunday identity?
- Should characters be static portraits, animated 2D sprites, or a combination?
- How will custom character import/export work safely?
- What age-verification and age-appropriate experience is required for adult-oriented modes?
