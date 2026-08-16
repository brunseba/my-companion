<script lang="ts">
  import { onMount } from "svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import type { Account, AccountCategory } from "$lib/types";
  import { CATEGORY_LABELS } from "$lib/types";
  import { listAccounts, deleteAccount } from "$lib/accounts";
  import AccountCard from "$lib/components/AccountCard.svelte";
  import AccountForm from "$lib/components/AccountForm.svelte";

  const categories: AccountCategory[] = ["ai", "csp", "k8s", "oidc"];

  let accounts = $state<Account[]>([]);
  let activeCategory = $state<AccountCategory>("ai");
  let loading = $state(true);
  let loadError = $state("");
  let formOpen = $state(false);
  let editingAccount = $state<Account | null>(null);

  const visibleAccounts = $derived(accounts.filter((a) => a.category === activeCategory));

  async function refresh() {
    loading = true;
    loadError = "";
    try {
      accounts = await listAccounts();
    } catch (e) {
      loadError = String(e);
    } finally {
      loading = false;
    }
  }

  function openAdd() {
    editingAccount = null;
    formOpen = true;
  }

  function openEdit(account: Account) {
    editingAccount = account;
    formOpen = true;
  }

  async function handleDelete(account: Account) {
    const confirmed = await confirm(`Delete "${account.name}"? This removes its stored credentials too.`, {
      title: "Delete account",
      kind: "warning",
    });
    if (!confirmed) return;
    try {
      await deleteAccount(account.id);
      await refresh();
    } catch (e) {
      loadError = String(e);
    }
  }

  async function handleSaved() {
    formOpen = false;
    editingAccount = null;
    await refresh();
  }

  function handleTested(updated: Account) {
    accounts = accounts.map((a) => (a.id === updated.id ? updated : a));
  }

  onMount(refresh);
</script>

<main class="container">
  <h1>Accounts</h1>

  <nav class="tabs">
    {#each categories as category (category)}
      <button
        class="tab"
        class:active={activeCategory === category}
        onclick={() => (activeCategory = category)}
      >
        {CATEGORY_LABELS[category]}
      </button>
    {/each}
  </nav>

  {#if loadError}
    <p class="error">{loadError}</p>
  {/if}

  <div class="toolbar">
    <button onclick={openAdd}>+ Add account</button>
  </div>

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if visibleAccounts.length === 0}
    <p class="muted">No {CATEGORY_LABELS[activeCategory]} accounts yet.</p>
  {:else}
    <div class="list">
      {#each visibleAccounts as account (account.id)}
        <AccountCard
          {account}
          onEdit={() => openEdit(account)}
          onDelete={() => handleDelete(account)}
          onTested={handleTested}
        />
      {/each}
    </div>
  {/if}
</main>

{#if formOpen}
  <AccountForm category={activeCategory} editing={editingAccount} onClose={() => (formOpen = false)} onSaved={handleSaved} />
{/if}

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  .container {
    margin: 0 auto;
    max-width: 640px;
    padding: 3rem 1.5rem;
  }

  h1 {
    margin-top: 0;
  }

  .tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1.2rem;
    border-bottom: 1px solid rgba(128, 128, 128, 0.25);
  }

  .tab {
    background: transparent;
    box-shadow: none;
    border: none;
    border-radius: 0;
    padding: 0.6em 0.9em;
    opacity: 0.65;
  }

  .tab.active {
    opacity: 1;
    border-bottom: 2px solid #396cd8;
  }

  .toolbar {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 1rem;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .muted {
    opacity: 0.6;
  }

  .error {
    color: #d33;
  }

  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
    cursor: pointer;
    outline: none;
  }

  button:hover {
    border-color: #396cd8;
  }
  button:active {
    border-color: #396cd8;
    background-color: #e8e8e8;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
      --surface: #2f2f2f;
    }

    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }
    button:active {
      background-color: #0f0f0f69;
    }
  }
</style>
