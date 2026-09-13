// 素材管理 API：素材目录列表、素材/音频/人像上传
import api, { extractData } from './index'
import type { MaterialInfo } from '@/types'

export function listMaterials(directory?: string) {
  return api.post('/materials/list', { directory }).then(extractData<{ list: MaterialInfo[]; total: number }>)
}

export function uploadMaterial(file: File) {
  const formData = new FormData()
  formData.append('file', file)
  return api.post('/materials/upload', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  }).then(extractData<MaterialInfo>)
}

export function uploadPortrait(file: File) {
  const formData = new FormData()
  formData.append('file', file)
  return api.post('/portraits/upload', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  }).then(extractData<{ name: string; path: string }>)
}
