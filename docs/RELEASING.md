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
```

This runs [`commit-and-tag-version`](https://github.com/absolute-version/commit-and-tag-version), configured in [`.versionrc.cjs`](../.versionrc.cjs). One command:

1. Reads every commit since the last tag and determines the semver bump (features → patch while pre-1.0, breaking changes → minor while pre-1.0, per standard semver-for-0.x behavior).
2. Bumps the version in **four files**, kept in sync via `bumpFiles`:
   - `package.json` / `package-lock.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/Cargo.lock` (just the `my-companion` package's own entry - see [`scripts/version-updaters/cargo-lock.cjs`](../scripts/version-updaters/cargo-lock.cjs))
   - `src-tauri/tauri.conf.json`
3. Writes the new entries to `CHANGELOG.md`.
4. Commits everything as `chore(release): X.Y.Z` and creates the `vX.Y.Z` git tag.

The very first release used `npm run release -- --first-release`, which generates the changelog and tag for the *current* version without bumping it - useful for establishing the first tag on an already-versioned project.

### Why Cargo.lock has its own updater

`Cargo.lock` isn't a simple key-value file - it has one `version = "..."` line per dependency, and only the one directly under `name = "my-companion"` is ours to bump. An earlier version of this setup tried to shell out to `cargo check` in a `postbump` hook to let Cargo regenerate it, then `git add` it separately - but `commit-and-tag-version`'s commit step uses an explicit file list (exactly the files in `bumpFiles`), so a file staged outside that list silently never made it into the release commit. The fix was a dedicated `bumpFiles` updater ([`cargo-lock.cjs`](../scripts/version-updaters/cargo-lock.cjs)) that patches the version in place directly, same mechanism as `Cargo.toml`.

## In-app history

The app's History section ([`lib/changelog.ts`](../src/lib/changelog.ts)) parses `CHANGELOG.md` at build time (bundled via a Vite `?raw` import) and only picks up headings shaped like a real tagged release - `## X.Y.Z (date)`, or `## [X.Y.Z](compare-url) (date)` once there's a previous tag to link to. Anything else in the file (prose, a hand-edited section) is simply not recognized, so what's shown is tag history, not raw commit history. This means the changelog and the in-app History section can never drift apart - they're the same file.

## No remote configured (yet)

This repository has no `git remote` set up - releases are local-only (commits + tags), nothing is pushed. `npm run release`'s own output suggests `git push --follow-tags origin main` as the next step once a remote exists.
