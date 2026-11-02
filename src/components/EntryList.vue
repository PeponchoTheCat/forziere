<script setup>
import { computed, ref } from "vue";

const props = defineProps({
  entries: { type: Array, required: true },
  selectedId: { type: String, default: null },
});
defineEmits(["select", "new"]);

const query = ref("");

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  const sorted = [...props.entries].sort((a, b) =>
    a.service.localeCompare(b.service, undefined, { sensitivity: "base" }),
  );
  if (!q) return sorted;
  return sorted.filter((e) =>
    [e.service, e.username, ...e.tags].some((v) => v.toLowerCase().includes(q)),
  );
});
</script>

<template>
  <aside class="list">
    <div class="toolbar">
      <input v-model="query" type="search" placeholder="Cerca..." title="Cerca per servizio, utente o tag" />
      <button class="primary" @click="$emit('new')">+ Nuova</button>
    </div>
    <p v-if="!filtered.length" class="empty">Nessuna voce.</p>
    <ul>
      <li
        v-for="e in filtered"
        :key="e.id"
        :class="{ active: e.id === selectedId }"
        @click="$emit('select', e.id)"
      >
        <strong>{{ e.service }}</strong>
        <span>{{ e.username }}</span>
        <small v-if="e.tags.length">{{ e.tags.join(", ") }}</small>
      </li>
    </ul>
  </aside>
</template>

<style scoped>
.list {
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--bg-panel);
  border-right: 1px solid var(--green);
  box-shadow: 4px 0 18px #4fb47733;
}
.toolbar {
  display: flex;
  gap: 6px;
  padding: 8px;
  border-bottom: 1px solid #4fb47744;
}
.toolbar input {
  flex: 1;
  min-width: 0;
}
ul {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
}
li {
  display: flex;
  flex-direction: column;
  padding: 5px 12px;
  cursor: pointer;
  border-left: 3px solid transparent;
  border-bottom: 1px solid #4fb47714;
}
li strong {
  color: var(--green);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
li:hover {
  background: #4fb47714;
  border-left-color: var(--green);
}
li.active {
  background: linear-gradient(90deg, #8fd6a033, transparent);
  border-left-color: var(--green-bright);
}
li.active strong {
  color: var(--lime);
  text-shadow: 0 0 8px color-mix(in srgb, var(--lime) 40%, transparent);
}
li span {
  color: var(--text);
  font-size: 0.85em;
}
li small {
  color: var(--green-bright);
  font-size: 0.8em;
}
li small::before {
  content: "#";
}
.empty {
  padding: 0 12px;
  color: var(--muted);
}
.empty::before {
  content: "> ";
  color: var(--green-bright);
}
</style>
