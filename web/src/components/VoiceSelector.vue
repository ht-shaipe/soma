<template>
  <div>
    <div class="form-row">
      <div class="form-label">{{ $t('audio.voiceName') }}</div>
      <el-select v-model="store.voiceName" style="width: 100%" filterable>
        <el-option
          v-for="voice in voiceList"
          :key="voice.value"
          :label="voice.label"
          :value="voice.value"
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
import { ref, computed, watch } from 'vue'
import { VideoPlay } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useVideoParamsStore } from '@/stores/videoParams'
import { useI18n } from 'vue-i18n'

const store = useVideoParamsStore()
const { t } = useI18n()
const playingVoice = ref(false)
const previewUrl = ref('')
const audioRef = ref<HTMLAudioElement>()

const edgeVoices = [
  { label: 'zh-CN-XiaoxiaoNeural (Female)', value: 'zh-CN-XiaoxiaoNeural-Female' },
  { label: 'zh-CN-YunxiNeural (Male)', value: 'zh-CN-YunxiNeural-Male' },
  { label: 'zh-CN-XiaoyiNeural (Female)', value: 'zh-CN-XiaoyiNeural-Female' },
  { label: 'zh-CN-YunjianNeural (Male)', value: 'zh-CN-YunjianNeural-Male' },
  { label: 'zh-HK-HiuGaaiNeural (Female)', value: 'zh-HK-HiuGaaiNeural-Female' },
  { label: 'zh-TW-HsiaoChenNeural (Female)', value: 'zh-TW-HsiaoChenNeural-Female' },
  { label: 'en-US-JennyNeural (Female)', value: 'en-US-JennyNeural-Female' },
  { label: 'en-US-GuyNeural (Male)', value: 'en-US-GuyNeural-Male' },
  { label: 'en-US-AriaNeural (Female)', value: 'en-US-AriaNeural-Female' },
  { label: 'en-US-DavisNeural (Male)', value: 'en-US-DavisNeural-Male' },
  { label: 'ja-JP-NanamiNeural (Female)', value: 'ja-JP-NanamiNeural-Female' },
  { label: 'ko-KR-SunHiNeural (Female)', value: 'ko-KR-SunHiNeural-Female' },
]

const voiceList = computed(() => {
  return edgeVoices
})

watch(() => store.ttsServer, () => {
  if (voiceList.value.length > 0) {
    store.voiceName = voiceList.value[0].value
  }
})

async function onPlayVoice() {
  playingVoice.value = true
  try {
    if (previewUrl.value && audioRef.value) {
      audioRef.value.play()
    } else {
      ElMessage.info('Voice preview requires backend API')
    }
  } finally {
    playingVoice.value = false
  }
}
</script>
