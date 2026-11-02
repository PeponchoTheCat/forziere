<script setup>
import { computed, onMounted, ref } from "vue";
import {
  changePassword,
  deleteEntry,
  listEntries,
  lockVault,
  saveEntry,
  setupVault,
  unlockVault,
  vaultStatus,
} from "./api";
import ChangePassword from "./components/ChangePassword.vue";
import EntryForm from "./components/EntryForm.vue";
import EntryList from "./components/EntryList.vue";
import UnlockScreen from "./components/UnlockScreen.vue";

const entries = ref([]);
const selectedId = ref(null);
const error = ref("");
const formKey = ref(0); // forza il reset del form su "Nuova"
// "loading" | "no_vault" | "legacy_plaintext" | "locked" | "unlocked"
const status = ref("loading");
const authError = ref("");
const busy = ref(false);
const showChange = ref(false);

const unlockMode = computed(
  () => ({ no_vault: "setup", legacy_plaintext: "migrate", locked: "unlock" })[status.value],
);

const selected = computed(() => entries.value.find((e) => e.id === selectedId.value) ?? null);

async function load() {
  try {
    entries.value = await listEntries();
  } catch (e) {
    error.value = String(e);
  }
}

function select(id) {
  error.value = "";
  formKey.value++;
  selectedId.value = id;
}

async function save(input) {
  error.value = "";
  try {
    const saved = await saveEntry(input);
    await load();
    selectedId.value = saved.id;
  } catch (e) {
    error.value = String(e);
  }
}

async function remove(id) {
  if (!window.confirm("Eliminare definitivamente questa voce?")) return;
  error.value = "";
  try {
    await deleteEntry(id);
    selectedId.value = null;
    await load();
  } catch (e) {
    error.value = String(e);
  }
}

async function submitPassword(password) {
  authError.value = "";
  busy.value = true;
  try {
    if (status.value === "locked") await unlockVault(password);
    else await setupVault(password);
    status.value = "unlocked";
    await load();
  } catch (e) {
    authError.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function lock() {
  try {
    await lockVault();
  } catch (e) {
    error.value = String(e);
    return;
  }
  // nessuna voce deve restare nello stato Vue o nel form
  entries.value = [];
  selectedId.value = null;
  error.value = "";
  formKey.value++;
  showChange.value = false;
  status.value = "locked";
}

async function submitChange({ current, next }) {
  authError.value = "";
  busy.value = true;
  try {
    await changePassword(current, next);
    showChange.value = false;
  } catch (e) {
    authError.value = String(e);
  } finally {
    busy.value = false;
  }
}

function openChange() {
  authError.value = "";
  showChange.value = true;
}

onMounted(async () => {
  try {
    status.value = await vaultStatus();
  } catch (e) {
    authError.value = String(e);
    status.value = "locked";
    return;
  }
  if (status.value === "unlocked") await load();
});
</script>

<template>
  <UnlockScreen
    v-if="status !== 'unlocked' && status !== 'loading'"
    :mode="unlockMode"
    :error="authError"
    :busy="busy"
    @submit="submitPassword"
  />
  <div v-else-if="status === 'unlocked'" class="shell">
    <header class="bar">
      <strong>Forziere</strong>
      <span class="spacer" />
      <button @click="openChange">Cambia password</button>
      <button @click="lock">Blocca</button>
    </header>
    <main class="app">
      <EntryList
        :entries="entries"
        :selected-id="selectedId"
        @select="select"
        @new="select(null)"
      />
      <EntryForm :key="formKey" :entry="selected" :error="error" @save="save" @delete="remove" />
    </main>
    <ChangePassword
      v-if="showChange"
      :error="authError"
      :busy="busy"
      @submit="submitChange"
      @cancel="showChange = false"
    />
  </div>
</template>

<style scoped>
.shell {
  display: grid;
  grid-template-rows: auto 1fr;
  height: 100vh;
}
.bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 10px;
  border-bottom: 1px solid var(--green);
  color: var(--green-bright);
}
.spacer {
  flex: 1;
}
.app {
  display: grid;
  grid-template-columns: 230px 1fr;
  min-height: 0;
}
</style>
