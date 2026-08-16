<script lang="ts">
  interface Props {
    /** How many placeholder rows to show. */
    rows?: number;
  }

  let { rows = 3 }: Props = $props();
</script>

<div class="stack">
  {#each Array(rows) as _, i (i)}
    <div class="row"></div>
  {/each}
</div>

<style>
  .stack {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .row {
    height: 3.4rem;
    border-radius: var(--radius-md);
    background: linear-gradient(90deg, var(--color-bg-subtle) 25%, var(--color-bg-hover) 37%, var(--color-bg-subtle) 63%);
    background-size: 400% 100%;
    animation: shimmer 1.6s ease-in-out infinite;
  }

  @keyframes shimmer {
    0% {
      background-position: 100% 50%;
    }
    100% {
      background-position: 0% 50%;
    }
  }
</style>
