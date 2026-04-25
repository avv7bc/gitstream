import { ref, watch } from "vue";

export type ThemeMode = "system" | "light" | "dark" | "material" | "smartgit" | "catppuccin" | "dracula";

const STORAGE_KEY = "gitstream-theme";
const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

function loadMode(): ThemeMode {
  const v = localStorage.getItem(STORAGE_KEY);
  return v === "light" || v === "dark" || v === "material" || v === "smartgit" || v === "catppuccin" || v === "dracula" || v === "system" ? (v as ThemeMode) : "system";
}

const mode = ref<ThemeMode>(loadMode());

function resolve(m: ThemeMode): "light" | "dark" | "material" | "smartgit" | "catppuccin" | "dracula" {
  if (m === "system") return mediaQuery.matches ? "dark" : "light";
  return m;
}

function apply() {
  document.documentElement.setAttribute("data-theme", resolve(mode.value));
}

apply();

watch(mode, (m) => {
  localStorage.setItem(STORAGE_KEY, m);
  apply();
});

mediaQuery.addEventListener("change", () => {
  if (mode.value === "system") apply();
});

export function useTheme() {
  return { mode };
}
