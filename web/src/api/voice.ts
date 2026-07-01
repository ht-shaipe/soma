import api, { extractData } from './index'

export interface VoiceItem {
  name: string
  label: string
}

export function listVoices() {
  return api.post('/voices/list', {}).then(extractData<{ voices: VoiceItem[] }>)
}
