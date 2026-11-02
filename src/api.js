import { invoke } from "@tauri-apps/api/core";

export const listEntries = () => invoke("list_entries");
export const saveEntry = (entry) => invoke("save_entry", { entry });
export const deleteEntry = (id) => invoke("delete_entry", { id });
export const generatePassword = (options) => invoke("generate_password", { options });

export const vaultStatus = () => invoke("vault_status");
export const setupVault = (password) => invoke("setup_vault", { password });
export const unlockVault = (password) => invoke("unlock", { password });
export const lockVault = () => invoke("lock");
export const changePassword = (oldPassword, newPassword) =>
  invoke("change_password", { old: oldPassword, new: newPassword });
