import { invoke } from "@tauri-apps/api/core";

export interface SearchResult {
  message_id: string;
  conversation_id: string;
  role: "user" | "assistant";
  content: string;
  /** Vector distance - lower is more similar. Not a 0-1 percentage. */
  distance: number;
}

/**
 * Semantic search over indexed chat messages. Only messages sent since this
 * feature shipped are indexed - there's no backfill of older conversations
 * yet.
 */
export function searchConversations(query: string): Promise<SearchResult[]> {
  return invoke("search_conversations", { query });
}
