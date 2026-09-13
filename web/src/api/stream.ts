// 流媒体 API：任务视频播放/下载地址获取，静态资源 URL 拼装
import api, { extractData } from './index'

export function getStreamUrl(taskId: string, index?: number) {
  return api.post('/stream/play', { taskId, index }).then(extractData<{ url: string; path: string }>)
}

export function getDownloadUrl(taskId: string, index?: number) {
  return api.post('/stream/download', { taskId, index }).then(extractData<{ url: string; path: string }>)
}

export function getStaticUrl(relativePath: string) {
  const storagePrefixes = ['./storage/', 'storage/', './storage', 'storage']
  let path = relativePath.replace(/\\/g, '/')
  for (const prefix of storagePrefixes) {
    if (path.startsWith(prefix)) {
      path = path.slice(prefix.length)
      break
    }
  }
  path = path.replace(/^\//, '')
  return `/storage/${path}`
}
