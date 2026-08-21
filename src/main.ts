import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { router } from "./router";
import { i18n } from "./lib/i18n";
import { useThemeStore } from "./stores/theme";
import "./styles.css";

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);
app.use(router);
app.use(i18n);

// 挂载前先应用主题，避免首屏闪烁
useThemeStore(pinia).init();

app.mount("#app");
