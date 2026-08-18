# Use cases

## Who this is for

Developers and platform engineers who routinely juggle credentials across many services at once: an AI provider key for a side project, an AWS profile for one client and Azure for another, a handful of Kubernetes clusters, a GitHub and a GitLab account, and a Jira/Confluence login for whichever team they're embedded with that week. The common failure mode this app targets: credentials scattered across shell profiles, `.env` files, and browser-saved passwords, with no single place to see what's configured, whether it still works, or when a session is about to expire.

## Core use cases

### Store a credential securely

Add an OpenAI account: pick the provider, paste an API key, name it. The key goes straight into the OS keychain - it's never written to disk in plain text, and the app never displays it again (editing means re-entering it, not viewing the old one).

### Confirm a credential actually works

"Test connection" isn't a format check - it's a real, read-only API call (list models for OpenAI, `GetCallerIdentity` for AWS, list namespaces for a kubeconfig context, and so on). A green status means the credential was just proven to work against the real service, not just that something non-empty was typed into a field.

### Sign in with OAuth instead of managing a token by hand

For GitHub, GitLab, Jira, Confluence, or a generic OIDC provider: choose "Sign in" as the auth method, click it, approve in the system browser, and the app holds a real session (access + refresh token) instead of a long-lived personal access token. No token to copy, store, or rotate by hand.

### Notice and recover from an expiring session

An OAuth account shows its session's expiry right on the card. Once it's passed, the card flags it - one click on "Refresh session" gets a new access token using the stored refresh token, with no browser round-trip needed unless the refresh token itself has been revoked (in which case the app says so plainly, rather than failing silently).

### Switch a GitHub/GitLab/Jira/Confluence account between token and OAuth

These four providers support either auth mode per account. Someone starting with a quick personal access token can later add an OAuth-based account for the same service instead (auth method is fixed per account, chosen at creation - switching means adding a new account with the other method).

### Put a stored AI credential to actual use

An OpenAI or Anthropic account isn't just something to validate and forget - Chat lets you start a real conversation with it directly, streamed token-by-token, without opening a browser tab or copying the key anywhere else. Multiple conversations, each pinned to whichever account (and model) you started it with, persist across restarts.

### Find that thing you asked about last week

Chat's search box finds a past message by what it *means*, not just words it contains - ask "that time I asked about retry backoff" and get the actual conversation, even if you never typed those exact words. Fully offline: no API call, no extra account, no cost per search.

### See everything at a glance, grouped by kind

The sidebar's category list (AI / Cloud / Kubernetes / Source Control / Atlassian / OIDC) answers "what do I have configured for X" in one glance, with a status dot per account so a stale or broken credential is visible before it causes a failure somewhere else.

### Get the one-screen summary

Clicking "my-companion" in the sidebar opens Overview: total account count, a status breakdown (valid / needs attention / untested), and a per-category grid you can click straight into. The answer to "is anything broken right now" without visiting every category individually.

### Make the app look and start the way you want

Settings covers the things that are about *you*, not your accounts: light/dark/system appearance (overriding the OS setting if you want), and which section opens when the app launches - useful if you live in Diagnostics or a specific category rather than the default Accounts view.

### Notice the app itself misbehaving

Diagnostics shows this app's own memory and CPU usage, live, plus how much disk space its data actually takes up - broken down per file (`accounts.json`, `conversations.json`, the search index, the embedding model cache), not just a single total. If the app starts feeling sluggish, or the data folder feels bigger than expected, this is where to check what's actually using it before going looking elsewhere. A second row shows how much Chat/search has actually accumulated: conversation count, message count, and how many of those messages are indexed for search.

### Start over cleanly

Settings' danger zone deletes every account - metadata and keychain secrets both - in one confirmed action. For when a fresh start is genuinely what's wanted, rather than deleting accounts one at a time.

### Know what version you're running, and what changed

The version badge and History section (sourced from the same `CHANGELOG.md` the release tooling generates, and published as real GitHub Releases with built app assets) mean there's never a question of "did I already get the fix for X" - the answer is in the app, not in a separate changelog file someone has to remember to check.
