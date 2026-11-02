<script setup>
import { computed, onMounted, ref } from "vue";
import { deleteEntry, listEntries, saveEntry } from "./api";
import EntryList from "./components/EntryList.vue";
import EntryForm from "./components/EntryForm.vue";

const entries = ref([]);
const selectedId = ref(null);
const error = ref("");
const formKey = ref(0); // forza il reset del form su "Nuova"

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

onMounted(load);
</script>

<template>
  <main class="app">
    <EntryList
      :entries="entries"
      :selected-id="selectedId"
      @select="select"
      @new="select(null)"
    />
    <EntryForm :key="formKey" :entry="selected" :error="error" @save="save" @delete="remove" />
  </main>
</template>

<style scoped>
.app {
  display: grid;
  grid-template-columns: 230px 1fr;
  height: 100vh;
}
</style>
