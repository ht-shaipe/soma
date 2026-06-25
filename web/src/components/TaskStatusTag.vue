<template>
  <el-tag :type="type" size="small" effect="dark">
    {{ label }}
  </el-tag>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { TaskStatus } from '@/types'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  status: TaskStatus
}>()

const { t } = useI18n()

const type = computed(() => {
  switch (props.status) {
    case TaskStatus.Completed: return 'success'
    case TaskStatus.Failed: return 'danger'
    case TaskStatus.Pending: return 'info'
    default: return 'warning'
  }
})

const label = computed(() => {
  const key = `task.status${props.status}`
  return t(key)
})
</script>
