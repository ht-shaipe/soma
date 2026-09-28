<template>
  <div class="tasks-view">
    <div class="tv-toolbar">
      <el-radio-group v-model="taskType" size="large" @change="onTypeChange">
        <el-radio-button value="video">{{ $t('task.typeVideo') }}</el-radio-button>
        <el-radio-button value="digital">{{ $t('task.typeDigital') }}</el-radio-button>
        <el-radio-button value="image">{{ $t('task.typeImage') }}</el-radio-button>
      </el-radio-group>
      <el-input
        v-model="keyword"
        :placeholder="$t('task.search')"
        :prefix-icon="Search"
        clearable
        size="large"
        class="tv-search"
      />
      <el-button :icon="Refresh" size="large" @click="onRefresh">
        {{ $t('task.refresh') }}
      </el-button>
    </div>

    <el-empty v-if="taskType === 'video' && filteredTasks.length === 0" :description="$t('task.noTasks')" />

    <div v-else-if="taskType === 'video'" class="tv-grid">
      <div v-for="row in filteredTasks" :key="row.taskId" class="tv-card">
        <div class="tv-card-top">
          <div class="tv-card-subject">{{ row.params?.video_subject || $t('task.untitled') }}</div>
          <TaskStatusTag :state="row.state" :progress="row.progress" />
        </div>

        <div class="tv-card-id">{{ row.taskId.slice(0, 8) }} · {{ formatDate(row.createdAt) }}</div>

        <p class="tv-card-script">
          {{ row.script ? row.script.substring(0, 76) + (row.script.length > 76 ? '…' : '') : '—' }}
        </p>

        <div class="tv-card-progress">
          <el-progress
            :percentage="row.progress"
            :status="getProgressStatus(row.state)"
            :stroke-width="7"
            :show-text="false"
          />
          <div class="tv-card-progress-meta">
            <span class="tv-aspect">{{ row.params?.video_aspect || '—' }}</span>
            <span>{{ row.progress }}%</span>
          </div>
        </div>

        <div class="tv-card-actions">
          <el-button
            v-if="row.combinedVideos && row.combinedVideos.length > 0"
            type="primary"
            size="small"
            round
            @click="onPlayVideo(row)"
          >
            <el-icon><VideoPlay /></el-icon>
            {{ $t('task.video') }}
          </el-button>
          <el-button size="small" round @click="onViewDetail(row)">
            <el-icon><View /></el-icon>
            {{ $t('task.detail') }}
          </el-button>
          <el-button v-if="isTaskEditable(row)" size="small" round @click="onContinueEdit(row)">
            <el-icon><Edit /></el-icon>
            {{ $t('task.continueEdit') }}
          </el-button>
          <el-button v-if="isTaskStuck(row)" size="small" round type="warning" @click="onStopTask(row)">
            <el-icon><VideoPause /></el-icon>
            {{ $t('task.stop') }}
          </el-button>
          <div class="tv-spacer" />
          <el-button size="small" circle type="danger" plain @click="onDeleteTask(row)">
            <el-icon><Delete /></el-icon>
          </el-button>
        </div>
      </div>
    </div>

    <!-- 数字人任务 -->
    <template v-if="taskType === 'digital'">
      <el-empty v-if="digitalTasks.length === 0" :description="$t('task.noTasks')" />
      <div v-else class="tv-grid">
        <div v-for="row in digitalTasks" :key="row.taskId" class="tv-card">
          <div class="tv-card-top">
            <div class="tv-card-subject">{{ row.params?.narration_text || $t('task.untitled') }}</div>
            <TaskStatusTag :state="row.state" :progress="row.progress" />
          </div>
          <div class="tv-card-id">{{ row.taskId.slice(0, 8) }} · {{ formatDate(row.createdAt) }}</div>
          <div class="tv-card-progress">
            <el-progress :percentage="row.progress" :status="getProgressStatus(row.state)" :stroke-width="7" :show-text="false" />
          </div>
          <div class="tv-card-actions">
            <el-button v-if="row.finalVideoPath" type="primary" size="small" round @click="playMedia(row.finalVideoPath)">
              <el-icon><VideoPlay /></el-icon>
              {{ $t('task.video') }}
            </el-button>
            <div class="tv-spacer" />
            <el-button size="small" circle type="danger" plain @click="deleteGeneric('/dh_tasks/delete', row.taskId, loadCurrent)">
              <el-icon><Delete /></el-icon>
            </el-button>
          </div>
        </div>
      </div>
    </template>

    <!-- 图片故事任务 -->
    <template v-if="taskType === 'image'">
      <el-empty v-if="imageTasks.length === 0" :description="$t('task.noTasks')" />
      <div v-else class="tv-grid">
        <div v-for="row in imageTasks" :key="row.taskId" class="tv-card">
          <div class="tv-card-top">
            <div class="tv-card-subject">{{ row.params?.story_subject || $t('task.untitled') }}</div>
            <TaskStatusTag :state="row.state" :progress="row.progress" />
          </div>
          <div class="tv-card-id">{{ row.taskId.slice(0, 8) }} · {{ formatDate(row.createdAt) }}</div>
          <div class="tv-card-progress">
            <el-progress :percentage="row.progress" :status="getProgressStatus(row.state)" :stroke-width="7" :show-text="false" />
          </div>
          <div v-if="row.errorMessage" class="tv-card-script">{{ row.errorMessage }}</div>
          <div class="tv-card-actions">
            <el-button v-if="row.videoPath" type="primary" size="small" round @click="playMedia(row.videoPath)">
              <el-icon><VideoPlay /></el-icon>
              {{ $t('task.video') }}
            </el-button>
            <div class="tv-spacer" />
            <el-button size="small" circle type="danger" plain @click="deleteGeneric('/image_story/delete', row.taskId, loadCurrent)">
              <el-icon><Delete /></el-icon>
            </el-button>
          </div>
        </div>
      </div>
    </template>

    <el-drawer v-model="detailVisible" :title="$t('task.detailTitle')" size="520px">
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
            style="width: 100%; margin-bottom: 8px; border-radius: 10px; background: #000;"
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
import { computed, onMounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Refresh, VideoPlay, VideoPause, Delete, View, Edit, Search } from '@element-plus/icons-vue'
import { useTaskStore } from '@/stores/task'
import { TaskStateCode, isTaskDraft, isTaskFailed, isTaskProcessing } from '@/types'
import type { TaskInfo } from '@/types'
import { getStaticUrl } from '@/api/stream'
import { stopTask } from '@/api/video'
import api, { extractData } from '@/api'
import TaskStatusTag from '@/components/TaskStatusTag.vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

// 任务中心：三类任务（视频 / 数字人 / 图片故事）页签化展示与轮询刷新
const taskStore = useTaskStore()
const router = useRouter()
const { t } = useI18n()
const detailVisible = ref(false)
const detailTask = ref<TaskInfo | null>(null)
const keyword = ref('')

// 任务类型页签：video / digital / image
const taskType = ref<'video' | 'digital' | 'image'>('video')
const digitalTasks = ref<any[]>([])
const imageTasks = ref<any[]>([])

onMounted(() => {
  taskStore.fetchTasks()
})

function loadCurrent() {
  if (taskType.value === 'video') taskStore.fetchTasks()
  else if (taskType.value === 'digital') loadDigital()
  else loadImage()
}

async function loadDigital() {
  try {
    const res = await api.post('/dh_tasks/list', { page: 1, pageSize: 100 }).then(extractData<any>)
    digitalTasks.value = res?.list || res || []
  } catch (e) {
    ElMessage.error((e as Error).message || '加载失败')
  }
}

async function loadImage() {
  try {
    const res = await api.post('/image_story/list', { page: 1, pageSize: 100 }).then(extractData<any>)
    imageTasks.value = res?.list || res || []
  } catch (e) {
    ElMessage.error((e as Error).message || '加载失败')
  }
}

function onTypeChange() {
  loadCurrent()
}

async function deleteGeneric(path: string, taskId: string, reload: () => void) {
  try {
    await ElMessageBox.confirm(t('task.deleteConfirm'), t('common.warning'), {
      confirmButtonText: t('common.ok'),
      cancelButtonText: t('common.cancel'),
      type: 'warning',
    })
    await api.post(path, { taskId })
    ElMessage.success(t('common.success'))
    reload()
  } catch {
    // cancelled
  }
}

function playMedia(path: string) {
  window.open(getStaticUrl(path), '_blank')
}

const filteredTasks = computed(() => {
  const k = keyword.value.trim().toLowerCase()
  if (!k) return taskStore.tasks
  return taskStore.tasks.filter(
    (t) =>
      (t.params?.video_subject || '').toLowerCase().includes(k) ||
      t.taskId.toLowerCase().includes(k)
  )
})

function onRefresh() {
  loadCurrent()
}

function getProgressStatus(state: number) {
  if (state === TaskStateCode.Completed) return 'success' as const
  if (state === TaskStateCode.Failed) return 'exception' as const
  return undefined
}

// Draft 和 Failed 状态都允许继续编辑/重试
function isTaskEditable(row: TaskInfo) {
  return isTaskDraft(row) || isTaskFailed(row)
}

function isTaskStuck(row: TaskInfo) {
  return isTaskProcessing(row)
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

async function onStopTask(row: TaskInfo) {
  try {
    await ElMessageBox.confirm(t('task.stopConfirm'), t('common.warning'), {
      confirmButtonText: t('common.ok'),
      cancelButtonText: t('common.cancel'),
      type: 'warning',
    })
    await stopTask(row.taskId)
    taskStore.fetchTasks()
    if (taskStore.currentTask?.taskId === row.taskId) {
      taskStore.stopPolling()
      taskStore.currentTask = null
      taskStore.isGenerating = false
    }
    ElMessage.success(t('common.success'))
  } catch {
    // cancelled or error
  }
}

function onContinueEdit(row: TaskInfo) {
  taskStore.resumeDraft(row.taskId)
  router.push({ name: 'home', query: { resume: row.taskId } })
}
</script>

<style scoped>
.tasks-view {
  max-width: 1200px;
  margin: 0 auto;
}

.tv-toolbar {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;

  .tv-search {
    max-width: 420px;
  }
}

.tv-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(330px, 1fr));
  gap: 16px;
}

.tv-card {
  border: 1px solid var(--soma-line);
  border-radius: var(--soma-radius);
  background: var(--soma-glass);
  backdrop-filter: blur(14px);
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  transition: all 0.2s ease;
}

.tv-card:hover {
  transform: translateY(-2px);
  border-color: rgba(91, 140, 255, 0.4);
  box-shadow: 0 14px 38px rgba(0, 0, 0, 0.38);
}

.tv-card-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.tv-card-subject {
  font-size: 15px;
  font-weight: 650;
  line-height: 1.4;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.tv-card-id {
  font-size: 11px;
  color: var(--soma-text-faint);
  font-family: 'JetBrains Mono', 'Menlo', monospace;
}

.tv-card-script {
  font-size: 12.5px;
  color: var(--soma-text-dim);
  line-height: 1.65;
  min-height: 34px;
  margin: 0;
}

.tv-card-progress {
  margin: 4px 0 2px;
}

.tv-card-progress-meta {
  display: flex;
  justify-content: space-between;
  font-size: 11.5px;
  color: var(--soma-text-faint);
  margin-top: 4px;

  .tv-aspect {
    font-family: 'JetBrains Mono', monospace;
  }
}

.tv-card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 6px;
  flex-wrap: wrap;

  .tv-spacer {
    flex: 1;
  }
}
</style>
