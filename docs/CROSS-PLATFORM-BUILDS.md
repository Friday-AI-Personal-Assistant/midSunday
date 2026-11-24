# Cross-platform build workflows

midSunday is being developed as one shared Tauri + React + Rust application with separate native builds for each desktop operating system.

## Current build targets

| Platform | GitHub Actions runner | Package output | Workflow |
|---|---|---|---|
| Windows | `windows-latest` | NSIS `.exe` installer | [Windows prototype build](../.github/workflows/windows-prototype-build.yml) |
| Linux | `ubuntu-24.04` | AppImage and Debian `.deb` | [Linux prototype build](../.github/workflows/linux-prototype-build.yml) |
| macOS | `macos-latest` | `.dmg` disk image | [macOS prototype build](../.github/workflows/macos-prototype-build.yml) |

All three workflows run automatically on pushes to `main` and pull requests targeting `main`. They also retain `workflow_dispatch` for an on-demand build. Windows, Linux, and macOS jobs are independent and can run in parallel; a failure on one platform does not block the other two.

The workflows use standard GitHub-hosted runners, not larger runners. GitHub currently states that standard hosted runners are free and unlimited for public repositories. Larger runners are billed. Automatic push/PR runs do not upload installers or artifacts; this avoids filling the repository's artifact-storage allowance with every CI run. Manual runs upload available packages and the generated npm lockfile for seven days. Keep artifact retention short and review GitHub's current storage limits before changing it.

## Important limitations

- These workflows prove compilation and packaging only. They do not prove the application launches correctly or behaves correctly on a real computer.
- The macOS workflow builds for the architecture of the selected GitHub-hosted runner. Separate Intel/Apple Silicon builds or a universal binary may require additional configuration.
- Linux packages can vary across distributions and desktop environments. AppImage and Debian packages are initial distribution formats, not a promise of compatibility with every Linux distribution.
- Signing, notarisation, auto-updates, installers, and release publishing are not configured.
- Tauri's Linux build requires system libraries supplied by the runner. The end user's runtime dependencies and packaging experience still need review.
- Platform-specific features, permissions, file paths, hardware detection, voice devices, and PC actions must be reviewed separately.

## How to run

Normal development is automatic: pushing a commit to `main` starts all three platform workflows, and opening/updating a pull request targeting `main` starts the same checks. The three operating-system builds run independently and in parallel when GitHub capacity is available.

For an on-demand package artifact, open the repository's **Actions** tab, choose a platform workflow, tap **Run workflow**, select `main`, and confirm. Manual runs upload any produced package and the generated npm lockfile as an artifact for seven days. Automatic CI runs validate the builds but deliberately do not retain installers.

## Development priority

Windows remains the first priority for feature development. Linux and macOS builds run alongside it to catch platform-specific compile and packaging problems early. CI compilation is not real-device validation; launch, installation, permissions, hardware, and runtime behaviour still need testing on actual computers.
