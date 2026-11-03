<script setup>
import { useI18n } from "vue-i18n";
import { setLocale } from "../i18n";

const { locale } = useI18n();

// Nomi nella rispettiva lingua: non vengono tradotti
const LANGUAGES = [
  { code: "en", name: "English" },
  { code: "it", name: "Italiano" },
];
</script>

<template>
  <select
    class="lang"
    :value="locale"
    :aria-label="$t('language.label')"
    @change="setLocale($event.target.value)"
  >
    <option v-for="l in LANGUAGES" :key="l.code" :value="l.code">{{ l.name }}</option>
  </select>
</template>
