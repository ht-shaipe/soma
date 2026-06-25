import api, { extractData } from './index'
import type { VideoParams, TaskInfo } from '@/types'

export function createVideo(params: VideoParams) {
  return api.post('/videos', { method: 'create', ...params }).then(extractData<TaskInfo>)
}

export function listTasks() {
  return api.post('/tasks', { method: 'list' }).then(extractData<TaskInfo[]>)
}

export function getTask(taskId: string) {
  return api.post(`/tasks/${taskId}`, { method: 'get' }).then(extractData<TaskInfo>)
}

export function deleteTask(taskId: string) {
  return api.post(`/tasks/${taskId}`, { method: 'delete' }).then(extractData<boolean>)
}
