import { Sparkles, Cloud, Boxes, ShieldCheck } from "lucide-svelte";

export type AccountCategory = "ai" | "csp" | "k8s" | "oidc";
export type AccountStatus = "unknown" | "valid" | "expired" | "error";
// All lucide-svelte icons share this component shape, so any one of them works as the type.
export type IconComponent = typeof Sparkles;

export interface Account {
  id: string;
  category: AccountCategory;
  provider: string;
  name: string;
  config: Record<string, unknown>;
  status: AccountStatus;
  created_at: string;
  updated_at: string;
  last_validated_at: string | null;
  last_error: string | null;
  session_expires_at: string | null;
}

export interface CreateAccountInput {
  category: AccountCategory;
  provider: string;
  name: string;
  config: Record<string, unknown>;
  secret?: Record<string, unknown>;
}

export interface UpdateAccountInput {
  name?: string;
  config?: Record<string, unknown>;
  secret?: Record<string, unknown>;
}

export interface FieldSchema {
  key: string;
  label: string;
  kind: "text" | "password" | "textarea";
  /** Goes into the secret payload (keychain) instead of the config payload. */
  secret?: boolean;
  required?: boolean;
  placeholder?: string;
}

export interface ProviderSchema {
  provider: string;
  category: AccountCategory;
  label: string;
  fields: FieldSchema[];
}

export const PROVIDER_SCHEMAS: ProviderSchema[] = [
  {
    provider: "openai",
    category: "ai",
    label: "OpenAI",
    fields: [
      { key: "api_key", label: "API key", kind: "password", secret: true, required: true },
      { key: "organization", label: "Organization ID", kind: "text" },
      { key: "base_url", label: "Base URL override", kind: "text", placeholder: "https://api.openai.com/v1" },
    ],
  },
  {
    provider: "anthropic",
    category: "ai",
    label: "Anthropic",
    fields: [
      { key: "api_key", label: "API key", kind: "password", secret: true, required: true },
      { key: "base_url", label: "Base URL override", kind: "text", placeholder: "https://api.anthropic.com" },
    ],
  },
  {
    provider: "aws",
    category: "csp",
    label: "AWS",
    fields: [
      { key: "region", label: "Region", kind: "text", placeholder: "us-east-1" },
      { key: "access_key_id", label: "Access key ID", kind: "text", secret: true, required: true },
      { key: "secret_access_key", label: "Secret access key", kind: "password", secret: true, required: true },
      { key: "session_token", label: "Session token (optional)", kind: "password", secret: true },
    ],
  },
  {
    provider: "azure",
    category: "csp",
    label: "Azure",
    fields: [
      { key: "tenant_id", label: "Tenant ID", kind: "text", required: true },
      { key: "subscription_id", label: "Subscription ID", kind: "text" },
      { key: "client_id", label: "Client ID", kind: "text", secret: true, required: true },
      { key: "client_secret", label: "Client secret", kind: "password", secret: true, required: true },
    ],
  },
  {
    provider: "gcp",
    category: "csp",
    label: "Google Cloud",
    fields: [
      { key: "project_id", label: "Project ID", kind: "text" },
      { key: "service_account_json", label: "Service account JSON", kind: "textarea", secret: true, required: true },
    ],
  },
  {
    provider: "scaleway",
    category: "csp",
    label: "Scaleway",
    fields: [
      { key: "region", label: "Region", kind: "text", placeholder: "fr-par" },
      { key: "project_id", label: "Project ID", kind: "text" },
      { key: "access_key", label: "Access key", kind: "text", secret: true, required: true },
      { key: "secret_key", label: "Secret key", kind: "password", secret: true, required: true },
    ],
  },
  {
    provider: "kubeconfig",
    category: "k8s",
    label: "Kubeconfig context",
    fields: [
      { key: "path", label: "Kubeconfig path", kind: "text", required: true, placeholder: "~/.kube/config" },
      { key: "context", label: "Context name", kind: "text", required: true },
      { key: "namespace", label: "Default namespace", kind: "text" },
    ],
  },
  {
    provider: "oidc",
    category: "oidc",
    label: "OIDC provider",
    fields: [
      { key: "issuer_url", label: "Issuer URL", kind: "text", required: true, placeholder: "https://issuer.example.com" },
      { key: "client_id", label: "Client ID", kind: "text", required: true },
      { key: "client_secret", label: "Client secret (optional)", kind: "password", secret: true },
      { key: "scopes", label: "Scopes (space separated)", kind: "text", placeholder: "openid profile email" },
    ],
  },
];

export const CATEGORY_LABELS: Record<AccountCategory, string> = {
  ai: "AI",
  csp: "Cloud",
  k8s: "Kubernetes",
  oidc: "OIDC",
};

export const CATEGORY_ICONS: Record<AccountCategory, IconComponent> = {
  ai: Sparkles,
  csp: Cloud,
  k8s: Boxes,
  oidc: ShieldCheck,
};

export function providersForCategory(category: AccountCategory): ProviderSchema[] {
  return PROVIDER_SCHEMAS.filter((p) => p.category === category);
}

export function schemaFor(provider: string): ProviderSchema | undefined {
  return PROVIDER_SCHEMAS.find((p) => p.provider === provider);
}
