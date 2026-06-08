import { createApp } from "vue";
import App from "./App.vue";
import { tooltip } from "./directives/tooltip";
import "./styles/main.css";
import "./styles/dialogs.css";

createApp(App).directive("tooltip", tooltip).mount("#app");
