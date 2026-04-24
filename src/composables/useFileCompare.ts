import { ref } from "vue";

export interface CompareTarget {
  path: string;
  // null/undefined => working tree vs index/HEAD
  oid?: string | null;
  staged?: boolean;
}

const target = ref<CompareTarget | null>(null);

export function useFileCompare() {
  function open(t: CompareTarget) {
    target.value = t;
  }
  function close() {
    target.value = null;
  }
  return { target, open, close };
}
