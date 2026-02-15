export interface Theme {
  name: string;
  label: string;
  vars: Record<string, string>;
}

export const themes: Theme[] = [
  {
    name: "dark",
    label: "Dark",
    vars: {
      "--bg-primary": "#0a0a0f",
      "--bg-secondary": "#13131a",
      "--bg-tertiary": "#1a1a24",
      "--bg-hover": "#1e1e2a",
      "--border": "#222233",
      "--border-hover": "#333355",
      "--text-primary": "#e0e0e0",
      "--text-secondary": "#888899",
      "--text-muted": "#555566",
      "--accent": "#7c5cff",
      "--accent-hover": "#6b4ae0",
      "--accent-glow": "rgba(124, 92, 255, 0.3)",
      "--success": "#44ff88",
      "--warning": "#ffaa44",
      "--danger": "#ff4444",
      "--danger-hover": "#dd3333",
      "--sidebar-bg": "#0d0d14",
      "--card-bg": "#13131a",
    },
  },
  {
    name: "light",
    label: "Light",
    vars: {
      "--bg-primary": "#f5f5f8",
      "--bg-secondary": "#ffffff",
      "--bg-tertiary": "#eeeef2",
      "--bg-hover": "#e8e8f0",
      "--border": "#d0d0dd",
      "--border-hover": "#b0b0cc",
      "--text-primary": "#1a1a2e",
      "--text-secondary": "#666680",
      "--text-muted": "#999aaa",
      "--accent": "#6c4ce0",
      "--accent-hover": "#5a3ac0",
      "--accent-glow": "rgba(108, 76, 224, 0.2)",
      "--success": "#22cc66",
      "--warning": "#ee8833",
      "--danger": "#ee3344",
      "--danger-hover": "#cc2233",
      "--sidebar-bg": "#ebebf0",
      "--card-bg": "#ffffff",
    },
  },
];

export function applyTheme(themeName: string) {
  const theme = themes.find((t) => t.name === themeName) || themes[0];
  const root = document.documentElement;
  for (const [key, value] of Object.entries(theme.vars)) {
    root.style.setProperty(key, value);
  }
}

export function getTheme(name: string): Theme {
  return themes.find((t) => t.name === name) || themes[0];
}
