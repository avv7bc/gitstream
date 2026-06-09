import { createApp } from "vue";
import App from "./App.vue";
import { tooltip } from "./directives/tooltip";
import { logError } from "./composables/useProgress";
import "./styles/main.css";
import "./styles/dialogs.css";

// Страховочная сеть: любая необработанная ошибка (в т.ч. отклонённый промис
// git-операции без локального catch) попадает в Git output, а не исчезает в
// консоли. Локальные обработчики, где они есть, отрабатывают раньше и обычно
// не пробрасывают — сюда долетает только то, что иначе упало бы молча.
window.addEventListener("unhandledrejection", (event) => {
  const reason = event.reason;
  const message =
    typeof reason === "string"
      ? reason
      : reason && typeof reason === "object" && "message" in reason
        ? String((reason as { message: unknown }).message)
        : String(reason);
  logError(message);
});

createApp(App).directive("tooltip", tooltip).mount("#app");
