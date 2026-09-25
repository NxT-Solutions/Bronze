# Contributing

Thanks for your interest in Bronze.

Bronze is a local-first macOS selection-to-action queue. It is not a clipboard
recorder, task manager, note vault, or hosted AI client.

## Development

See `README.md` (Develop) for the first-time macOS setup: Xcode, `rust-toolchain.toml`, Node 24.21.0, pnpm 12.4.2, CMake, clone, `pnpm install`, `pnpm --filter desktop tauri dev`, `pnpm verify`, and the GitNexus index command.

Maintainer bar:

```bash
pnpm verify
```

Pull requests must stay green on `.github/workflows/ci.yml` (JS, docs, portable Rust, macOS Rust except `bronze-desktop`, and WebView screenshot baselines). That is not a notarization or WCAG claim.

Local debug package (not notarized):

```bash
tooling/package-debug.sh
```

## Pull requests

- Keep changes focused.
- When a pull request adds or changes a screen, control, or layout, update the visual scenario and baselines in that pull request (`pnpm visual:update`). See README Develop.
- Preserve requirement IDs in notes and tests.
- Add a changelog line under `## Unreleased` when behavior changes.
- Do not claim WCAG, VoiceOver, or notarization without `docs/evidence/`.
- ADR-002 is Accepted (split arm64 and Intel packages; the operator asked for the Intel build). ADR-009, ADR-018, and ADR-023 stay Proposed unless a human accepts them.
- Do not start QUEUE-ATTACHMENTS work unless a human names that story.

New work on this repo goes through Hedgehog (`hedgehog status`, `hedgehog next`,
`hedgehog verify`). Do not resume bmad-loop run `6a79`.

## License

Contributions are MIT, same as [LICENSE](LICENSE).

## Release

Land the version in `apps/desktop` plus a `CHANGELOG.md` section (`## v0.1.0 - YYYY-MM-DD`) on `main` first. Put plain sentences under `### What's new` in that section. The update dialog shows those sentences. Commit subjects stay under `### Changes`, and the GitHub release page keeps the generated commit list after the plain block. Then cut a build from Actions → **Release** with that version. The workflow reuses tag `v*`, builds `aarch64-apple-darwin` on `macos-15` and `x86_64-apple-darwin` on `macos-15-intel`, and uploads `bronze-macos-arm64.pkg` and `bronze-macos-x86_64.pkg` (not a notarization claim). It does not push `main`.

When `PACKAGES_REPO_TOKEN` is set, **Publish Homebrew** updates
`Casks/bronze.rb` in `NxT-Solutions/homebrew-nxt-solutions-packages` with
`on_arm` / `on_intel`. The tap is a cask, not `Formula/bronze.rb`. Do not
protect the tap with required pull requests.
