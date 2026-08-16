<script lang="ts">
  import { onMount } from "svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { getVersion } from "@tauri-apps/api/app";
  import { Wallet, History, Plus } from "lucide-svelte";
  import type { Account, AccountCategory } from "$lib/types";
  import { CATEGORY_LABELS, CATEGORY_ICONS } from "$lib/types";
  import { listAccounts, deleteAccount } from "$lib/accounts";
  import AccountCard from "$lib/components/AccountCard.svelte";
  import AccountForm from "$lib/components/AccountForm.svelte";
  import ChangelogList from "$lib/components/ChangelogList.svelte";

  type Section = "accounts" | "history";

  const categories: AccountCategory[] = ["ai", "csp", "k8s", "scm", "tracker", "oidc"];

  let activeSection = $state<Section>("accounts");
  let accounts = $state<Account[]>([]);
  let activeCategory = $state<AccountCategory>("ai");
  let loading = $state(true);
  let loadError = $state("");
  let formOpen = $state(false);
  let editingAccount = $state<Account | null>(null);
  let appVersion = $state("");

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

  onMount(() => {
    refresh();
    getVersion().then((v) => (appVersion = v));
  });
</script>

<main class="container">
  <div class="header">
    <h1>my-companion</h1>
    {#if appVersion}
      <button class="version" onclick={() => (activeSection = "history")}>v{appVersion}</button>
    {/if}
  </div>

  <nav class="top-nav">
    <button class:active={activeSection === "accounts"} onclick={() => (activeSection = "accounts")}>
      <Wallet size={16} />
      Accounts
    </button>
    <button class:active={activeSection === "history"} onclick={() => (activeSection = "history")}>
      <History size={16} />
      History
    </button>
  </nav>

  {#if activeSection === "accounts"}
    <nav class="tabs">
      {#each categories as category (category)}
        {@const Icon = CATEGORY_ICONS[category]}
        <button
          class="tab"
          class:active={activeCategory === category}
          onclick={() => (activeCategory = category)}
        >
          <Icon size={15} />
          {CATEGORY_LABELS[category]}
        </button>
      {/each}
    </nav>

    {#if loadError}
      <p class="error">{loadError}</p>
    {/if}

    <div class="toolbar">
      <button onclick={openAdd}><Plus size={16} /> Add account</button>
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
  {:else}
    <ChangelogList />
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

  .header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.2rem;
  }

  h1 {
    margin: 0;
  }

  .top-nav {
    display: flex;
    gap: 0.4rem;
    margin-bottom: 1.5rem;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid rgba(128, 128, 128, 0.25);
  }

  .top-nav button {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: transparent;
    box-shadow: none;
    border: 1px solid transparent;
    font-weight: 600;
    opacity: 0.6;
  }

  .top-nav button.active {
    opacity: 1;
    border-color: rgba(128, 128, 128, 0.35);
    background: rgba(128, 128, 128, 0.08);
  }

  .tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1.2rem;
    border-bottom: 1px solid rgba(128, 128, 128, 0.25);
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 0.35rem;
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

  .version {
    background: transparent;
    box-shadow: none;
    border: none;
    font-size: 0.8rem;
    opacity: 0.5;
    padding: 0.3em 0.6em;
    flex-shrink: 0;
  }

  .version:hover {
    opacity: 0.85;
    border-color: transparent;
  }

  .error {
    color: #d33;
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: 0.4em;
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
