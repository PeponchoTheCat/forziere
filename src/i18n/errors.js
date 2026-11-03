import { i18n } from "./index";

/**
 * Traduce un errore del backend. Il backend restituisce un codice (`codice` o `codice:arg1:arg2`);
 * un testo che non è un codice noto (es. errore di I/O del sistema operativo) passa invariato.
 */
export function translateError(e) {
  const raw = String(e);
  const [code, ...args] = raw.split(":");
  const key = `errors.${code}`;
  const { te, t } = i18n.global;
  return te(key) ? t(key, args) : raw;
}
