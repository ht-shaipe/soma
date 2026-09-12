import api, { extractData } from './index'
import axios from 'axios'

export interface VoiceItem {
  name: string
  label: string
}

export interface ClonedVoiceItem {
  id: string
  name: string
  engine: string
  remoteId: string
  sourceFile: string
  samplePath: string
  createdAt: string
  voiceName: string
}

export function listVoices() {
  return api.post('/voices/list', {}).then(extractData<{ voices: VoiceItem[] }>)
}

export function listClonedVoices() {
  return api.post('/voices/list_cloned', {}).then(extractData<{ voices: ClonedVoiceItem[] }>)
}

export function deleteClonedVoice(id: string) {
  return api.post('/voices/delete_cloned', { id }).then(extractData<{ id: string; deleted: boolean }>)
}

export function cloneVoice(file: File, engine: string, name: string) {
  const formData = new FormData()
  formData.append('file', file)
  formData.append('engine', engine)
  formData.append('name', name)
  return axios
    .post('/api/v1/voices/clone', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
      timeout: 120000,
    })
    .then((response) => {
      const data = response.data
      if (data && typeof data === 'object' && 'code' in data) {
        if (data.code !== 200) {
          return Promise.reject(new Error(data.message || data.msg || 'Clone failed'))
        }
        return data.result as ClonedVoiceItem
      }
      return data as ClonedVoiceItem
    })
}
