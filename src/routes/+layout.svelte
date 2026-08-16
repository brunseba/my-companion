<script lang="ts">
  import "../app.css";
  import ToastHost from "$lib/components/ui/ToastHost.svelte";
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

{@render children()}
<ToastHost />
