import { defineConfig } from "vitest/config";
import { resolve } from "path";

// Юнит-тесты на чистую логику фронтенда. Компонентные/DOM-тесты не нужны —
// окружение node, цель — парсинг/форматирование/escaping без браузера.
export default defineConfig({
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
