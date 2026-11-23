# midSunday — Privacy, Security, and Safety Requirements

**Status:** Mandatory product requirements; detailed threat model and implementation remain open.

## Privacy defaults
- Local storage by default for conversations, memory, settings, and local activity.
- No account required for core offline use.
- No conversation, audio, file, or memory transfer to cloud services by default.
- No hidden analytics, advertising trackers, or content collection.
- Clearly identify network-dependent features and the destination/provider before data leaves the device.
- Provide controls to disable optional network features.

## User control over data
Users should be able to:
- Inspect and manage conversation history.
- Inspect, edit, and delete individual memory records.
- Disable long-term memory.
- Clear history separately from memory.
- Export personal data in a documented format.
- Back up and restore data with clear warnings.
- Choose storage locations where practical.
- Understand what is and is not included in an export or backup.

Deletion must explain any backups or copies that remain until overwritten or explicitly removed.

## Security architecture
- Treat all model output, imported content, web pages, and documents as untrusted.
- Validate all frontend-to-backend inputs.
- Use least privilege for OS access.
- Keep secrets out of source code, logs, crash reports, and diagnostics.
- Use OS credential storage for optional provider keys where supported.
- Define encryption and key-recovery requirements before storing sensitive data.
- Apply dependency review, vulnerability monitoring, and secure update practices.
- Do not claim security certification or an independent audit unless one has actually occurred.

## Desktop action safety
- The model must not execute arbitrary shell or terminal commands.
- Expose explicit allow-listed operations through a permission broker.
- Validate action names, arguments, file paths, and scope in the native backend.
- Require confirmation for destructive, external, financial, or privacy-sensitive actions.
- Show a preview where an action changes or sends user data.
- Provide cancellation and recovery options where practical.
- Web content and prompt injection must never be allowed to grant permissions.

## Cloud and BYOK
- Optional cloud providers must be opt-in and off by default.
- Explain exactly what context is sent, to which provider, and possible provider charges.
- Never silently fall back from a local model to a cloud model.
- Store keys securely and provide a clear disconnect/delete control.
- Provider terms and privacy policies must be reviewed before integration.

## Companion wellbeing
Characters must be transparent AI entities and must not manipulate users, encourage isolation, or maximise dependency. Users can reset or disable a character experience.

## Incident response
Maintain a security contact and a process for triaging, fixing, and disclosing vulnerabilities. Do not include real user data in issue reports. Provide a way to export diagnostics with sensitive fields excluded by default and user review before sharing.

## Open questions
- What local database encryption model and recovery experience are appropriate?
- Which actions require confirmation in every case?
- What is the security boundary for file access and browser interaction?
- What is the update-signing and rollback strategy?
