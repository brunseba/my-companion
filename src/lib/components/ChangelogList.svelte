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
    opacity: 0.6;
  }

  .releases {
    display: flex;
    flex-direction: column;
    gap: 1.2rem;
    text-align: left;
  }

  section h3 {
    margin: 0 0 0.3rem;
    font-size: 1rem;
  }

  .date {
    font-weight: 400;
    opacity: 0.6;
    font-size: 0.85rem;
  }

  section :global(h4) {
    margin: 0.6rem 0 0.2rem;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    opacity: 0.7;
  }

  section :global(ul) {
    margin: 0;
    padding-left: 1.2rem;
  }

  section :global(li) {
    font-size: 0.9rem;
    margin: 0.15rem 0;
  }

  section :global(p) {
    font-size: 0.9rem;
    margin: 0.3rem 0;
  }
</style>
