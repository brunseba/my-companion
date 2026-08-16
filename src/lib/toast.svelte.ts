export interface Toast {
  id: number;
  kind: "success" | "error" | "info";
  message: string;
}

let nextId = 0;
const toasts = $state<Toast[]>([]);

const DURATIONS: Record<Toast["kind"], number> = {
  success: 3000,
  info: 3000,
  error: 6000, // errors get more time to read - they're often longer messages
};

function push(kind: Toast["kind"], message: string) {
  const id = nextId++;
  toasts.push({ id, kind, message });
  setTimeout(() => dismiss(id), DURATIONS[kind]);
}

function dismiss(id: number) {
  const index = toasts.findIndex((t) => t.id === id);
  if (index !== -1) toasts.splice(index, 1);
}

export const toast = {
  success: (message: string) => push("success", message),
  error: (message: string) => push("error", message),
  info: (message: string) => push("info", message),
  dismiss,
  get all() {
    return toasts;
  },
};
