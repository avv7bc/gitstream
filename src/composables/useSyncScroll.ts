import { ref, onMounted, onBeforeUnmount } from "vue";

export function useSyncScroll() {
  const leftPanelRef = ref<HTMLElement | null>(null);
  const rightPanelRef = ref<HTMLElement | null>(null);
  const isSyncing = ref(false);

  function setupSync() {
    if (!leftPanelRef.value || !rightPanelRef.value) return;

    const leftPanel = leftPanelRef.value;
    const rightPanel = rightPanelRef.value;

    const handleLeftScroll = () => {
      if (isSyncing.value) return;
      isSyncing.value = true;
      rightPanel.scrollTop = leftPanel.scrollTop;
      isSyncing.value = false;
    };

    const handleRightScroll = () => {
      if (isSyncing.value) return;
      isSyncing.value = true;
      leftPanel.scrollTop = rightPanel.scrollTop;
      isSyncing.value = false;
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
