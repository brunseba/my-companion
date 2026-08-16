import { invoke } from "@tauri-apps/api/core";
import type { Account, CreateAccountInput, UpdateAccountInput } from "./types";

export function listAccounts(): Promise<Account[]> {
  return invoke("list_accounts");
}

export function createAccount(input: CreateAccountInput): Promise<Account> {
  return invoke("create_account", { input });
}

export function updateAccount(id: string, input: UpdateAccountInput): Promise<Account> {
  return invoke("update_account", { id, input });
}

export function deleteAccount(id: string): Promise<void> {
  return invoke("delete_account", { id });
}

export function testAccount(id: string): Promise<Account> {
  return invoke("test_account", { id });
}

export function oidcLogin(id: string): Promise<Account> {
  return invoke("oidc_login", { id });
}

export function refreshOidcSession(id: string): Promise<Account> {
  return invoke("refresh_oidc_session", { id });
}
