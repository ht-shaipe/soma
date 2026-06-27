<template>
  <el-tag :type="type" size="small" effect="dark">
    {{ label }}
  </el-tag>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { TaskStateCode, isTaskCompleted, isTaskFailed, isTaskProcessing } from '@/types'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  state: number
}>()

const { t } = useI18n()

const type = computed(() => {
  if (isTaskCompleted({ state: props.state } as any)) return 'success'
  if (isTaskFailed({ state: props.state } as any)) return 'danger'
  if (isTaskProcessing({ state: props.state } as any)) return 'warning'
  return 'info'
})

const label = computed(() => {
  if (props.state === TaskStateCode.Completed) return t('task.statusCompleted')
  if (props.state === TaskStateCode.Failed) return t('task.statusFailed')
  return t('task.statusProcessing')
})
</script>
