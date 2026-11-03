<script setup>
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import LanguageSwitch from "./LanguageSwitch.vue";

const { t } = useI18n();

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
const title = computed(() =>
  t({ setup: "unlock.titleSetup", migrate: "unlock.titleMigrate", unlock: "unlock.titleUnlock" }[props.mode]),
);

function submit() {
  localError.value = "";
  if (creating.value) {
    if (password.value.length < MIN_LEN) {
      localError.value = t("unlock.tooShort", { min: MIN_LEN });
      return;
    }
    if (password.value !== confirm.value) {
      localError.value = t("unlock.mismatch");
      return;
    }
  } else if (!password.value) {
    localError.value = t("unlock.required");
    return;
  }
  emit("submit", password.value);
}
</script>

<template>
  <main class="unlock">
    <LanguageSwitch class="corner" />
    <form @submit.prevent="submit">
      <h1>{{ title }}</h1>
      <i18n-t v-if="mode === 'setup'" keypath="unlock.hintSetup" tag="p" class="hint">
        <template #noRecovery>
          <strong>{{ t("unlock.noRecovery") }}</strong>
        </template>
      </i18n-t>
      <i18n-t v-if="mode === 'migrate'" keypath="unlock.hintMigrate" tag="p" class="hint">
        <template #noRecovery>
          <strong>{{ t("unlock.noRecovery") }}</strong>
        </template>
      </i18n-t>
      <label>
        {{ t("unlock.masterPassword") }}
        <input v-model="password" type="password" autocomplete="off" autofocus />
      </label>
      <label v-if="creating">
        {{ t("unlock.confirmPassword") }}
        <input v-model="confirm" type="password" autocomplete="off" />
      </label>
      <p v-if="localError || error" class="error">{{ localError || error }}</p>
      <button class="primary" type="submit" :disabled="busy">
        {{ busy ? t("unlock.busy") : mode === "unlock" ? t("unlock.unlock") : t("unlock.encrypt") }}
      </button>
    </form>
  </main>
</template>

<style scoped>
.corner {
  position: absolute;
  top: 8px;
  right: 10px;
}
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
