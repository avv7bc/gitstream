function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function escapeRegex(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * Returns HTML string with matches of `query` wrapped in <mark class="hl">.
 * Safe for v-html — escapes input first.
 */
export function highlight(text: string | number | null | undefined, query: string): string {
  const s = text == null ? "" : String(text);
  const q = query.trim();
  if (!q) return escapeHtml(s);
  const escaped = escapeHtml(s);
  const re = new RegExp(`(${escapeRegex(escapeHtml(q))})`, "gi");
  return escaped.replace(re, '<mark class="hl">$1</mark>');
}
