# Architecture

## System overview

my-companion is a Tauri 2 app: a native window hosting a SvelteKit frontend, backed by a Rust process that owns all account data and every network call. The frontend never talks to a provider's API, a keychain, or the filesystem directly - it only ever calls Tauri commands.

```mermaid
flowchart LR
    subgraph Frontend["Frontend (WebView) - SvelteKit / Svelte 5"]
        Sidebar["Sidebar.svelte\nOverview / Accounts / History / Diagnostics / Settings"]
        UI["Per-section views\nOverview, AccountCard/AccountForm, ChangelogList,\nDiagnostics, Settings"]
        Types["lib/types.ts\nprovider field schemas"]
        Prefs[("localStorage\ntheme, default section")]
        Sidebar --> UI
        UI --> Types
        UI <--> Prefs
    end

    subgraph Backend["Backend - Rust (src-tauri)"]
        Commands["accounts::commands\n#[tauri::command]"]
        Store["accounts::store\nin-memory Vec&lt;Account&gt;"]
        Secrets["accounts::secrets\nkeyring wrapper"]
        Providers["accounts::providers\nvalidate / oauth_login / oauth_refresh"]
        Diagnostics["diagnostics\nresource_usage (sysinfo)"]
        Commands --> Store
        Commands --> Secrets
        Commands --> Providers
    end

    UI -- "invoke()" --> Commands
    UI -- "invoke()" --> Diagnostics
    Commands -- "Account (no secrets)" --> UI
    Diagnostics -- "RAM / CPU / disk footprint" --> UI

    Store -- "accounts.json" --> DataDir[("App data dir")]
    Secrets -- "one entry per account id" --> Keychain[("OS Keychain")]
    Providers -- "HTTPS" --> APIs[("Provider APIs\nOpenAI, AWS, GitHub, ...")]
    Providers -- "system browser" --> Browser[("OAuth authorize page")]
```

## The secrets boundary

This is the single most important design decision in the app: **secret values never cross into the frontend.**

- `Account` (the struct returned by every command) holds only non-secret metadata - `id`, `category`, `provider`, `name`, a `config` object, and status fields. See [`src-tauri/src/accounts/model.rs`](../src-tauri/src/accounts/model.rs).
- Secret values (API keys, tokens, OAuth client secrets, session tokens) live only in the OS keychain, read and written exclusively by [`accounts::secrets`](../src-tauri/src/accounts/secrets.rs). No Tauri command ever returns a secret to the frontend - not even on edit (the edit form re-enters secrets rather than displaying the old one).
- The frontend's `create`/`update` calls *send* secret values in, but never receive them back out.

See [`SECURITY.md`](SECURITY.md) for the full threat-model rationale.

## Why `config`/`secret` are untyped JSON

Rather than a Rust enum per provider (`OpenAiConfig`, `AwsConfig`, ...), `Account.config` and every secret payload are plain `serde_json::Value`. The frontend's [`PROVIDER_SCHEMAS`](../src/lib/types.ts) is the single source of truth for what fields a provider needs; Rust just stores and forwards whatever JSON it's given, and each provider module pulls the specific keys it needs out of that JSON at the point of use (`required_str`/`optional_str` helpers in `providers/mod.rs`).

This means adding a new provider never requires changing the `Account` model, the storage layer, or the Tauri command signatures - only a new `providers/<name>.rs` module and a schema entry in `types.ts`. See [`DEVELOPMENT.md`](DEVELOPMENT.md#adding-a-new-provider) for the concrete steps.

## Module layout

```
src-tauri/src/
  lib.rs                    Tauri app setup, command registration
  accounts/
    mod.rs                  wires the submodules together, init_state()
    model.rs                Account, AccountCategory, AccountStatus, Create/UpdateAccountInput
    store.rs                accounts.json persistence, AccountsState (Mutex<Vec<Account>>)
    secrets.rs               keyring wrapper: set / get / merge / delete
    commands.rs              #[tauri::command] functions - the account API surface,
                              plus app_data_info / reset_all_data for the Settings page
    providers/
      mod.rs                 validate() / oauth_login() / oauth_refresh() dispatch by provider name
      oauth.rs                shared PKCE + loopback-listener + token-exchange machinery
      oidc.rs, gitlab.rs, github.rs, atlassian.rs, jira.rs, confluence.rs
      openai.rs, anthropic.rs, aws.rs, azure.rs, gcp.rs, scaleway.rs, kubeconfig.rs
  diagnostics.rs             resource_usage command (sysinfo) - the app's own RAM/CPU/disk use

src/
  routes/
    +layout.svelte            imports app.css globally, mounts the toast host
    +page.svelte               owns activeSection/activeCategory state, wires Sidebar to the
                                per-section views
  app.css                     design tokens (color/spacing/radius/shadow/motion), light+dark
  lib/
    types.ts                   Account/provider types + PROVIDER_SCHEMAS (drives the dynamic form)
    accounts.ts                 invoke() wrappers (accounts CRUD, test, oauth, data info/reset)
    diagnostics.ts               invoke() wrapper + byte-formatting for resource_usage
    changelog.ts                 parses CHANGELOG.md (bundled via a Vite ?raw import) for History
    settings.svelte.ts            theme + default-landing-section, localStorage-backed
    toast.svelte.ts                notification queue (Svelte 5 module-state pattern)
    components/
      Sidebar.svelte              Overview/Accounts+categories/History/Diagnostics/Settings nav
      Overview.svelte              stats + per-category grid
      AccountCard.svelte           one account: status, test/edit/delete/sign-in/refresh
      AccountForm.svelte           add/edit modal, built dynamically from PROVIDER_SCHEMAS
      ChangelogList.svelte         renders parsed release history
      Diagnostics.svelte            live RAM/CPU/disk, polled every 2s
      Settings.svelte                appearance, default section, data info, reset
      ui/
        Button.svelte, Modal.svelte, Skeleton.svelte, ToastHost.svelte
```

## Storage

Three separate stores, none of them overlapping in what they hold:

- **Metadata** - a single JSON file at `<app data dir>/accounts.json`, loaded into an in-memory `Mutex<Vec<Account>>` at startup ([`store::load`](../src-tauri/src/accounts/store.rs)) and rewritten in full on every create/update/delete ([`store::save`](../src-tauri/src/accounts/store.rs)). Fine at the scale this app deals with (tens of accounts, not thousands).
- **Secrets** - the OS keychain (`keyring` crate; Keychain on macOS), one entry per account under the service name `com.brun_s.my-companion.accounts`. [`secrets::merge`](../src-tauri/src/accounts/secrets.rs) does a shallow merge so an OAuth login can add a `session` sub-object without disturbing a stored `client_secret`.
- **UI preferences** - theme and default landing section live in the browser's `localStorage`, not Rust at all ([`lib/settings.svelte.ts`](../src/lib/settings.svelte.ts)). They're pure display state with no bearing on account data or secrets, so there's no reason for the backend to own them.

## Command surface

Account commands live in [`accounts::commands`](../src-tauri/src/accounts/commands.rs); the one resource-monitoring command lives in [`diagnostics`](../src-tauri/src/diagnostics.rs). All are registered in [`lib.rs`](../src-tauri/src/lib.rs):

| Command | Purpose |
|---|---|
| `list_accounts` | Returns all accounts (metadata only). |
| `create_account` | Creates an account; stores its secret (if any) in the keychain. |
| `update_account` | Updates name/config, optionally replaces the secret; resets status to `unknown`. |
| `delete_account` | Removes both the metadata entry and its keychain secret. |
| `test_account` | Runs the provider's live validation call, updates `status`/`last_error`. |
| `oauth_login` | Runs the authorization-code + PKCE browser flow for an OAuth-capable account. |
| `refresh_oauth_session` | Refreshes a stored OAuth session using its refresh token. |
| `app_data_info` | Read-only: resolved `accounts.json` path + keychain service name, for Settings. |
| `reset_all_data` | Deletes every account's keychain secret and clears `accounts.json`. Irreversible. |
| `diagnostics::resource_usage` | This process's RSS memory, CPU%, and its own on-disk footprint. |

## The OAuth flow

Five providers support interactive sign-in: generic OIDC, GitLab, GitHub, Jira, and Confluence (see [`PROVIDERS.md`](PROVIDERS.md) for which ones also offer a plain-token alternative). They all share one implementation, [`providers::oauth`](../src-tauri/src/accounts/providers/oauth.rs), parameterized on an `Endpoints` struct - either discovered from a `.well-known/openid-configuration` document (generic OIDC, GitLab) or hardcoded (GitHub, Atlassian, which don't publish one).

```mermaid
sequenceDiagram
    participant UI as Frontend
    participant Cmd as oauth_login command
    participant OAuth as providers::oauth
    participant Listener as Local loopback (tiny_http)
    participant Browser as System browser
    participant IdP as Provider (authorize + token endpoint)

    UI->>Cmd: invoke("oauth_login", { id })
    Cmd->>OAuth: login(app, endpoints, client_id, ...)
    OAuth->>Listener: bind 127.0.0.1:0 (OS-assigned port)
    OAuth->>Browser: open authorize URL (PKCE challenge, state)
    Browser->>IdP: user authenticates + approves
    IdP-->>Listener: redirect with ?code&state
    Listener-->>OAuth: authorization code
    OAuth->>IdP: POST token endpoint (code + PKCE verifier)
    IdP-->>OAuth: access/refresh/id token
    OAuth-->>Cmd: session patch {"session": {...}}
    Cmd->>Cmd: secrets::merge + update Account.status/session_expires_at
    Cmd-->>UI: updated Account
```

The listener accepts exactly one request (or times out after 5 minutes), validates the `state` parameter to guard against CSRF, and is torn down immediately after - it only exists for the duration of a single sign-in.

## Frontend architecture

SvelteKit in static-adapter (SPA) mode - `ssr` is disabled (Tauri has no Node server to render against), so the whole app is one client-rendered page. Svelte 5 runes (`$state`, `$derived`, `$effect`) throughout, no external state management library; shared reactive state that isn't tied to one component (toasts, UI preferences) lives in `.svelte.ts` modules using the same runes at module scope.

`lib/types.ts` is the frontend's equivalent of the backend's provider dispatch: `PROVIDER_SCHEMAS` declares every provider's fields (and, where relevant, its `authMethods`), and `AccountForm.svelte` renders itself entirely from that data rather than having a bespoke form per provider.

### Navigation

`+page.svelte` owns a single `activeSection` state (`overview | accounts | history | diagnostics | settings`) and an `activeCategory` state for which of the 6 account categories is showing. `Sidebar.svelte` is purely presentational - it renders whichever section is active and calls back up through props (`onSelectOverview`, `onSelectCategory`, ...) rather than owning navigation state itself.

### Design system

[`app.css`](../src/app.css) defines the whole app's design tokens as CSS custom properties - color (with a full separate dark palette), spacing scale, radius, shadow, and motion timing. Every component reads from these tokens rather than hardcoding values, and dark mode is a token swap, not a per-component concern (see the theme-selection mechanics below).

A handful of reusable primitives in `lib/components/ui/` back most of the interactive chrome instead of every component styling its own button or modal:

- **`Button`** - variant (`primary`/`secondary`/`danger`/`ghost`) x size (`md`/`sm`/`icon`)
- **`Modal`** - shared backdrop/panel/open-close transition, used by `AccountForm`
- **`ToastHost`** + `toast.svelte.ts` - non-blocking notifications (mounted once, in `+layout.svelte`)
- **`Skeleton`** - shimmer loading placeholder

### Theme selection

Three states, matching how the tokens are structured in `app.css`: the OS preference (`prefers-color-scheme`) is the default; `settings.svelte.ts` can override it by setting a `data-theme="light"` or `data-theme="dark"` attribute on `<html>`, persisted to `localStorage`. The CSS layers accordingly - OS-dark tokens apply under `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { ... } }` (so an explicit Light choice beats a dark OS), and `:root[data-theme="dark"]` redefines the same tokens again so an explicit Dark choice wins regardless of the OS setting.
