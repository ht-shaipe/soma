<template>
  <div class="tasks-view">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px">
      <h2>{{ $t('task.title') }}</h2>
      <el-button :icon="Refresh" @click="onRefresh">
        {{ $t('task.refresh') }}
      </el-button>
    </div>

    <el-table :data="taskStore.tasks" stripe style="width: 100%" empty-text="No tasks">
      <el-table-column prop="taskId" :label="$t('task.taskId')" width="280">
        <template #default="{ row }">
          <el-text size="small" truncated>{{ row.taskId }}</el-text>
        </template>
      </el-table-column>
      <el-table-column prop="script" :label="$t('task.topic')" min-width="120">
        <template #default="{ row }">
          {{ row.script ? row.script.substring(0, 50) + '...' : '—' }}
        </template>
      </el-table-column>
      <el-table-column prop="state" :label="$t('task.status')" width="140">
        <template #default="{ row }">
          <TaskStatusTag :state="row.state" />
        </template>
      </el-table-column>
      <el-table-column prop="progress" :label="$t('task.progress')" width="100">
        <template #default="{ row }">
          <el-progress :percentage="row.progress" :status="getProgressStatus(row.state)" :stroke-width="10" />
        </template>
      </el-table-column>
      <el-table-column prop="createdAt" :label="$t('task.createdAt')" width="180">
        <template #default="{ row }">
          {{ formatDate(row.createdAt) }}
        </template>
      </el-table-column>
      <el-table-column :label="$t('task.video')" width="80">
        <template #default="{ row }">
          <el-button
            v-if="row.combinedVideos && row.combinedVideos.length > 0"
            type="primary"
            size="small"
            circle
            @click="onPlayVideo(row)"
          >
            <el-icon><VideoPlay /></el-icon>
          </el-button>
          <span v-else>—</span>
        </template>
      </el-table-column>
      <el-table-column :label="$t('task.actions')" width="120" fixed="right">
        <template #default="{ row }">
          <el-button type="danger" size="small" text @click="onDeleteTask(row)">
            <el-icon><Delete /></el-icon>
            {{ $t('task.delete') }}
          </el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Refresh, VideoPlay, Delete } from '@element-plus/icons-vue'
import { useTaskStore } from '@/stores/task'
import { TaskStateCode, isTaskCompleted, isTaskFailed } from '@/types'
import type { TaskInfo } from '@/types'
import { getStaticUrl } from '@/api/stream'
import TaskStatusTag from '@/components/TaskStatusTag.vue'
import { useI18n } from 'vue-i18n'

const taskStore = useTaskStore()
const { t } = useI18n()

onMounted(() => {
  taskStore.fetchTasks()
})

function onRefresh() {
  taskStore.fetchTasks()
}

function getProgressStatus(state: number) {
  if (state === TaskStateCode.Completed) return 'success' as const
  if (state === TaskStateCode.Failed) return 'exception' as const
  return undefined
}

function formatDate(dateStr: string) {
  if (!dateStr) return '—'
  return new Date(dateStr).toLocaleString()
}

async function onDeleteTask(row: TaskInfo) {
  try {
    await ElMessageBox.confirm(t('task.deleteConfirm'), t('common.warning'), {
      confirmButtonText: t('common.ok'),
      cancelButtonText: t('common.cancel'),
      type: 'warning',
    })
    await taskStore.removeTask(row.taskId)
    ElMessage.success(t('common.success'))
  } catch {
    // cancelled
  }
}

function onPlayVideo(row: TaskInfo) {
  if (row.combinedVideos && row.combinedVideos.length > 0) {
    window.open(getStaticUrl(row.combinedVideos[0]), '_blank')
  }
}
</script>
