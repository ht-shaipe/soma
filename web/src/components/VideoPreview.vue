<template>
  <div v-if="task && task.video_url" class="video-results">
    <div class="section-title">{{ $t('task.playVideo') }}</div>
    <div class="video-item">
      <video
        :src="videoSrc"
        controls
        preload="metadata"
      />
      <div style="margin-top: 8px; display: flex; gap: 8px">
        <el-button type="primary" size="small" @click="onDownload">
          <el-icon><Download /></el-icon>
          {{ $t('task.downloadVideo') }}
        </el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Download } from '@element-plus/icons-vue'
import { useTaskStore } from '@/stores/task'
import { getStreamUrl, getDownloadUrl } from '@/api/stream'

const taskStore = useTaskStore()
const task = computed(() => taskStore.currentTask)

const videoSrc = computed(() => {
  if (task.value?.video_url) {
    return getStreamUrl(task.value.video_url)
  }
  return ''
})

function onDownload() {
  if (task.value?.video_url) {
    const url = getDownloadUrl(task.value.video_url)
    window.open(url, '_blank')
  }
}
</script>
