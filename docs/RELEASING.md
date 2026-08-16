# Releasing

## Commit convention

Commits follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:`, `docs:`, ...), enforced by [`commitlint`](../commitlint.config.cjs) via a `commit-msg` git hook (installed by `husky` - see [`.husky/commit-msg`](../.husky/commit-msg)). A non-conforming commit message is rejected locally, before it's even created.

- `feat!:` or a `BREAKING CHANGE:` footer marks a breaking change.
- The hook is installed automatically by `npm install` (via the `prepare` script).

## Cutting a release

```sh
npm run release              # bump version, update CHANGELOG.md, commit, tag
npm run release -- --dry-run  # preview without changing anything
npm run release -- --release-as minor   # override the auto-detected bump
git push --follow-tags origin main       # push the release commit and its tag
```

The `npm run release` step runs [`commit-and-tag-version`](https://github.com/absolute-version/commit-and-tag-version), configured in [`.versionrc.cjs`](../.versionrc.cjs):

1. Reads every commit since the last tag and determines the semver bump (features → patch while pre-1.0, breaking changes → minor while pre-1.0, per standard semver-for-0.x behavior).
2. Bumps the version in **four files**, kept in sync via `bumpFiles`:
   - `package.json` / `package-lock.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/Cargo.lock` (just the `my-companion` package's own entry - see [`scripts/version-updaters/cargo-lock.cjs`](../scripts/version-updaters/cargo-lock.cjs))
   - `src-tauri/tauri.conf.json`
3. Writes the new entries to `CHANGELOG.md`.
4. Commits everything as `chore(release): X.Y.Z` and creates the `vX.Y.Z` git tag.

The very first release used `npm run release -- --first-release`, which generates the changelog and tag for the *current* version without bumping it - useful for establishing the first tag on an already-versioned project.

That's the local half. Pushing the tag is what actually publishes anything - see below.

### Why Cargo.lock has its own updater

`Cargo.lock` isn't a simple key-value file - it has one `version = "..."` line per dependency, and only the one directly under `name = "my-companion"` is ours to bump. An earlier version of this setup tried to shell out to `cargo check` in a `postbump` hook to let Cargo regenerate it, then `git add` it separately - but `commit-and-tag-version`'s commit step uses an explicit file list (exactly the files in `bumpFiles`), so a file staged outside that list silently never made it into the release commit. The fix was a dedicated `bumpFiles` updater ([`cargo-lock.cjs`](../scripts/version-updaters/cargo-lock.cjs)) that patches the version in place directly, same mechanism as `Cargo.toml`.

## In-app history

The app's History section ([`lib/changelog.ts`](../src/lib/changelog.ts)) parses `CHANGELOG.md` at build time (bundled via a Vite `?raw` import) and only picks up headings shaped like a real tagged release - `## X.Y.Z (date)`, or `## [X.Y.Z](compare-url) (date)` once there's a previous tag to link to. Anything else in the file (prose, a hand-edited section) is simply not recognized, so what's shown is tag history, not raw commit history. This means the changelog and the in-app History section can never drift apart - they're the same file.

## CI

[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) runs on every push to `main` and every pull request, on a `macos-latest` runner (matching how this app is actually built): `npm run check`, `npm run build`, `cargo check`. No tests exist yet, so this is a type-checking/build-health gate, not a test suite.

## Automated GitHub Release

Pushing a `vX.Y.Z` tag (i.e. the result of `npm run release` followed by `git push --follow-tags`) triggers [`.github/workflows/release.yml`](../.github/workflows/release.yml), which:

1. Checks out that tag.
2. Extracts that version's section from `CHANGELOG.md` via [`scripts/changelog-section.sh`](../scripts/changelog-section.sh) - the same heading-parsing logic the in-app History section uses, so the GitHub Release notes, the CHANGELOG.md entry, and the in-app History entry are all generated from (or read) the same source.
3. Runs `tauri-apps/tauri-action`, which builds the macOS `.app`/`.dmg` in release mode and publishes them to a GitHub Release for that tag - creating the release if it doesn't exist yet, so this fully replaces any manual `gh release create` step.

Can also be triggered manually (`workflow_dispatch`, with a `tag` input) to rebuild and republish an existing tag without cutting a new version.

Local commands, for reference:

```sh
gh release view vX.Y.Z          # see a release's notes and attached assets
gh run list --limit 5            # recent workflow runs
gh run watch <run-id>              # follow a run to completion
```
