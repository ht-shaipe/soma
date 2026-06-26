import api, { extractData } from './index'
import type { MusicInfo } from '@/types'

export function listMusics() {
  return api.post('/musics/list').then(extractData<{ list: MusicInfo[]; total: number }>)
}

export function uploadMusic(file: File) {
  const formData = new FormData()
  formData.append('file', file)
  return api.post('/musics/upload', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  }).then(extractData<MusicInfo>)
}
