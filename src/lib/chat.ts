import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  created_at: string;
}

export interface Conversation {
  id: string;
  account_id: string;
  model: string;
  title: string;
  messages: ChatMessage[];
  created_at: string;
  updated_at: string;
}

export interface CreateConversationInput {
  account_id: string;
  model: string;
}

export function listConversations(): Promise<Conversation[]> {
  return invoke("list_conversations");
}

export function createConversation(input: CreateConversationInput): Promise<Conversation> {
  return invoke("create_conversation", { input });
}

export function deleteConversation(id: string): Promise<void> {
  return invoke("delete_conversation", { id });
}

export function sendMessage(conversationId: string, content: string): Promise<ChatMessage> {
  return invoke("send_message", { conversationId, content });
}

interface ChatDeltaPayload {
  conversation_id: string;
  text: string;
}

/** Fires once per streamed token/chunk while a reply is being generated. */
export function onChatDelta(callback: (conversationId: string, text: string) => void): Promise<UnlistenFn> {
  return listen<ChatDeltaPayload>("chat:delta", (event) => {
    callback(event.payload.conversation_id, event.payload.text);
  });
}
