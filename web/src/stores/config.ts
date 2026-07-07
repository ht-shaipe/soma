import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { AppConfig } from '@/types'
import { getConfig, saveConfig } from '@/api/config'

export const useConfigStore = defineStore('config', () => {
  const config = ref<AppConfig>({
    app: {
      name: 'Soma',
      version: '0.1.0',
      host: '0.0.0.0',
      port: 8080,
      storage_path: './storage',
      concurrent_tasks: 1,
    },
    llm: {
      provider: 'openai',
      model: 'gpt-4o-mini',
      api_key: '',
      base_url: 'https://api.openai.com/v1',
    },
    tts: {
      provider: 'edge-tts',
      voice_name: 'zh-CN-XiaoxiaoNeural-Female',
    },
    stock: {
      pexels_api_key: '',
      pixabay_api_key: '',
      coverr_api_key: '',
    },
    aivideo: {
      zhipu_video_api_key: '',
      zhipu_video_model: 'cogvideox-flash',
      kling_access_key: '',
      kling_secret_key: '',
      kling_video_model: 'kling-v2-master',
      minimax_video_api_key: '',
      minimax_video_model: 'MiniMax-Hailuo-2.3',
      video_gen_timeout: 300,
    },
    ffmpeg: {
      path: 'ffmpeg',
      threads: 2,
    },
    whisper: {
      provider: 'local',
      model: 'base',
    },
  })

  const loaded = ref(false)
  const uiLanguage = ref('zh-CN')
  const hideConfig = ref(false)
  const hideLog = ref(false)

  async function loadConfig() {
    try {
      const data = await getConfig()
      if (data) {
        config.value = { ...config.value, ...data }
        loaded.value = true
      }
    } catch (e) {
      console.error('Failed to load config:', e)
    }
  }

  async function save() {
    try {
      await saveConfig(config.value)
      return true
    } catch (e) {
      console.error('Failed to save config:', e)
      return false
    }
  }

  function updateLlmConfig(payload: Partial<AppConfig['llm']>) {
    Object.assign(config.value.llm, payload)
  }

  function updateTtsConfig(payload: Partial<AppConfig['tts']>) {
    Object.assign(config.value.tts, payload)
  }

  function updateStockConfig(payload: Partial<AppConfig['stock']>) {
    Object.assign(config.value.stock, payload)
  }

  function updateFfmpegConfig(payload: Partial<AppConfig['ffmpeg']>) {
    Object.assign(config.value.ffmpeg, payload)
  }

  function updateAppConfig(payload: Partial<AppConfig['app']>) {
    Object.assign(config.value.app, payload)
  }

  function updateAiVideoConfig(payload: Partial<AppConfig['aivideo']>) {
    if (!config.value.aivideo) {
      config.value.aivideo = {}
    }
    Object.assign(config.value.aivideo, payload)
  }

  return {
    config,
    loaded,
    uiLanguage,
    hideConfig,
    hideLog,
    loadConfig,
    save,
    updateLlmConfig,
    updateTtsConfig,
    updateStockConfig,
    updateFfmpegConfig,
    updateAppConfig,
    updateAiVideoConfig,
  }
})
