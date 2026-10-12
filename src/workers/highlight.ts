/// <reference lib="webworker" />

export function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/\"/g, "&quot;");
}

/** Lightweight, dependency-free token colouring for common source files. */
export function highlightSource(source: string, language = ""): string {
  const escaped = escapeHtml(source);
  const lang = language.toLowerCase();
  if (!(lang.match(/(js|ts|jsx|tsx|rs|go|py|java|c|cpp|css|html|vue|json|sh)/) || /[{};]|=>/.test(source))) return escaped;
  return escaped
    .replace(/(\/\/.*|#.*|\/\*[\s\S]*?\*\/)/g, '<span class="tok-comment">$1</span>')
    .replace(/(&quot;.*?&quot;|&#39;.*?&#39;|`.*?`)/g, '<span class="tok-string">$1</span>')
    .replace(/\b(const|let|var|function|return|if|else|for|while|class|import|from|export|pub|fn|struct|impl|match|async|await|def|true|false|null|None)\b/g, '<span class="tok-keyword">$1</span>');
}

export function highlightVisible(lines: string[], language: string, start = 0, count = 200): string[] {
  const out = lines.map(escapeHtml);
  const bytes = lines.reduce((sum, line) => sum + line.length, 0);
  if (lines.length > 20000 || bytes > 1024 * 1024) return out;
  for (let i = Math.max(0, start); i < Math.min(lines.length, start + count); i++) out[i] = highlightSource(lines[i], language);
  return out;
}

const workerScope = globalThis as unknown as { onmessage?: (event: MessageEvent<{ id: number; lines: string[]; language?: string; start?: number; count?: number }>) => void; postMessage?: (value: unknown) => void };
if (typeof workerScope.postMessage === "function") {
  workerScope.onmessage = ({ data }) => {
    workerScope.postMessage?.({ id: data.id, lines: highlightVisible(data.lines, data.language ?? "", data.start, data.count) });
  };
}
