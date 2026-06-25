<template>
  <div>
    <div v-if="keys.length === 0" class="no-keys">{{ $t('apiKey.noKeys') }}</div>
    <div v-else class="api-key-list">
      <div v-for="key in keys" :key="key" class="key-item">
        <span class="key-text">{{ maskKey(key) }}</span>
        <el-button type="danger" size="small" text @click="$emit('delete', key)">
          <el-icon><Delete /></el-icon>
        </el-button>
      </div>
    </div>
    <div style="display: flex; gap: 8px; margin-top: 8px">
      <el-input
        v-model="newKey"
        :placeholder="$t('apiKey.keyPlaceholder')"
        size="small"
        @keyup.enter="onAdd"
      />
      <el-button type="primary" size="small" @click="onAdd">
        {{ $t('apiKey.addKey') }}
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Delete } from '@element-plus/icons-vue'

const props = defineProps<{
  keys: string[]
}>()

const emit = defineEmits<{
  add: [key: string]
  delete: [key: string]
}>()

const newKey = ref('')

function onAdd() {
  const key = newKey.value.trim()
  if (key) {
    emit('add', key)
    newKey.value = ''
  }
}

function maskKey(key: string) {
  if (key.length <= 8) return '****'
  return key.slice(0, 4) + '****' + key.slice(-4)
}
</script>
