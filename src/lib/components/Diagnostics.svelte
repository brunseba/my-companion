<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { MemoryStick, Cpu, HardDrive } from "lucide-svelte";
  import { getResourceUsage, formatBytes, type ResourceUsage } from "../diagnostics";
  import { toast } from "../toast.svelte";

  const POLL_MS = 2000;
  const HISTORY_LENGTH = 30;

  let usage = $state<ResourceUsage | null>(null);
  let memoryHistory = $state<number[]>([]);
  let cpuHistory = $state<number[]>([]);
  let timer: ReturnType<typeof setInterval> | undefined;

  function pushHistory(history: number[], value: number): number[] {
    const next = [...history, value];
    return next.length > HISTORY_LENGTH ? next.slice(next.length - HISTORY_LENGTH) : next;
  }

  async function poll() {
    try {
      const next = await getResourceUsage();
      usage = next;
      memoryHistory = pushHistory(memoryHistory, next.memory_bytes);
      cpuHistory = pushHistory(cpuHistory, next.cpu_percent);
    } catch (e) {
      toast.error(String(e));
      if (timer) clearInterval(timer);
    }
  }

  // Builds an SVG polyline's `points` attribute from a history array,
  // normalized to fill a 100x28 viewBox. A flat/empty history draws a flat
  // midline instead of collapsing to nothing.
  function sparklinePoints(history: number[]): string {
    if (history.length < 2) return "";
    const max = Math.max(...history, 1);
    const min = Math.min(...history, 0);
    const range = max - min || 1;
    const step = 100 / (HISTORY_LENGTH - 1);
    const offset = HISTORY_LENGTH - history.length;
    return history
      .map((value, i) => {
        const x = (offset + i) * step;
        const y = 28 - ((value - min) / range) * 26 - 1;
        return `${x.toFixed(1)},${y.toFixed(1)}`;
      })
      .join(" ");
  }

  onMount(() => {
    poll();
    timer = setInterval(poll, POLL_MS);
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });
</script>

<p class="intro">
  Live resource usage for my-companion's own process - not the whole system. Updates every {POLL_MS / 1000}s
  while this page is open.
</p>

<div class="grid">
  <div class="card">
    <div class="card-head">
      <MemoryStick size={16} />
      <span>Memory</span>
    </div>
    <p class="value">{usage ? formatBytes(usage.memory_bytes) : "—"}</p>
    <svg class="sparkline" viewBox="0 0 100 28" preserveAspectRatio="none">
      <polyline points={sparklinePoints(memoryHistory)} fill="none" stroke="var(--color-accent)" stroke-width="1.5" />
    </svg>
  </div>

  <div class="card">
    <div class="card-head">
      <Cpu size={16} />
      <span>CPU</span>
    </div>
    <p class="value">{usage ? `${usage.cpu_percent.toFixed(1)}%` : "—"}</p>
    <svg class="sparkline" viewBox="0 0 100 28" preserveAspectRatio="none">
      <polyline points={sparklinePoints(cpuHistory)} fill="none" stroke="var(--color-success)" stroke-width="1.5" />
    </svg>
  </div>

  <div class="card">
    <div class="card-head">
      <HardDrive size={16} />
      <span>Disk</span>
    </div>
    {#if usage}
      <div class="disk-rows">
        <div class="disk-row">
          <span>accounts.json</span>
          <span class="disk-value">{formatBytes(usage.accounts_file_bytes)}</span>
        </div>
        <div class="disk-row">
          <span>App binary</span>
          <span class="disk-value">{formatBytes(usage.binary_bytes)}</span>
        </div>
      </div>
    {:else}
      <p class="value">—</p>
    {/if}
  </div>
</div>

<style>
  .intro {
    color: var(--color-text-muted);
    font-size: 0.9rem;
    max-width: 640px;
    margin-bottom: var(--space-5);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--space-3);
    max-width: 720px;
  }

  .card {
    padding: var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
  }

  .card-head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--color-text-muted);
    font-size: 0.8rem;
    margin-bottom: var(--space-2);
  }

  .value {
    margin: 0 0 var(--space-2);
    font-size: 1.5rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    line-height: 1.1;
  }

  .sparkline {
    display: block;
    width: 100%;
    height: 28px;
  }

  .disk-rows {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin-top: var(--space-1);
  }

  .disk-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }

  .disk-value {
    font-variant-numeric: tabular-nums;
    color: var(--color-text);
    font-weight: 600;
  }
</style>
