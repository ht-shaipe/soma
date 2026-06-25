<template>
  <div class="tasks-view">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px">
      <h2>{{ $t('task.title') }}</h2>
      <el-button :icon="Refresh" @click="onRefresh">
        {{ $t('task.refresh') }}
      </el-button>
    </div>

    <el-table :data="taskStore.tasks" stripe style="width: 100%" empty-text="No tasks">
      <el-table-column prop="task_id" :label="$t('task.taskId')" width="280">
        <template #default="{ row }">
          <el-text size="small" truncated>{{ row.task_id }}</el-text>
        </template>
      </el-table-column>
      <el-table-column prop="params.video_subject" :label="$t('task.topic')" min-width="120">
        <template #default="{ row }">
          {{ row.params?.video_subject || '—' }}
        </template>
      </el-table-column>
      <el-table-column prop="status" :label="$t('task.status')" width="140">
        <template #default="{ row }">
          <TaskStatusTag :status="row.status" />
        </template>
      </el-table-column>
      <el-table-column prop="progress" :label="$t('task.progress')" width="100">
        <template #default="{ row }">
          <el-progress :percentage="row.progress" :status="getProgressStatus(row.status)" :stroke-width="10" />
        </template>
      </el-table-column>
      <el-table-column prop="created_at" :label="$t('task.createdAt')" width="180">
        <template #default="{ row }">
          {{ formatDate(row.created_at) }}
        </template>
      </el-table-column>
      <el-table-column :label="$t('task.video')" width="80">
        <template #default="{ row }">
          <el-button
            v-if="row.video_url"
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
import { useI18n } from 'vue-i18n'
import { TaskStatus } from '@/types'
import type { TaskInfo } from '@/types'
import TaskStatusTag from '@/components/TaskStatusTag.vue'

const taskStore = useTaskStore()
const { t } = useI18n()

onMounted(() => {
  taskStore.fetchTasks()
})

function onRefresh() {
  taskStore.fetchTasks()
}

function getProgressStatus(status: TaskStatus) {
  if (status === TaskStatus.Completed) return 'success' as const
  if (status === TaskStatus.Failed) return 'exception' as const
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
    await taskStore.removeTask(row.task_id)
    ElMessage.success(t('common.success'))
  } catch {
    // cancelled
  }
}

function onPlayVideo(row: TaskInfo) {
  if (row.video_url) {
    window.open(`/api/v1/stream/${encodeURIComponent(row.video_url)}`, '_blank')
  }
}
</script>
