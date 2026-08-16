<script lang="ts">
  import { Wallet, History, Settings } from "lucide-svelte";
  import type { AccountCategory } from "../types";
  import { CATEGORY_LABELS, CATEGORY_ICONS } from "../types";

  interface Props {
    categories: AccountCategory[];
    activeSection: "overview" | "accounts" | "history" | "settings";
    activeCategory: AccountCategory;
    appVersion: string;
    onSelectOverview: () => void;
    onSelectAccounts: () => void;
    onSelectCategory: (category: AccountCategory) => void;
    onSelectHistory: () => void;
    onSelectSettings: () => void;
  }

  let {
    categories,
    activeSection,
    activeCategory,
    appVersion,
    onSelectOverview,
    onSelectAccounts,
    onSelectCategory,
    onSelectHistory,
    onSelectSettings,
  }: Props = $props();
</script>

<aside class="sidebar">
  <button class="brand" class:active={activeSection === "overview"} onclick={onSelectOverview}>
    my-companion
  </button>

  <nav>
    <button class="section" class:active={activeSection === "accounts"} onclick={onSelectAccounts}>
      <Wallet size={16} />
      Accounts
    </button>

    {#if activeSection === "accounts"}
      <ul class="sublist">
        {#each categories as category (category)}
          {@const Icon = CATEGORY_ICONS[category]}
          <li>
            <button class="subitem" class:active={activeCategory === category} onclick={() => onSelectCategory(category)}>
              <Icon size={14} />
              {CATEGORY_LABELS[category]}
            </button>
          </li>
        {/each}
      </ul>
    {/if}

    <button class="section" class:active={activeSection === "history"} onclick={onSelectHistory}>
      <History size={16} />
      History
    </button>

    <button class="section" class:active={activeSection === "settings"} onclick={onSelectSettings}>
      <Settings size={16} />
      Settings
    </button>
  </nav>

  <div class="spacer"></div>

  {#if appVersion}
    <button class="version" onclick={onSelectHistory}>v{appVersion}</button>
  {/if}
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: 220px;
    flex-shrink: 0;
    height: 100%;
    padding: var(--space-4) var(--space-3);
    background: var(--color-bg-subtle);
    border-right: 1px solid var(--color-border);
  }

  .brand {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    box-shadow: none;
    border-radius: var(--radius-sm);
    color: var(--color-text);
    font-weight: 600;
    font-size: 0.95rem;
    letter-spacing: -0.01em;
    padding: var(--space-2);
    margin-bottom: var(--space-3);
    cursor: pointer;
    transition: background-color var(--duration-fast) var(--ease-out), color var(--duration-fast) var(--ease-out);
  }

  .brand:hover {
    background: var(--color-bg-hover);
  }

  .brand.active {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .section,
  .subitem {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    box-shadow: none;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    cursor: pointer;
    transition: background-color var(--duration-fast) var(--ease-out), color var(--duration-fast) var(--ease-out);
  }

  .section {
    padding: var(--space-2) var(--space-2);
    font-weight: 600;
    font-size: 0.88rem;
    color: var(--color-text);
  }

  .section:hover {
    background: var(--color-bg-hover);
  }

  .section.active {
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .sublist {
    list-style: none;
    margin: 0 0 var(--space-2);
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .subitem {
    padding: var(--space-1) var(--space-2) var(--space-1) var(--space-6);
    font-size: 0.82rem;
  }

  .subitem:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .subitem.active {
    color: var(--color-accent);
    font-weight: 600;
  }

  .spacer {
    flex: 1;
  }

  .version {
    align-self: flex-start;
    background: transparent;
    box-shadow: none;
    border: none;
    font-size: 0.78rem;
    color: var(--color-text-muted);
    padding: var(--space-1) var(--space-2);
  }

  .version:hover {
    color: var(--color-text);
    border-color: transparent;
  }
</style>
