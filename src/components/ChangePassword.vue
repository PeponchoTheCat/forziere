<script setup>
import { ref } from "vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

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
    localError.value = t("change.currentRequired");
  } else if (next.value.length < MIN_LEN) {
    localError.value = t("change.tooShort", { min: MIN_LEN });
  } else if (next.value !== confirm.value) {
    localError.value = t("change.mismatch");
  } else {
    emit("submit", { current: current.value, next: next.value });
  }
}
</script>

<template>
  <div class="backdrop" @click.self="emit('cancel')">
    <form role="dialog" :aria-label="t('change.title')" @submit.prevent="submit">
      <h2>{{ t("change.title") }}</h2>
      <label>
        {{ t("change.current") }}
        <input v-model="current" type="password" autocomplete="off" />
      </label>
      <label>
        {{ t("change.next") }}
        <input v-model="next" type="password" autocomplete="off" />
      </label>
      <label>
        {{ t("change.confirm") }}
        <input v-model="confirm" type="password" autocomplete="off" />
      </label>
      <p v-if="localError || error" class="error">{{ localError || error }}</p>
      <div class="actions">
        <button type="button" @click="emit('cancel')">{{ t("change.cancel") }}</button>
        <button class="primary" type="submit" :disabled="busy">{{ t("change.submit") }}</button>
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
