<script lang="ts">
  import { toast } from "../../toast.svelte";
  import { CircleCheck, CircleX, Info, X } from "lucide-svelte";

  const icons = { success: CircleCheck, error: CircleX, info: Info };
</script>

<div class="host" role="status" aria-live="polite">
  {#each toast.all as item (item.id)}
    {@const Icon = icons[item.kind]}
    <div class="toast {item.kind}">
      <Icon size={16} />
      <p>{item.message}</p>
      <button class="dismiss" aria-label="Dismiss" onclick={() => toast.dismiss(item.id)}>
        <X size={14} />
      </button>
    </div>
  {/each}
</div>

<style>
  .host {
    position: fixed;
    top: var(--space-4);
    right: var(--space-4);
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    max-width: 340px;
  }

  .toast {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-3);
    border-radius: var(--radius-md);
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    box-shadow: var(--shadow-md);
    animation: slide-in var(--duration-base) var(--ease-out);
  }

  .toast p {
    flex: 1;
    font-size: 0.85rem;
    line-height: 1.4;
    word-break: break-word;
  }

  .toast.success {
    color: var(--color-success);
  }
  .toast.success p {
    color: var(--color-text);
  }

  .toast.error {
    color: var(--color-danger);
  }
  .toast.error p {
    color: var(--color-text);
  }

  .toast.info {
    color: var(--color-accent);
  }
  .toast.info p {
    color: var(--color-text);
  }

  .dismiss {
    display: inline-flex;
    background: transparent;
    box-shadow: none;
    border: none;
    padding: 2px;
    opacity: 0.5;
    color: var(--color-text);
    flex-shrink: 0;
  }

  .dismiss:hover {
    opacity: 1;
    border-color: transparent;
  }

  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
