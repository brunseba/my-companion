<script lang="ts">
  import type { Account } from "../types";
  import { schemaFor } from "../types";
  import { testAccount, oidcLogin, refreshOidcSession } from "../accounts";

  interface Props {
    account: Account;
    onEdit: () => void;
    onDelete: () => void;
    onTested: (account: Account) => void;
  }

  let { account, onEdit, onDelete, onTested }: Props = $props();

  const providerLabel = $derived(schemaFor(account.provider)?.label ?? account.provider);
  const isOidc = $derived(account.provider === "oidc");
  const sessionExpired = $derived(
    account.session_expires_at !== null && new Date(account.session_expires_at).getTime() < Date.now(),
  );

  let busy = $state(false);

  async function run(action: () => Promise<Account>) {
    busy = true;
    try {
      const updated = await action();
      onTested(updated);
    } catch (e) {
      onTested({ ...account, status: "error", last_error: String(e) });
    } finally {
      busy = false;
    }
  }
</script>

<div class="card">
  <div class="row">
    <div class="info">
      <span class="status {account.status}" title="Status: {account.status}"></span>
      <div>
        <p class="name">{account.name}</p>
        <p class="provider">{providerLabel}</p>
      </div>
    </div>
    <div class="actions">
      {#if isOidc}
        {#if account.session_expires_at}
          <button class="secondary" onclick={() => run(() => refreshOidcSession(account.id))} disabled={busy}>
            {busy ? "Refreshing…" : "Refresh session"}
          </button>
        {:else}
          <button class="secondary" onclick={() => run(() => oidcLogin(account.id))} disabled={busy}>
            {busy ? "Signing in…" : "Sign in"}
          </button>
        {/if}
      {/if}
      <button class="secondary" onclick={() => run(() => testAccount(account.id))} disabled={busy}>
        {busy ? "Testing…" : "Test connection"}
      </button>
      <button class="secondary" onclick={onEdit}>Edit</button>
      <button class="secondary danger" onclick={onDelete}>Delete</button>
    </div>
  </div>
  {#if isOidc && account.session_expires_at}
    <p class="session" class:expired={sessionExpired}>
      {sessionExpired ? "Session expired at" : "Session expires"}
      {new Date(account.session_expires_at).toLocaleString()}
    </p>
  {/if}
  {#if account.status === "error" && account.last_error}
    <p class="error">{account.last_error}</p>
  {/if}
</div>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.8rem 1rem;
    border-radius: 8px;
    border: 1px solid rgba(128, 128, 128, 0.25);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .info {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    text-align: left;
    min-width: 0;
  }

  .status {
    width: 0.6rem;
    height: 0.6rem;
    border-radius: 50%;
    flex-shrink: 0;
    background: #999;
  }

  .status.valid {
    background: #2ea043;
  }

  .status.expired {
    background: #d4a72c;
  }

  .status.error {
    background: #d33;
  }

  .name {
    margin: 0;
    font-weight: 600;
  }

  .provider {
    margin: 0;
    font-size: 0.85rem;
    opacity: 0.7;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  button.secondary {
    background: transparent;
    font-size: 0.85rem;
    padding: 0.4em 0.8em;
  }

  button.danger {
    color: #d33;
    border-color: rgba(221, 51, 51, 0.4);
  }

  .session {
    margin: 0;
    font-size: 0.8rem;
    opacity: 0.7;
    text-align: left;
  }

  .session.expired {
    color: #d4a72c;
    opacity: 1;
  }

  .error {
    margin: 0;
    font-size: 0.8rem;
    color: #d33;
    text-align: left;
    word-break: break-word;
  }
</style>
