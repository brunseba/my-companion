<script lang="ts">
  import type { Account, AccountCategory } from "../types";
  import { providersForCategory, schemaFor, fieldsFor } from "../types";
  import { createAccount, updateAccount } from "../accounts";

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
    <h2>{editing ? "Edit account" : "Add account"}</h2>

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
        <p class="provider-label">{schema?.label}</p>
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
          <p class="provider-label">{schema.authMethods.find((m) => m.id === authMethod)?.label}</p>
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
        <button type="button" class="secondary" onclick={onClose}>Cancel</button>
        <button type="submit" disabled={saving}>{saving ? "Saving…" : "Save"}</button>
      </div>
    </form>
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
    width: min(420px, 90vw);
    max-height: 85vh;
    overflow-y: auto;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
  }

  h2 {
    margin-top: 0;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.9rem;
    text-align: left;
  }

  .hint {
    font-weight: 400;
    opacity: 0.6;
    font-size: 0.8rem;
  }

  input,
  select,
  textarea {
    border-radius: 6px;
    border: 1px solid rgba(128, 128, 128, 0.4);
    padding: 0.5em 0.7em;
    font-size: 0.95em;
    font-family: inherit;
    background: transparent;
    color: inherit;
  }

  .provider-label {
    margin: 0;
    font-weight: 600;
  }

  .error {
    color: #d33;
    margin: 0;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    margin-top: 0.5rem;
  }

  button.secondary {
    background: transparent;
  }
</style>
