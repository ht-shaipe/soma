<template>
  <el-tag :type="type" size="small" effect="dark">
    {{ label }}
  </el-tag>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { TaskStateCode, isTaskCompleted, isTaskFailed, isTaskProcessing, isTaskDraft, isTaskQueued, isTaskPaused, getTaskStepLabel } from '@/types'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  state: number
  progress?: number
}>()

const { t } = useI18n()

const type = computed(() => {
  const mock = { state: props.state } as any
  if (isTaskCompleted(mock)) return 'success'
  if (isTaskFailed(mock)) return 'danger'
  if (isTaskPaused(mock)) return 'info'
  if (isTaskQueued(mock)) return ''
  if (isTaskDraft(mock)) return 'info'
  return 'warning'
})

const label = computed(() => {
  const mock = { state: props.state, progress: props.progress ?? 0 } as any
  if (props.state === TaskStateCode.Completed) return t('task.statusCompleted')
  if (props.state === TaskStateCode.Failed) return t('task.statusFailed')
  if (props.state === TaskStateCode.Draft) return t('task.statusDraft')
  if (props.state === TaskStateCode.Queued) return t('task.statusQueued')
  if (props.state === TaskStateCode.Paused) return t('task.statusPaused')
  const stepLabel = getTaskStepLabel(mock)
  return t(`task.status${stepLabel}`)
})
</script>
