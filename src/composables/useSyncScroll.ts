import { ref, onMounted, onBeforeUnmount } from "vue";

export function useSyncScroll() {
  const leftPanelRef = ref<HTMLElement | null>(null);
  const rightPanelRef = ref<HTMLElement | null>(null);

  function setupSync() {
    if (!leftPanelRef.value || !rightPanelRef.value) return;

    const leftPanel = leftPanelRef.value;
    const rightPanel = rightPanelRef.value;
    let isSyncing = false;

    const handleLeftScroll = () => {
      if (isSyncing) return;
      isSyncing = true;
      rightPanel.scrollTop = leftPanel.scrollTop;
      requestAnimationFrame(() => { isSyncing = false; });
    };

    const handleRightScroll = () => {
      if (isSyncing) return;
      isSyncing = true;
      leftPanel.scrollTop = rightPanel.scrollTop;
      requestAnimationFrame(() => { isSyncing = false; });
    };

    leftPanel.addEventListener("scroll", handleLeftScroll);
    rightPanel.addEventListener("scroll", handleRightScroll);

    return () => {
      leftPanel.removeEventListener("scroll", handleLeftScroll);
      rightPanel.removeEventListener("scroll", handleRightScroll);
    };
  }

  let cleanup: (() => void) | null = null;

  onMounted(() => {
    if (!leftPanelRef.value || !rightPanelRef.value) {
      console.warn('[useSyncScroll] Refs not attached to DOM elements');
      return;
    }
    cleanup = setupSync() || null;
  });

  onBeforeUnmount(() => {
    if (cleanup) cleanup();
  });

  return {
    leftPanelRef,
    rightPanelRef,
    setupSync,
  };
}
