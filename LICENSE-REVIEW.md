# Licence Review Register

This register tracks licence checks for code, models, voices, and creative assets considered for midSunday. An entry marked "Pending" is not cleared for bundling or commercial distribution.

## Review status definitions
- **Pending:** Not yet reviewed for the exact version/asset.
- **In review:** Licence and terms are being checked.
- **Approved:** Exact item/version reviewed and documented for the intended use.
- **Rejected:** Terms are incompatible or unclear enough that the item will not be used.
- **Re-review required:** Version, asset, licence, or intended use changed.

## Candidate register

| Component / asset | Intended role | Exact version or asset | Status | Notes |
|---|---|---|---|---|
| Tauri 2 | Desktop shell | Not selected | Pending | Review framework and bundled dependency notices |
| Rust crates | Native backend | Not selected | Pending | Review each crate and transitive dependency |
| React / TypeScript / Vite | UI and build | Not selected | Pending | Review exact package versions and notices |
| llama.cpp | Local inference runtime | Not selected | Pending | Runtime licence separate from model weights |
| GGUF model weights | Local language model | Not selected | Pending | Review exact model card and redistribution terms |
| Vosk | Speech recognition candidate | Not selected | Pending | Runtime, model, and language asset review |
| whisper.cpp | Speech recognition candidate | Not selected | Pending | Runtime and model asset review |
| Piper | Speech synthesis candidate | Not selected | Pending | Runtime licence and each voice licence separately |
| Kokoro | Speech synthesis candidate | Not selected | Pending | Exact runtime, weights, and voice terms required |
| SQLite | Local database candidate | Not selected | Pending | Review distribution notices and build configuration |
| Character artwork | Companion visuals | Not selected | Pending | Require commercial and redistribution rights |
| Fonts, icons, sounds | UI assets | Not selected | Pending | Record source and attribution per asset |

## Approval checklist
- [ ] Exact version, revision, and source recorded.
- [ ] Full licence or terms archived/referenced.
- [ ] Commercial use permitted.
- [ ] Redistribution/bundling permitted.
- [ ] Attribution and notice requirements recorded.
- [ ] Restrictions and obligations understood.
- [ ] Runtime and usage costs identified.
- [ ] Owner approval obtained where costs or material obligations apply.
- [ ] Third-party notices prepared for the product.
- [ ] Review date and reviewer recorded.

## Important
This register is not legal advice and does not grant permission to use any item. Do not bundle or distribute candidates until their exact terms have been reviewed and the status is updated.
