# Changelog

Tutte le modifiche rilevanti al progetto sono documentate in questo file.
Il formato segue [Keep a Changelog](https://keepachangelog.com/it-IT/1.1.0/) e il progetto adotta il
[Versionamento Semantico](https://semver.org/lang/it/).

## [Unreleased]

### Changed
- 2026-11-02: progetto rinominato in **Forziere** (crate `forziere`, lib `forziere_lib`, productName, titolo finestra e pagina, identificativo `com.giorgio-marini.forziere`); corretto il refuso "mamanger"; documentazione aggiornata.

### Added
- 2026-11-02: migrazione automatica di `vault.json` dalla vecchia cartella dati (`com.giorgio-marini.fcs-password-mamanger`) alla nuova, senza sovrascrivere né cancellare il vecchio file; 3 test AAA.

## [1.0.0] - 2026-10-23

Prima versione rilasciata. Verificata a mano dall'utente e installata da pacchetto `.deb` su Linux.

### Changed
- 2026-10-23: UI in stile cyberpunk (solo tema scuro) in toni di verde, con glow tenue (niente magenta né colori fluo): angoli tagliati, scanline CRT, font monospace. Solo CSS: markup e comportamento invariati.
- 2026-10-23: UI compatta: font 13px, form a due colonne (campi e generatore affiancati), lista 230px, finestra di default 980×640 così tutto sta senza scorrere.

### Added
- 2026-10-23: icona dell'applicazione (lucchetto verde con tracce di circuito, angoli tagliati): sorgente `design/icon.svg`, set completo in `src-tauri/icons/` (PNG, `.ico`, `.icns`), favicon in `public/icon.svg`; titolo pagina "Forziere".
- 2026-10-23: `docs/BUILD-E-INSTALLAZIONE.md`: build, installazione e rimozione su Linux (verificato: .deb, .rpm, AppImage), Windows e macOS (da documentazione Tauri, non provati), posizione dei dati, versioning, CI.
- 2026-10-23: `scripts/dev-clean.sh` per avviare `tauri dev` da un terminale snap di VS Code (variabili GTK/GIO/XDG ereditate causavano il crash all'avvio).
- 2026-10-23: unit test AAA: 33 Rust (generatore, store, comandi Tauri via `tauri::test`) e 52 frontend (Vitest + Vue Test Utils); script `test`, `test:coverage`, `test:rust`, `regression`.
- 2026-10-23: frontend Vue: lista con ricerca, form voce (mostra/nascondi, copia), generatore password (lunghezza, maiuscole, minuscole, numeri, speciali), wrapper `api.js`, stili in `assets/main.css`.
- 2026-10-23: backend Rust: modello `Entry`, storage `vault.json` in chiaro con scrittura atomica (`store.rs`), generatore password (`generator.rs`), comandi Tauri `list_entries`, `save_entry`, `delete_entry`, `generate_password`, con test unitari.
- 2026-10-23: documentazione iniziale (`README.md`, `TODO.md`, `CHANGELOG.md`).

### Removed
- 2026-10-23: logo dello scaffold (`public/vite.svg`, `public/tauri.svg`) e icone Android/iOS non usate.
- 2026-10-23: comando `greet` e plugin `tauri-plugin-opener` (Rust, capability e dipendenza npm).
