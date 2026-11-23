# midSunday — Voice and Language Requirements

**Status:** Requirements confirmed; engines, models, and voices not selected.

## Language priorities
1. English.
2. Urdu, including Pakistani Urdu vocabulary and pronunciation.
3. Roman Urdu.
4. Urdu-English code-switching within a sentence.
5. Additional languages based on demand and validated assets.

## Speech recognition (STT)
Evaluate candidates using real-world samples and consented test data. Test:
- Pakistani Urdu accents and regional variation.
- Urdu script transcription.
- Roman Urdu output where practical.
- Mixed Urdu-English sentences and technical vocabulary.
- English accents common among intended users.
- Quiet and noisy environments, different microphones, and low-volume speech.

Do not claim Urdu support based only on a model's language list. Record word error rate or another suitable quality measure, test-set composition, latency, memory, and failure cases.

## Speech synthesis (TTS)
Evaluate:
- Naturalness and intelligibility in English and Urdu.
- Pakistani Urdu pronunciation and common names.
- Urdu script handling and code-switching.
- Voice quality at different speeds.
- CPU use, memory, startup time, and offline operation.
- Voice-specific commercial redistribution terms and attribution.

A TTS runtime licence does not automatically grant rights to redistribute every voice model.

## Interaction requirements
- Begin with push-to-talk.
- Show a clear microphone-active indicator.
- Provide explicit microphone permission and device selection.
- Allow users to stop playback and interrupt speech.
- Provide editable transcripts before sending where practical.
- Hands-free/wake-word mode is a later feature and requires a separate privacy review.
- No audio may be sent to a remote service without explicit disclosure and consent.

## Candidate engines
Vosk, whisper.cpp, Piper, and Kokoro are research candidates only. Each exact release, runtime, model, and voice must be reviewed for language quality, performance, maintenance, and commercial distribution rights before adoption.

## Test governance
Maintain a small, representative, permissioned evaluation set. Avoid retaining personal recordings unnecessarily. Document consent, anonymisation, storage, and deletion. Never use private user audio as a training dataset without separate informed permission.

## Release labels
Use precise labels such as "English offline speech recognition tested" or "Urdu transcription experimental". Do not use a broad "multilingual voice" claim unless each advertised language has passed defined quality and hardware tests.

## Open questions
- What quality threshold is acceptable for Pakistani Urdu STT and TTS?
- Should speech output support Urdu script and Roman Urdu independently?
- Which voices have clear commercial redistribution rights?
- What is the best UX for correcting mixed-language transcripts?
