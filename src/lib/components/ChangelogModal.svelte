<script lang="ts">
  import { releases, renderReleaseBody } from "../changelog";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();
</script>

<div
  class="backdrop"
  role="button"
  tabindex="-1"
  onclick={onClose}
  onkeydown={(e) => e.key === "Escape" && onClose()}
>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <h2>Changelog</h2>

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

    <div class="actions">
      <button type="button" onclick={onClose}>Close</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
  }

  .modal {
    background: var(--surface, #fff);
    color: inherit;
    border-radius: 10px;
    padding: 1.5rem;
    width: min(480px, 90vw);
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
    text-align: left;
  }

  h2 {
    margin-top: 0;
  }

  .muted {
    opacity: 0.6;
  }

  .releases {
    display: flex;
    flex-direction: column;
    gap: 1.2rem;
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

  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 1.2rem;
  }
</style>
