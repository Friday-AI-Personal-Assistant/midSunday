# Cross-platform build workflows

midSunday is being developed as one shared Tauri + React + Rust application with separate native builds for each desktop operating system.

## Current build targets

| Platform | GitHub Actions runner | Package output | Workflow |
|---|---|---|---|
| Windows | `windows-latest` | NSIS `.exe` installer | [Windows prototype build](../.github/workflows/windows-prototype-build.yml) |
| Linux | `ubuntu-24.04` | AppImage and Debian `.deb` | [Linux prototype build](../.github/workflows/linux-prototype-build.yml) |
| macOS | `macos-latest` | `.dmg` disk image | [macOS prototype build](../.github/workflows/macos-prototype-build.yml) |

Each workflow is manual-only (`workflow_dispatch`). No workflow runs automatically on every push or pull request. Standard GitHub-hosted runners are used; larger paid runners are not configured. Artifact retention is seven days. Check current GitHub plan and usage terms before changing runner classes or retention.

## Important limitations

- These workflows prove compilation and packaging only. They do not prove the application launches correctly or behaves correctly on a real computer.
- The macOS workflow builds for the architecture of the selected GitHub-hosted runner. Separate Intel/Apple Silicon builds or a universal binary may require additional configuration.
- Linux packages can vary across distributions and desktop environments. AppImage and Debian packages are initial distribution formats, not a promise of compatibility with every Linux distribution.
- Signing, notarisation, auto-updates, installers, and release publishing are not configured.
- Tauri's Linux build requires system libraries supplied by the runner. The end user's runtime dependencies and packaging experience still need review.
- Platform-specific features, permissions, file paths, hardware detection, voice devices, and PC actions must be reviewed separately.

## How to run

Open the repository's **Actions** tab, choose the desired platform workflow, tap **Run workflow**, select `main`, and confirm. Run one workflow at a time while the prototype is being debugged. Each run uploads any produced package and the generated npm lockfile as an artifact for seven days.

## Development priority

Windows remains the first priority for feature development. Linux and macOS builds are added early to catch platform-specific compile and packaging problems rather than postponing all platform work until after the Windows application is complete. Real-device validation remains a later milestone.
