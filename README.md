# Forziere

Applicazione desktop (Tauri 2 + Vue 3) per storicizzare utenti e password associate e generare password sicure.

## Funzionalità

- Archivio di voci: servizio, URL, utente, password, note, tag/categoria, date di creazione e modifica.
- Creazione, modifica, eliminazione e ricerca (servizio, utente, tag).
- Generatore di password sicure configurabile:
  - numero di caratteri (4–128)
  - lettere maiuscole sì/no
  - lettere minuscole sì/no
  - numeri sì/no
  - caratteri speciali sì/no
  - almeno un carattere per ogni classe selezionata; casualità da generatore crittografico.
- Copia della password negli appunti, mostra/nascondi password.

## Requisiti

- Node.js (>= 20) e npm
- Rust (stable, edition 2024) e cargo
- Dipendenze di sistema Tauri per Linux: vedi <https://tauri.app/start/prerequisites/>

## Comandi

```bash
npm install                # installa le dipendenze frontend
npm run tauri dev          # avvia l'app in sviluppo
npm run tauri build        # build di produzione
npm test                   # test frontend (Vitest)
npm run test:rust          # test backend (cargo test)
npm run regression         # regressione completa: test frontend + Rust + build
```

> **Terminale dello snap di VS Code (Linux):** se `npm run tauri dev` crasha con `GLib-GIO-ERROR ... antialiasing`
> o `symbol lookup error ... libpthread`, usa `./scripts/dev-clean.sh`, che ripulisce le variabili d'ambiente ereditate.

## Build, installazione e rimozione

`npm run tauri build` produce i pacchetti nativi in `src-tauri/target/release/bundle/` (va eseguito sul sistema
operativo di destinazione). Guida completa per Linux, Windows e macOS, con installazione, disinstallazione, firma e
posizione dei dati: [`docs/BUILD-E-INSTALLAZIONE.md`](docs/BUILD-E-INSTALLAZIONE.md).

Esempio Linux (verificato): `sudo apt install ./src-tauri/target/release/bundle/deb/forziere_1.0.0_amd64.deb`
per installare, `sudo apt remove forziere` per rimuovere (i dati in `vault.json` non vengono cancellati).

### Icona

Sorgente: `design/icon.svg`. Per rigenerare tutte le dimensioni (PNG, `.ico`, `.icns`) dopo una modifica:

```bash
inkscape design/icon.svg -o design/icon.png -w 1024 -h 1024   # oppure un altro renderer SVG
npm run tauri icon design/icon.png
rm -rf src-tauri/icons/android src-tauri/icons/ios            # non servono (app solo desktop)
cp design/icon.svg public/icon.svg                            # favicon del frontend
```

## Dati

Le voci sono salvate in `vault.json` nella cartella dati dell'app (`app_data_dir`, su Linux tipicamente
`~/.local/share/com.giorgio-marini.forziere/`). La scrittura è atomica (file temporaneo + rename).

> **Attenzione sicurezza:** il file NON è cifrato e non c'è master password. Le password sono leggibili in chiaro
> da chiunque abbia accesso al file. La cifratura è tra gli sviluppi futuri (vedi `TODO.md`).

## Struttura

- `src/` – frontend Vue (componenti, wrapper `api.js`)
- `design/icon.svg` – sorgente dell'icona (lucchetto verde, stile cyberpunk); `design/icon.png` è il render 1024×1024
- `src-tauri/src/` – backend Rust (`store.rs`, `generator.rs`, comandi Tauri in `lib.rs`)

## Documenti di progetto

- `docs/BUILD-E-INSTALLAZIONE.md` – build, installazione e rimozione per OS
- `TODO.md` – avanzamento lavori
- `CHANGELOG.md` – storico delle modifiche
