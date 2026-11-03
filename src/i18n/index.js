import { createI18n } from "vue-i18n";
import en from "./locales/en.json";
import it from "./locales/it.json";

export const SUPPORTED = ["en", "it"];
export const DEFAULT_LOCALE = "en";
const STORAGE_KEY = "forziere.locale";

/** Lingua salvata dall'utente, altrimenti `en` (localStorage può non essere disponibile). */
export function getInitialLocale() {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (SUPPORTED.includes(saved)) return saved;
  } catch {
    // ignorato: si usa il default
  }
  return DEFAULT_LOCALE;
}

export const i18n = createI18n({
  legacy: false,
  locale: getInitialLocale(),
  fallbackLocale: DEFAULT_LOCALE,
  messages: { en, it },
});

/** Cambia lingua, la ricorda e aggiorna l'attributo `lang` della pagina. */
export function setLocale(code) {
  if (!SUPPORTED.includes(code)) return;
  i18n.global.locale.value = code;
  document.documentElement.lang = code;
  try {
    localStorage.setItem(STORAGE_KEY, code);
  } catch {
    // ignorato: la scelta vale solo per questa sessione
  }
}
