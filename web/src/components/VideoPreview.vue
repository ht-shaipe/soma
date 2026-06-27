<template>
  <div v-if="task && task.combinedVideos && task.combinedVideos.length > 0" class="video-results">
    <div class="section-title">{{ $t('task.playVideo') }}</div>
    <div v-for="(video, idx) in task.combinedVideos" :key="idx" class="video-item">
      <video
        :src="getStaticUrl(video)"
        controls
        preload="metadata"
      />
      <div style="margin-top: 8px; display: flex; gap: 8px">
        <el-button type="primary" size="small" @click="onDownload(video)">
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
import { getStaticUrl } from '@/api/stream'

const taskStore = useTaskStore()
const task = computed(() => taskStore.currentTask)

function onDownload(videoPath: string) {
  window.open(getStaticUrl(videoPath), '_blank')
}
</script>
