# Security

## The secrets boundary

The core guarantee: **secret values never cross from Rust into the frontend.**

- Every Tauri command that returns account data returns the `Account` struct, which holds only non-secret metadata - see [`model.rs`](../src-tauri/src/accounts/model.rs). There is no command that reads a secret out of the keychain and hands it to the frontend.
- Creating or updating an account *sends* a secret value in (from a form field, over the same in-process Tauri IPC channel every other command uses - it never touches the network until a provider call is made), but the response never echoes it back.
- Editing an account with a secret field shows it blank, not pre-filled - leaving it blank means "keep the existing value" ([`update_account`](../src-tauri/src/accounts/commands.rs) only overwrites the secret if a new value was actually provided).
- Deleting an account purges both the metadata entry and its keychain entry ([`secrets::delete`](../src-tauri/src/accounts/secrets.rs)) - nothing is left behind.

## Storage

- **Metadata** (`accounts.json` in the app data directory) is plain JSON, readable by anything with filesystem access to that user account - by design, since it contains no secrets, only things like provider names, regions, and issuer URLs.
- **Secrets** live in the OS keychain (Keychain Access on macOS) via the `keyring` crate, one entry per account `id`, under the service name `com.brun_s.my-companion.accounts`. The OS keychain, not this app, is what actually gates access - reading a secret triggers whatever OS-level prompt/authorization the keychain is configured to require.

## Error messages

Provider validators return `Result<(), String>` and that string can end up shown directly in the UI as `last_error`. Every validator is written to describe *what failed* (an HTTP status, a missing field, "session token rejected - sign in again") without ever including the secret value itself - errors come from HTTP status codes and library error `Display` output, never from echoing back what was sent.

## OAuth flow protections

The shared OAuth implementation ([`providers/oauth.rs`](../src-tauri/src/accounts/providers/oauth.rs)) used by OIDC/GitLab/GitHub/Jira/Confluence:

- **PKCE** (`code_challenge`/`code_verifier`, S256) on every flow, regardless of whether the provider requires it - the local redirect target is a loopback port, not a registered HTTPS URL, so PKCE is the meaningful protection against authorization-code interception here.
- **CSRF state**: a random `state` value is generated per login attempt and checked against the callback; a mismatch aborts the flow with an explicit error rather than silently proceeding.
- **Ephemeral local listener**: the redirect target is `http://127.0.0.1:<OS-assigned port>/callback` - a fresh port each time, bound only for the duration of one sign-in attempt (5-minute timeout), then closed. It never listens on a fixed or predictable port.
- **Client secrets**, where a provider requires one (GitHub OAuth Apps, Atlassian 3LO), are stored the same way as any other secret - in the keychain, never in `config`.

## Settings and Diagnostics commands

Two commands added for the Settings page touch data but never secrets:

- `app_data_info` returns the resolved `accounts.json` path and the keychain service name - filesystem/keychain *locations*, not their contents. Useful for troubleshooting ("where did my data go"), not a disclosure risk on its own.
- `reset_all_data` is destructive (deletes every account's keychain secret and clears `accounts.json`) but requires the frontend to have already confirmed with the user via the same native dialog used for single-account delete - there's no silent or automatic path to it.

`diagnostics::resource_usage` (the Diagnostics page) reports this process's own memory/CPU usage and the byte size of `accounts.json` and the app binary - sizes only, never content, and nothing about other processes or the system as a whole.

## Tauri capability scoping

[`src-tauri/capabilities/default.json`](../src-tauri/capabilities/default.json) grants only what's actually used: `core:default` (window/app basics, including `getVersion()`), `opener:default` (needed to launch the system browser for OAuth), and `dialog:default` (native confirm dialogs - see below). No filesystem, shell-execution, or arbitrary-HTTP capability is granted to the frontend; all of that happens in Rust, behind the command surface.

## A concrete lesson from building this

Early on, account deletion used the browser's native `window.confirm()`, which does nothing in a Tauri WKWebView by default - it silently fails without showing a dialog, since Tauri doesn't wire up the WebView's confirm/alert delegate methods out of the box. The fix was `@tauri-apps/plugin-dialog`, which shows a real native dialog. Worth knowing if you're tempted to reach for a browser-native confirm/alert/prompt anywhere else in this app - it won't behave the way it does in a regular browser tab.
