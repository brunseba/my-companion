<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { Plus, Bot, SendHorizontal, Trash2, Search } from "lucide-svelte";
  import type { Account } from "../types";
  import {
    listConversations,
    createConversation,
    deleteConversation,
    sendMessage,
    onChatDelta,
    type Conversation,
    type ChatMessage,
  } from "../chat";
  import { searchConversations, type SearchResult } from "../search";
  import { toast } from "../toast.svelte";
  import Button from "./ui/Button.svelte";
  import Modal from "./ui/Modal.svelte";

  interface Props {
    accounts: Account[];
    onAddAccount: () => void;
  }

  let { accounts, onAddAccount }: Props = $props();

  const aiAccounts = $derived(accounts.filter((a) => a.category === "ai"));

  let conversations = $state<Conversation[]>([]);
  let activeId = $state<string | null>(null);
  let draft = $state("");
  let sending = $state(false);
  let streamingText = $state("");
  let showNewChat = $state(false);
  let newChatAccountId = $state("");
  let newChatModel = $state("");
  let threadEl = $state<HTMLDivElement | undefined>(undefined);
  let unlisten: (() => void) | undefined;

  let searchQuery = $state("");
  let searchResults = $state<SearchResult[]>([]);
  let searching = $state(false);
  let searchDebounce: ReturnType<typeof setTimeout> | undefined;

  const activeConversation = $derived(conversations.find((c) => c.id === activeId) ?? null);

  function defaultModelFor(provider: string): string {
    return provider === "anthropic" ? "claude-3-5-haiku-20241022" : "gpt-4o-mini";
  }

  // Keeps the thread scrolled to the newest message as messages arrive or a
  // reply streams in.
  $effect(() => {
    void activeConversation?.messages.length;
    void streamingText;
    if (threadEl) threadEl.scrollTop = threadEl.scrollHeight;
  });

  onMount(async () => {
    try {
      conversations = await listConversations();
      if (conversations.length > 0) activeId = conversations[0].id;
    } catch (e) {
      toast.error(String(e));
    }
    unlisten = await onChatDelta((conversationId, text) => {
      if (conversationId === activeId) streamingText += text;
    });
  });

  onDestroy(() => {
    unlisten?.();
    clearTimeout(searchDebounce);
  });

  function openNewChat() {
    if (aiAccounts.length === 0) return;
    newChatAccountId = aiAccounts[0].id;
    newChatModel = defaultModelFor(aiAccounts[0].provider);
    showNewChat = true;
  }

  function handleAccountChange() {
    const account = aiAccounts.find((a) => a.id === newChatAccountId);
    newChatModel = defaultModelFor(account?.provider ?? "");
  }

  async function confirmNewChat(event: Event) {
    event.preventDefault();
    if (!newChatAccountId || !newChatModel.trim()) return;
    try {
      const conversation = await createConversation({ account_id: newChatAccountId, model: newChatModel.trim() });
      conversations = [conversation, ...conversations];
      activeId = conversation.id;
      showNewChat = false;
    } catch (e) {
      toast.error(String(e));
    }
  }

  async function handleDeleteConversation(id: string) {
    const confirmed = await confirm("Delete this conversation? This can't be undone.", {
      title: "Delete conversation",
      kind: "warning",
    });
    if (!confirmed) return;
    try {
      await deleteConversation(id);
      conversations = conversations.filter((c) => c.id !== id);
      if (activeId === id) activeId = conversations[0]?.id ?? null;
    } catch (e) {
      toast.error(String(e));
    }
  }

  function handleSearchInput() {
    clearTimeout(searchDebounce);
    if (!searchQuery.trim()) {
      searchResults = [];
      searching = false;
      return;
    }
    searching = true;
    searchDebounce = setTimeout(runSearch, 300);
  }

  async function runSearch() {
    const query = searchQuery.trim();
    try {
      const results = await searchConversations(query);
      // The debounced query may have changed while this was in flight.
      if (query === searchQuery.trim()) searchResults = results;
    } catch (e) {
      toast.error(String(e));
    } finally {
      searching = false;
    }
  }

  function jumpToResult(result: SearchResult) {
    activeId = result.conversation_id;
    searchQuery = "";
    searchResults = [];
  }

  async function handleSend(event: Event) {
    event.preventDefault();
    if (!activeConversation || !draft.trim() || sending) return;
    const content = draft.trim();
    const targetId = activeConversation.id;
    draft = "";
    sending = true;
    streamingText = "";

    const optimisticUser: ChatMessage = {
      id: crypto.randomUUID(),
      role: "user",
      content,
      created_at: new Date().toISOString(),
    };
    conversations = conversations.map((c) =>
      c.id === targetId ? { ...c, messages: [...c.messages, optimisticUser] } : c,
    );

    try {
      const assistantMessage = await sendMessage(targetId, content);
      conversations = conversations.map((c) =>
        c.id === targetId ? { ...c, messages: [...c.messages, assistantMessage], title: c.title } : c,
      );
    } catch (e) {
      toast.error(String(e));
    } finally {
      sending = false;
      streamingText = "";
    }
  }
</script>

<div class="chat">
  <aside class="conversations">
    <Button variant="secondary" onclick={openNewChat} disabled={aiAccounts.length === 0}>
      <Plus size={15} /> New chat
    </Button>

    <label class="search-box">
      <Search size={14} />
      <input type="text" bind:value={searchQuery} oninput={handleSearchInput} placeholder="Search conversations…" />
    </label>

    {#if searchQuery.trim()}
      {#if searching}
        <p class="search-status">Searching…</p>
      {:else if searchResults.length === 0}
        <p class="search-status">No matches.</p>
      {:else}
        <ul class="conv-list">
          {#each searchResults as result (result.message_id)}
            <li>
              <button class="search-result" onclick={() => jumpToResult(result)}>
                <span class="search-role">{result.role === "user" ? "You" : "Assistant"}</span>
                <span class="search-snippet">{result.content}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    {:else if conversations.length > 0}
      <ul class="conv-list">
        {#each conversations as conversation (conversation.id)}
          <li class="conv-row">
            <button class="conv-item" class:active={conversation.id === activeId} onclick={() => (activeId = conversation.id)}>
              {conversation.title || "New conversation"}
            </button>
            <button
              class="conv-delete"
              aria-label="Delete conversation"
              onclick={() => handleDeleteConversation(conversation.id)}
            >
              <Trash2 size={12} />
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </aside>

  <div class="thread-pane">
    {#if aiAccounts.length === 0}
      <div class="empty-state">
        <Bot size={28} />
        <p>Add an OpenAI or Anthropic account to start chatting.</p>
        <Button variant="primary" onclick={onAddAccount}>Go to AI accounts</Button>
      </div>
    {:else if !activeConversation}
      <div class="empty-state">
        <Bot size={28} />
        <p>Pick a conversation, or start a new one.</p>
        <Button variant="primary" onclick={openNewChat}><Plus size={15} /> New chat</Button>
      </div>
    {:else}
      <div class="thread" bind:this={threadEl}>
        {#each activeConversation.messages as message (message.id)}
          <div class="bubble {message.role}">
            <p>{message.content}</p>
          </div>
        {/each}
        {#if sending}
          <div class="bubble assistant streaming">
            {#if streamingText}
              <p>{streamingText}</p>
            {:else}
              <p class="thinking">Thinking…</p>
            {/if}
          </div>
        {/if}
      </div>
      <form class="composer" onsubmit={handleSend}>
        <input type="text" bind:value={draft} placeholder="Message…" disabled={sending} />
        <Button variant="primary" size="icon" type="submit" disabled={sending || !draft.trim()} aria-label="Send">
          <SendHorizontal size={16} />
        </Button>
      </form>
    {/if}
  </div>
</div>

{#if showNewChat}
  <Modal title="New chat" onClose={() => (showNewChat = false)}>
    <form class="new-chat-form" onsubmit={confirmNewChat}>
      <label>
        Account
        <select bind:value={newChatAccountId} onchange={handleAccountChange}>
          {#each aiAccounts as account (account.id)}
            <option value={account.id}>{account.name}</option>
          {/each}
        </select>
      </label>
      <label>
        Model
        <input type="text" bind:value={newChatModel} placeholder="e.g. gpt-4o-mini" />
      </label>
      <div class="actions">
        <Button variant="secondary" type="button" onclick={() => (showNewChat = false)}>Cancel</Button>
        <Button variant="primary" type="submit">Start</Button>
      </div>
    </form>
  </Modal>
{/if}

<style>
  .chat {
    display: flex;
    height: calc(100vh - var(--space-6) - var(--space-7) - 2.6rem);
    max-height: 720px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--color-bg-elevated);
  }

  .conversations {
    width: 220px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
    border-right: 1px solid var(--color-border);
    background: var(--color-bg-subtle);
    overflow-y: auto;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text-muted);
  }

  .search-box input {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    color: var(--color-text);
    font-size: 0.82rem;
    padding: var(--space-1) 0;
  }

  .search-box input:focus {
    outline: none;
  }

  .search-status {
    color: var(--color-text-muted);
    font-size: 0.8rem;
    padding: var(--space-2);
  }

  .search-result {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    box-shadow: none;
    border-radius: var(--radius-sm);
    padding: var(--space-2);
    cursor: pointer;
  }

  .search-result:hover {
    background: var(--color-bg-hover);
  }

  .search-role {
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--color-accent);
  }

  .search-snippet {
    font-size: 0.82rem;
    color: var(--color-text-muted);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .conv-list {
    list-style: none;
    margin: var(--space-2) 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .conv-row {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .conv-item {
    flex: 1;
    min-width: 0;
    text-align: left;
    background: transparent;
    border: none;
    box-shadow: none;
    border-radius: var(--radius-sm);
    padding: var(--space-2);
    font-size: 0.82rem;
    color: var(--color-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
  }

  .conv-item:hover {
    background: var(--color-bg-hover);
  }

  .conv-item.active {
    background: var(--color-accent-soft);
    color: var(--color-accent);
    font-weight: 600;
  }

  .conv-delete {
    flex-shrink: 0;
    display: inline-flex;
    background: transparent;
    box-shadow: none;
    border: none;
    padding: var(--space-1);
    color: var(--color-text-muted);
    opacity: 0;
    transition: opacity var(--duration-fast) var(--ease-out);
  }

  .conv-row:hover .conv-delete {
    opacity: 1;
  }

  .conv-delete:hover {
    color: var(--color-danger);
    border-color: transparent;
  }

  .thread-pane {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    color: var(--color-text-muted);
    text-align: center;
    padding: var(--space-6);
  }

  .thread {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
  }

  .bubble {
    max-width: 70%;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
  }

  .bubble p {
    font-size: 0.88rem;
    line-height: 1.55;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .bubble.user {
    align-self: flex-end;
    background: var(--color-accent);
  }

  .bubble.user p {
    color: white;
  }

  .bubble.assistant {
    align-self: flex-start;
    background: var(--color-bg-subtle);
    border: 1px solid var(--color-border);
  }

  .thinking {
    color: var(--color-text-muted);
    animation: pulse 1.2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
  }

  .composer {
    display: flex;
    gap: var(--space-2);
    padding: var(--space-3);
    border-top: 1px solid var(--color-border);
  }

  .composer input {
    flex: 1;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    padding: 0.55em 0.7em;
    font-size: 0.9rem;
    font-family: inherit;
    background: var(--color-bg);
    color: var(--color-text);
  }

  .composer input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .new-chat-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .new-chat-form label {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font-size: 0.86rem;
    font-weight: 500;
  }

  .new-chat-form input,
  .new-chat-form select {
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    padding: 0.55em 0.7em;
    font-size: 0.9rem;
    font-family: inherit;
    background: var(--color-bg);
    color: var(--color-text);
  }

  .new-chat-form .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }
</style>
