#!/bin/bash
# Controlla le dipendenze Rust contro il database RustSec (https://rustsec.org).
# Fallisce solo per vulnerabilità note; unmaintained/yanked sono warning. Richiede rete.
set -euo pipefail
if ! command -v cargo-audit >/dev/null 2>&1; then
  echo "cargo-audit non trovato. Installalo con: cargo install cargo-audit --locked" >&2
  exit 1
fi
cd "$(dirname "$0")/.."
exec cargo audit --file src-tauri/Cargo.lock
