# my-companion

A desktop app for managing accounts and credentials across the tools developers juggle daily: AI providers, cloud platforms, Kubernetes clusters, source control, and issue tracking - one place to store, validate, and (where supported) sign into all of them.

Built with [Tauri 2](https://tauri.app) (Rust backend, native window) and [SvelteKit](https://svelte.dev) (Svelte 5, TypeScript). [![CI](https://github.com/brunseba/my-companion/actions/workflows/ci.yml/badge.svg)](https://github.com/brunseba/my-companion/actions/workflows/ci.yml)

## Features

- **12 providers across 6 categories**: AI (OpenAI, Anthropic), Cloud (AWS, Azure, GCP, Scaleway), Kubernetes (kubeconfig contexts), Source Control (GitHub, GitLab), Atlassian (Jira, Confluence), and generic OIDC.
- **Secrets in the OS keychain** - never written to disk in plain text, never sent back to the frontend after creation.
- **Live validation** ("Test connection") - a real, read-only API call per provider, not just a stored-credential check.
- **OAuth sign-in** for GitHub, GitLab, Jira, Confluence, and OIDC - full authorization-code + PKCE flow via the system browser, with session refresh. GitHub/GitLab/Jira/Confluence also support plain token auth as an alternative, chosen per account.
- **Overview**: total account count, a status breakdown, and a per-category grid - one click from anywhere via the sidebar.
- **Diagnostics**: the app's own live memory/CPU usage and on-disk footprint, not the whole system.
- **Settings**: appearance (light/dark/system), which section opens on launch, where your data actually lives, and a reset-everything danger zone.
- **Version + release history** in the app itself, sourced from the same `CHANGELOG.md` the release tooling generates - and published as real [GitHub Releases](https://github.com/brunseba/my-companion/releases) with built `.app`/`.dmg` assets on every version bump.

See [`docs/USE_CASES.md`](docs/USE_CASES.md) for the scenarios this is built around, and [`docs/PROVIDERS.md`](docs/PROVIDERS.md) for the full provider reference.

## Getting started

Prerequisites: [Node.js](https://nodejs.org) and [Rust](https://www.rust-lang.org/tools/install) (via `rustup`). On macOS, Xcode Command Line Tools too.

```sh
npm install
npm run tauri dev    # launch the app in dev mode
```

Other useful commands:

```sh
npm run check         # type-check the frontend (svelte-check)
npm run build          # production frontend build
npm run tauri build    # full native app build/bundle
cargo check --manifest-path src-tauri/Cargo.toml   # type-check the Rust backend
```

## Documentation

- [**Use cases**](docs/USE_CASES.md) - who this is for and the scenarios it's built around
- [**Architecture**](docs/ARCHITECTURE.md) - how the frontend/backend split, storage, and OAuth flow work
- [**Providers**](docs/PROVIDERS.md) - full reference of every supported provider and auth method
- [**Development**](docs/DEVELOPMENT.md) - project structure, and how to add a new provider
- [**Security**](docs/SECURITY.md) - the secrets boundary, keychain usage, and OAuth flow protections
- [**Releasing**](docs/RELEASING.md) - Conventional Commits, semver, and the release process

## License

[MIT](LICENSE)
