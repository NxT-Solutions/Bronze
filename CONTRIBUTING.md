# Contributing

Thanks for your interest in Bronze.

Bronze is a local-first macOS selection-to-action queue. It is not a clipboard
recorder, task manager, note vault, or hosted AI client.

## Development

See `README.md` for toolchain pins and `pnpm --filter desktop tauri dev`.

Maintainer bar:

```bash
pnpm verify
```

Local debug package (not notarized):

```bash
tooling/package-debug.sh
```

## Pull requests

- Keep changes focused.
- Preserve requirement IDs in notes and tests.
- Add a changelog line under `## Unreleased` when behavior changes.
- Do not claim WCAG, VoiceOver, or notarization without `docs/evidence/`.
- ADR-002, ADR-009, ADR-018, and ADR-023 stay Proposed unless a human accepts them.
- Do not start QUEUE-ATTACHMENTS work unless a human names that story.

New work on this repo goes through Hedgehog (`hedgehog status`, `hedgehog next`,
`hedgehog verify`). Do not resume bmad-loop run `6a79`.

## License

Contributions are MIT, same as [LICENSE](LICENSE).

## Release

Maintainers cut a build from `main` with Actions → **Release** and a version such
as `0.1.0`. That workflow tags `v*`, writes `CHANGELOG.md` from commits, builds
an arm64 `.app`, and uploads `bronze-macos.pkg` (not a notarization claim).

When `PACKAGES_REPO_TOKEN` is set, **Publish Homebrew** updates
`Casks/bronze.rb` in `NoahNxT/homebrew-nxt-solutions-packages`. The tap is a
cask, not `Formula/bronze.rb`.
