<template>
  <div>
    <div class="form-row">
      <div class="form-label">{{ $t('audio.voiceName') }}</div>
      <el-select v-model="store.voiceName" style="width: 100%" filterable :loading="loadingVoices">
        <el-option
          v-for="voice in voiceList"
          :key="voice.name"
          :label="voice.label"
          :value="voice.name"
        />
      </el-select>
    </div>
    <div style="margin-top: 8px">
      <el-button size="small" :loading="playingVoice" @click="onPlayVoice">
        <el-icon><VideoPlay /></el-icon>
        {{ $t('audio.playVoice') }}
      </el-button>
      <audio v-if="previewUrl" ref="audioRef" :src="previewUrl" style="display:none" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { VideoPlay } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useVideoParamsStore } from '@/stores/videoParams'
import { useI18n } from 'vue-i18n'
import { listVoices, type VoiceItem } from '@/api/voice'

const store = useVideoParamsStore()
const { t } = useI18n()
const playingVoice = ref(false)
const previewUrl = ref('')
const audioRef = ref<HTMLAudioElement>()
const loadingVoices = ref(false)
const voiceList = ref<VoiceItem[]>([])

async function fetchVoices() {
  loadingVoices.value = true
  try {
    const data = await listVoices()
    voiceList.value = data.voices || []
    if (voiceList.value.length > 0 && !voiceList.value.some(v => v.name === store.voiceName)) {
      store.voiceName = voiceList.value[0].name
    }
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    ElMessage.warning(t('audio.voiceListFailed') + ': ' + msg)
    voiceList.value = [
      { name: 'zh-CN-XiaoxiaoNeural', label: 'Xiaoxiao (F, Chinese)' },
      { name: 'zh-CN-YunxiNeural', label: 'Yunxi (M, Chinese)' },
      { name: 'en-US-JennyNeural', label: 'Jenny (F, English)' },
      { name: 'no-voice', label: t('audio.noVoice') },
    ]
  } finally {
    loadingVoices.value = false
  }
}

onMounted(() => {
  fetchVoices()
})

watch(() => store.ttsServer, () => {
  fetchVoices()
})

async function onPlayVoice() {
  const voiceName = store.voiceName
  if (!voiceName || voiceName === 'no-voice') {
    ElMessage.info(t('audio.noVoicePreview'))
    return
  }
  playingVoice.value = true
  try {
    const params = new URLSearchParams({ voice: voiceName, text: t('audio.previewText') })
    previewUrl.value = `/api/v1/voices/preview?${params.toString()}`
    await new Promise<void>((resolve) => {
      if (audioRef.value) {
        audioRef.value.oncanplaythrough = () => {
          audioRef.value?.play()
          resolve()
        }
        audioRef.value.onerror = () => {
          ElMessage.warning(t('audio.voicePreviewFailed'))
          resolve()
        }
        audioRef.value.load()
      } else {
        resolve()
      }
    })
  } finally {
    playingVoice.value = false
  }
}
</script>
