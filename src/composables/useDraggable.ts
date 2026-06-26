import { ref, computed, onUnmounted } from "vue";

export function useDraggable() {
  const offset = ref({ x: 0, y: 0 });
  let dragStart: { px: number; py: number; ox: number; oy: number } | null = null;

  function onMove(e: MouseEvent) {
    if (!dragStart) return;
    offset.value = {
      x: dragStart.ox + (e.clientX - dragStart.px),
      y: dragStart.oy + (e.clientY - dragStart.py),
    };
  }

  function onEnd() {
    dragStart = null;
    window.removeEventListener("mousemove", onMove);
    window.removeEventListener("mouseup", onEnd);
  }

  function onDragStart(e: MouseEvent) {
    if ((e.target as HTMLElement).closest(".close-btn, button, input, textarea, select, a")) return;
    dragStart = {
      px: e.clientX,
      py: e.clientY,
      ox: offset.value.x,
      oy: offset.value.y,
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onEnd);
    e.preventDefault();
  }

  const dragStyle = computed(() => ({
    transform: `translate(${offset.value.x}px, ${offset.value.y}px)`,
  }));

  // Если компонент размонтируется с зажатой мышью во время перетаскивания
  // (например, диалог закрыли по Esc прямо во время drag), снять висящие
  // window-listener'ы, иначе они утекут.
  onUnmounted(onEnd);

  return { dragStyle, onDragStart };
}
