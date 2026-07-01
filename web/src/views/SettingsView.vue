<template>
  <div class="settings-view">
    <h2 style="margin-bottom: 20px">{{ $t('settings.title') }}</h2>

    <el-row :gutter="16">
      <el-col :xs="24" :sm="24" :md="12" :lg="12">
        <el-card class="panel-card" shadow="hover">
          <template #header>{{ $t('settings.llm.title') }}</template>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.llm.provider') }}</div>
            <el-select v-model="llmProvider" style="width: 100%" @change="onProviderChange">
              <el-option-group v-for="group in providerGroups" :key="group.label" :label="group.label">
                <el-option
                  v-for="p in group.providers"
                  :key="p.value"
                  :label="p.label"
                  :value="p.value"
                />
              </el-option-group>
            </el-select>
          </div>
          <div v-if="currentProviderInfo?.needApiKey" class="form-row">
            <div class="form-label">{{ $t('settings.llm.apiKey') }}</div>
            <el-input v-model="llmApiKey" type="password" show-password />
          </div>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.llm.baseUrl') }}</div>
            <el-input v-model="llmBaseUrl" />
          </div>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.llm.model') }}</div>
            <el-input v-model="llmModel" />
          </div>
          <div v-if="currentProviderInfo?.needSecretKey" class="form-row">
            <div class="form-label">{{ $t('settings.llm.secretKey') }}</div>
            <el-input v-model="llmSecretKey" type="password" show-password />
          </div>
          <div v-if="currentProviderInfo?.needAccountId" class="form-row">
            <div class="form-label">{{ $t('settings.llm.accountId') }}</div>
            <el-input v-model="llmAccountId" />
          </div>
          <el-alert
            v-if="currentProviderInfo?.tip"
            :title="currentProviderInfo.tip"
            type="info"
            :closable="false"
            style="margin-top: 8px"
          />
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="24" :md="12" :lg="12">
        <el-card class="panel-card" shadow="hover">
          <template #header>{{ $t('settings.tts.title') }}</template>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.tts.defaultServer') }}</div>
            <el-select v-model="ttsProvider" style="width: 100%">
              <el-option label="Edge TTS (Free)" value="edge-tts" />
              <el-option label="Azure TTS V2" value="azure-v2" />
              <el-option label="SiliconFlow TTS" value="siliconflow" />
              <el-option label="Google Gemini TTS" value="gemini" />
              <el-option label="Xiaomi MiMo TTS" value="mimo" />
              <el-option label="ElevenLabs TTS" value="elevenlabs" />
            </el-select>
          </div>
          <div v-if="ttsProvider === 'azure-v2'" class="form-row">
            <div class="form-label">{{ $t('settings.tts.azureKey') }}</div>
            <el-input v-model="azureSpeechKey" type="password" show-password />
          </div>
          <div v-if="ttsProvider === 'azure-v2'" class="form-row">
            <div class="form-label">{{ $t('settings.tts.azureRegion') }}</div>
            <el-input v-model="azureSpeechRegion" />
          </div>
          <div v-if="ttsProvider === 'siliconflow'" class="form-row">
            <div class="form-label">{{ $t('settings.tts.siliconflowKey') }}</div>
            <el-input v-model="siliconflowKey" type="password" show-password />
          </div>
          <div v-if="ttsProvider === 'elevenlabs'" class="form-row">
            <div class="form-label">{{ $t('settings.tts.elevenlabsKey') }}</div>
            <el-input v-model="elevenlabsKey" type="password" show-password />
          </div>
          <div v-if="ttsProvider === 'elevenlabs'" class="form-row">
            <div class="form-label">{{ $t('settings.tts.elevenlabsModel') }}</div>
            <el-select v-model="elevenlabsModel" style="width: 100%">
              <el-option label="eleven_multilingual_v2" value="eleven_multilingual_v2" />
              <el-option label="eleven_flash_v2_5" value="eleven_flash_v2_5" />
              <el-option label="eleven_v3" value="eleven_v3" />
            </el-select>
          </div>
          <div v-if="ttsProvider === 'mimo'" class="form-row">
            <div class="form-label">{{ $t('settings.tts.mimoKey') }}</div>
            <el-input v-model="mimoKey" type="password" show-password />
          </div>
        </el-card>

        <el-card class="panel-card" shadow="hover">
          <template #header>{{ $t('settings.stock.title') }}</template>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.stock.pexelsKey') }}</div>
            <el-input v-model="pexelsApiKey" type="password" show-password />
          </div>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.stock.pixabayKey') }}</div>
            <el-input v-model="pixabayApiKey" type="password" show-password />
          </div>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.stock.coverrKey') }}</div>
            <el-input v-model="coverrApiKey" type="password" show-password />
          </div>
        </el-card>

        <el-card class="panel-card" shadow="hover">
          <template #header>{{ $t('settings.system.title') }}</template>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.system.ffmpegPath') }}</div>
            <el-input v-model="ffmpegPath" />
          </div>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.system.threads') }}</div>
            <el-input-number v-model="ffmpegThreads" :min="1" :max="32" style="width: 100%" />
          </div>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.system.storagePath') }}</div>
            <el-input v-model="storagePath" />
          </div>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.system.concurrentTasks') }}</div>
            <el-input-number v-model="concurrentTasks" :min="1" :max="10" style="width: 100%" />
          </div>
        </el-card>
      </el-col>
    </el-row>

    <div style="margin-top: 16px; text-align: center">
      <el-button type="primary" size="large" @click="onSave" style="width: 300px">
        {{ $t('settings.save') }}
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useConfigStore } from '@/stores/config'
import { useI18n } from 'vue-i18n'
import type { LlmProviderOption } from '@/types'

const configStore = useConfigStore()
const { t } = useI18n()

const providerGroups = ref([
  {
    label: 'OpenAI Compatible',
    providers: [
      { label: 'OpenAI', value: 'openai', defaultBaseUrl: 'https://api.openai.com/v1', defaultModel: 'gpt-4o-mini', needApiKey: true },
      { label: 'AIHubMix (Recommended)', value: 'aihubmix', defaultBaseUrl: 'https://api.aihubmix.com/v1', defaultModel: 'gpt-4o-mini', needApiKey: true },
      { label: 'DeepSeek', value: 'deepseek', defaultBaseUrl: 'https://api.deepseek.com/v1', defaultModel: 'deepseek-chat', needApiKey: true },
      { label: 'Moonshot', value: 'moonshot', defaultBaseUrl: 'https://api.moonshot.cn/v1', defaultModel: 'moonshot-v1-8k', needApiKey: true },
      { label: 'Qwen (DashScope)', value: 'qwen', defaultBaseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1', defaultModel: 'qwen-turbo', needApiKey: true },
      { label: 'MiniMax', value: 'minimax', defaultBaseUrl: 'https://api.minimax.chat/v1', defaultModel: 'abab6.5s-chat', needApiKey: true },
      { label: 'Groq', value: 'groq', defaultBaseUrl: 'https://api.groq.com/openai/v1', defaultModel: 'llama3-8b-8192', needApiKey: true },
      { label: 'Ollama', value: 'ollama', defaultBaseUrl: 'http://localhost:11434/v1', defaultModel: 'llama3', needApiKey: false },
      { label: 'OneAPI', value: 'oneapi', defaultBaseUrl: 'http://localhost:3000/v1', defaultModel: 'gpt-4o-mini', needApiKey: true },
      { label: 'ModelScope', value: 'modelscope', defaultBaseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1', defaultModel: 'qwen-turbo', needApiKey: true },
      { label: 'EvoLink', value: 'evolink', defaultBaseUrl: 'https://api.evolink.com/v1', defaultModel: 'gpt-4o-mini', needApiKey: true },
      { label: 'AIML API', value: 'aiml', defaultBaseUrl: 'https://api.aimlapi.com/v1', defaultModel: 'gpt-4o-mini', needApiKey: true },
    ] as LlmProviderOption[],
  },
  {
    label: 'Special Providers',
    providers: [
      { label: 'Azure OpenAI', value: 'azure', defaultBaseUrl: '', defaultModel: 'gpt-4o-mini', needApiKey: true, tip: 'Format: https://{resource}.openai.azure.com' },
      { label: 'Google Gemini', value: 'gemini', defaultBaseUrl: 'https://generativelanguage.googleapis.com/v1beta', defaultModel: 'gemini-pro', needApiKey: true },
      { label: 'Cloudflare Workers AI', value: 'cloudflare', defaultBaseUrl: 'https://api.cloudflare.com/client/v4/accounts', defaultModel: '@cf/meta/llama-3-8b-instruct', needApiKey: true, needAccountId: true },
      { label: 'Grok (xAI)', value: 'grok', defaultBaseUrl: 'https://api.x.ai/v1', defaultModel: 'grok-beta', needApiKey: true },
      { label: 'Pollinations AI', value: 'pollinations', defaultBaseUrl: 'https://text.pollinations.ai', defaultModel: 'openai', needApiKey: false },
      { label: 'LiteLLM', value: 'litellm', defaultBaseUrl: 'http://localhost:4000', defaultModel: 'gpt-4o-mini', needApiKey: true },
      { label: 'G4F', value: 'g4f', defaultBaseUrl: '', defaultModel: 'gpt-4o-mini', needApiKey: false },
      { label: 'MiMo (Xiaomi)', value: 'mimo', defaultBaseUrl: 'https://api.mimo.com/v1', defaultModel: 'mimo-chat', needApiKey: true },
      { label: 'Doubao (Volcengine)', value: 'doubao', defaultBaseUrl: 'https://ark.cn-beijing.volces.com/api/v3', defaultModel: 'doubao-pro-32k', needApiKey: true },
      { label: 'Hunyuan (Tencent)', value: 'hunyuan', defaultBaseUrl: 'https://hunyuan.tencentcloudapi.com', defaultModel: 'hunyuan-turbo', needApiKey: true },
      { label: 'Zhipu (BigModel)', value: 'zhipu', defaultBaseUrl: 'https://open.bigmodel.cn/api/paas/v4', defaultModel: 'glm-4-flash', needApiKey: true },
      { label: 'Wenxin (Baidu)', value: 'wenxin', defaultBaseUrl: 'https://aip.baidubce.com', defaultModel: 'ernie-4.0-8k', needApiKey: true, needSecretKey: true },
      { label: 'Xunfei (iFlytek)', value: 'xunfei', defaultBaseUrl: 'https://spark-api.xf-yun.com/v1', defaultModel: 'generalv3.5', needApiKey: true },
    ] as LlmProviderOption[],
  },
])

const llmProvider = ref(configStore.config.llm.provider)
const llmApiKey = ref(configStore.config.llm.api_key)
const llmBaseUrl = ref(configStore.config.llm.base_url)
const llmModel = ref(configStore.config.llm.model)
const llmSecretKey = ref(configStore.config.llm.secret_key || '')
const llmAccountId = ref(configStore.config.llm.account_id || '')

const ttsProvider = ref(configStore.config.tts.provider)
const azureSpeechKey = ref(configStore.config.tts.azure_speech_key || '')
const azureSpeechRegion = ref(configStore.config.tts.azure_speech_region || '')
const siliconflowKey = ref(configStore.config.tts.siliconflow_key || '')
const elevenlabsKey = ref(configStore.config.tts.elevenlabs_key || '')
const elevenlabsModel = ref(configStore.config.tts.elevenlabs_model || 'eleven_multilingual_v2')
const mimoKey = ref(configStore.config.tts.mimo_key || '')

const pexelsApiKey = ref(configStore.config.stock.pexels_api_key)
const pixabayApiKey = ref(configStore.config.stock.pixabay_api_key)
const coverrApiKey = ref(configStore.config.stock.coverr_api_key)

const ffmpegPath = ref(configStore.config.ffmpeg.path)
const ffmpegThreads = ref(configStore.config.ffmpeg.threads)
const storagePath = ref(configStore.config.app.storage_path)
const concurrentTasks = ref(configStore.config.app.concurrent_tasks)

watch(() => configStore.config, (cfg) => {
  llmProvider.value = cfg.llm.provider
  llmApiKey.value = cfg.llm.api_key
  llmBaseUrl.value = cfg.llm.base_url
  llmModel.value = cfg.llm.model
  llmSecretKey.value = cfg.llm.secret_key || ''
  llmAccountId.value = cfg.llm.account_id || ''
  ttsProvider.value = cfg.tts.provider
  azureSpeechKey.value = cfg.tts.azure_speech_key || ''
  azureSpeechRegion.value = cfg.tts.azure_speech_region || ''
  siliconflowKey.value = cfg.tts.siliconflow_key || ''
  elevenlabsKey.value = cfg.tts.elevenlabs_key || ''
  elevenlabsModel.value = cfg.tts.elevenlabs_model || 'eleven_multilingual_v2'
  mimoKey.value = cfg.tts.mimo_key || ''
  pexelsApiKey.value = cfg.stock.pexels_api_key
  pixabayApiKey.value = cfg.stock.pixabay_api_key
  coverrApiKey.value = cfg.stock.coverr_api_key
  ffmpegPath.value = cfg.ffmpeg.path
  ffmpegThreads.value = cfg.ffmpeg.threads
  storagePath.value = cfg.app.storage_path
  concurrentTasks.value = cfg.app.concurrent_tasks
}, { deep: true })

const currentProviderInfo = computed(() => {
  for (const group of providerGroups.value) {
    const found = group.providers.find(p => p.value === llmProvider.value)
    if (found) return found
  }
  return null
})

function onProviderChange(value: string) {
  const info = currentProviderInfo.value
  if (info) {
    if (info.defaultBaseUrl) llmBaseUrl.value = info.defaultBaseUrl
    if (info.defaultModel) llmModel.value = info.defaultModel
    if (!info.needApiKey) llmApiKey.value = ''
  }
}

async function onSave() {
  configStore.updateLlmConfig({
    provider: llmProvider.value,
    api_key: llmApiKey.value,
    base_url: llmBaseUrl.value,
    model: llmModel.value,
    secret_key: llmSecretKey.value,
    account_id: llmAccountId.value,
  })
  configStore.updateTtsConfig({
    provider: ttsProvider.value,
    azure_speech_key: azureSpeechKey.value,
    azure_speech_region: azureSpeechRegion.value,
    siliconflow_key: siliconflowKey.value,
    elevenlabs_key: elevenlabsKey.value,
    elevenlabs_model: elevenlabsModel.value,
    mimo_key: mimoKey.value,
  })
  configStore.updateStockConfig({
    pexels_api_key: pexelsApiKey.value,
    pixabay_api_key: pixabayApiKey.value,
    coverr_api_key: coverrApiKey.value,
  })
  configStore.updateFfmpegConfig({
    path: ffmpegPath.value,
    threads: ffmpegThreads.value,
  })
  configStore.updateAppConfig({
    storage_path: storagePath.value,
    concurrent_tasks: concurrentTasks.value,
  })
  const ok = await configStore.save()
  if (ok) {
    ElMessage.success(t('settings.saveSuccess'))
  } else {
    ElMessage.error(t('settings.saveFailed'))
  }
}
</script>
