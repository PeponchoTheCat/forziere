# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Forziere: desktop password manager (Tauri 2 + Vue 3, JavaScript, no TypeScript). Stores entries (service, URL, username, password, notes, tags) and generates secure passwords with configurable length and character classes (upper, lower, digits, symbols). UI/docs are in Italian. Not a git repository yet.

Work progress is tracked in `TODO.md` and `CHANGELOG.md` (Keep a Changelog, `[Unreleased]`): update both when completing a step. The planned design lives in `~/.claude/plans/voglio-scrivere-una-applicazione-mutable-ember.md`; phases 1 (backend) and 2 (Vue UI) are done; unit tests are in place for both; end-to-end verification in `npm run tauri dev` is pending.

## Commands

```bash
npm install                        # frontend deps
npm run tauri dev                  # run the app (starts Vite on fixed port 1420, strictPort)
npm run tauri build                # production bundle (runs `vite build` to ../dist)
npm run dev                        # Vite alone (no Tauri backend; `invoke` calls will fail)
cd src-tauri && cargo test         # Rust tests
cd src-tauri && cargo test <name>  # single Rust test
```

Tests: Vitest + @vue/test-utils (jsdom) for the frontend, `cargo test` for Rust.

```bash
npm test                    # frontend tests (vitest run); single file: npx vitest run src/App.test.js
npm run test:coverage       # frontend coverage (v8)
npm run test:rust           # Rust tests
npm run regression          # FULL regression: frontend tests + Rust tests + vite build
```

No JS linter is configured.

## Testing rules (required)

- Every new feature/function ships with unit tests in the same change; check coverage before finishing.
- Tests follow **AAA** (Arrange / Act / Assert), marked with `// Arrange`, `// Act`, `// Assert` comments, one behaviour per test.
- A task is NOT done until the whole regression (`npm run regression`) is green.
- Frontend tests sit next to the source (`*.test.js`) and mock `@tauri-apps/api/core` / `./api`; Rust tests live in `#[cfg(test)]` modules (commands are tested via `tauri::test::mock_app`).

## Architecture

- **Frontend** `src/` (Vue 3 `<script setup>`, Vite). Talks to Rust only via `invoke` from `@tauri-apps/api/core`; planned to be wrapped in `src/api.js`.
- **Backend** `src-tauri/src/`: `main.rs` just calls `forziere_lib::run()`; all setup (plugins, `invoke_handler`, managed state) goes in `lib.rs`. Every new Tauri command must be registered in `generate_handler![...]`.
- **Persistence (decided design)**: a plain, **unencrypted** `vault.json` in `app_data_dir()`, written atomically (tmp file + rename), held in a `Mutex<Vec<Entry>>` managed state. Encryption was deliberately declined by the user; keep storage isolated in its own module so it can be added later. Passwords must be generated in Rust with a CSPRNG, guaranteeing at least one char per selected class.

## Gotchas

- The app was renamed from `fcs-password-mamanger` to **Forziere** (crate `forziere`, lib `forziere_lib`, identifier `com.giorgio-marini.forziere`). On startup `store::migrate_legacy_vault` copies `vault.json` from the old identifier folder if the new one has none.
- `tauri.conf.json` has `csp: null` and Cargo uses edition 2024.
