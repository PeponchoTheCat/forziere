# Build, installazione e rimozione

Guida per produrre l'eseguibile di **Forziere**, installarlo e rimuoverlo su Linux, Windows e macOS.

> **Regola fondamentale:** Tauri non fa cross-compilazione affidabile. Il pacchetto per un sistema operativo va
> costruito **su quel sistema operativo** (oppure in CI con un runner per ciascun OS, vedi [Build in CI](#build-in-ci-github-actions)).

Stato di verifica: il build **Linux (x86_64, Ubuntu)** è stato eseguito e verificato su questo progetto.
Le sezioni Windows e macOS seguono la documentazione ufficiale di Tauri 2 ma **non sono state provate** su queste macchine.

## 1. Prerequisiti comuni (tutti gli OS)

- Node.js >= 20 e npm
- Rust stabile (`rustup`), edition 2024 → Rust >= 1.85
- Dipendenze del progetto: `npm install`

Il build produce due cose in sequenza: `vite build` (frontend in `dist/`) e la compilazione Rust in profilo `release`
(LTO attivo, quindi la prima compilazione richiede alcuni minuti).

Comando unico, uguale su ogni OS:

```bash
npm run tauri build
```

I pacchetti finiscono in `src-tauri/target/release/bundle/<formato>/`.
Per costruire un solo formato: `npm run tauri build -- --bundles <deb|rpm|appimage|nsis|msi|app|dmg>`.

Prima di distribuire esegui la regressione: `npm run regression` (deve essere tutta verde).

## 2. Linux

### Prerequisiti di sistema

Debian / Ubuntu:

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Altre distribuzioni (Fedora, Arch, ...): vedi <https://tauri.app/start/prerequisites/#linux>.

### Build

```bash
npm install
npm run tauri build
```

Artefatti prodotti (nomi reali del build verificato, versione 1.0.0):

| Formato | File | Dimensione |
|---|---|---|
| Debian/Ubuntu | `bundle/deb/forziere_1.0.0_amd64.deb` | ~1,8 MB |
| Fedora/RHEL/openSUSE | `bundle/rpm/forziere-1.0.0-1.x86_64.rpm` | ~1,8 MB |
| Universale | `bundle/appimage/forziere_1.0.0_amd64.AppImage` | ~84 MB |

Il binario "nudo" è anche in `src-tauri/target/release/forziere`.
Il bundle AppImage scarica `linuxdeploy` da GitHub al primo build: serve connessione a internet.

### Installazione

**.deb (Debian, Ubuntu, Mint)** — le dipendenze (`libwebkit2gtk-4.1-0`, `libgtk-3-0`) sono installate automaticamente:

```bash
sudo apt install ./src-tauri/target/release/bundle/deb/forziere_1.0.0_amd64.deb
```

**.rpm (Fedora, RHEL, openSUSE)**:

```bash
sudo dnf install ./src-tauri/target/release/bundle/rpm/forziere-1.0.0-1.x86_64.rpm
```

**AppImage (qualsiasi distribuzione, nessuna installazione)**:

```bash
chmod +x forziere_1.0.0_amd64.AppImage
./forziere_1.0.0_amd64.AppImage
```

Dopo l'installazione da .deb/.rpm l'app compare nel menu applicazioni; da terminale: `forziere`.

### Rimozione

```bash
sudo apt remove forziere     # .deb
sudo dnf remove forziere     # .rpm
rm forziere_1.0.0_amd64.AppImage   # AppImage: basta cancellare il file
```

I dati dell'utente **non vengono toccati** dalla disinstallazione (vedi [Dati dell'utente](#5-dati-dellutente)).

### Se l'app non si avvia da un terminale VS Code (snap)

Vedi `./scripts/dev-clean.sh` e la nota nel README: vale per `tauri dev`; per l'app installata basta
avviarla dal menu applicazioni o da un terminale di sistema.

## 3. Windows

### Prerequisiti di sistema

- **Microsoft C++ Build Tools** (workload "Sviluppo di applicazioni desktop con C++") da Visual Studio Installer.
- **WebView2**: già presente su Windows 10 (aggiornato) e 11; in caso contrario l'installer lo scarica.
- Rust con toolchain MSVC: `rustup default stable-msvc`.
- Node.js >= 20.

Per generare il pacchetto **MSI** serve anche la funzione "VBSCRIPT" di Windows (Impostazioni → App → Funzionalità facoltative); l'installer **NSIS** non la richiede.

### Build (PowerShell)

```powershell
npm install
npm run tauri build
```

Artefatti attesi (nomi secondo la convenzione di Tauri 2):

| Formato | File |
|---|---|
| Installer NSIS (consigliato) | `bundle\nsis\forziere_1.0.0_x64-setup.exe` |
| Installer MSI | `bundle\msi\forziere_1.0.0_x64_en-US.msi` |

### Installazione

Doppio clic su `...-setup.exe` (o sull'`.msi`) e segui la procedura guidata.
Da riga di comando, per l'MSI: `msiexec /i forziere_1.0.0_x64_en-US.msi`.

Gli installer **non sono firmati**: Windows SmartScreen mostrerà "Windows ha protetto il PC".
Scegli "Ulteriori informazioni" → "Esegui comunque". Per evitare l'avviso serve un certificato di code signing
(vedi <https://tauri.app/distribute/sign/windows/>).

### Rimozione

Impostazioni → App → App installate → *forziere* → Disinstalla.
Oppure, per l'MSI: `msiexec /x forziere_1.0.0_x64_en-US.msi`.

I dati utente restano in `%APPDATA%\com.giorgio-marini.forziere\` (vedi sezione 5).

## 4. macOS

### Prerequisiti di sistema

- Xcode Command Line Tools: `xcode-select --install`
- Rust: `rustup` (toolchain predefinita per l'architettura del Mac)
- Node.js >= 20

### Build

```bash
npm install
npm run tauri build
```

Produce l'architettura della macchina su cui compili (Apple Silicon → `aarch64`, Intel → `x64`).
Per un binario universale: `rustup target add aarch64-apple-darwin x86_64-apple-darwin` e poi
`npm run tauri build -- --target universal-apple-darwin`.

Artefatti attesi:

| Formato | File |
|---|---|
| Applicazione | `bundle/macos/forziere.app` |
| Immagine disco | `bundle/dmg/forziere_1.0.0_aarch64.dmg` (o `x64`) |

### Installazione

Apri il `.dmg` e trascina `forziere.app` nella cartella **Applicazioni**.

L'app **non è firmata né notarizzata**: al primo avvio Gatekeeper la blocca. Apri con **clic destro → Apri**
(poi "Apri" nel dialogo), oppure da Impostazioni di Sistema → Privacy e sicurezza → "Apri comunque".
Se macOS dice che l'app è "danneggiata" (file scaricato): `xattr -dr com.apple.quarantine /Applications/forziere.app`.
Per distribuirla senza avvisi servono un Apple Developer ID e la notarizzazione
(vedi <https://tauri.app/distribute/sign/macos/>).

### Rimozione

Trascina `forziere.app` dalla cartella Applicazioni al Cestino.
I dati restano in `~/Library/Application Support/com.giorgio-marini.forziere/` (vedi sezione 5).

## 5. Dati dell'utente

Le voci sono nel file `vault.json` dentro la cartella dati dell'app (identificativo `com.giorgio-marini.forziere`):

| OS | Percorso |
|---|---|
| Linux | `~/.local/share/com.giorgio-marini.forziere/vault.json` |
| Windows | `%APPDATA%\com.giorgio-marini.forziere\vault.json` |
| macOS | `~/Library/Application Support/com.giorgio-marini.forziere/vault.json` |

- **Disinstallare l'app NON cancella `vault.json`**: reinstallando ritrovi le tue voci.
- **Rimozione completa** (cancella definitivamente tutte le password salvate): elimina l'intera cartella indicata sopra.
- **Backup / spostamento su un altro computer**: copia `vault.json` nella stessa posizione sulla nuova macchina, con l'app chiusa.
- Il file è **in chiaro, non cifrato**: proteggi la cartella e i backup di conseguenza.
- La cartella contiene anche cache del motore web (`WebKitCache`, `storage`, ...): si possono cancellare senza perdere dati.

## 6. Aggiornare la versione

La versione compare in tre file; tienili allineati prima del build:

- `package.json` → `"version"`
- `src-tauri/Cargo.toml` → `version`
- `src-tauri/tauri.conf.json` → `"version"` (determina il nome dei pacchetti)

Aggiorna anche `CHANGELOG.md` spostando le voci da `[Unreleased]` alla nuova versione.

## 7. Build in CI (GitHub Actions)

Per ottenere i pacchetti di tutti e tre gli OS senza avere le tre macchine, usa l'azione ufficiale
[`tauri-apps/tauri-action`](https://github.com/tauri-apps/tauri-action) con una matrice
`ubuntu-22.04`, `windows-latest`, `macos-latest`. Il progetto oggi **non è un repository git** e non ha workflow CI:
è un passo da fare quando verrà pubblicato.

## 8. Problemi comuni

| Sintomo | Causa / rimedio |
|---|---|
| `Package webkit2gtk-4.1 was not found` | Mancano le dipendenze Linux: installa `libwebkit2gtk-4.1-dev` (sezione 2). |
| Build AppImage fallito offline | Il primo build scarica `linuxdeploy` da GitHub: serve internet. Usa `--bundles deb` per evitarlo. |
| Primo build molto lento | Normale: LTO + `codegen-units = 1`. I successivi sono incrementali. |
| Crash all'avvio con `GLib-GIO-ERROR` o `symbol lookup error` | Variabili ereditate dal terminale snap di VS Code: avvia da un terminale di sistema o con `scripts/dev-clean.sh`. |
| Windows: avviso SmartScreen / macOS: "app danneggiata" | App non firmata, vedi sezioni 3 e 4. |
