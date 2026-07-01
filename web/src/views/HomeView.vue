<template>
  <div class="home-view">
    <el-row :gutter="16" class="responsive-row">
      <el-col :xs="24" :sm="24" :md="8" :lg="8">
        <ScriptPanel />
      </el-col>
      <el-col :xs="24" :sm="24" :md="8" :lg="8">
        <VideoSettingsPanel />
        <AudioSettingsPanel />
      </el-col>
      <el-col :xs="24" :sm="24" :md="8" :lg="8">
        <SubtitleSettingsPanel />
        <SocialMetadataPanel />
        <ApiKeyManager />
      </el-col>
    </el-row>

    <div class="generate-section">
      <el-button
        type="primary"
        size="large"
        :loading="taskStore.isGenerating"
        @click="onGenerateVideo"
      >
        <el-icon v-if="!taskStore.isGenerating"><VideoCamera /></el-icon>
        {{ taskStore.isGenerating ? $t('generate.generating') : $t('generate.button') }}
      </el-button>
    </div>

    <TaskProgress v-if="taskStore.currentTask" />
    <LogOutput />
    <VideoPreview />
  </div>
</template>

<script setup lang="ts">
import { ElMessage } from 'element-plus'
import { VideoCamera } from '@element-plus/icons-vue'
import { useVideoParamsStore } from '@/stores/videoParams'
import { useTaskStore } from '@/stores/task'
import { useConfigStore } from '@/stores/config'
import { useI18n } from 'vue-i18n'
import ScriptPanel from '@/components/ScriptPanel.vue'
import VideoSettingsPanel from '@/components/VideoSettingsPanel.vue'
import AudioSettingsPanel from '@/components/AudioSettingsPanel.vue'
import SubtitleSettingsPanel from '@/components/SubtitleSettingsPanel.vue'
import SocialMetadataPanel from '@/components/SocialMetadataPanel.vue'
import ApiKeyManager from '@/components/ApiKeyManager.vue'
import TaskProgress from '@/components/TaskProgress.vue'
import LogOutput from '@/components/LogOutput.vue'
import VideoPreview from '@/components/VideoPreview.vue'

const videoParamsStore = useVideoParamsStore()
const taskStore = useTaskStore()
const configStore = useConfigStore()
const { t } = useI18n()

async function onGenerateVideo() {
  if (!videoParamsStore.videoSubject && !videoParamsStore.videoScript) {
    ElMessage.warning(t('generate.validate.subjectRequired'))
    return
  }
  if (videoParamsStore.videoSource !== 'local') {
    const stockConfig = configStore.config.stock
    const source = videoParamsStore.videoSource
    if (source === 'pexels' && !stockConfig.pexels_api_key) {
      ElMessage.warning(t('generate.validate.apiKeyRequired'))
      return
    }
    if (source === 'pixabay' && !stockConfig.pixabay_api_key) {
      ElMessage.warning(t('generate.validate.apiKeyRequired'))
      return
    }
    if (source === 'coverr' && !stockConfig.coverr_api_key) {
      ElMessage.warning(t('generate.validate.apiKeyRequired'))
      return
    }
  }

  try {
    const params = videoParamsStore.toVideoParams()
    await taskStore.startGeneration(params)
    ElMessage.success(t('generate.success'))
  } catch {
    ElMessage.error(t('generate.failed'))
  }
}
</script>
