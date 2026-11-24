# Windows prototype build workflow

This workflow builds the first midSunday Tauri desktop-shell prototype on a GitHub-hosted Windows runner.

## Trigger

It is deliberately **manual only** (workflow_dispatch). Commits and pull requests do not start builds automatically.

To start it:
1. Open the repository on GitHub.
2. Select **Actions**.
3. Select **Windows prototype build**.
4. Press **Run workflow**, choose the main branch, and confirm.

Do not run repeatedly while diagnosing one failure; inspect the first run's logs first.

## What it does

- Checks out the repository.
- Sets up Node.js 22.
- Installs the Rust MSVC toolchain.
- Installs frontend dependencies.
- Builds the React/Vite frontend.
- Builds the Tauri Windows application and NSIS installer.
- Uploads the installer as a workflow artifact for 7 days.

The workflow does not publish a release, deploy anything, upload source code elsewhere, run AI models, or invoke paid third-party services. It uses GitHub-hosted Actions minutes and artifact storage, which are subject to the repository/account's current included quota and billing settings. Check GitHub Actions usage before starting a run; disable Actions or stop here if GitHub indicates that charges would apply.

## Current limitations

- The workflow was manually attempted twice, but both attempts failed during GitHub Actions setup before checkout or compilation because the pinned artifact-upload action v3 had been deprecated. The workflow now pins the official v4 action. No frontend, Rust, or installer build has completed.
- A successful compile would not prove that the app has been tested on a user's PC.
- The first build may expose missing configuration or dependency issues; fix those before treating the installer as test-ready.
- The dependency install currently uses the package ranges in package.json; a committed lockfile should be generated and reviewed before repeatable release builds.
- The NSIS installer is a development artifact, not a signed public release.
- WebView2 installation behaviour, startup, idle memory, accessibility, and offline installation still need separate validation.

## Artifact retrieval

After a successful run, open its summary and download the artifact named midsunday-windows-prototype-installer. GitHub keeps it for 7 days.
