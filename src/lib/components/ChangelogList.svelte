<script lang="ts">
  import { releases, renderReleaseBody } from "../changelog";
</script>

{#if releases.length === 0}
  <p class="muted">No tagged releases yet.</p>
{:else}
  <div class="releases">
    {#each releases as release (release.version)}
      <section>
        <h3>v{release.version} <span class="date">{release.date}</span></h3>
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        {@html renderReleaseBody(release.body)}
      </section>
    {/each}
  </div>
{/if}

<style>
  .muted {
    color: var(--color-text-muted);
  }

  .releases {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    text-align: left;
    max-width: 640px;
  }

  section {
    padding: var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
  }

  section h3 {
    margin: 0 0 var(--space-2);
    font-size: 1rem;
  }

  .date {
    font-weight: 400;
    color: var(--color-text-muted);
    font-size: 0.82rem;
  }

  section :global(h4) {
    margin: var(--space-3) 0 var(--space-1);
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-text-muted);
  }

  section :global(ul) {
    margin: 0;
    padding-left: 1.2rem;
  }

  section :global(li) {
    font-size: 0.88rem;
    margin: 0.2rem 0;
  }

  section :global(p) {
    font-size: 0.88rem;
    margin: var(--space-1) 0;
  }
</style>
