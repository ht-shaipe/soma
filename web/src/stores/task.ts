import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { TaskInfo, VideoParams, StoryboardScene } from '@/types'
import { isTaskCompleted, isTaskFailed, isTaskDraft, getTaskStepLabel } from '@/types'
import { createVideo, createDraftTask, updateTaskConfig, startDraftTask, listTasks, getTask, deleteTask, fetchMaterials } from '@/api/video'

export const useTaskStore = defineStore('task', () => {
  const tasks = ref<TaskInfo[]>([])
  const currentTask = ref<TaskInfo | null>(null)
  const draftTaskId = ref('')
  const isGenerating = ref(false)
  const logs = ref<string[]>([])
  const pollTimer = ref<ReturnType<typeof setInterval> | null>(null)

  const pipelineSteps = computed(() => {
    if (!currentTask.value) return []
    const stepLabel = getTaskStepLabel(currentTask.value)
    const steps = [
      { key: 'Intent', label: 'Intent' },
      { key: 'Script', label: 'Script' },
      { key: 'Storyboard', label: 'Storyboard' },
      { key: 'Materials', label: 'Materials' },
      { key: 'Audio', label: 'Audio' },
      { key: 'Compose', label: 'Compose' },
    ]
    const order = ['Intent', 'Script', 'Storyboard', 'Materials', 'Audio', 'Compose', 'Completed', 'Failed']
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
      console.error('Failed to fetch tasks')
    }
  }

  async function startGeneration(params: VideoParams) {
    isGenerating.value = true
    logs.value = []
    try {
      const result = await createVideo(params)
      const task = await getTask(result.taskId)
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

  async function initDraftTask(params: VideoParams) {
    if (draftTaskId.value) return draftTaskId.value
    try {
      const result = await createDraftTask(params)
      draftTaskId.value = result.taskId
      return result.taskId
    } catch (e) {
      logs.value.push(`Draft creation error: ${e instanceof Error ? e.message : String(e)}`)
      return ''
    }
  }

  async function saveStepConfig(stepParams: Partial<VideoParams> & {
    script?: string
    terms?: string[]
    storyboard?: StoryboardScene[]
  }) {
    if (!draftTaskId.value) return
    try {
      await updateTaskConfig(draftTaskId.value, stepParams)
    } catch (e) {
      logs.value.push(`Config save error: ${e instanceof Error ? e.message : String(e)}`)
    }
  }

  async function launchFromDraft() {
    if (!draftTaskId.value) return
    isGenerating.value = true
    logs.value = []
    try {
      const result = await startDraftTask(draftTaskId.value)
      const task = await getTask(result.taskId)
      currentTask.value = task
      startPolling()
      return task
    } catch (e) {
      logs.value.push(`Error: ${e instanceof Error ? e.message : String(e)}`)
      isGenerating.value = false
      throw e
    }
  }

  function resetDraft() {
    draftTaskId.value = ''
  }

  function resetAll() {
    stopPolling()
    draftTaskId.value = ''
    currentTask.value = null
    isGenerating.value = false
    logs.value = []
  }

  async function fetchDraftMaterials() {
    if (!draftTaskId.value) return
    try {
      const result = await fetchMaterials(draftTaskId.value)
      return result
    } catch (e) {
      logs.value.push(`Materials fetch error: ${e instanceof Error ? e.message : String(e)}`)
      return undefined
    }
  }

  function resumeDraft(taskId: string) {
    draftTaskId.value = taskId
  }

  return {
    tasks,
    currentTask,
    draftTaskId,
    isGenerating,
    logs,
    pipelineSteps,
    fetchTasks,
    startGeneration,
    stopPolling,
    removeTask,
    addLog,
    initDraftTask,
    saveStepConfig,
    launchFromDraft,
    resetDraft,
    fetchDraftMaterials,
    resumeDraft,
    resetAll,
  }
})
