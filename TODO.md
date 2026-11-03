# TODO – Avanzamento lavori

Ultimo aggiornamento: 2026-11-03 – versione corrente: 1.0.0

## Fase 0 – Documentazione
- [x] README.md
- [x] TODO.md
- [x] CHANGELOG.md

## Fase 1 – Backend (Rust)
- [x] Dipendenze: `rand`, `uuid`, `chrono`
- [x] Modello `Entry` (servizio, URL, utente, password, note, tag, date)
- [x] Modulo `store.rs`: lettura/scrittura atomica di `vault.json`, state `Mutex`
- [x] Modulo `generator.rs`: `GenOptions` e generazione password
- [x] Comandi Tauri: `list_entries`, `save_entry`, `delete_entry`, `generate_password`
- [x] Rimozione `greet` e plugin opener se non usato
- [x] Test unitari generatore

## Fase 2 – Frontend (Vue)
- [x] `api.js` (wrapper `invoke`)
- [x] Stili globali in `assets/main.css`
- [x] `EntryList.vue` con ricerca
- [x] `EntryForm.vue` (mostra/nascondi, copia)
- [x] `PasswordGenerator.vue`
- [x] `App.vue` layout lista + dettaglio

## Fase 2b – Test unitari (AAA)
- [x] Rust: `generator` (10), `store` (17), comandi Tauri (6) = 33 test
- [x] Frontend: `api`, `EntryList`, `PasswordGenerator`, `EntryForm`, `App` = 52 test
- [x] Script `npm run regression` (frontend + Rust + build)

## Fase 3 – Verifica
- [x] Regressione completa verde (`npm run regression`)
- [x] Avvio dell'app con `./scripts/dev-clean.sh`: compila, Vite parte, nessun panic/errore nel log
- [x] Prova manuale in finestra: CRUD + persistenza di `vault.json` dopo riavvio (eseguita dall'utente, esito ok)
- [x] Prova generatore con tutte le combinazioni di opzioni (eseguita dall'utente, esito ok)

## Fase 4 – Restyling
- [x] Tema cyberpunk (CSS globale + stili scoped dei componenti)
- [x] Verifica visiva in finestra (screenshot dell'utente: tutto visibile senza scorrere)

## Fase 5 – Documentazione distribuzione
- [x] Guida build/installazione/rimozione per OS
- [x] Build Linux verificato (.deb, .rpm, AppImage)
- [ ] Build e prova su Windows e macOS (non disponibili qui)
- [ ] Workflow CI GitHub Actions (progetto non ancora su git)

## Fase 6 – Icona
- [x] Icona SVG + set completo Tauri, favicon, verifica nel pacchetto .deb
- [x] Verifica visiva dell'icona su desktop (menu applicazioni / barra)

## Fase 7 – Master password e cifratura
- [x] `crypto.rs`: Argon2id + AES-256-GCM, nonce nuovo a ogni scrittura, header autenticato (AAD)
- [x] `store.rs` a stati (NoVault / LegacyPlaintext / Locked / Unlocked), formato v2, migrazione in chiaro → cifrato
- [x] Comandi `vault_status`, `setup_vault`, `unlock`, `lock`, `change_password`
- [x] Frontend: `UnlockScreen`, `ChangePassword`, pulsante Blocca in `App.vue`
- [x] Test AAA Rust (72) e frontend (83); regressione verde
- [x] Prova manuale in finestra (eseguita dall'utente, esito ok): migrazione di un vault in chiaro, voci intatte, blocca/sblocco con password errata e giusta, cambio password, riavvio con la nuova

## Fase 8 – Audit dipendenze
- [x] `scripts/audit-rust.sh` + `npm run audit:rust` (errore chiaro se manca `cargo-audit`)
- [x] `audit:rust` inserito in `npm run regression`
- [x] `cargo-audit` installato; prima esecuzione: 0 vulnerabilità, 2 warning noti (`proc-macro-error` unmaintained RUSTSEC-2024-0370, `glib` 0.18.5 unsound RUSTSEC-2024-0429, dallo stack GTK3 di Tauri); regressione verde

## Fase 9 – Multilingua (en predefinita, it)
- [x] `vue-i18n`, `src/i18n/` (index, errors, `locales/en.json`, `locales/it.json`), `test-setup.js`
- [x] Tutti i componenti tradotti; `LanguageSwitch` in barra e schermata di sblocco; scelta in `localStorage`
- [x] Errori Rust come codici + `translateError`; test Rust e frontend aggiornati (104 frontend, 72 Rust)
- [x] Test di parità chiavi en/it; regressione verde
- [x] Prova manuale in finestra (eseguita dall'utente): primo avvio in inglese, cambio lingua, riavvio, errori tradotti

## Futuro
- [ ] Import/export (CSV/JSON)
- [ ] Blocco automatico per inattività
- [ ] Svuotamento automatico degli appunti
