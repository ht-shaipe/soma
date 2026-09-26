<template>
  <el-card class="panel-card" shadow="hover">
    <template #header>
      <span>{{ $t('audio.title') }}</span>
    </template>
    <div class="form-row">
      <div class="form-label">{{ $t('audio.ttsServer') }}</div>
      <el-select v-model="store.ttsServer" style="width: 100%">
        <el-option :label="$t('audio.noVoice')" value="none" />
        <el-option :label="$t('audio.edgeTts')" value="edge-tts" />
        <el-option :label="$t('audio.azureTtsV2')" value="azure-v2" />
        <el-option :label="$t('audio.siliconflowTts')" value="siliconflow" />
        <el-option :label="$t('audio.geminiTts')" value="gemini" />
        <el-option :label="$t('audio.mimoTts')" value="mimo" />
        <el-option :label="$t('audio.elevenlabsTts')" value="elevenlabs" />
        <el-option :label="$t('audio.volcengineTts')" value="volcengine" />
        <el-option :label="$t('audio.xfyunTts')" value="xfyun" />
        <el-option :label="$t('audio.fishspeechTts')" value="fishspeech" />
      </el-select>
    </div>
    <VoiceSelector v-if="store.ttsServer !== 'none'" />
    <el-collapse style="margin-top: 8px">
      <el-collapse-item :title="$t('audio.voiceClone.title')" name="clone">
        <VoiceClonePanel />
      </el-collapse-item>
    </el-collapse>
    <div v-if="store.ttsServer === 'azure-v2'" class="form-row">
      <div class="form-label">{{ $t('audio.azureSpeechKey') }}</div>
      <el-input v-model="azureSpeechKey" type="password" show-password @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'azure-v2'" class="form-row">
      <div class="form-label">{{ $t('audio.azureSpeechRegion') }}</div>
      <el-input v-model="azureSpeechRegion" @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'siliconflow'" class="form-row">
      <div class="form-label">{{ $t('audio.siliconflowApiKey') }}</div>
      <el-input v-model="siliconflowKey" type="password" show-password @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'mimo'" class="form-row">
      <div class="form-label">{{ $t('audio.mimoApiKey') }}</div>
      <el-input v-model="mimoKey" type="password" show-password @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'elevenlabs'" class="form-row">
      <div class="form-label">{{ $t('audio.elevenlabsApiKey') }}</div>
      <el-input v-model="elevenlabsKey" type="password" show-password @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'elevenlabs'" class="form-row">
      <div class="form-label">{{ $t('audio.elevenlabsModel') }}</div>
      <el-select v-model="elevenlabsModel" style="width: 100%" @change="onSaveTtsConfig">
        <el-option label="eleven_multilingual_v2" value="eleven_multilingual_v2" />
        <el-option label="eleven_flash_v2_5" value="eleven_flash_v2_5" />
        <el-option label="eleven_v3" value="eleven_v3" />
      </el-select>
    </div>
    <div v-if="store.ttsServer === 'gemini'" class="form-row">
      <div class="form-label">{{ $t('audio.geminiApiKey') }}</div>
      <el-input v-model="geminiKey" type="password" show-password @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'volcengine'" class="form-row">
      <div class="form-label">{{ $t('audio.volcengineAppId') }}</div>
      <el-input v-model="volcAppId" @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'volcengine'" class="form-row">
      <div class="form-label">{{ $t('audio.volcengineAccessToken') }}</div>
      <el-input v-model="volcAccessToken" type="password" show-password @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'xfyun'" class="form-row">
      <div class="form-label">{{ $t('audio.xfyunAppId') }}</div>
      <el-input v-model="xfyunAppId" @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'xfyun'" class="form-row">
      <div class="form-label">{{ $t('audio.xfyunApiKey') }}</div>
      <el-input v-model="xfyunApiKey" type="password" show-password @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'xfyun'" class="form-row">
      <div class="form-label">{{ $t('audio.xfyunApiSecret') }}</div>
      <el-input v-model="xfyunApiSecret" type="password" show-password @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'fishspeech'" class="form-row">
      <div class="form-label">{{ $t('audio.fishspeechBaseUrl') }}</div>
      <el-input v-model="fishspeechBaseUrl" placeholder="https://api.fish.audio" @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'fishspeech'" class="form-row">
      <div class="form-label">{{ $t('audio.fishspeechApiKey') }}</div>
      <el-input v-model="fishspeechApiKey" type="password" show-password @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer === 'fishspeech'" class="form-row">
      <div class="form-label">{{ $t('audio.fishspeechRefId') }}</div>
      <el-input v-model="fishspeechReferenceId" @change="onSaveTtsConfig" />
    </div>
    <div v-if="store.ttsServer !== 'none'" class="form-row">
      <div class="form-label">{{ $t('audio.speechVolume') }}</div>
      <el-select v-model="store.voiceVolume" style="width: 100%">
        <el-option v-for="v in [0.6,0.8,1.0,1.2,1.5,2.0,3.0,4.0,5.0]" :key="v" :label="v" :value="v" />
      </el-select>
    </div>
    <div v-if="store.ttsServer !== 'none'" class="form-row">
      <div class="form-label">{{ $t('audio.speechRate') }}</div>
      <el-select v-model="store.voiceRate" style="width: 100%">
        <el-option v-for="r in [0.8,0.9,1.0,1.1,1.2,1.3,1.5,1.8,2.0]" :key="r" :label="r" :value="r" />
      </el-select>
    </div>
    <div class="form-row">
      <div class="form-label">{{ $t('audio.customAudio') }}</div>
      <el-upload
        :auto-upload="true"
        :action="'/api/v1/audio/upload'"
        :accept="'.mp3,.wav,.m4a,.aac,.flac,.ogg'"
        :limit="1"
        :on-success="onAudioUploadSuccess"
        :on-error="onAudioUploadError"
        name="file"
      >
        <el-button size="small">
          <el-icon><Upload /></el-icon>
          {{ $t('common.upload') }}
        </el-button>
        <template #tip>
          <div class="el-upload__tip">{{ $t('audio.customAudioTip') }}</div>
        </template>
      </el-upload>
    </div>
    <BgmSelector />
  </el-card>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Upload } from '@element-plus/icons-vue'
import { useVideoParamsStore } from '@/stores/videoParams'
import { useConfigStore } from '@/stores/config'
import { useI18n } from 'vue-i18n'
import VoiceSelector from './VoiceSelector.vue'
import VoiceClonePanel from './VoiceClonePanel.vue'
import BgmSelector from './BgmSelector.vue'

const store = useVideoParamsStore()
const configStore = useConfigStore()
const { t } = useI18n()

const azureSpeechKey = ref('')
const azureSpeechRegion = ref('')
const siliconflowKey = ref('')
const mimoKey = ref('')
const elevenlabsKey = ref('')
const elevenlabsModel = ref('eleven_multilingual_v2')
const geminiKey = ref('')
const volcAppId = ref('')
const volcAccessToken = ref('')
const xfyunAppId = ref('')
const xfyunApiKey = ref('')
const xfyunApiSecret = ref('')
const fishspeechBaseUrl = ref('')
const fishspeechApiKey = ref('')
const fishspeechReferenceId = ref('')

function loadTtsKeys() {
  const tts = configStore.config.tts
  azureSpeechKey.value = tts.azure_speech_key || ''
  azureSpeechRegion.value = tts.azure_speech_region || ''
  siliconflowKey.value = tts.siliconflow_key || ''
  mimoKey.value = tts.mimo_key || ''
  elevenlabsKey.value = tts.elevenlabs_key || ''
  elevenlabsModel.value = tts.elevenlabs_model || 'eleven_multilingual_v2'
  geminiKey.value = tts.gemini_key || ''
  volcAppId.value = tts.volcengine_app_id || ''
  volcAccessToken.value = tts.volcengine_access_token || ''
  xfyunAppId.value = tts.xfyun_app_id || ''
  xfyunApiKey.value = tts.xfyun_api_key || ''
  xfyunApiSecret.value = tts.xfyun_api_secret || ''
  fishspeechBaseUrl.value = tts.fishspeech_base_url || ''
  fishspeechApiKey.value = tts.fishspeech_api_key || ''
  fishspeechReferenceId.value = tts.fishspeech_reference_id || ''
}

function onSaveTtsConfig() {
  configStore.updateTtsConfig({
    azure_speech_key: azureSpeechKey.value,
    azure_speech_region: azureSpeechRegion.value,
    siliconflow_key: siliconflowKey.value,
    mimo_key: mimoKey.value,
    elevenlabs_key: elevenlabsKey.value,
    elevenlabs_model: elevenlabsModel.value,
    gemini_key: geminiKey.value,
    volcengine_app_id: volcAppId.value,
    volcengine_access_token: volcAccessToken.value,
    xfyun_app_id: xfyunAppId.value,
    xfyun_api_key: xfyunApiKey.value,
    xfyun_api_secret: xfyunApiSecret.value,
    fishspeech_base_url: fishspeechBaseUrl.value,
    fishspeech_api_key: fishspeechApiKey.value,
    fishspeech_reference_id: fishspeechReferenceId.value,
  })
  configStore.save()
}

onMounted(() => {
  if (configStore.loaded) {
    loadTtsKeys()
  }
})

watch(() => configStore.loaded, (val) => {
  if (val) loadTtsKeys()
})

function onAudioUploadSuccess(response: { data?: { name?: string; path?: string } }) {
  const name = response.data?.name || response.data?.path || ''
  if (name) {
    store.customAudioFileName = name
    ElMessage.success(t('audio.uploadSuccess'))
  }
}

function onAudioUploadError() {
  ElMessage.error(t('audio.uploadFailed'))
}
</script>
