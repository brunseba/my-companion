<script lang="ts">
  import type { Account } from "../types";
  import { schemaFor, isOAuthAccount } from "../types";
  import { testAccount, oauthLogin, refreshOauthSession } from "../accounts";
  import { Zap, Pencil, Trash2, LogIn, RefreshCw } from "lucide-svelte";

  interface Props {
    account: Account;
    onEdit: () => void;
    onDelete: () => void;
    onTested: (account: Account) => void;
  }

  let { account, onEdit, onDelete, onTested }: Props = $props();

  const providerLabel = $derived(schemaFor(account.provider)?.label ?? account.provider);
  const showOAuthActions = $derived(isOAuthAccount(account));
  const sessionExpired = $derived(
    account.session_expires_at !== null && new Date(account.session_expires_at).getTime() < Date.now(),
  );

  // Tracks *which* action is in flight (not just whether one is) so only that
  // button's icon pulses, while every button stays disabled to avoid racing
  // two actions against the same account.
  let busyKind = $state<"test" | "login" | "refresh" | null>(null);
  const busy = $derived(busyKind !== null);

  async function run(kind: "test" | "login" | "refresh", action: () => Promise<Account>) {
    busyKind = kind;
    try {
      const updated = await action();
      onTested(updated);
    } catch (e) {
      onTested({ ...account, status: "error", last_error: String(e) });
    } finally {
      busyKind = null;
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
      {#if showOAuthActions}
        {#if account.session_expires_at}
          <button
            class="icon-button"
            title={busyKind === "refresh" ? "Refreshing…" : "Refresh session"}
            aria-label="Refresh session"
            onclick={() => run("refresh", () => refreshOauthSession(account.id))}
            disabled={busy}
          >
            <RefreshCw size={16} class={busyKind === "refresh" ? "busy" : ""} />
          </button>
        {:else}
          <button
            class="icon-button"
            title={busyKind === "login" ? "Signing in…" : "Sign in"}
            aria-label="Sign in"
            onclick={() => run("login", () => oauthLogin(account.id))}
            disabled={busy}
          >
            <LogIn size={16} class={busyKind === "login" ? "busy" : ""} />
          </button>
        {/if}
      {/if}
      <button
        class="icon-button"
        title={busyKind === "test" ? "Testing…" : "Test connection"}
        aria-label="Test connection"
        onclick={() => run("test", () => testAccount(account.id))}
        disabled={busy}
      >
        <Zap size={16} class={busyKind === "test" ? "busy" : ""} />
      </button>
      <button class="icon-button" title="Edit" aria-label="Edit" onclick={onEdit} disabled={busy}>
        <Pencil size={16} />
      </button>
      <button class="icon-button danger" title="Delete" aria-label="Delete" onclick={onDelete} disabled={busy}>
        <Trash2 size={16} />
      </button>
    </div>
  </div>
  {#if showOAuthActions && account.session_expires_at}
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
    gap: 0.35rem;
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .icon-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    box-shadow: none;
    padding: 0.4em;
    width: 2rem;
    height: 2rem;
  }

  .icon-button.danger {
    color: #d33;
  }

  .icon-button.danger:hover {
    border-color: rgba(221, 51, 51, 0.4);
  }

  .icon-button :global(.busy) {
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.35;
    }
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
