<template>
  <div v-if="logs.length > 0" class="log-section">
    <div class="section-title">{{ $t('common.log') }}</div>
    <div class="log-container" ref="logContainer">
      <div v-for="(log, i) in logs" :key="i">{{ log }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useTaskStore } from '@/stores/task'

const taskStore = useTaskStore()
const logs = computed(() => taskStore.logs)
const logContainer = ref<HTMLDivElement>()

watch(logs, () => {
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  })
}, { deep: true })
</script>
