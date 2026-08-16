<script lang="ts">
  import type { Account, AccountCategory } from "../types";
  import { CATEGORY_LABELS, CATEGORY_ICONS } from "../types";
  import { CircleCheck, TriangleAlert, CircleDashed, Wallet } from "lucide-svelte";

  interface Props {
    accounts: Account[];
    categories: AccountCategory[];
    onSelectCategory: (category: AccountCategory) => void;
  }

  let { accounts, categories, onSelectCategory }: Props = $props();

  const total = $derived(accounts.length);
  const valid = $derived(accounts.filter((a) => a.status === "valid").length);
  const needsAttention = $derived(accounts.filter((a) => a.status === "error" || a.status === "expired").length);
  const untested = $derived(accounts.filter((a) => a.status === "unknown").length);

  const perCategory = $derived(
    categories.map((category) => ({
      category,
      count: accounts.filter((a) => a.category === category).length,
    })),
  );
</script>

<p class="intro">
  my-companion centralizes credentials and sessions for the tools you use daily - AI providers, cloud
  platforms, Kubernetes clusters, source control, and issue tracking - in one place. Store a key or sign in
  once, confirm it still works with a real API call, and see at a glance what's configured and what needs
  attention.
</p>

<div class="stats">
  <div class="stat">
    <Wallet size={18} />
    <div>
      <p class="value">{total}</p>
      <p class="label">Total accounts</p>
    </div>
  </div>
  <div class="stat success">
    <CircleCheck size={18} />
    <div>
      <p class="value">{valid}</p>
      <p class="label">Valid</p>
    </div>
  </div>
  <div class="stat warning">
    <TriangleAlert size={18} />
    <div>
      <p class="value">{needsAttention}</p>
      <p class="label">Needs attention</p>
    </div>
  </div>
  <div class="stat muted">
    <CircleDashed size={18} />
    <div>
      <p class="value">{untested}</p>
      <p class="label">Untested</p>
    </div>
  </div>
</div>

<h2>By category</h2>
<div class="grid">
  {#each perCategory as { category, count } (category)}
    {@const Icon = CATEGORY_ICONS[category]}
    <button class="category-card" onclick={() => onSelectCategory(category)}>
      <Icon size={20} />
      <span class="name">{CATEGORY_LABELS[category]}</span>
      <span class="count">{count}</span>
    </button>
  {/each}
</div>

<style>
  .intro {
    max-width: 640px;
    color: var(--color-text-muted);
    font-size: 0.9rem;
    line-height: 1.6;
    margin-bottom: var(--space-6);
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: var(--space-3);
    max-width: 720px;
    margin-bottom: var(--space-6);
  }

  .stat {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
    color: var(--color-text-muted);
  }

  .stat.success {
    color: var(--color-success);
  }
  .stat.warning {
    color: var(--color-warning);
  }
  .stat.muted {
    color: var(--color-text-muted);
  }

  .value {
    margin: 0;
    font-size: 1.4rem;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1.1;
  }

  .label {
    margin: 0;
    font-size: 0.78rem;
    color: var(--color-text-muted);
  }

  h2 {
    font-size: 0.95rem;
    margin: 0 0 var(--space-3);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: var(--space-3);
    max-width: 720px;
  }

  .category-card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
    transition:
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  .category-card:hover {
    border-color: color-mix(in srgb, var(--color-accent) 35%, var(--color-border));
    box-shadow: var(--shadow-sm);
  }

  .name {
    font-size: 0.86rem;
    font-weight: 600;
  }

  .count {
    font-size: 1.4rem;
    font-weight: 700;
    color: var(--color-text-muted);
  }
</style>
