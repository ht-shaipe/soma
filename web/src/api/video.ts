import api, { extractData } from './index'
import type { VideoParams, TaskInfo } from '@/types'

export function createVideo(params: VideoParams) {
  return api.post('/videos/create', params).then(extractData<TaskInfo>)
}

export function listTasks(page?: number, pageSize?: number) {
  return api.post('/tasks/list', { page, pageSize }).then(extractData<{ list: TaskInfo[]; total: number }>)
}

export function getTask(taskId: string) {
  return api.post('/tasks/get', { taskId }).then(extractData<TaskInfo>)
}

export function deleteTask(taskId: string) {
  return api.post('/tasks/delete', { taskId }).then(extractData<boolean>)
}
