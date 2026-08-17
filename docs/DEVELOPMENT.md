# Development

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for how the pieces fit together; this page is about working in the codebase day to day.

## Prerequisites

- [Node.js](https://nodejs.org) (npm)
- [Rust](https://www.rust-lang.org/tools/install) via `rustup`
- macOS: Xcode Command Line Tools (`xcode-select --install`)

## Commands

```sh
npm install              # install frontend deps + set up the git hooks (husky)
npm run tauri dev        # launch the app in dev mode (hot-reloads the frontend)
npm run check             # type-check the frontend (svelte-check)
npm run build              # production frontend build only
npm run tauri build        # full native app build/bundle

cargo check --manifest-path src-tauri/Cargo.toml   # type-check the Rust backend
cargo build --manifest-path src-tauri/Cargo.toml    # build the Rust backend
```

Note: `npm run tauri dev` opens a real native window, so it needs a display - it won't do anything useful in a headless/CI shell. `cargo check`/`cargo build` and `npm run check`/`npm run build` all work headlessly and are what to run to verify changes. [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) runs exactly these two headless checks (`npm run check`, `npm run build`, `cargo check`) on every push and PR - if they pass locally, CI passes.

## Project structure

```
src-tauri/src/               backend - see ARCHITECTURE.md#module-layout for the full breakdown
  accounts/                   account CRUD, providers, OAuth
  diagnostics.rs               app resource usage (RAM/CPU/disk)
  chat/                         conversations + SSE-streamed replies via an existing AI account
src/                          frontend - see ARCHITECTURE.md#module-layout for the full breakdown
  app.css                      design tokens
  lib/                          types, invoke() wrappers, shared state, components/
  routes/                       +layout.svelte, +page.svelte (the whole app is one page)
docs/                          this documentation
scripts/version-updaters/       custom bump-file updaters for the release tooling (see RELEASING.md)
.github/workflows/               CI (every push/PR) and Release (on a version tag) - see RELEASING.md
```

## Adding a new provider

This pattern has repeated eight times now (OpenAI, Anthropic, AWS, Azure, GCP, Scaleway, kubeconfig, then GitHub/GitLab/Jira/Confluence/OIDC on top). Because `Account.config`/secret payloads are untyped JSON (see [`ARCHITECTURE.md`](ARCHITECTURE.md#why-configsecret-are-untyped-json)), adding a provider never touches the storage layer or command signatures - it's purely additive.

1. **Backend validator** - create `src-tauri/src/accounts/providers/<name>.rs` with an async `validate(config: &Value, secret: &Value) -> Result<(), String>` that makes one cheap, read-only, authenticated API call. Use `super::{required_str, optional_str}` to pull fields out of the JSON.
2. **Wire it into the dispatcher** - add `mod <name>;` and a match arm in `providers::validate()` in [`providers/mod.rs`](../src-tauri/src/accounts/providers/mod.rs).
3. **(Optional) OAuth support** - if the provider supports interactive sign-in, add `login`/`refresh` functions using the shared [`providers::oauth`](../src-tauri/src/accounts/providers/oauth.rs) module (see GitHub/GitLab/Atlassian for examples of discovery-based vs. hardcoded endpoints), then add match arms in `oauth_login()`/`oauth_refresh()` and list the provider name in `OAUTH_CAPABLE_PROVIDERS`.
4. **Frontend schema** - add an entry to `PROVIDER_SCHEMAS` in [`src/lib/types.ts`](../src/lib/types.ts): `provider`, `category` (add a new `AccountCategory` + label + icon in the same file if it doesn't fit an existing one), `fields`, and `authMethods` if step 3 applies.
5. **Verify**: `cargo check` (backend), `npm run check` (frontend), then `npm run tauri dev` to actually add an account and click Test connection.

No changes are needed to `model.rs`, `store.rs`, `commands.rs`, `AccountForm.svelte`, or `AccountCard.svelte` unless the new provider needs something genuinely new (a new category, a third auth method, etc.) - they're all driven by the schema/dispatch tables above.

## Conventions

- **Commits**: [Conventional Commits](https://www.conventionalcommits.org/), enforced by a `commit-msg` git hook - see [`RELEASING.md`](RELEASING.md).
- **Rust style**: comments explain *why*, not *what*; errors are `Result<_, String>` with human-readable messages (these can end up shown directly in the UI as `last_error`) - never include secret values in an error message.
- **Frontend style**: Svelte 5 runes (`$state`/`$derived`/`$effect`), no external state management library. Components stay presentational; data flow goes through `lib/accounts.ts`.
