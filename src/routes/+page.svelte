<script lang="ts">
  import { onMount } from "svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { getVersion } from "@tauri-apps/api/app";
  import { Plus } from "lucide-svelte";
  import type { Account, AccountCategory } from "$lib/types";
  import { CATEGORY_LABELS } from "$lib/types";
  import { listAccounts, deleteAccount } from "$lib/accounts";
  import { toast } from "$lib/toast.svelte";
  import { settings } from "$lib/settings.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Overview from "$lib/components/Overview.svelte";
  import AccountCard from "$lib/components/AccountCard.svelte";
  import AccountForm from "$lib/components/AccountForm.svelte";
  import ChangelogList from "$lib/components/ChangelogList.svelte";
  import SettingsView from "$lib/components/Settings.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";

  type Section = "overview" | "accounts" | "history" | "settings";

  const categories: AccountCategory[] = ["ai", "csp", "k8s", "scm", "tracker", "oidc"];

  let activeSection = $state<Section>(settings.defaultSection);
  let accounts = $state<Account[]>([]);
  let activeCategory = $state<AccountCategory>("ai");
  let loading = $state(true);
  let formOpen = $state(false);
  let editingAccount = $state<Account | null>(null);
  let appVersion = $state("");

  const visibleAccounts = $derived(accounts.filter((a) => a.category === activeCategory));

  async function refresh() {
    loading = true;
    try {
      accounts = await listAccounts();
    } catch (e) {
      toast.error(String(e));
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
      toast.success(`Deleted "${account.name}"`);
    } catch (e) {
      toast.error(String(e));
    }
  }

  async function handleSaved() {
    const wasEditing = editingAccount !== null;
    formOpen = false;
    editingAccount = null;
    await refresh();
    toast.success(wasEditing ? "Account updated" : "Account added");
  }

  function handleTested(updated: Account) {
    accounts = accounts.map((a) => (a.id === updated.id ? updated : a));
    if (updated.status === "error" && updated.last_error) {
      toast.error(updated.last_error);
    }
  }

  onMount(() => {
    refresh();
    getVersion().then((v) => (appVersion = v));
  });
</script>

<div class="shell">
  <Sidebar
    {categories}
    {activeSection}
    {activeCategory}
    {appVersion}
    onSelectOverview={() => (activeSection = "overview")}
    onSelectAccounts={() => (activeSection = "accounts")}
    onSelectCategory={(category) => {
      activeSection = "accounts";
      activeCategory = category;
    }}
    onSelectHistory={() => (activeSection = "history")}
    onSelectSettings={() => (activeSection = "settings")}
  />

  <main class="content">
    {#if activeSection === "overview"}
      <div class="content-header">
        <h1>Overview</h1>
      </div>
      <Overview
        {accounts}
        {categories}
        onSelectCategory={(category) => {
          activeSection = "accounts";
          activeCategory = category;
        }}
      />
    {:else if activeSection === "accounts"}
      <div class="content-header">
        <h1>{CATEGORY_LABELS[activeCategory]}</h1>
        <Button variant="primary" onclick={openAdd}><Plus size={16} /> Add account</Button>
      </div>

      {#if loading}
        <Skeleton rows={3} />
      {:else if visibleAccounts.length === 0}
        <p class="empty">No {CATEGORY_LABELS[activeCategory]} accounts yet.</p>
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
    {:else if activeSection === "history"}
      <div class="content-header">
        <h1>History</h1>
      </div>
      <ChangelogList />
    {:else}
      <div class="content-header">
        <h1>Settings</h1>
      </div>
      <SettingsView onReset={refresh} />
    {/if}
  </main>
</div>

{#if formOpen}
  <AccountForm category={activeCategory} editing={editingAccount} onClose={() => (formOpen = false)} onSaved={handleSaved} />
{/if}

<style>
  .shell {
    display: flex;
    height: 100vh;
  }

  .content {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: var(--space-6) var(--space-6) var(--space-7);
  }

  .content-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    margin-bottom: var(--space-5);
  }

  h1 {
    margin: 0;
    font-size: 1.3rem;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    max-width: 640px;
  }

  .empty {
    color: var(--color-text-muted);
    max-width: 640px;
  }
</style>
