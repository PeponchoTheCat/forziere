<script setup>
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import PasswordGenerator from "./PasswordGenerator.vue";

const { t } = useI18n();

const props = defineProps({
  entry: { type: Object, default: null }, // null = nuova voce
  error: { type: String, default: "" },
});
const emit = defineEmits(["save", "delete"]);

const empty = () => ({
  id: null,
  service: "",
  url: "",
  username: "",
  password: "",
  notes: "",
  tags: "",
});

const form = ref(empty());
const showPassword = ref(false);
const copied = ref(false);

watch(
  () => props.entry,
  (e) => {
    form.value = e ? { ...e, tags: e.tags.join(", ") } : empty();
    showPassword.value = false;
    copied.value = false;
  },
  { immediate: true },
);

function submit() {
  const { id, service, url, username, password, notes, tags } = form.value;
  emit("save", {
    id,
    service,
    url,
    username,
    password,
    notes,
    tags: tags.split(",").map((t) => t.trim()).filter(Boolean),
  });
}

async function copyPassword() {
  try {
    await navigator.clipboard.writeText(form.value.password);
    copied.value = true;
    setTimeout(() => (copied.value = false), 1500);
  } catch {
    copied.value = false;
  }
}

function useGenerated(pw) {
  form.value.password = pw;
  showPassword.value = true;
}
</script>

<template>
  <form class="form" @submit.prevent="submit">
    <h2 class="full">{{ entry ? t("form.titleEdit") : t("form.titleNew") }}</h2>

    <label>{{ t("form.service") }}<input v-model="form.service" required /></label>
    <label>{{ t("form.url") }}<input v-model="form.url" type="text" placeholder="https://..." /></label>
    <label>{{ t("form.username") }}<input v-model="form.username" required autocomplete="off" /></label>

    <label>
      {{ t("form.password") }}
      <div class="pw">
        <input
          v-model="form.password"
          :type="showPassword ? 'text' : 'password'"
          autocomplete="off"
        />
        <button type="button" @click="showPassword = !showPassword">
          {{ showPassword ? t("form.hide") : t("form.show") }}
        </button>
        <button type="button" :disabled="!form.password" @click="copyPassword">
          {{ copied ? t("form.copied") : t("form.copy") }}
        </button>
      </div>
    </label>

    <PasswordGenerator class="gen-slot" @use="useGenerated" />

    <label>{{ t("form.tags") }}<input v-model="form.tags" :placeholder="t('form.tagsPlaceholder')" /></label>
    <label>{{ t("form.notes") }}<textarea v-model="form.notes" rows="2"></textarea></label>

    <p v-if="error" class="error full">{{ error }}</p>

    <div class="actions full">
      <button type="submit" class="primary">{{ t("form.save") }}</button>
      <button v-if="entry" type="button" class="danger" @click="emit('delete', entry.id)">
        {{ t("form.delete") }}
      </button>
    </div>
  </form>
</template>

<style scoped>
.form {
  display: grid;
  grid-template-columns: 1fr 1fr;
  align-content: start;
  gap: 8px 14px;
  padding: 10px 16px;
  overflow-y: auto;
  min-height: 0;
}
.full {
  grid-column: 1 / -1;
}
.gen-slot {
  grid-column: 2;
  grid-row: 2 / span 4;
  align-self: start;
}
h2 {
  margin: 0;
  color: var(--green);
  text-transform: uppercase;
  letter-spacing: 0.14em;
  text-shadow: 0 0 8px #4fb47766;
  border-bottom: 1px solid var(--green);
  padding-bottom: 4px;
  font-size: 1.2em;
}
h2::before {
  content: "> ";
  color: var(--green-bright);
}
label {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 0.78em;
  color: var(--green);
  text-transform: uppercase;
  letter-spacing: 0.1em;
}
label input,
label textarea {
  font-size: 1.2em;
  text-transform: none;
  letter-spacing: normal;
}
.pw {
  display: flex;
  gap: 6px;
}
.pw input {
  flex: 1;
  min-width: 0;
}
.actions {
  display: flex;
  gap: 8px;
}
</style>
