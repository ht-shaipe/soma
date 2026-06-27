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
import { isTaskCompleted, isTaskFailed, getTaskStepLabel } from '@/types'
import { useI18n } from 'vue-i18n'

const taskStore = useTaskStore()
const { t } = useI18n()
const task = computed(() => taskStore.currentTask)

const activeStep = computed(() => {
  if (!task.value) return -1
  const stepLabel = getTaskStepLabel(task.value)
  const steps = ['Script', 'Terms', 'Audio', 'Subtitle', 'Materials', 'Video']
  if (stepLabel === 'Completed') return 6
  if (stepLabel === 'Failed') {
    const idx = steps.indexOf(taskStore.pipelineSteps.find(s => s.active)?.key || '')
    return Math.max(0, idx - 1)
  }
  const idx = steps.indexOf(stepLabel)
  return Math.max(0, idx)
})

const statusTagType = computed(() => {
  if (!task.value) return 'info'
  if (isTaskCompleted(task.value)) return 'success'
  if (isTaskFailed(task.value)) return 'danger'
  return 'warning'
})

const statusLabel = computed(() => {
  if (!task.value) return ''
  const stepLabel = getTaskStepLabel(task.value)
  const key = `task.status${stepLabel}`
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
