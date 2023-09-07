<script setup lang="ts">
import { ref, watchEffect } from "vue";

const a = ref(0);
const b = ref(0);

const c = ref(0);

type Api = {
  add: (a: number, b: number) => Promise<number>;
};

watchEffect(async () => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const api = (window as any).electron as Api;

  const result = await api.add(a.value, b.value);
  c.value = result;
});
</script>

<template>
  <label>
    <span>A:</span>
    <input v-model="a" type="number" />
  </label>
  <label>
    <span>A:</span>
    <input v-model="b" type="number" />
  </label>
  <div>
    Result:
    <code>
      <pre>{{ JSON.stringify(c, null, 2) }}</pre>
    </code>
  </div>
</template>
