<template>
  <div class="task-progress-section">
    <el-steps :active="activeStep" finish-status="success" align-center>
      <el-step :title="$t('task.statusScript')" :description="getStepDesc(0)" />
      <el-step :title="$t('task.statusTerms')" :description="getStepDesc(1)" />
      <el-step :title="$t('task.statusAudio')" :description="getStepDesc(2)" />
      <el-step :title="$t('task.statusSubtitle')" :description="getStepDesc(3)" />
      <el-step :title="$t('task.statusMaterials')" :description="getStepDesc(4)" />
      <el-step :title="$t('task.statusVideo')" :description="getStepDesc(5)" />
    </el-steps>
    <div v-if="task" style="text-align: center; margin-top: 8px">
      <el-tag :type="statusTagType" size="large">
        {{ statusLabel }} — {{ task.progress }}%
      </el-tag>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useTaskStore } from '@/stores/task'
import { TaskStatus } from '@/types'
import { useI18n } from 'vue-i18n'

const taskStore = useTaskStore()
const { t } = useI18n()
const task = computed(() => taskStore.currentTask)

const statusOrder: string[] = [
  TaskStatus.Pending,
  TaskStatus.Script,
  TaskStatus.Terms,
  TaskStatus.Audio,
  TaskStatus.Subtitle,
  TaskStatus.Materials,
  TaskStatus.Video,
  TaskStatus.Completed,
  TaskStatus.Failed,
]

const activeStep = computed(() => {
  if (!task.value) return -1
  const idx = statusOrder.indexOf(task.value.status)
  if (task.value.status === TaskStatus.Completed) return 6
  if (task.value.status === TaskStatus.Failed) return idx - 1
  return Math.max(0, idx - 1)
})

const statusTagType = computed(() => {
  if (!task.value) return 'info'
  switch (task.value.status) {
    case TaskStatus.Completed: return 'success'
    case TaskStatus.Failed: return 'danger'
    case TaskStatus.Pending: return 'info'
    default: return 'warning'
  }
})

const statusLabel = computed(() => {
  if (!task.value) return ''
  const key = `task.status${task.value.status}`
  return t(key)
})

function getStepDesc(index: number) {
  if (!task.value) return ''
  const steps = taskStore.pipelineSteps
  if (steps[index]?.done) return '✓'
  if (steps[index]?.active) return `${task.value.progress}%`
  return ''
}
</script>
