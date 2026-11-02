<script setup>
import { computed, ref } from "vue";

const MIN_LEN = 12;

const props = defineProps({
  // "setup" = primo avvio, "migrate" = vault in chiaro da cifrare, "unlock" = sblocco
  mode: { type: String, required: true },
  error: { type: String, default: "" },
  busy: { type: Boolean, default: false },
});
const emit = defineEmits(["submit"]);

const password = ref("");
const confirm = ref("");
const localError = ref("");

const creating = computed(() => props.mode !== "unlock");
const title = computed(
  () => ({ setup: "Crea la master password", migrate: "Proteggi il vault", unlock: "Vault bloccato" })[props.mode],
);

function submit() {
  localError.value = "";
  if (creating.value) {
    if (password.value.length < MIN_LEN) {
      localError.value = `La master password deve avere almeno ${MIN_LEN} caratteri`;
      return;
    }
    if (password.value !== confirm.value) {
      localError.value = "Le due password non coincidono";
      return;
    }
  } else if (!password.value) {
    localError.value = "Inserisci la master password";
    return;
  }
  emit("submit", password.value);
}
</script>

<template>
  <main class="unlock">
    <form @submit.prevent="submit">
      <h1>{{ title }}</h1>
      <p v-if="mode === 'setup'" class="hint">
        Protegge tutte le voci con cifratura AES-256. <strong>Non esiste recupero:</strong> se la dimentichi,
        i dati sono persi.
      </p>
      <p v-if="mode === 'migrate'" class="hint">
        Il vault esistente è in chiaro. Scegli una master password: le voci verranno cifrate e il file in chiaro
        sostituito. <strong>Non esiste recupero</strong> se la dimentichi.
      </p>
      <label>
        Master password
        <input v-model="password" type="password" autocomplete="off" autofocus />
      </label>
      <label v-if="creating">
        Conferma password
        <input v-model="confirm" type="password" autocomplete="off" />
      </label>
      <p v-if="localError || error" class="error">{{ localError || error }}</p>
      <button class="primary" type="submit" :disabled="busy">
        {{ busy ? "Attendere..." : mode === "unlock" ? "Sblocca" : "Cifra e continua" }}
      </button>
    </form>
  </main>
</template>

<style scoped>
.unlock {
  display: grid;
  place-items: center;
  height: 100vh;
}
form {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 360px;
  padding: 20px;
  background: var(--bg-panel);
  border: 1px solid var(--green);
}
h1 {
  margin: 0;
  font-size: 1.3em;
  color: var(--green-bright);
}
label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--muted);
}
.hint {
  margin: 0;
  color: var(--muted);
}
.error {
  margin: 0;
  color: var(--red);
}
</style>
