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

export function oauthLogin(id: string): Promise<Account> {
  return invoke("oauth_login", { id });
}

export function refreshOauthSession(id: string): Promise<Account> {
  return invoke("refresh_oauth_session", { id });
}

export interface DataInfo {
  accounts_file: string;
  keychain_service: string;
}

export function getAppDataInfo(): Promise<DataInfo> {
  return invoke("app_data_info");
}

export function resetAllData(): Promise<void> {
  return invoke("reset_all_data");
}
