import { ref, computed, onMounted, onBeforeUnmount, type Ref } from "vue";

const LINE_HEIGHT = 20;
const OVERSCAN = 15;

export function useVirtualList<T>(
  items: Ref<T[]>,
  containerRef: Ref<HTMLElement | null>,
) {
  const scrollTop = ref(0);
  const containerHeight = ref(600);

  function onScroll() {
    const el = containerRef.value;
    if (!el) return;
    scrollTop.value = el.scrollTop;
    containerHeight.value = el.clientHeight;
  }

  onMounted(() => {
    const el = containerRef.value;
    if (el) {
      containerHeight.value = el.clientHeight;
      el.addEventListener("scroll", onScroll, { passive: true });
    }
  });

  onBeforeUnmount(() => {
    containerRef.value?.removeEventListener("scroll", onScroll);
  });

  const totalHeight = computed(() => items.value.length * LINE_HEIGHT);

  const range = computed(() => {
    const start = Math.max(0, Math.floor(scrollTop.value / LINE_HEIGHT) - OVERSCAN);
    const end = Math.min(
      items.value.length,
      Math.ceil((scrollTop.value + containerHeight.value) / LINE_HEIGHT) + OVERSCAN,
    );
    return { start, end };
  });

  const visibleItems = computed(() =>
    items.value.slice(range.value.start, range.value.end).map((item, i) => ({
      item,
      index: range.value.start + i,
    })),
  );

  const paddingTop = computed(() => range.value.start * LINE_HEIGHT);
  const paddingBottom = computed(() =>
    Math.max(0, (items.value.length - range.value.end) * LINE_HEIGHT),
  );

  return { visibleItems, paddingTop, paddingBottom, totalHeight };
}
