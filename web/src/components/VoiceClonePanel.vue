<template>
  <div class="voice-clone-panel">
    <div class="clone-section">
      <div class="form-row">
        <div class="form-label">{{ $t('audio.voiceClone.engine') }}</div>
        <el-select v-model="cloneEngine" style="width: 100%">
          <el-option label="ElevenLabs" value="elevenlabs" />
          <el-option label="SiliconFlow (CosyVoice2)" value="siliconflow" />
          <el-option label="火山引擎 (豆包)" value="volcengine" />
          <el-option label="科大讯飞 (声音复刻)" value="xfyun" />
        </el-select>
      </div>
      <div class="form-row">
        <div class="form-label">{{ $t('audio.voiceClone.voiceName') }}</div>
        <el-input v-model="cloneName" :placeholder="$t('audio.voiceClone.voiceNamePlaceholder')" />
      </div>
      <div class="form-row">
        <div class="form-label">{{ $t('audio.voiceClone.uploadAudio') }}</div>
        <el-upload
          :auto-upload="false"
          :accept="'.mp3,.wav,.m4a,.aac,.flac,.ogg'"
          :limit="1"
          :on-change="onFileChange"
          :on-exceed="onFileExceed"
          name="file"
        >
          <el-button size="small">
            <el-icon><Upload /></el-icon>
            {{ $t('audio.voiceClone.selectFile') }}
          </el-button>
          <template #tip>
            <div class="el-upload__tip">{{ $t('audio.voiceClone.uploadTip') }}</div>
          </template>
        </el-upload>
      </div>
      <el-button
        type="primary"
        size="small"
        :loading="cloning"
        :disabled="!cloneFile"
        style="margin-top: 8px"
        @click="onClone"
      >
        <el-icon><Microphone /></el-icon>
        {{ $t('audio.voiceClone.startClone') }}
      </el-button>
    </div>

    <el-divider v-if="clonedVoices.length > 0" />

    <div v-if="clonedVoices.length > 0" class="cloned-list">
      <div class="form-label">{{ $t('audio.voiceClone.clonedList') }}</div>
      <div
        v-for="voice in clonedVoices"
        :key="voice.id"
        class="cloned-item"
      >
        <div class="cloned-item-info">
          <span class="cloned-item-name">{{ voice.name }}</span>
          <el-tag size="small" type="info">{{ voice.engine }}</el-tag>
        </div>
        <div class="cloned-item-actions">
          <el-button size="small" text @click="onUseVoice(voice)">
            {{ $t('audio.voiceClone.use') }}
          </el-button>
          <el-button size="small" text type="danger" @click="onDeleteVoice(voice)">
            {{ $t('common.delete') }}
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Upload, Microphone } from '@element-plus/icons-vue'
import { useVideoParamsStore } from '@/stores/videoParams'
import { useI18n } from 'vue-i18n'
import {
  listClonedVoices,
  deleteClonedVoice,
  cloneVoice,
  type ClonedVoiceItem,
} from '@/api/voice'
import type { UploadFile } from 'element-plus'

const store = useVideoParamsStore()
const { t } = useI18n()

const cloneEngine = ref('elevenlabs')
const cloneName = ref('')
const cloneFile = ref<File | null>(null)
const cloning = ref(false)
const clonedVoices = ref<ClonedVoiceItem[]>([])

async function fetchClonedVoices() {
  try {
    const data = await listClonedVoices()
    clonedVoices.value = data.voices || []
  } catch {
    clonedVoices.value = []
  }
}

function onFileChange(file: UploadFile) {
  cloneFile.value = file.raw || null
}

function onFileExceed() {
  ElMessage.warning(t('audio.voiceClone.fileLimit'))
}

async function onClone() {
  if (!cloneFile.value) {
    ElMessage.warning(t('audio.voiceClone.noFile'))
    return
  }
  if (!cloneName.value.trim()) {
    ElMessage.warning(t('audio.voiceClone.noName'))
    return
  }

  cloning.value = true
  try {
    await cloneVoice(cloneFile.value, cloneEngine.value, cloneName.value.trim())
    ElMessage.success(t('audio.voiceClone.cloneSuccess'))
    cloneFile.value = null
    cloneName.value = ''
    await fetchClonedVoices()
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    ElMessage.error(t('audio.voiceClone.cloneFailed') + ': ' + msg)
  } finally {
    cloning.value = false
  }
}

function onUseVoice(voice: ClonedVoiceItem) {
  store.voiceName = voice.voiceName
  if (voice.engine === 'elevenlabs') {
    store.ttsServer = 'elevenlabs'
  } else if (voice.engine === 'siliconflow') {
    store.ttsServer = 'siliconflow'
  } else if (voice.engine === 'volcengine') {
    store.ttsServer = 'volcengine'
  } else if (voice.engine === 'xfyun') {
    store.ttsServer = 'xfyun'
  }
  ElMessage.success(t('audio.voiceClone.voiceSelected') + ': ' + voice.name)
}

async function onDeleteVoice(voice: ClonedVoiceItem) {
  try {
    await ElMessageBox.confirm(
      t('audio.voiceClone.deleteConfirm') + ': ' + voice.name + '?',
      t('common.confirm'),
      { type: 'warning' },
    )
  } catch {
    return
  }

  try {
    await deleteClonedVoice(voice.id)
    ElMessage.success(t('common.deleteSuccess'))
    await fetchClonedVoices()
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    ElMessage.error(t('common.deleteFailed') + ': ' + msg)
  }
}

onMounted(() => {
  fetchClonedVoices()
})

defineExpose({ fetchClonedVoices })
</script>

<style scoped>
.voice-clone-panel {
  width: 100%;
}
.clone-section {
  padding: 8px 0;
}
.cloned-list {
  margin-top: 8px;
}
.cloned-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}
.cloned-item:last-child {
  border-bottom: none;
}
.cloned-item-info {
  display: flex;
  align-items: center;
  gap: 8px;
}
.cloned-item-name {
  font-size: 14px;
}
.cloned-item-actions {
  display: flex;
  gap: 4px;
}
.form-row {
  margin-bottom: 12px;
}
.form-label {
  margin-bottom: 4px;
  font-size: 13px;
  color: var(--el-text-color-regular);
}
</style>
