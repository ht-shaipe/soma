import api, { extractData } from './index'
import type { MusicInfo } from '@/types'

export function listMusics() {
  return api.post('/musics', { method: 'list' }).then(extractData<MusicInfo[]>)
}

export function uploadMusic(file: File) {
  const formData = new FormData()
  formData.append('file', file)
  return api.post('/musics', formData, {
    method: 'post' as never,
    headers: { 'Content-Type': 'multipart/form-data' },
    params: { method: 'upload' },
  }).then(extractData<MusicInfo>)
}
