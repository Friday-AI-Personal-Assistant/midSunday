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
- Uploads the NSIS installer (if the build succeeds) and generated `package-lock.json` as a workflow artifact for 7 days, even if a later build step fails.

The workflow does not publish a release, deploy anything, upload source code elsewhere, run AI models, or invoke paid third-party services. The repository is public and the workflow uses the standard `windows-latest` GitHub-hosted runner; GitHub documents standard hosted runners in public repositories as free. Do not change this to a larger runner, which can be paid. Artifact storage is a separate consideration, so outputs are limited to the installer and lockfile with a 7-day retention period. See [GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners) and [Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions).

## Current limitations

- The workflow was manually attempted twice, but both attempts failed during GitHub Actions setup before checkout or compilation because the pinned artifact-upload action v3 had been deprecated. The workflow now pins the official v4 action. No frontend, Rust, or installer build has completed.
- A successful compile would not prove that the app has been tested on a user's PC.
- The first build may expose missing configuration or dependency issues; fix those before treating the installer as test-ready.
- The dependency install currently uses the package ranges in package.json. The workflow now retains the generated lockfile as an artifact so it can be reviewed and committed before repeatable release builds.
- The NSIS installer is a development artifact, not a signed public release.
- WebView2 installation behaviour, startup, idle memory, accessibility, and offline installation still need separate validation.

## Artifact retrieval

After a run, open its summary and download the artifact named `midsunday-windows-prototype-build`. It contains the generated `package-lock.json` and, if compilation succeeds, the NSIS installer. GitHub keeps it for 7 days.
