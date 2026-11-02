# TODO – Avanzamento lavori

Ultimo aggiornamento: 2026-10-23 – versione corrente: 1.0.0

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
- [ ] Verifica visiva dell'icona su desktop (menu applicazioni / barra)

## Futuro
- [ ] Master password e cifratura del vault (Argon2 + AES-256-GCM)
- [ ] Import/export (CSV/JSON)
- [ ] Blocco automatico per inattività
- [ ] Svuotamento automatico degli appunti
