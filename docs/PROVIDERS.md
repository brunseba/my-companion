# Provider reference

Every provider is defined in two places that must stay in sync: the field schema in [`src/lib/types.ts`](../src/lib/types.ts) (what the add/edit form shows) and a validator/login module under [`src-tauri/src/accounts/providers/`](../src-tauri/src/accounts/providers/) (what "Test connection" and OAuth sign-in actually do). This page documents both.

Fields marked **secret** are stored in the OS keychain, never on disk and never sent back to the frontend. Fields without that marker are stored as plain (non-secret) metadata.

## AI

### OpenAI
- **Fields**: API key (secret, required), Organization ID, Base URL override (default `https://api.openai.com/v1`)
- **Test connection**: `GET {base_url}/models` with `Authorization: Bearer <api_key>` (+ `OpenAI-Organization` header if set)

### Anthropic
- **Fields**: API key (secret, required), Base URL override (default `https://api.anthropic.com`)
- **Test connection**: `GET {base_url}/v1/models` with `x-api-key: <api_key>`, `anthropic-version: 2023-06-01`

## Cloud

### AWS
- **Fields**: Region (default `us-east-1`), Access key ID (secret, required), Secret access key (secret, required), Session token (secret, optional)
- **Test connection**: `sts:GetCallerIdentity` via `aws-sdk-sts`, using the stored credentials as a static `Credentials` provider

### Azure
- **Fields**: Tenant ID (required), Subscription ID, Client ID (secret, required), Client secret (secret, required)
- **Test connection**: client-credentials token acquisition against `https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token` (scope `https://management.azure.com/.default`) - proves the service principal can get a token, without calling any resource API

### Google Cloud
- **Fields**: Project ID, Service account JSON (secret, required, textarea)
- **Test connection**: signs a JWT with the service account's private key (RS256) and exchanges it for an access token at `https://oauth2.googleapis.com/token`

### Scaleway
- **Fields**: Region (default `fr-par`), Project ID, Access key (secret, required), Secret key (secret, required)
- **Test connection**: `GET https://api.scaleway.com/iam/v1alpha1/api-keys/{access_key}` with `X-Auth-Token: <secret_key>`

## Kubernetes

### Kubeconfig context
- **Fields**: Kubeconfig path (required, e.g. `~/.kube/config`), Context name (required), Default namespace
- **Test connection**: lists namespaces (capped at 1 result) via the real `kube` client - handles TLS certs and `exec`-based auth plugins (`aws-iam-authenticator`, `gke-gcloud-auth-plugin`, ...) exactly like `kubectl` would

## Source Control

Both support either auth method, chosen per account (stored as `config.auth_method`).

### GitHub
- **Common field**: Enterprise Server URL (optional - github.com is used if blank; when set, the API base becomes `{base_url}/api/v3`)
- **Token**: Personal access token (secret, required)
  - Validated via `GET {api_base}/user` with `Authorization: Bearer <token>`
- **OAuth**: Client ID (required), Client secret (secret, required)
  - Fixed endpoints (`github.com/login/oauth/{authorize,access_token}`), scope `read:user repo`
  - Session validated the same way as the token path, using the stored access token

### GitLab
- **Common field**: Instance URL (default `https://gitlab.com`)
- **Token**: Personal access token (secret, required)
  - Validated via `GET {base_url}/api/v4/user` with `PRIVATE-TOKEN: <token>`
- **OAuth**: Client ID (required), Client secret (secret, optional - GitLab supports public/native OAuth apps)
  - GitLab publishes a real OIDC discovery document, so this reuses the same discovery-based flow as generic OIDC below, scope `read_api read_user`

## Atlassian

Jira and Confluence share the same OAuth app registration shape and the same Basic-auth token scheme - see [`providers/atlassian.rs`](../src-tauri/src/accounts/providers/atlassian.rs). Both support either auth method per account.

### Jira
- **Common field**: Site URL (required, e.g. `https://yoursite.atlassian.net`)
- **Token**: Email (secret, required) + API token (secret, required) - Atlassian Cloud's standard Basic-auth API token scheme
  - Validated via `GET {base_url}/rest/api/3/myself`
- **OAuth**: Client ID (required), Client secret (secret, required)
  - Fixed endpoints (`auth.atlassian.com`), scope `read:jira-user read:jira-work offline_access`
  - Session validated via `GET https://api.atlassian.com/oauth/token/accessible-resources` (Atlassian's 3LO flow has no plain userinfo endpoint)

### Confluence
- Same shape as Jira, different scopes (`read:confluence-user read:confluence-content.all offline_access`) and whoami path (`/wiki/rest/api/user/current`)

## OIDC

### Generic OIDC provider
- **Fields**: Issuer URL (required), Client ID (required), Client secret (secret, optional), Scopes (default `openid profile email`)
- OAuth-only - there's no token-based alternative for this category
- Endpoints discovered from `{issuer_url}/.well-known/openid-configuration`
- **Test connection**: reachability check on the discovery document, plus - if a session is stored - a live check against the discovered `userinfo_endpoint`

## Auth method storage

For the four providers that support both modes, the chosen method is stored as `config.auth_method` (`"token"` or `"oauth"`) - non-secret, so it's visible in `accounts.json` and drives both the frontend's `isOAuthAccount()` check (whether to show Sign in/Refresh) and the backend's provider dispatch (which validator to run).
