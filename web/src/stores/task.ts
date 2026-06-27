import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { TaskInfo, VideoParams } from '@/types'
import { isTaskCompleted, isTaskFailed, getTaskStepLabel } from '@/types'
import { createVideo, listTasks, getTask, deleteTask } from '@/api/video'

export const useTaskStore = defineStore('task', () => {
  const tasks = ref<TaskInfo[]>([])
  const currentTask = ref<TaskInfo | null>(null)
  const isGenerating = ref(false)
  const logs = ref<string[]>([])
  const pollTimer = ref<ReturnType<typeof setInterval> | null>(null)

  const pipelineSteps = computed(() => {
    if (!currentTask.value) return []
    const stepLabel = getTaskStepLabel(currentTask.value)
    const steps = [
      { key: 'Script', label: 'Script' },
      { key: 'Terms', label: 'Keywords' },
      { key: 'Audio', label: 'Audio' },
      { key: 'Subtitle', label: 'Subtitle' },
      { key: 'Materials', label: 'Materials' },
      { key: 'Video', label: 'Video' },
    ]
    const order = ['Script', 'Terms', 'Audio', 'Subtitle', 'Materials', 'Video', 'Completed', 'Failed']
    const currentIndex = order.indexOf(stepLabel)
    return steps.map((s, i) => ({
      ...s,
      done: currentIndex > i || stepLabel === 'Completed',
      active: currentIndex === i,
    }))
  })

  async function fetchTasks() {
    try {
      const result = await listTasks()
      tasks.value = result.list
    } catch {
      tasks.value = []
    }
  }

  async function startGeneration(params: VideoParams) {
    isGenerating.value = true
    logs.value = []
    try {
      const task = await createVideo(params)
      currentTask.value = task
      startPolling()
      return task
    } catch (e) {
      logs.value.push(`Error: ${e instanceof Error ? e.message : String(e)}`)
      isGenerating.value = false
      throw e
    }
  }

  function startPolling() {
    stopPolling()
    pollTimer.value = setInterval(async () => {
      if (!currentTask.value) return
      try {
        const task = await getTask(currentTask.value.taskId)
        currentTask.value = task
        const stepLabel = getTaskStepLabel(task)
        logs.value.push(`[${stepLabel}] Progress: ${task.progress}%`)
        if (isTaskCompleted(task) || isTaskFailed(task)) {
          stopPolling()
          isGenerating.value = false
          if (isTaskFailed(task)) {
            logs.value.push(`Failed: ${task.errorMessage || 'Unknown error'}`)
          } else {
            logs.value.push('Video generation completed!')
          }
        }
      } catch {
        stopPolling()
        isGenerating.value = false
      }
    }, 2000)
  }

  function stopPolling() {
    if (pollTimer.value) {
      clearInterval(pollTimer.value)
      pollTimer.value = null
    }
  }

  async function removeTask(taskId: string) {
    await deleteTask(taskId)
    tasks.value = tasks.value.filter((t) => t.taskId !== taskId)
    if (currentTask.value?.taskId === taskId) {
      currentTask.value = null
    }
  }

  function addLog(message: string) {
    logs.value.push(message)
  }

  return {
    tasks,
    currentTask,
    isGenerating,
    logs,
    pipelineSteps,
    fetchTasks,
    startGeneration,
    stopPolling,
    removeTask,
    addLog,
  }
})
