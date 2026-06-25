import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { TaskInfo, VideoParams } from '@/types'
import { TaskStatus } from '@/types'
import { createVideo, listTasks, getTask, deleteTask } from '@/api/video'

export const useTaskStore = defineStore('task', () => {
  const tasks = ref<TaskInfo[]>([])
  const currentTask = ref<TaskInfo | null>(null)
  const isGenerating = ref(false)
  const logs = ref<string[]>([])
  const pollTimer = ref<ReturnType<typeof setInterval> | null>(null)

  const pipelineSteps = computed(() => {
    if (!currentTask.value) return []
    const status = currentTask.value.status
    const steps = [
      { key: TaskStatus.Script, label: 'Script', done: false },
      { key: TaskStatus.Terms, label: 'Keywords', done: false },
      { key: TaskStatus.Audio, label: 'Audio', done: false },
      { key: TaskStatus.Subtitle, label: 'Subtitle', done: false },
      { key: TaskStatus.Materials, label: 'Materials', done: false },
      { key: TaskStatus.Video, label: 'Video', done: false },
    ]
    const order: string[] = [
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
    const currentIndex = order.indexOf(status)
    return steps.map((s, i) => ({
      ...s,
      done: currentIndex > i + 1 || status === TaskStatus.Completed,
      active: currentIndex === i + 1,
    }))
  })

  async function fetchTasks() {
    try {
      tasks.value = await listTasks()
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
        const task = await getTask(currentTask.value.task_id)
        currentTask.value = task
        logs.value.push(`[${task.status}] Progress: ${task.progress}%`)
        if (task.status === TaskStatus.Completed || task.status === TaskStatus.Failed) {
          stopPolling()
          isGenerating.value = false
          if (task.status === TaskStatus.Failed) {
            logs.value.push(`Failed: ${task.error_message || 'Unknown error'}`)
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
    tasks.value = tasks.value.filter((t) => t.task_id !== taskId)
    if (currentTask.value?.task_id === taskId) {
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
