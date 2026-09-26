<template>
  <div v-if="logs.length > 0" class="ai-video-logs">
    <div class="ai-video-logs-title">
      AI 视频生成日志
      <el-tag size="small" type="info">{{ successCount }}/{{ logs.length }} 成功</el-tag>
    </div>
    <div class="ai-video-logs-list">
      <div v-for="log in logs" :key="log.scene_id" class="ai-video-log-item">
        <div class="ai-video-log-header">
          <el-tag :type="statusTagType(log.status)" size="small">#{{ log.scene_id }}</el-tag>
          <span class="ai-video-log-status">{{ statusLabel(log.status) }}</span>
        </div>
        <div class="ai-video-log-prompt">{{ log.prompt }}</div>
        <div v-if="log.message" class="ai-video-log-message">{{ log.message }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useTaskStore } from '@/stores/task'
import type { AiVideoSegmentLog } from '@/types'

const taskStore = useTaskStore()

const logs = computed<AiVideoSegmentLog[]>(() => {
  return taskStore.currentTask?.aiVideoLogs ?? []
})

const successCount = computed(() => logs.value.filter(l => l.status === 'success').length)

function statusTagType(status: string) {
  switch (status) {
    case 'success': return 'success'
    case 'failed': case 'timeout': return 'danger'
    case 'processing': case 'submitted': return 'warning'
    default: return 'info'
  }
}

function statusLabel(status: string) {
  switch (status) {
    case 'pending': return '等待中'
    case 'submitted': return '已提交'
    case 'processing': return '生成中'
    case 'success': return '生成成功'
    case 'failed': return '生成失败'
    case 'timeout': return '生成超时'
    default: return status
  }
}
</script>

<style scoped>
.ai-video-logs {
  margin-top: 12px;
}

.ai-video-logs-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.ai-video-logs-list {
  max-height: 320px;
  overflow-y: auto;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 4px;
  padding: 4px 0;
}

.ai-video-log-item {
  padding: 8px 12px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.ai-video-log-item:last-child {
  border-bottom: none;
}

.ai-video-log-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.ai-video-log-status {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.ai-video-log-prompt {
  font-size: 12px;
  color: var(--el-text-color-primary);
  word-break: break-all;
  line-height: 1.5;
  background: var(--el-fill-color-light);
  padding: 6px 8px;
  border-radius: 3px;
  margin-bottom: 2px;
}

.ai-video-log-message {
  font-size: 11px;
  color: var(--el-color-danger);
  margin-top: 2px;
}
</style>
