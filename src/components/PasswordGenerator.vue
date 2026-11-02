<script setup>
import { reactive, ref } from "vue";
import { generatePassword } from "../api";

const emit = defineEmits(["use"]);

const opts = reactive({
  length: 16,
  uppercase: true,
  lowercase: true,
  digits: true,
  symbols: true,
});
const generated = ref("");
const error = ref("");

async function generate() {
  error.value = "";
  try {
    generated.value = await generatePassword({ ...opts, length: Number(opts.length) });
  } catch (e) {
    generated.value = "";
    error.value = String(e);
  }
}
</script>

<template>
  <fieldset class="gen">
    <legend>Generatore password</legend>
    <label class="len">
      Caratteri: <strong>{{ opts.length }}</strong>
      <input v-model="opts.length" type="range" min="4" max="128" />
    </label>
    <div class="checks">
      <label><input v-model="opts.uppercase" type="checkbox" /> Maiuscole</label>
      <label><input v-model="opts.lowercase" type="checkbox" /> Minuscole</label>
      <label><input v-model="opts.digits" type="checkbox" /> Numeri</label>
      <label><input v-model="opts.symbols" type="checkbox" /> Caratteri speciali</label>
    </div>
    <div class="actions">
      <button type="button" @click="generate">Genera</button>
      <button type="button" class="primary" :disabled="!generated" @click="emit('use', generated)">
        Usa questa password
      </button>
    </div>
    <code v-if="generated" class="out">{{ generated }}</code>
    <p v-if="error" class="error">{{ error }}</p>
  </fieldset>
</template>

<style scoped>
.gen {
  border: 1px solid var(--green-bright);
  background: #8fd6a00d;
  padding: 2px 10px 10px;
  box-shadow: inset 0 0 14px #8fd6a022;
}
legend {
  padding: 0 8px;
  color: var(--green-bright);
  text-transform: uppercase;
  letter-spacing: 0.12em;
  text-shadow: 0 0 8px color-mix(in srgb, var(--green-bright) 40%, transparent);
}
.len {
  display: flex;
  align-items: center;
  gap: 10px;
}
.len strong {
  min-width: 3ch;
  color: var(--lime);
  text-shadow: 0 0 8px color-mix(in srgb, var(--lime) 40%, transparent);
}
.len input {
  flex: 1;
}
.checks {
  display: grid;
  grid-template-columns: auto auto;
  gap: 4px 8px;
  white-space: nowrap;
  margin: 8px 0;
}
.actions {
  display: flex;
  gap: 8px;
}
.out {
  font-size: 0.95em;
  display: block;
  margin-top: 8px;
  padding: 6px;
  color: var(--lime);
  background: #000;
  border: 1px dashed var(--lime);
  text-shadow: 0 0 6px color-mix(in srgb, var(--lime) 40%, transparent);
  word-break: break-all;
  user-select: all;
}
</style>
