<script lang="ts">
  import type { Account, AccountCategory } from "../types";
  import { providersForCategory, schemaFor, fieldsFor } from "../types";
  import { createAccount, updateAccount } from "../accounts";
  import Modal from "./ui/Modal.svelte";
  import Button from "./ui/Button.svelte";

  interface Props {
    category: AccountCategory;
    editing: Account | null;
    onClose: () => void;
    onSaved: () => void;
  }

  let { category, editing, onClose, onSaved }: Props = $props();

  const providers = providersForCategory(category);

  let provider = $state(editing?.provider ?? providers[0]?.provider ?? "");
  let name = $state(editing?.name ?? "");
  let authMethod = $state(String(editing?.config.auth_method ?? ""));
  let values = $state<Record<string, string>>(
    editing ? Object.fromEntries(Object.entries(editing.config).map(([k, v]) => [k, String(v ?? "")])) : {},
  );
  let error = $state("");
  let saving = $state(false);

  const schema = $derived(schemaFor(provider));
  const fields = $derived(schema ? fieldsFor(schema, authMethod) : []);

  // Keeps authMethod pointing at a real option for whichever provider is
  // currently selected - covers both the initial default and switching
  // providers in the (non-editing) dropdown.
  $effect(() => {
    if (schema?.authMethods && !schema.authMethods.some((m) => m.id === authMethod)) {
      authMethod = schema.authMethods[0].id;
    }
  });

  function fieldValue(key: string): string {
    return values[key] ?? "";
  }

  function setFieldValue(key: string, v: string) {
    values = { ...values, [key]: v };
  }

  async function submit(event: Event) {
    event.preventDefault();
    if (!schema) return;
    error = "";

    const config: Record<string, unknown> = {};
    const secret: Record<string, unknown> = {};
    for (const field of fields) {
      const raw = values[field.key];
      if (field.secret) {
        // Blank secret field on edit means "keep the existing value".
        if (raw) secret[field.key] = raw;
      } else if (raw !== undefined && raw !== "") {
        config[field.key] = raw;
      }
      if (field.required && field.secret && !editing && !raw) {
        error = `${field.label} is required`;
        return;
      }
      if (field.required && !field.secret && !raw) {
        error = `${field.label} is required`;
        return;
      }
    }
    if (schema.authMethods) {
      config.auth_method = authMethod;
    }
    if (!name.trim()) {
      error = "Name is required";
      return;
    }

    saving = true;
    try {
      if (editing) {
        await updateAccount(editing.id, {
          name,
          config,
          secret: Object.keys(secret).length ? secret : undefined,
        });
      } else {
        await createAccount({
          category,
          provider,
          name,
          config,
          secret: Object.keys(secret).length ? secret : undefined,
        });
      }
      onSaved();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<Modal title={editing ? "Edit account" : "Add account"} {onClose}>
  <form onsubmit={submit}>
    {#if !editing}
      <label>
        Provider
        <select bind:value={provider}>
          {#each providers as p (p.provider)}
            <option value={p.provider}>{p.label}</option>
          {/each}
        </select>
      </label>
    {:else}
      <p class="static-label">{schema?.label}</p>
    {/if}

    {#if schema?.authMethods}
      {#if !editing}
        <label>
          Auth method
          <select bind:value={authMethod}>
            {#each schema.authMethods as method (method.id)}
              <option value={method.id}>{method.label}</option>
            {/each}
          </select>
        </label>
      {:else}
        <p class="static-label">{schema.authMethods.find((m) => m.id === authMethod)?.label}</p>
      {/if}
    {/if}

    <label>
      Name
      <input type="text" bind:value={name} placeholder="e.g. Personal OpenAI key" />
    </label>

    {#each fields as field (field.key)}
      <label>
        {field.label}{field.required ? " *" : ""}
        {#if editing && field.secret}
          <span class="hint">(leave blank to keep existing)</span>
        {/if}
        {#if field.kind === "textarea"}
          <textarea
            value={fieldValue(field.key)}
            oninput={(e) => setFieldValue(field.key, (e.target as HTMLTextAreaElement).value)}
            placeholder={field.placeholder}
            rows="4"
          ></textarea>
        {:else}
          <input
            type={field.kind}
            value={fieldValue(field.key)}
            oninput={(e) => setFieldValue(field.key, (e.target as HTMLInputElement).value)}
            placeholder={field.placeholder}
          />
        {/if}
      </label>
    {/each}

    {#if error}
      <p class="error">{error}</p>
    {/if}

    <div class="actions">
      <Button variant="secondary" onclick={onClose}>Cancel</Button>
      <Button variant="primary" type="submit" disabled={saving}>{saving ? "Saving…" : "Save"}</Button>
    </div>
  </form>
</Modal>

<style>
  form {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  label {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font-size: 0.86rem;
    font-weight: 500;
  }

  .hint {
    font-weight: 400;
    color: var(--color-text-muted);
    font-size: 0.8rem;
  }

  input,
  select,
  textarea {
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    padding: 0.55em 0.7em;
    font-size: 0.9rem;
    font-family: inherit;
    background: var(--color-bg);
    color: var(--color-text);
    transition: border-color var(--duration-fast) var(--ease-out);
  }

  input:focus,
  select:focus,
  textarea:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .static-label {
    margin: 0;
    font-weight: 600;
    font-size: 0.9rem;
  }

  .error {
    color: var(--color-danger);
    margin: 0;
    font-size: 0.85rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }
</style>
