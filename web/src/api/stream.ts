import api, { extractData } from './index'

export function getStreamUrl(taskId: string, index?: number) {
  return api.post('/stream/play', { taskId, index }).then(extractData<{ url: string; path: string }>)
}

export function getDownloadUrl(taskId: string, index?: number) {
  return api.post('/stream/download', { taskId, index }).then(extractData<{ url: string; path: string }>)
}

export function getStaticUrl(relativePath: string) {
  return `/storage/${relativePath}`
}
