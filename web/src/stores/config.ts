import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { AppConfig } from '@/types'

export const useConfigStore = defineStore('config', () => {
  const config = ref<AppConfig>({
    app: {
      name: 'MoneyPrinterTurbo',
      version: '1.3.0',
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
    ffmpeg: {
      path: 'ffmpeg',
      threads: 2,
    },
    whisper: {
      provider: 'local',
      model: 'base',
    },
  })

  const uiLanguage = ref('zh-CN')
  const hideConfig = ref(false)
  const hideLog = ref(false)

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

  return {
    config,
    uiLanguage,
    hideConfig,
    hideLog,
    updateLlmConfig,
    updateTtsConfig,
    updateStockConfig,
    updateFfmpegConfig,
    updateAppConfig,
  }
})
