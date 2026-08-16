<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variant?: "primary" | "secondary" | "danger" | "ghost";
    size?: "md" | "sm" | "icon";
    type?: "button" | "submit";
    disabled?: boolean;
    title?: string;
    "aria-label"?: string;
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
  }

  let {
    variant = "secondary",
    size = "md",
    type = "button",
    disabled = false,
    title,
    "aria-label": ariaLabel,
    onclick,
    children,
  }: Props = $props();
</script>

<button
  class="btn {variant} {size}"
  {type}
  {disabled}
  {title}
  aria-label={ariaLabel}
  {onclick}
>
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    font-family: inherit;
    font-weight: 500;
    cursor: pointer;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }

  .btn:active:not(:disabled) {
    transform: scale(0.97);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  /* Sizes */
  .md {
    padding: 0.55em 1.1em;
    font-size: 0.9rem;
  }

  .sm {
    padding: 0.4em 0.8em;
    font-size: 0.82rem;
  }

  .icon {
    padding: var(--space-2);
    width: 2rem;
    height: 2rem;
  }

  /* Variants */
  .primary {
    background: var(--color-accent);
    color: white;
  }
  .primary:hover:not(:disabled) {
    background: var(--color-accent-hover);
  }

  .secondary {
    background: var(--color-bg-elevated);
    border-color: var(--color-border);
    color: var(--color-text);
  }
  .secondary:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-accent);
  }

  .danger {
    background: transparent;
    border-color: var(--color-border);
    color: var(--color-danger);
  }
  .danger:hover:not(:disabled) {
    background: var(--color-danger-soft);
    border-color: var(--color-danger);
  }

  .ghost {
    background: transparent;
    color: var(--color-text-muted);
  }
  .ghost:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }
</style>
