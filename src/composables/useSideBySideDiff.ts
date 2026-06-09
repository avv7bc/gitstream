// Тонкая обёртка над чистой логикой word-diff (src/utils/wordDiff.ts), вынесенной
// для юнит-тестов. Типы ре-экспортируются для обратной совместимости с
// компонентами, импортирующими их отсюда.
import { enrichAllHunks, enrichHunkWithWordDiff } from "@/utils/wordDiff";

export type {
  WordDiffSpan,
  DiffLineWithWordDiff,
  DiffHunkWithWordDiff,
} from "@/utils/wordDiff";

export function useSideBySideDiff() {
  return {
    enrichHunkWithWordDiff,
    enrichAllHunks,
  };
}
