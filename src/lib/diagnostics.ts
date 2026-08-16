import { invoke } from "@tauri-apps/api/core";

export interface ResourceUsage {
  memory_bytes: number;
  cpu_percent: number;
  accounts_file_bytes: number;
  binary_bytes: number;
}

export function getResourceUsage(): Promise<ResourceUsage> {
  return invoke("resource_usage");
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  return `${value.toFixed(value < 10 ? 1 : 0)} ${units[unitIndex]}`;
}
