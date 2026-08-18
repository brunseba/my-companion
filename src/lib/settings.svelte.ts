export type Theme = "system" | "light" | "dark";
export type DefaultSection = "overview" | "accounts" | "history";

const THEME_KEY = "my-companion:theme";
const DEFAULT_SECTION_KEY = "my-companion:default-section";

function readTheme(): Theme {
  const stored = typeof localStorage !== "undefined" ? localStorage.getItem(THEME_KEY) : null;
  return stored === "light" || stored === "dark" ? stored : "system";
}

function readDefaultSection(): DefaultSection {
  const stored = typeof localStorage !== "undefined" ? localStorage.getItem(DEFAULT_SECTION_KEY) : null;
  return stored === "overview" || stored === "accounts" || stored === "history" ? stored : "accounts";
}

function applyTheme(value: Theme) {
  if (typeof document === "undefined") return;
  // Empty string removes the attribute entirely, falling back to the OS
  // preference via the @media query in app.css.
  document.documentElement.dataset.theme = value === "system" ? "" : value;
}

let theme = $state<Theme>(readTheme());
let defaultSection = $state<DefaultSection>(readDefaultSection());

// Apply on load, before anything renders a visible flash of the wrong theme.
applyTheme(theme);

export const settings = {
  get theme() {
    return theme;
  },
  setTheme(value: Theme) {
    theme = value;
    localStorage.setItem(THEME_KEY, value);
    applyTheme(value);
  },
  get defaultSection() {
    return defaultSection;
  },
  setDefaultSection(value: DefaultSection) {
    defaultSection = value;
    localStorage.setItem(DEFAULT_SECTION_KEY, value);
  },
};
