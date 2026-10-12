import { onBeforeUnmount, ref, type Ref, watch } from "vue";
import { escapeHtml, highlightVisible } from "@/workers/highlight";

/** Highlight only the rows a virtualized view asks for. Large files stay plain text. */
export function useCodeHighlight(lines: Ref<string[]>, language: Ref<string>, visibleRange: Ref<{ start: number; count: number }>) {
  const highlighted = ref<string[]>([]);
  let worker: Worker | null = null;
  let requestId = 0;
  try {
    if (typeof Worker !== "undefined") {
      worker = new Worker(new URL("../workers/highlight.ts", import.meta.url), { type: "module" });
      worker.onmessage = (event: MessageEvent<{ id: number; lines: string[] }>) => {
        if (event.data.id === requestId) highlighted.value = event.data.lines;
      };
    }
  } catch {
    worker = null;
  }
  const refresh = () => {
    const range = visibleRange.value;
    const bytes = lines.value.reduce((sum, line) => sum + line.length, 0);
    if (lines.value.length > 20000 || bytes > 1024 * 1024) {
      highlighted.value = lines.value.map(escapeHtml);
      return;
    }
    requestId += 1;
    if (worker) worker.postMessage({ id: requestId, lines: lines.value, language: language.value, start: range.start, count: range.count });
    else highlighted.value = highlightVisible(lines.value, language.value, range.start, range.count);
  };
  watch([lines, language, visibleRange], refresh, { immediate: true, deep: true });
  onBeforeUnmount(() => worker?.terminate());
  return { highlighted, refresh };
}
