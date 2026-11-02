#!/bin/bash
# Avvia `tauri dev` ripulendo le variabili ereditate dal terminale dello snap di VS Code,
# che altrimenti fanno crashare GTK/WebKit all'avvio (GLib-GIO-ERROR / symbol lookup error).
unset GTK_PATH GTK_EXE_PREFIX GTK_IM_MODULE_FILE GIO_MODULE_DIR LOCPATH GSETTINGS_SCHEMA_DIR GDK_BACKEND LD_LIBRARY_PATH
[ -n "$XDG_DATA_DIRS_VSCODE_SNAP_ORIG" ] && export XDG_DATA_DIRS="$XDG_DATA_DIRS_VSCODE_SNAP_ORIG"
[ -n "$XDG_CONFIG_DIRS_VSCODE_SNAP_ORIG" ] && export XDG_CONFIG_DIRS="$XDG_CONFIG_DIRS_VSCODE_SNAP_ORIG"
cd "$(dirname "$0")/.." && exec npm run tauri dev
