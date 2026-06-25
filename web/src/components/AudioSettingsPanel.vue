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
      </el-select>
    </div>
    <VoiceSelector v-if="store.ttsServer !== 'none'" />
    <div v-if="store.ttsServer === 'azure-v2'" class="form-row">
      <div class="form-label">{{ $t('audio.azureSpeechKey') }}</div>
      <el-input v-model="azureSpeechKey" type="password" show-password />
    </div>
    <div v-if="store.ttsServer === 'azure-v2'" class="form-row">
      <div class="form-label">{{ $t('audio.azureSpeechRegion') }}</div>
      <el-input v-model="azureSpeechRegion" />
    </div>
    <div v-if="store.ttsServer === 'siliconflow'" class="form-row">
      <div class="form-label">{{ $t('audio.siliconflowApiKey') }}</div>
      <el-input v-model="siliconflowKey" type="password" show-password />
    </div>
    <div v-if="store.ttsServer === 'mimo'" class="form-row">
      <div class="form-label">{{ $t('audio.mimoApiKey') }}</div>
      <el-input v-model="mimoKey" type="password" show-password />
    </div>
    <div v-if="store.ttsServer === 'elevenlabs'" class="form-row">
      <div class="form-label">{{ $t('audio.elevenlabsApiKey') }}</div>
      <el-input v-model="elevenlabsKey" type="password" show-password />
    </div>
    <div v-if="store.ttsServer === 'elevenlabs'" class="form-row">
      <div class="form-label">{{ $t('audio.elevenlabsModel') }}</div>
      <el-select v-model="elevenlabsModel" style="width: 100%">
        <el-option label="eleven_multilingual_v2" value="eleven_multilingual_v2" />
        <el-option label="eleven_flash_v2_5" value="eleven_flash_v2_5" />
        <el-option label="eleven_v3" value="eleven_v3" />
      </el-select>
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
        :auto-upload="false"
        :accept="'.mp3,.wav,.m4a,.aac,.flac,.ogg'"
        :limit="1"
        :on-change="onAudioChange"
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
import { ref } from 'vue'
import type { UploadFile } from 'element-plus'
import { Upload } from '@element-plus/icons-vue'
import { useVideoParamsStore } from '@/stores/videoParams'
import VoiceSelector from './VoiceSelector.vue'
import BgmSelector from './BgmSelector.vue'

const store = useVideoParamsStore()
const azureSpeechKey = ref('')
const azureSpeechRegion = ref('')
const siliconflowKey = ref('')
const mimoKey = ref('')
const elevenlabsKey = ref('')
const elevenlabsModel = ref('eleven_multilingual_v2')

function onAudioChange(file: UploadFile) {
  if (file.raw) {
    store.customAudioFile = file.raw
  }
}
</script>
