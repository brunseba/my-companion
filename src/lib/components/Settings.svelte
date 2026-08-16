<script lang="ts">
  import { onMount } from "svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { Sun, Moon, Monitor, FolderOpen, KeyRound } from "lucide-svelte";
  import { settings, type Theme, type DefaultSection } from "../settings.svelte";
  import { getAppDataInfo, resetAllData, type DataInfo } from "../accounts";
  import { toast } from "../toast.svelte";
  import Button from "./ui/Button.svelte";

  interface Props {
    /** Called after a successful reset, so the parent can refresh its account list. */
    onReset: () => Promise<void>;
  }

  let { onReset }: Props = $props();

  const themeOptions: { value: Theme; label: string; icon: typeof Sun }[] = [
    { value: "system", label: "System", icon: Monitor },
    { value: "light", label: "Light", icon: Sun },
    { value: "dark", label: "Dark", icon: Moon },
  ];

  const sectionOptions: { value: DefaultSection; label: string }[] = [
    { value: "overview", label: "Overview" },
    { value: "accounts", label: "Accounts" },
    { value: "history", label: "History" },
  ];

  let dataInfo = $state<DataInfo | null>(null);
  let resetting = $state(false);

  onMount(async () => {
    try {
      dataInfo = await getAppDataInfo();
    } catch (e) {
      toast.error(String(e));
    }
  });

  async function handleReset() {
    const confirmed = await confirm(
      "Delete every account, including all stored secrets in the keychain? This cannot be undone.",
      { title: "Reset all data", kind: "warning" },
    );
    if (!confirmed) return;

    resetting = true;
    try {
      await resetAllData();
      await onReset();
      toast.success("All data reset");
    } catch (e) {
      toast.error(String(e));
    } finally {
      resetting = false;
    }
  }
</script>

<div class="settings">
  <section>
    <h2>Appearance</h2>
    <div class="segmented">
      {#each themeOptions as option (option.value)}
        {@const Icon = option.icon}
        <button class:active={settings.theme === option.value} onclick={() => settings.setTheme(option.value)}>
          <Icon size={15} />
          {option.label}
        </button>
      {/each}
    </div>
  </section>

  <section>
    <h2>Default landing section</h2>
    <p class="hint">Which section opens when the app starts.</p>
    <div class="segmented">
      {#each sectionOptions as option (option.value)}
        <button
          class:active={settings.defaultSection === option.value}
          onclick={() => settings.setDefaultSection(option.value)}
        >
          {option.label}
        </button>
      {/each}
    </div>
  </section>

  <section>
    <h2>Data & storage</h2>
    {#if dataInfo}
      <div class="info-row">
        <FolderOpen size={15} />
        <div>
          <p class="info-label">Account metadata</p>
          <code>{dataInfo.accounts_file}</code>
        </div>
      </div>
      <div class="info-row">
        <KeyRound size={15} />
        <div>
          <p class="info-label">Keychain service (secrets)</p>
          <code>{dataInfo.keychain_service}</code>
        </div>
      </div>
    {/if}
  </section>

  <section class="danger-zone">
    <h2>Danger zone</h2>
    <p class="hint">
      Deletes every account's metadata and its keychain secret. There's no undo - you'd need to re-add each
      account and re-enter its credentials.
    </p>
    <Button variant="danger" onclick={handleReset} disabled={resetting}>
      {resetting ? "Resetting…" : "Reset all data"}
    </Button>
  </section>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    max-width: 560px;
  }

  section h2 {
    font-size: 0.95rem;
    margin: 0 0 var(--space-2);
  }

  .hint {
    color: var(--color-text-muted);
    font-size: 0.85rem;
    margin-bottom: var(--space-3);
  }

  .segmented {
    display: inline-flex;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .segmented button {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    background: var(--color-bg-elevated);
    border: none;
    box-shadow: none;
    border-radius: 0;
    padding: var(--space-2) var(--space-3);
    font-size: 0.85rem;
    color: var(--color-text-muted);
    cursor: pointer;
    border-right: 1px solid var(--color-border);
    transition: background-color var(--duration-fast) var(--ease-out), color var(--duration-fast) var(--ease-out);
  }

  .segmented button:last-child {
    border-right: none;
  }

  .segmented button:hover {
    background: var(--color-bg-hover);
  }

  .segmented button.active {
    background: var(--color-accent-soft);
    color: var(--color-accent);
    font-weight: 600;
  }

  .info-row {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
    color: var(--color-text-muted);
    margin-bottom: var(--space-2);
  }

  .info-label {
    margin: 0 0 2px;
    font-size: 0.78rem;
    color: var(--color-text-muted);
  }

  code {
    font-size: 0.8rem;
    color: var(--color-text);
    word-break: break-all;
  }

  .danger-zone {
    padding: var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-danger-soft);
    background: var(--color-danger-soft);
  }

  .danger-zone h2 {
    color: var(--color-danger);
  }
</style>
