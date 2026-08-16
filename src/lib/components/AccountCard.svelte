<script lang="ts">
  import type { Account } from "../types";
  import { schemaFor, isOAuthAccount } from "../types";
  import { testAccount, oauthLogin, refreshOauthSession } from "../accounts";
  import { Zap, Pencil, Trash2, LogIn, RefreshCw } from "lucide-svelte";
  import Button from "./ui/Button.svelte";

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
          <Button
            variant="ghost"
            size="icon"
            title={busyKind === "refresh" ? "Refreshing…" : "Refresh session"}
            aria-label="Refresh session"
            onclick={() => run("refresh", () => refreshOauthSession(account.id))}
            disabled={busy}
          >
            <RefreshCw size={16} class={busyKind === "refresh" ? "busy" : ""} />
          </Button>
        {:else}
          <Button
            variant="ghost"
            size="icon"
            title={busyKind === "login" ? "Signing in…" : "Sign in"}
            aria-label="Sign in"
            onclick={() => run("login", () => oauthLogin(account.id))}
            disabled={busy}
          >
            <LogIn size={16} class={busyKind === "login" ? "busy" : ""} />
          </Button>
        {/if}
      {/if}
      <Button
        variant="ghost"
        size="icon"
        title={busyKind === "test" ? "Testing…" : "Test connection"}
        aria-label="Test connection"
        onclick={() => run("test", () => testAccount(account.id))}
        disabled={busy}
      >
        <Zap size={16} class={busyKind === "test" ? "busy" : ""} />
      </Button>
      <Button variant="ghost" size="icon" title="Edit" aria-label="Edit" onclick={onEdit} disabled={busy}>
        <Pencil size={16} />
      </Button>
      <Button variant="danger" size="icon" title="Delete" aria-label="Delete" onclick={onDelete} disabled={busy}>
        <Trash2 size={16} />
      </Button>
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
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
    transition:
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  .card:hover {
    border-color: color-mix(in srgb, var(--color-accent) 35%, var(--color-border));
    box-shadow: var(--shadow-sm);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .info {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    text-align: left;
    min-width: 0;
  }

  .status {
    width: 0.55rem;
    height: 0.55rem;
    border-radius: 50%;
    flex-shrink: 0;
    background: var(--color-text-muted);
    transition: background-color var(--duration-base) var(--ease-out);
  }

  .status.valid {
    background: var(--color-success);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-success) 18%, transparent);
  }

  .status.expired {
    background: var(--color-warning);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-warning) 18%, transparent);
  }

  .status.error {
    background: var(--color-danger);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-danger) 18%, transparent);
  }

  .name {
    margin: 0;
    font-weight: 600;
    font-size: 0.92rem;
  }

  .provider {
    margin: 0;
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .actions :global(.busy) {
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
    font-size: 0.78rem;
    color: var(--color-text-muted);
    text-align: left;
  }

  .session.expired {
    color: var(--color-warning);
  }

  .error {
    margin: 0;
    font-size: 0.78rem;
    color: var(--color-danger);
    text-align: left;
    word-break: break-word;
  }
</style>
