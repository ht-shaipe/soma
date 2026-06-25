<template>
  <div v-if="logs.length > 0" class="log-section">
    <div class="section-title">Log</div>
    <div class="log-container" ref="logContainer">
      <div v-for="(log, i) in logs" :key="i">{{ log }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useTaskStore } from '@/stores/task'

const taskStore = useTaskStore()
const logs = computed(() => taskStore.logs)
const logContainer = ref<HTMLDivElement>()

import { computed } from 'vue'

watch(logs, () => {
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  })
}, { deep: true })
</script>
