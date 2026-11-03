import { createApp } from "vue";
import App from "./App.vue";
import { getInitialLocale, i18n, setLocale } from "./i18n";
import "./assets/main.css";

setLocale(getInitialLocale());
createApp(App).use(i18n).mount("#app");
