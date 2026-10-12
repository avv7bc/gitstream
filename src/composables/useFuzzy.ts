import { computed, type Ref } from "vue";

export function fuzzyScore(value: string, query: string): number {
  const haystack = value.toLocaleLowerCase();
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return 0;
  let cursor = 0;
  let score = 0;
  let previous = -2;
  for (const char of needle) {
    const at = haystack.indexOf(char, cursor);
    if (at < 0) return -1;
    score += at === previous + 1 ? 4 : at === 0 || " /_-.:".includes(haystack[at - 1] ?? "") ? 3 : 1;
    previous = at;
    cursor = at + 1;
  }
  return score - value.length / 1000;
}

export function useFuzzy<T>(source: Ref<T[]>, text: Ref<string>, label: (item: T) => string) {
  const results = computed(() => source.value
    .map((item, index) => ({ item, index, score: fuzzyScore(label(item), text.value) }))
    .filter((entry) => !text.value.trim() || entry.score >= 0)
    .sort((a, b) => b.score - a.score));
  return { results };
}

export function rememberRecent(key: string, value: string, limit = 8): string[] {
  if (typeof localStorage === "undefined") return [value];
  let previous: unknown = [];
  try { previous = JSON.parse(localStorage.getItem(key) ?? "[]") as unknown; } catch { /* recover from corrupt storage */ }
  const list = Array.isArray(previous) ? previous.filter((item): item is string => typeof item === "string") : [];
  const next = [value, ...list.filter((item) => item !== value)].slice(0, limit);
  try { localStorage.setItem(key, JSON.stringify(next)); } catch { /* storage may be unavailable */ }
  return next;
}
