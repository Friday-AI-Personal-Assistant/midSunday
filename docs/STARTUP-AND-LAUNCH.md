# Startup and launch-at-sign-in

## Purpose

The Settings → System & hardware section provides an optional **Launch at sign-in** control. When enabled, the operating system registers midSunday to start when the current user signs in. This replaces the earlier non-functional Windows-only placeholder and uses the Tauri v2 autostart integration for Windows, Linux, and macOS.

## User control and default

- Launch at sign-in is **off by default**.
- midSunday checks the operating-system registration state when the app starts; the UI does not assume a preference from an in-memory default.
- The user must explicitly enable the setting. Turning it off asks the OS integration to remove the registration.
- After a change, the UI reads the OS state again and displays the confirmed state. Errors are shown without pretending the change succeeded.
- The setting does not start a local model, enable a cloud provider, or create an internet connection. It only registers the desktop app for sign-in launch.

## Platform scope

The Tauri autostart plugin documents support for Windows, Linux, and macOS. The macOS integration uses the LaunchAgent strategy. Actual login behavior can vary with packaging, desktop environment, OS policy, and installation location, so hosted CI compilation is not proof of successful login startup. Physical-device validation remains required on each supported OS.

## Permissions and privacy

The application grants the plugin's default permission set, which is limited to checking, enabling, and disabling autostart. No arbitrary shell command, startup script, or unrelated operating-system setting is exposed. Registration is local to the current user's OS account; no account data or usage telemetry is transmitted. The feature remains opt-in.

## Dependency and licence review

The implementation uses tauri-plugin-autostart and its JavaScript guest package. The upstream plugin workspace states MIT or MIT/Apache-2.0 licensing; the exact resolved crate/npm versions and transitive dependency notices must still be checked before commercial distribution. See LICENSE-REVIEW.md.

## Validation checklist

- [ ] Fresh install starts with launch-at-sign-in disabled.
- [ ] Enabling registers the app and the UI confirms the OS-reported enabled state.
- [ ] Disabling removes the registration and the UI confirms the disabled state.
- [ ] A failed OS operation produces an error and does not falsely update the displayed state.
- [ ] The setting survives app restarts because the OS registration is authoritative.
- [ ] Login launch works from the installed package on Windows 10/11, common Linux desktop environments, and macOS.
- [ ] Uninstall behavior is checked so stale startup registrations are not left behind.
- [ ] No model runtime, cloud provider, or network feature is started as a side effect.

## Upstream references

- Tauri v2 Autostart plugin: https://v2.tauri.app/plugin/autostart/
- Plugin source and licence: https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/autostart
