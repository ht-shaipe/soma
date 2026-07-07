import api, { extractData } from './index'
import type { VideoParams, TaskInfo, StoryboardScene } from '@/types'

export function createVideo(params: VideoParams) {
  return api.post('/videos/create', params).then(extractData<{ taskId: string }>)
}

export function createDraftTask(params: VideoParams) {
  return api.post('/videos/draft', params).then(extractData<{ taskId: string }>)
}

export function updateTaskConfig(taskId: string, params: Partial<VideoParams> & {
  script?: string
  terms?: string[]
  storyboard?: StoryboardScene[]
}) {
  return api.post('/videos/updateConfig', { taskId, ...params }).then(extractData<{ updated: boolean }>)
}

export function startDraftTask(taskId: string, stopAt?: string) {
  return api.post('/videos/start', { taskId, stopAt: stopAt || '' }).then(extractData<{ started: boolean; taskId: string }>)
}

export function listTasks(page?: number, pageSize?: number) {
  return api.post('/tasks/list', { page, pageSize }).then(extractData<{ list: TaskInfo[]; total: number }>)
}

export function getTask(taskId: string) {
  return api.post('/tasks/get', { taskId }).then(extractData<TaskInfo>)
}

export function deleteTask(taskId: string) {
  return api.post('/tasks/delete', { taskId }).then(extractData<{ deleted: boolean }>)
}

export function fetchMaterials(taskId: string) {
  return api.post('/videos/fetchMaterials', { taskId }).then(extractData<{ taskId: string; materials: string[]; total: number }>)
}

export function generateAudio(taskId: string) {
  return api.post('/videos/generateAudio', { taskId }).then(extractData<{ taskId: string; audioFile: string; audioDuration: number }>)
}
