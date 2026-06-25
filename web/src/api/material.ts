import api, { extractData } from './index'
import type { MaterialInfo } from '@/types'

export function listMaterials() {
  return api.post('/materials', { method: 'list' }).then(extractData<MaterialInfo[]>)
}

export function uploadMaterial(file: File) {
  const formData = new FormData()
  formData.append('file', file)
  return api.post('/materials', formData, {
    method: 'post' as never,
    headers: { 'Content-Type': 'multipart/form-data' },
    params: { method: 'upload' },
  }).then(extractData<MaterialInfo>)
}
