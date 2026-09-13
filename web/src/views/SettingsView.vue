<template>
  <div class="settings-view">
    <!-- 左侧锚点导航 -->
    <aside class="set-nav">
      <div
        v-for="sec in sections"
        :key="sec.key"
        class="set-nav-item"
        :class="{ 'is-active': activeSection === sec.key }"
        @click="scrollTo(sec.key)"
      >
        <span class="set-nav-dot" :class="sec.status" />
        <span>{{ $t(sec.label) }}</span>
      </div>
    </aside>

    <!-- 右侧内容分区 -->
    <div class="set-body">
      <!-- 环境状态 -->
      <section class="set-section">
        <div class="sec-head">
          <div class="sec-icon system"><el-icon :size="20"><CircleCheck /></el-icon></div>
          <div class="sec-titles">
            <div class="sec-title">{{ $t('settings.env.title') }}</div>
            <div class="sec-desc">{{ $t('settings.env.desc') }}</div>
          </div>
          <el-button size="small" :loading="envLoading" @click="runPreflight">{{ $t('settings.env.rerun') }}</el-button>
        </div>
        <div v-if="envChecks.length" class="sec-fields env-list">
          <div v-for="c in envChecks" :key="c.name" class="env-row">
            <span class="env-dot" :class="c.ok ? 'ok' : 'bad'" />
            <span class="env-name">{{ c.name }}</span>
            <span class="env-detail">{{ c.detail }}</span>
            <span v-if="!c.ok" class="env-hint">{{ c.hint }}</span>
          </div>
        </div>
      </section>
      <!-- ── 大语言模型 ── -->
      <section id="sec-llm" class="set-section">
        <div class="sec-head">
          <div class="sec-icon llm"><el-icon :size="20"><MagicStick /></el-icon></div>
          <div class="sec-titles">
            <div class="sec-title">{{ $t('settings.llm.title') }}</div>
            <div class="sec-desc">{{ $t('settings.desc.llm') }}</div>
          </div>
          <el-tag :type="sections[0].status === 'ok' ? 'success' : 'warning'" round effect="light">
            {{ sections[0].status === 'ok' ? $t('settings.status.configured') : $t('settings.status.notConfigured') }}
          </el-tag>
        </div>
        <div class="sec-fields">
          <div class="form-row">
            <div class="form-label">{{ $t('settings.llm.provider') }}</div>
            <el-select v-model="llmProvider" style="width: 100%" @change="onProviderChange">
              <el-option-group v-for="group in providerGroups" :key="group.label" :label="group.label">
                <el-option v-for="p in group.providers" :key="p.value" :label="p.label" :value="p.value" />
              </el-option-group>
            </el-select>
          </div>
          <div class="form-grid">
            <div v-if="currentProviderInfo?.needApiKey" class="form-row">
              <div class="form-label">{{ $t('settings.llm.apiKey') }}</div>
              <el-input v-model="llmApiKey" type="password" show-password />
            </div>
            <div class="form-row">
              <div class="form-label">{{ $t('settings.llm.model') }}</div>
              <el-select v-model="llmModel" filterable allow-create default-first-option style="width: 100%">
                <el-option v-for="m in llmModelSuggestions" :key="m" :label="m" :value="m" />
              </el-select>
            </div>
          </div>
          <div class="form-row">
            <div class="form-label">{{ $t('settings.llm.baseUrl') }}</div>
            <el-input v-model="llmBaseUrl" />
          </div>
          <div v-if="currentProviderInfo?.needSecretKey" class="form-row">
            <div class="form-label">{{ $t('settings.llm.secretKey') }}</div>
            <el-input v-model="llmSecretKey" type="password" show-password />
          </div>
          <div v-if="currentProviderInfo?.needAccountId" class="form-row">
            <div class="form-label">{{ $t('settings.llm.accountId') }}</div>
            <el-input v-model="llmAccountId" />
          </div>
          <el-alert v-if="currentProviderInfo?.tip" :title="currentProviderInfo.tip" type="info" :closable="false" />
        </div>
      </section>

      <!-- ── 语音合成 ── -->
      <section id="sec-tts" class="set-section">
        <div class="sec-head">
          <div class="sec-icon tts"><el-icon :size="20"><Microphone /></el-icon></div>
          <div class="sec-titles">
            <div class="sec-title">{{ $t('settings.tts.title') }}</div>
            <div class="sec-desc">{{ $t('settings.desc.tts') }}</div>
          </div>
          <el-tag :type="sections[1].status === 'ok' ? 'success' : 'warning'" round effect="light">
            {{ sections[1].status === 'ok' ? $t('settings.status.configured') : $t('settings.status.notConfigured') }}
          </el-tag>
        </div>
        <div class="sec-fields">
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
          <div class="form-grid">
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
        </div>
      </section>

      <!-- ── 素材服务 ── -->
      <section id="sec-stock" class="set-section">
        <div class="sec-head">
          <div class="sec-icon stock"><el-icon :size="20"><PictureFilled /></el-icon></div>
          <div class="sec-titles">
            <div class="sec-title">{{ $t('settings.stock.title') }}</div>
            <div class="sec-desc">{{ $t('settings.desc.stock') }}</div>
          </div>
          <el-tag :type="sections[2].status === 'ok' ? 'success' : 'warning'" round effect="light">
            {{ sections[2].status === 'ok' ? $t('settings.status.configured') : $t('settings.status.notConfigured') }}
          </el-tag>
        </div>
        <div class="sec-fields">
          <div class="form-grid">
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
          </div>
        </div>
      </section>

      <!-- ── AI 视频生成 ── -->
      <section id="sec-aivideo" class="set-section">
        <div class="sec-head">
          <div class="sec-icon aivideo"><el-icon :size="20"><VideoCamera /></el-icon></div>
          <div class="sec-titles">
            <div class="sec-title">{{ $t('settings.aivideo.title') }}</div>
            <div class="sec-desc">{{ $t('settings.desc.aivideo') }}</div>
          </div>
          <el-tag :type="sections[3].status === 'ok' ? 'success' : 'warning'" round effect="light">
            {{ sections[3].status === 'ok' ? $t('settings.status.configured') : $t('settings.status.notConfigured') }}
          </el-tag>
        </div>
        <div class="sec-fields">
          <el-alert type="info" :closable="false" class="!mb-4">
            <template #title>{{ $t('settings.aivideo.hint') }}</template>
          </el-alert>
          <el-collapse>
            <el-collapse-item :title="$t('settings.aivideo.zhipu')" name="zhipu">
              <div class="form-row">
                <div class="form-label">{{ $t('settings.aivideo.zhipuApiKey') }}</div>
                <el-input v-model="zhipuVideoApiKey" type="password" show-password />
              </div>
              <div class="form-row">
                <div class="form-label">{{ $t('settings.aivideo.zhipuModel') }}</div>
                <el-select v-model="zhipuVideoModel" style="width: 100%">
                  <el-option-group label="CogVideoX">
                    <el-option label="cogvideox-flash (Free)" value="cogvideox-flash" />
                    <el-option label="cogvideox-3 (1元/次, 4K)" value="cogvideox-3" />
                  </el-option-group>
                  <el-option-group label="Vidu2 (720P)">
                    <el-option label="vidu2-image (图生视频)" value="vidu2-image" />
                    <el-option label="vidu2-start-end (首尾帧)" value="vidu2-start-end" />
                    <el-option label="vidu2-reference (参考生视频)" value="vidu2-reference" />
                  </el-option-group>
                  <el-option-group label="ViduQ1 (1080P)">
                    <el-option label="viduq1-text (文生视频)" value="viduq1-text" />
                    <el-option label="viduq1-image (图生视频)" value="viduq1-image" />
                    <el-option label="viduq1-start-end (首尾帧)" value="viduq1-start-end" />
                  </el-option-group>
                </el-select>
              </div>
            </el-collapse-item>
            <el-collapse-item :title="$t('settings.aivideo.kling')" name="kling">
              <div class="form-row">
                <div class="form-label">{{ $t('settings.aivideo.klingAccessKey') }}</div>
                <el-input v-model="klingAccessKey" type="password" show-password />
              </div>
              <div class="form-row">
                <div class="form-label">{{ $t('settings.aivideo.klingSecretKey') }}</div>
                <el-input v-model="klingSecretKey" type="password" show-password />
              </div>
              <div class="form-row">
                <div class="form-label">{{ $t('settings.aivideo.klingModel') }}</div>
                <el-select v-model="klingVideoModel" style="width: 100%">
                  <el-option label="kling-v1" value="kling-v1" />
                  <el-option label="kling-v1-pro" value="kling-v1-pro" />
                  <el-option label="kling-v2-master" value="kling-v2-master" />
                </el-select>
              </div>
            </el-collapse-item>
            <el-collapse-item :title="$t('settings.aivideo.minimax')" name="minimax">
              <div class="form-row">
                <div class="form-label">{{ $t('settings.aivideo.minimaxApiKey') }}</div>
                <el-input v-model="minimaxVideoApiKey" type="password" show-password />
              </div>
              <div class="form-row">
                <div class="form-label">{{ $t('settings.aivideo.minimaxModel') }}</div>
                <el-select v-model="minimaxVideoModel" style="width: 100%">
                  <el-option label="MiniMax-Hailuo-2.3" value="MiniMax-Hailuo-2.3" />
                  <el-option label="T2V-01 (文生视频)" value="T2V-01" />
                  <el-option label="I2V-01 (图生视频)" value="I2V-01" />
                  <el-option label="video-01-live2d (角色动画)" value="video-01-live2d" />
                  <el-option label="S2V-01 (主体参考)" value="S2V-01" />
                </el-select>
              </div>
            </el-collapse-item>
          </el-collapse>
          <div class="form-row" style="margin-top: 12px">
            <div class="form-label">{{ $t('settings.aivideo.timeout') }}</div>
            <el-input-number v-model="videoGenTimeout" :min="60" :max="900" :step="30" style="width: 100%" />
          </div>
        </div>
      </section>

      <!-- ── 系统与存储 ── -->
      <section id="sec-system" class="set-section">
        <div class="sec-head">
          <div class="sec-icon system"><el-icon :size="20"><Setting /></el-icon></div>
          <div class="sec-titles">
            <div class="sec-title">{{ $t('settings.system.title') }}</div>
            <div class="sec-desc">{{ $t('settings.desc.system') }}</div>
          </div>
          <el-tag :type="sections[4].status === 'ok' ? 'success' : 'warning'" round effect="light">
            {{ sections[4].status === 'ok' ? $t('settings.status.configured') : $t('settings.status.notConfigured') }}
          </el-tag>
        </div>
        <div class="sec-fields">
          <div class="form-grid">
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
          </div>
        </div>
      </section>
    </div>

    <!-- 底部保存栏 -->
    <div class="save-bar" :class="{ dirty: isDirty }">
      <span class="save-hint">{{ isDirty ? $t('settings.dirty') : $t('settings.clean') }}</span>
      <el-button type="primary" size="large" :disabled="!isDirty" :loading="saving" @click="onSave">
        {{ $t('settings.save') }}
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { ElMessage } from 'element-plus'
import { MagicStick, Microphone, PictureFilled, VideoCamera, Setting, CircleCheck } from '@element-plus/icons-vue'
import { useConfigStore } from '@/stores/config'
import api, { extractData } from '@/api'
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

/** 各提供商常用模型建议（下拉可搜索、可自填任意模型名） */
const SUGGESTED_MODELS: Record<string, string[]> = {
  openai: ['gpt-4o-mini', 'gpt-4o', 'gpt-4.1-mini', 'o4-mini'],
  deepseek: ['deepseek-chat', 'deepseek-reasoner'],
  qwen: ['qwen-turbo', 'qwen-plus', 'qwen-max'],
  moonshot: ['moonshot-v1-8k', 'moonshot-v1-32k'],
  zhipu: ['glm-4-flash', 'glm-4-plus'],
  doubao: ['doubao-pro-32k', 'doubao-lite-32k'],
  gemini: ['gemini-2.0-flash', 'gemini-1.5-pro'],
  groq: ['llama3-8b-8192', 'llama-3.1-70b-versatile'],
  minimax: ['abab6.5s-chat'],
  ollama: ['llama3', 'qwen2.5:7b'],
}
const llmModelSuggestions = computed(() => {
  const v = llmProvider.value
  return SUGGESTED_MODELS[v] ?? (currentProviderInfo.value?.defaultModel ? [currentProviderInfo.value.defaultModel] : [])
})
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

const zhipuVideoApiKey = ref(configStore.config.aivideo?.zhipu_video_api_key || '')
const zhipuVideoModel = ref(configStore.config.aivideo?.zhipu_video_model || 'cogvideox-flash')
const klingAccessKey = ref(configStore.config.aivideo?.kling_access_key || '')
const klingSecretKey = ref(configStore.config.aivideo?.kling_secret_key || '')
const klingVideoModel = ref(configStore.config.aivideo?.kling_video_model || 'kling-v2-master')
const minimaxVideoApiKey = ref(configStore.config.aivideo?.minimax_video_api_key || '')
const minimaxVideoModel = ref(configStore.config.aivideo?.minimax_video_model || 'MiniMax-Hailuo-2.3')
const videoGenTimeout = ref(configStore.config.aivideo?.video_gen_timeout || 300)

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
  zhipuVideoApiKey.value = cfg.aivideo?.zhipu_video_api_key || ''
  zhipuVideoModel.value = cfg.aivideo?.zhipu_video_model || 'cogvideox-flash'
  klingAccessKey.value = cfg.aivideo?.kling_access_key || ''
  klingSecretKey.value = cfg.aivideo?.kling_secret_key || ''
  klingVideoModel.value = cfg.aivideo?.kling_video_model || 'kling-v2-master'
  minimaxVideoApiKey.value = cfg.aivideo?.minimax_video_api_key || ''
  minimaxVideoModel.value = cfg.aivideo?.minimax_video_model || 'MiniMax-Hailuo-2.3'
  videoGenTimeout.value = cfg.aivideo?.video_gen_timeout || 300
  ffmpegPath.value = cfg.ffmpeg.path
  ffmpegThreads.value = cfg.ffmpeg.threads
  storagePath.value = cfg.app.storage_path
  concurrentTasks.value = cfg.app.concurrent_tasks
}, { deep: true })

// 配置从后端重新加载后，同步基线
watch(() => configStore.config, () => {
  refreshBaseline()
}, { flush: 'post' })

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

const llmOk = computed(() => {
  if (currentProviderInfo.value && !currentProviderInfo.value.needApiKey) return true
  return !!llmApiKey.value.trim()
})
const ttsOk = computed(() => {
  if (ttsProvider.value === 'edge-tts') return true
  if (ttsProvider.value === 'azure-v2') return !!azureSpeechKey.value.trim()
  if (ttsProvider.value === 'siliconflow') return !!siliconflowKey.value.trim()
  if (ttsProvider.value === 'elevenlabs') return !!elevenlabsKey.value.trim()
  if (ttsProvider.value === 'mimo') return !!mimoKey.value.trim()
  return true
})
const stockOk = computed(() => !!pexelsApiKey.value.trim() || !!pixabayApiKey.value.trim() || !!coverrApiKey.value.trim())
const aivideoOk = computed(() => !!zhipuVideoApiKey.value.trim() || !!klingAccessKey.value.trim() || !!minimaxVideoApiKey.value.trim())
const systemOk = computed(() => !!ffmpegPath.value.trim())

// 分区状态（响应式，供徽标与导航使用）
const sections = computed(() => [
  { key: 'llm', label: 'settings.llm.title', desc: 'settings.desc.llm', status: llmOk.value ? 'ok' : 'todo' },
  { key: 'tts', label: 'settings.tts.title', desc: 'settings.desc.tts', status: ttsOk.value ? 'ok' : 'todo' },
  { key: 'stock', label: 'settings.stock.title', desc: 'settings.desc.stock', status: stockOk.value ? 'ok' : 'todo' },
  { key: 'aivideo', label: 'settings.aivideo.title', desc: 'settings.desc.aivideo', status: aivideoOk.value ? 'ok' : 'todo' },
  { key: 'system', label: 'settings.system.title', desc: 'settings.desc.system', status: systemOk.value ? 'ok' : 'todo' },
])

// ── 未保存更改检测（以最近一次保存/加载的表单快照为基线）──
const savedBaseline = ref('')
function refreshBaseline() {
  savedBaseline.value = formSnapshot()
}
function formSnapshot(): string {
  return JSON.stringify([
    {
      provider: llmProvider.value, api_key: llmApiKey.value, base_url: llmBaseUrl.value,
      model: llmModel.value, secret_key: llmSecretKey.value, account_id: llmAccountId.value,
    },
    {
      provider: ttsProvider.value, azure_speech_key: azureSpeechKey.value,
      azure_speech_region: azureSpeechRegion.value, siliconflow_key: siliconflowKey.value,
      elevenlabs_key: elevenlabsKey.value, elevenlabs_model: elevenlabsModel.value, mimo_key: mimoKey.value,
    },
    { pexels_api_key: pexelsApiKey.value, pixabay_api_key: pixabayApiKey.value, coverr_api_key: coverrApiKey.value },
    {
      zhipu_video_api_key: zhipuVideoApiKey.value, zhipu_video_model: zhipuVideoModel.value,
      kling_access_key: klingAccessKey.value, kling_secret_key: klingSecretKey.value,
      kling_video_model: klingVideoModel.value, minimax_video_api_key: minimaxVideoApiKey.value,
      minimax_video_model: minimaxVideoModel.value, video_gen_timeout: videoGenTimeout.value,
    },
    { path: ffmpegPath.value, threads: ffmpegThreads.value },
    { storage_path: storagePath.value, concurrent_tasks: concurrentTasks.value },
  ])
}
const isDirty = computed(() => formSnapshot() !== savedBaseline.value)

// ── 环境检测（M2.5）──
const envChecks = ref<Array<{ name: string; ok: boolean; detail: string; hint: string }>>([])
const envLoading = ref(false)

async function runPreflight() {
  envLoading.value = true
  try {
    const res = await api.post('/system/preflight', {}).then(extractData<{ checks: any[] }>)
    envChecks.value = (res?.checks as any[]) || []
  } catch (e) {
    ElMessage.error((e as Error).message || '环境检测失败')
  } finally {
    envLoading.value = false
  }
}

const saving = ref(false)
async function onSave() {
  saving.value = true
  try {
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
    configStore.updateAiVideoConfig({
      zhipu_video_api_key: zhipuVideoApiKey.value,
      zhipu_video_model: zhipuVideoModel.value,
      kling_access_key: klingAccessKey.value,
      kling_secret_key: klingSecretKey.value,
      kling_video_model: klingVideoModel.value,
      minimax_video_api_key: minimaxVideoApiKey.value,
      minimax_video_model: minimaxVideoModel.value,
      video_gen_timeout: videoGenTimeout.value,
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
      refreshBaseline()
      ElMessage.success(t('settings.saveSuccess'))
    } else {
      ElMessage.error(t('settings.saveFailed'))
    }
  } finally {
    saving.value = false
  }
}

// ── 锚点导航 + 滚动高亮 ──
const activeSection = ref('llm')
let observer: IntersectionObserver | null = null

function scrollTo(key: string) {
  document.getElementById(`sec-${key}`)?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

onMounted(() => {
  refreshBaseline()
  runPreflight()
  observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          activeSection.value = entry.target.id.replace('sec-', '')
        }
      }
    },
    { rootMargin: '-20% 0px -70% 0px' },
  )
  for (const sec of sections.value) {
    const el = document.getElementById(`sec-${sec.key}`)
    if (el) observer.observe(el)
  }
})

onBeforeUnmount(() => observer?.disconnect())
</script>

<style scoped>
.settings-view {
  max-width: 1080px;
  margin: 0 auto;
  display: flex;
  gap: 24px;
  align-items: flex-start;
  padding-bottom: 90px;
}

/* 左侧锚点导航 */
.set-nav {
  width: 168px;
  flex-shrink: 0;
  position: sticky;
  top: 20px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.set-nav-item {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 9px 12px;
  border-radius: 10px;
  font-size: 13px;
  color: var(--soma-text-dim);
  cursor: pointer;
  transition: all 0.15s ease;
}

.set-nav-item:hover {
  color: var(--soma-text);
  background: var(--soma-glass);
}

.set-nav-item.is-active {
  color: var(--soma-text);
  background: var(--soma-gradient-soft);
  border: 1px solid rgba(91, 140, 255, 0.3);
  font-weight: 600;
}

.set-nav-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--soma-warning);
  flex-shrink: 0;
}

.set-nav-dot.ok {
  background: var(--soma-success);
  box-shadow: 0 0 8px rgba(52, 211, 153, 0.6);
}

/* 内容分区 */
.set-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.set-section {
  scroll-margin-top: 20px;
}

.sec-head {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 16px;
}

.sec-icon {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.sec-icon.llm { background: rgba(91, 140, 255, 0.14); color: #7ba3ff; }
.sec-icon.tts { background: rgba(94, 234, 212, 0.12); color: #5eead4; }
.sec-icon.stock { background: rgba(244, 114, 182, 0.12); color: #f472b6; }
.sec-icon.aivideo { background: rgba(167, 139, 250, 0.12); color: #a78bfa; }
.sec-icon.system { background: rgba(148, 163, 184, 0.12); color: #94a3b8; }

.sec-titles {
  flex: 1;
  min-width: 0;
}

.sec-title {
  font-size: 15.5px;
  font-weight: 650;
}

.sec-desc {
  font-size: 12.5px;
  color: var(--soma-text-faint);
  margin-top: 2px;
}

.sec-fields {
  border: 1px solid var(--soma-line);
  border-radius: var(--soma-radius);
  background: var(--soma-glass);
  backdrop-filter: blur(14px);
  padding: 20px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  column-gap: 16px;
}

/* 底部保存栏 */
.save-bar {
  position: fixed;
  left: calc(232px + 28px);
  right: 28px;
  bottom: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 20px;
  border-radius: var(--soma-radius) var(--soma-radius) 0 0;
  background: rgba(16, 22, 42, 0.92);
  backdrop-filter: blur(18px);
  border: 1px solid var(--soma-line);
  border-bottom: none;
  transition: border-color 0.2s ease;
}

.save-bar.dirty {
  border-color: rgba(91, 140, 255, 0.45);
}

.save-hint {
  flex: 1;
  font-size: 13px;
  color: var(--soma-text-faint);
}

.save-bar.dirty .save-hint {
  color: var(--soma-accent);
}

html:not(.dark) .save-bar {
  background: rgba(255, 255, 255, 0.94);
}

.env-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.env-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  flex-wrap: wrap;
}

.env-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.env-dot.ok {
  background: var(--soma-success);
  box-shadow: 0 0 8px rgba(52, 211, 153, 0.6);
}

.env-dot.bad {
  background: var(--soma-danger);
  box-shadow: 0 0 8px rgba(248, 113, 113, 0.6);
}

.env-name {
  font-weight: 600;
  min-width: 110px;
}

.env-detail {
  color: var(--soma-text-dim);
  flex: 1;
  min-width: 160px;
}

.env-hint {
  color: var(--soma-warning);
  font-size: 12px;
}
</style>
