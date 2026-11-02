import { invoke } from "@tauri-apps/api/core";

export const listEntries = () => invoke("list_entries");
export const saveEntry = (entry) => invoke("save_entry", { entry });
export const deleteEntry = (id) => invoke("delete_entry", { id });
export const generatePassword = (options) => invoke("generate_password", { options });
