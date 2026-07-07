<template>
  <div class="tasks-view">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px">
      <h2>{{ $t('task.title') }}</h2>
      <el-button :icon="Refresh" @click="onRefresh">
        {{ $t('task.refresh') }}
      </el-button>
    </div>

    <el-table :data="taskStore.tasks" stripe style="width: 100%" :empty-text="$t('task.noTasks')">
      <el-table-column prop="taskId" :label="$t('task.taskId')" width="220">
        <template #default="{ row }">
          <el-text size="small" truncated>{{ row.taskId }}</el-text>
        </template>
      </el-table-column>
      <el-table-column :label="$t('task.subject')" min-width="140">
        <template #default="{ row }">
          <el-text truncated>{{ row.params?.video_subject || '—' }}</el-text>
        </template>
      </el-table-column>
      <el-table-column :label="$t('task.script')" min-width="120">
        <template #default="{ row }">
          <el-text truncated>{{ row.script ? row.script.substring(0, 50) + '...' : '—' }}</el-text>
        </template>
      </el-table-column>
      <el-table-column :label="$t('task.aspect')" width="80" align="center">
        <template #default="{ row }">
          {{ row.params?.video_aspect || '—' }}
        </template>
      </el-table-column>
      <el-table-column prop="state" :label="$t('task.status')" width="140">
        <template #default="{ row }">
          <TaskStatusTag :state="row.state" :progress="row.progress" />
        </template>
      </el-table-column>
      <el-table-column prop="progress" :label="$t('task.progress')" width="100">
        <template #default="{ row }">
          <el-progress :percentage="row.progress" :status="getProgressStatus(row.state)" :stroke-width="10" />
        </template>
      </el-table-column>
      <el-table-column prop="createdAt" :label="$t('task.createdAt')" width="170">
        <template #default="{ row }">
          {{ formatDate(row.createdAt) }}
        </template>
      </el-table-column>
      <el-table-column :label="$t('task.video')" width="70" align="center">
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
      <el-table-column :label="$t('task.actions')" width="240" fixed="right">
        <template #default="{ row }">
          <el-button v-if="isTaskDraft(row)" type="warning" size="small" text @click="onContinueEdit(row)">
            <el-icon><Edit /></el-icon>
            {{ $t('task.continueEdit') }}
          </el-button>
          <el-button type="primary" size="small" text @click="onViewDetail(row)">
            <el-icon><View /></el-icon>
            {{ $t('task.detail') }}
          </el-button>
          <el-button type="danger" size="small" text @click="onDeleteTask(row)">
            <el-icon><Delete /></el-icon>
            {{ $t('task.delete') }}
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-drawer v-model="detailVisible" :title="$t('task.detailTitle')" size="500px">
      <template v-if="detailTask">
        <el-descriptions :column="1" border>
          <el-descriptions-item :label="$t('task.taskId')">{{ detailTask.taskId }}</el-descriptions-item>
          <el-descriptions-item :label="$t('task.status')">
            <TaskStatusTag :state="detailTask.state" />
          </el-descriptions-item>
          <el-descriptions-item :label="$t('task.progress')">
            <el-progress :percentage="detailTask.progress" :status="getProgressStatus(detailTask.state)" />
          </el-descriptions-item>
          <el-descriptions-item :label="$t('task.createdAt')">{{ formatDate(detailTask.createdAt) }}</el-descriptions-item>
          <el-descriptions-item v-if="detailTask.videoSubject" :label="$t('video.subject')">{{ detailTask.videoSubject }}</el-descriptions-item>
          <el-descriptions-item v-if="detailTask.videoScript" :label="$t('script.title')">
            <div style="max-height: 200px; overflow-y: auto; white-space: pre-wrap;">{{ detailTask.videoScript }}</div>
          </el-descriptions-item>
          <el-descriptions-item v-if="detailTask.audioFile" :label="$t('audio.title')">
            <audio controls :src="getStaticUrl(detailTask.audioFile)" style="width: 100%" />
          </el-descriptions-item>
          <el-descriptions-item v-if="detailTask.subtitleFile" :label="$t('subtitle.title')">{{ detailTask.subtitleFile }}</el-descriptions-item>
        </el-descriptions>
        <div v-if="detailTask.combinedVideos && detailTask.combinedVideos.length > 0" style="margin-top: 16px;">
          <div style="font-weight: bold; margin-bottom: 8px;">{{ $t('task.video') }}</div>
          <video
            v-for="v in detailTask.combinedVideos"
            :key="v"
            controls
            :src="getStaticUrl(v)"
            style="width: 100%; margin-bottom: 8px;"
          />
        </div>
        <div v-if="detailTask.errorMessage" style="margin-top: 16px;">
          <el-alert :title="detailTask.errorMessage" type="error" show-icon :closable="false" />
        </div>
      </template>
    </el-drawer>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Refresh, VideoPlay, Delete, View, Edit } from '@element-plus/icons-vue'
import { useTaskStore } from '@/stores/task'
import { TaskStateCode, isTaskDraft } from '@/types'
import type { TaskInfo } from '@/types'
import { getStaticUrl } from '@/api/stream'
import TaskStatusTag from '@/components/TaskStatusTag.vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

const taskStore = useTaskStore()
const router = useRouter()
const { t } = useI18n()
const detailVisible = ref(false)
const detailTask = ref<TaskInfo | null>(null)

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

function onViewDetail(row: TaskInfo) {
  detailTask.value = row
  detailVisible.value = true
}

function onContinueEdit(row: TaskInfo) {
  taskStore.resumeDraft(row.taskId)
  router.push({ name: 'home', query: { resume: row.taskId } })
}
</script>
