<script setup>
import { ref } from "vue";

const MIN_LEN = 12;

defineProps({
  error: { type: String, default: "" },
  busy: { type: Boolean, default: false },
});
const emit = defineEmits(["submit", "cancel"]);

const current = ref("");
const next = ref("");
const confirm = ref("");
const localError = ref("");

function submit() {
  localError.value = "";
  if (!current.value) {
    localError.value = "Inserisci la password attuale";
  } else if (next.value.length < MIN_LEN) {
    localError.value = `La nuova password deve avere almeno ${MIN_LEN} caratteri`;
  } else if (next.value !== confirm.value) {
    localError.value = "Le due password non coincidono";
  } else {
    emit("submit", { current: current.value, next: next.value });
  }
}
</script>

<template>
  <div class="backdrop" @click.self="emit('cancel')">
    <form role="dialog" aria-label="Cambia master password" @submit.prevent="submit">
      <h2>Cambia master password</h2>
      <label>
        Password attuale
        <input v-model="current" type="password" autocomplete="off" />
      </label>
      <label>
        Nuova password
        <input v-model="next" type="password" autocomplete="off" />
      </label>
      <label>
        Conferma nuova password
        <input v-model="confirm" type="password" autocomplete="off" />
      </label>
      <p v-if="localError || error" class="error">{{ localError || error }}</p>
      <div class="actions">
        <button type="button" @click="emit('cancel')">Annulla</button>
        <button class="primary" type="submit" :disabled="busy">Cambia</button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  background: #000000b0;
  z-index: 50;
}
form {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 340px;
  padding: 20px;
  background: var(--bg-panel);
  border: 1px solid var(--green);
}
h2 {
  margin: 0;
  font-size: 1.2em;
  color: var(--green-bright);
}
label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--muted);
}
.error {
  margin: 0;
  color: var(--red);
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
