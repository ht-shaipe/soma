// API 传输层入口：axios(HTTP) 与 Tauri invoke 双轨封装，业务模块零改动切换宿主
import axios from 'axios'
import { invoke } from '@tauri-apps/api/core'

/**
 * 统一 API 传输层（0.1.2 M2.2）
 *
 * 双轨透明切换，业务 API 模块（video.ts / llm.ts / ...）零改动：
 * - 浏览器 / vite 开发：axios → HTTP `POST /api/v1/{module}/{method}`（走 vite 代理到 soma-server）
 * - Tauri 桌面：invoke('api') → 桌面宿主内置的业务分发命令（与 HTTP 同构的 {code, result, message} 信封）
 *
 * 桌面检测用 __TAURI_INTERNALS__（v2 运行时无条件注入的 IPC 桥），
 * 不依赖 withGlobalTauri 的 window.__TAURI__ 全局包装。
 */
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

const http = axios.create({
  baseURL: '/api/v1',
  timeout: 120000,
  headers: { 'Content-Type': 'application/json' },
})

http.interceptors.response.use(
  (response) => {
    const data = response.data
    if (data && typeof data === 'object' && 'code' in data) {
      if (data.code !== 200) {
        return Promise.reject(new Error(data.message || data.msg || 'Request failed'))
      }
    }
    return response
  },
  (error) => {
    return Promise.reject(error)
  }
)

/** Tauri 桌面传输层：与 axios 实例对外行为一致（仅实现业务用到的 post） */
const tauriApi = {
  async post(path: string, payload?: unknown) {
    if (payload instanceof FormData) {
      // 桌面端上传：文件转 base64 走 upload_file 命令，落 storage/uploads/
      if (path !== '/materials/upload') {
        return Promise.reject(new Error('桌面版暂不支持该上传类型'))
      }
      const file = payload.get('file')
      if (!(file instanceof File)) {
        return Promise.reject(new Error('上传内容不是文件'))
      }
      const buf = await file.arrayBuffer()
      const bytes = new Uint8Array(buf)
      let binary = ''
      const chunk = 0x8000
      for (let i = 0; i < bytes.length; i += chunk) {
        binary += String.fromCharCode(...bytes.subarray(i, i + chunk))
      }
      const envelope = (await invoke('upload_file', {
        fileName: file.name,
        dataBase64: btoa(binary),
      })) as { code?: number; message?: string; result?: { path: string; url: string } }
      if (!envelope || envelope.code !== 200) {
        return Promise.reject(new Error(envelope?.message || '上传失败'))
      }
      return { data: envelope }
    }
    const clean = path.replace(/^\//, '')
    const sep = clean.indexOf('/')
    const module = sep > 0 ? clean.slice(0, sep) : clean
    const method = sep > 0 ? clean.slice(sep + 1) : ''
    const envelope = (await invoke('api', {
      module,
      method,
      payload: payload ?? {},
    })) as { code?: number; message?: string; msg?: string }

    if (envelope && typeof envelope === 'object' && 'code' in envelope && envelope.code !== 200) {
      return Promise.reject(new Error(envelope.message || envelope.msg || 'Request failed'))
    }
    // 与 axios 响应形状对齐：extractData 从 .data 中解包
    return { data: envelope }
  },
}

/** 统一 API 客户端 */
const api = (isTauri ? tauriApi : http) as typeof http

export function extractData<T>(response: { data: unknown }): T {
  const d = response.data
  if (d && typeof d === 'object' && 'code' in d && 'result' in d) {
    return (d as { code: number; result: T; message: string }).result
  }
  return d as T
}

export default api
