<template>
  <div class="toolbox-view">
    <!-- 工具卡片网格 -->
    <div class="tb-grid">
      <div v-for="t in tools" :key="t.id" class="tb-card" @click="openTool(t)">
        <div class="tb-card-icon" :style="{ background: t.chipBg, color: t.color }">
          <el-icon :size="22"><component :is="t.icon" /></el-icon>
        </div>
        <div class="tb-card-name">{{ $t(t.titleKey) }}</div>
        <div class="tb-card-desc">{{ $t(t.descKey) }}</div>
        <div class="tb-card-foot">
          <el-tag size="small" type="info" effect="plain" round>{{ t.module }}</el-tag>
          <el-button type="primary" size="small" round>{{ $t('toolbox.use') }}</el-button>
        </div>
      </div>
    </div>

    <!-- 工具运行对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="currentTool ? $t(currentTool.titleKey) : ''"
      width="620px"
      destroy-on-close
      append-to-body
    >
      <div v-if="currentTool" class="tb-run-desc">{{ $t(currentTool.descKey) }}</div>

      <el-form v-if="currentTool" label-position="top" class="tb-form">
        <el-form-item v-for="f in currentTool.fields" :key="f.name">
          <template #label>
            <span class="tb-field-label">
              {{ $t(f.labelKey) }}
              <code class="tb-field-key">{{ f.name }}</code>
            </span>
          </template>
          <el-switch v-if="f.type === 'switch'" v-model="formModel[f.name]" />
          <el-input-number
            v-else-if="f.type === 'number'"
            v-model="formModel[f.name]"
            :min="f.min ?? 1"
            :max="f.max"
            :step="f.step ?? 1"
            class="!w-full"
          />
          <el-select v-else-if="f.options" v-model="formModel[f.name]" class="!w-full">
            <el-option v-for="o in f.options" :key="o" :label="o" :value="o" />
          </el-select>
          <el-input
            v-else-if="f.type === 'textarea'"
            v-model="formModel[f.name]"
            type="textarea"
            :rows="3"
            :placeholder="f.placeholder"
          />
          <el-input v-else v-model="formModel[f.name]" :placeholder="f.placeholder" />
        </el-form-item>
      </el-form>

      <div class="tb-actions">
        <el-button
          v-for="a in currentTool?.actions"
          :key="a.key"
          :type="a.primary ? 'primary' : 'default'"
          :loading="running === a.key"
          @click="runAction(a)"
        >
          {{ $t(a.labelKey) }}
        </el-button>
      </div>

      <div v-if="wmPreview" class="tb-result-box">
        <div class="tb-result-title">{{ $t('toolbox.wmPreview') }}</div>
        <div class="tb-wm-frame">
          <img :src="wmPreview.frame" class="tb-wm-img" alt="frame" />
          <img v-if="wmPreview.showMask" :src="wmPreview.mask" class="tb-wm-mask" alt="mask" />
        </div>
        <div class="tb-wm-meta">
          <el-checkbox v-model="wmPreview.showMask">{{ $t('toolbox.wmMaskToggle') }}</el-checkbox>
          <span class="tb-wm-coverage">
            {{ $t('toolbox.wmCoverage') }}: {{ (wmPreview.coverage * 100).toFixed(1) }}%
          </span>
        </div>
      </div>

      <el-alert
        v-if="runError"
        type="error"
        :title="runError"
        show-icon
        :closable="false"
        class="tb-result"
      />

      <div v-if="resultJson" class="tb-result-box">
        <div class="tb-result-title">{{ $t('toolbox.result') }}</div>
        <pre class="tb-result-json">{{ resultJson }}</pre>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Bell, DataAnalysis, Document, Download, Film, Link, MagicStick } from '@element-plus/icons-vue'
import {
  convertSubtitle,
  createJianyingDraft,
  detectWatermark,
  douyinDetail,
  douyinPosts,
  downloadVideo,
  exportData,
  getYtdlpInfo,
  mergeSubtitles,
  previewData,
  removeWatermark,
  sendNotify,
  translateSubtitle,
} from '@/api/tools'

type FieldType = 'input' | 'textarea' | 'number' | 'switch' | 'select'

interface ToolField {
  name: string
  labelKey: string
  type: FieldType
  placeholder?: string
  options?: string[]
  defaultValue?: string | number | boolean
  min?: number
  max?: number
  step?: number
}

interface ToolAction {
  key: string
  labelKey: string
  primary?: boolean
  handler: (form: Record<string, string | number | boolean>) => Promise<unknown>
}

interface Tool {
  id: string
  module: string
  icon: unknown
  color: string
  chipBg: string
  titleKey: string
  descKey: string
  fields: ToolField[]
  actions: ToolAction[]
}

const str = (form: Record<string, string | number | boolean>, key: string): string =>
  String(form[key] ?? '')
const num = (form: Record<string, string | number | boolean>, key: string, fallback: number): number => {
  const v = form[key]
  return typeof v === 'number' ? v : fallback
}
const bool = (form: Record<string, string | number | boolean>, key: string): boolean => form[key] === true
const lines = (s: string): string[] =>
  s
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean)

const tools: Tool[] = [
  {
    id: 'download',
    module: 'yt-dlp',
    icon: Download,
    color: '#5b8cff',
    chipBg: 'rgba(91, 140, 255, 0.16)',
    titleKey: 'toolbox.tools.download.title',
    descKey: 'toolbox.tools.download.desc',
    fields: [
      { name: 'url', labelKey: 'toolbox.f.url', type: 'input', placeholder: 'https://...' },
      { name: 'saveDir', labelKey: 'toolbox.f.saveDir', type: 'input' },
      { name: 'audioOnly', labelKey: 'toolbox.f.audioOnly', type: 'switch' },
      { name: 'proxy', labelKey: 'toolbox.f.proxy', type: 'input', placeholder: 'http://127.0.0.1:7890' },
    ],
    actions: [
      {
        key: 'info',
        labelKey: 'toolbox.a.info',
        handler: (f) => getYtdlpInfo(str(f, 'url'), str(f, 'proxy')),
      },
      {
        key: 'download',
        labelKey: 'toolbox.a.download',
        primary: true,
        handler: (f) =>
          downloadVideo({
            url: str(f, 'url'),
            saveDir: str(f, 'saveDir'),
            audioOnly: bool(f, 'audioOnly'),
            proxy: str(f, 'proxy'),
          }),
      },
    ],
  },
  {
    id: 'douyin',
    module: 'douyin',
    icon: Link,
    color: '#22d3ee',
    chipBg: 'rgba(34, 211, 238, 0.16)',
    titleKey: 'toolbox.tools.douyin.title',
    descKey: 'toolbox.tools.douyin.desc',
    fields: [
      { name: 'awemeId', labelKey: 'toolbox.f.awemeId', type: 'input', placeholder: 'https://www.douyin.com/video/...' },
      { name: 'secUserId', labelKey: 'toolbox.f.secUserId', type: 'input', placeholder: 'https://www.douyin.com/user/MS4w...' },
      { name: 'count', labelKey: 'toolbox.f.count', type: 'number', defaultValue: 20 },
      { name: 'cookie', labelKey: 'toolbox.f.cookie', type: 'textarea', placeholder: 'ttwid=...' },
    ],
    actions: [
      {
        key: 'detail',
        labelKey: 'toolbox.a.detail',
        handler: (f) => douyinDetail(str(f, 'awemeId'), str(f, 'cookie')),
      },
      {
        key: 'posts',
        labelKey: 'toolbox.a.posts',
        primary: true,
        handler: (f) => douyinPosts(str(f, 'secUserId'), num(f, 'count', 20), 0, str(f, 'cookie')),
      },
    ],
  },
  {
    id: 'subtitle',
    module: 'subtitle',
    icon: Document,
    color: '#34d399',
    chipBg: 'rgba(52, 211, 153, 0.16)',
    titleKey: 'toolbox.tools.subtitle.title',
    descKey: 'toolbox.tools.subtitle.desc',
    fields: [
      { name: 'inputPath', labelKey: 'toolbox.f.inputPath', type: 'input', placeholder: 'storage/.../*.srt' },
      { name: 'paths', labelKey: 'toolbox.f.paths', type: 'textarea', placeholder: 'one path per line' },
      { name: 'outputPath', labelKey: 'toolbox.f.outputPath', type: 'input' },
      { name: 'targetLang', labelKey: 'toolbox.f.targetLang', type: 'input', placeholder: 'en' },
    ],
    actions: [
      {
        key: 'convert',
        labelKey: 'toolbox.a.convert',
        handler: (f) => convertSubtitle(str(f, 'inputPath'), str(f, 'outputPath')),
      },
      {
        key: 'merge',
        labelKey: 'toolbox.a.merge',
        handler: (f) => mergeSubtitles(lines(str(f, 'paths')), str(f, 'outputPath')),
      },
      {
        key: 'translate',
        labelKey: 'toolbox.a.translate',
        primary: true,
        handler: (f) => translateSubtitle(str(f, 'inputPath'), str(f, 'outputPath'), str(f, 'targetLang') || 'en'),
      },
    ],
  },
  {
    id: 'jianying',
    module: 'jianying',
    icon: Film,
    color: '#9b6bff',
    chipBg: 'rgba(155, 107, 255, 0.16)',
    titleKey: 'toolbox.tools.jianying.title',
    descKey: 'toolbox.tools.jianying.desc',
    fields: [
      { name: 'name', labelKey: 'toolbox.f.name', type: 'input', defaultValue: 'soma_draft' },
      { name: 'videos', labelKey: 'toolbox.f.videos', type: 'textarea', placeholder: 'one video path per line' },
      { name: 'audio', labelKey: 'toolbox.f.audio', type: 'input' },
      { name: 'outputDir', labelKey: 'toolbox.f.outputDir', type: 'input', placeholder: 'storage/jianying/' },
      { name: 'width', labelKey: 'toolbox.f.width', type: 'number', defaultValue: 1920 },
      { name: 'height', labelKey: 'toolbox.f.height', type: 'number', defaultValue: 1080 },
    ],
    actions: [
      {
        key: 'create',
        labelKey: 'toolbox.a.create',
        primary: true,
        handler: (f) =>
          createJianyingDraft({
            name: str(f, 'name') || 'soma_draft',
            videos: lines(str(f, 'videos')),
            audio: str(f, 'audio') || undefined,
            width: num(f, 'width', 1920),
            height: num(f, 'height', 1080),
            outputDir: str(f, 'outputDir'),
          }),
      },
    ],
  },
  {
    id: 'notify',
    module: 'notify',
    icon: Bell,
    color: '#fbbf24',
    chipBg: 'rgba(251, 191, 36, 0.16)',
    titleKey: 'toolbox.tools.notify.title',
    descKey: 'toolbox.tools.notify.desc',
    fields: [
      { name: 'channel', labelKey: 'toolbox.f.channel', type: 'select', options: ['bark', 'dingtalk', 'telegram'], defaultValue: 'bark' },
      { name: 'webhook', labelKey: 'toolbox.f.webhook', type: 'input' },
      { name: 'secret', labelKey: 'toolbox.f.secret', type: 'input' },
      { name: 'title', labelKey: 'toolbox.f.title', type: 'input' },
      { name: 'body', labelKey: 'toolbox.f.body', type: 'textarea' },
    ],
    actions: [
      {
        key: 'send',
        labelKey: 'toolbox.a.send',
        primary: true,
        handler: (f) =>
          sendNotify({
            channel: str(f, 'channel'),
            webhook: str(f, 'webhook'),
            title: str(f, 'title'),
            body: str(f, 'body'),
            secret: str(f, 'secret') || undefined,
          }),
      },
    ],
  },
  {
    id: 'dataexport',
    module: 'dataexport',
    icon: DataAnalysis,
    color: '#f87171',
    chipBg: 'rgba(248, 113, 113, 0.16)',
    titleKey: 'toolbox.tools.dataexport.title',
    descKey: 'toolbox.tools.dataexport.desc',
    fields: [
      { name: 'data', labelKey: 'toolbox.f.data', type: 'textarea', placeholder: '[{"k": "v"}]' },
      { name: 'inputPath', labelKey: 'toolbox.f.inputPath', type: 'input', placeholder: 'input.json / input.jsonl' },
      { name: 'outputPath', labelKey: 'toolbox.f.outputPath', type: 'input', placeholder: 'output.csv' },
      { name: 'format', labelKey: 'toolbox.f.format', type: 'select', options: ['auto', 'csv', 'json', 'jsonl'], defaultValue: 'auto' },
    ],
    actions: [
      {
        key: 'preview',
        labelKey: 'toolbox.a.preview',
        handler: (f) => {
          const data = parseJsonArray(str(f, 'data'))
          return previewData(data)
        },
      },
      {
        key: 'export',
        labelKey: 'toolbox.a.export',
        primary: true,
        handler: (f) =>
          exportData({
            outputPath: str(f, 'outputPath'),
            data: str(f, 'data') ? parseJsonArray(str(f, 'data')) : undefined,
            inputPath: str(f, 'inputPath') || undefined,
            format: str(f, 'format') === 'auto' ? undefined : str(f, 'format'),
          }),
      },
    ],
  },
  {
    id: 'watermark',
    module: 'watermark',
    icon: MagicStick,
    color: '#f472b6',
    chipBg: 'rgba(244, 114, 182, 0.16)',
    titleKey: 'toolbox.tools.watermark.title',
    descKey: 'toolbox.tools.watermark.desc',
    fields: [
      { name: 'videoPath', labelKey: 'toolbox.f.videoPath', type: 'input', placeholder: 'storage/tasks/.../final.mp4' },
      { name: 'keyframes', labelKey: 'toolbox.f.keyframes', type: 'number', defaultValue: 30, min: 2, max: 120 },
      { name: 'gradThreshold', labelKey: 'toolbox.f.gradThreshold', type: 'number', defaultValue: 8, min: 0.5, max: 120, step: 0.5 },
      { name: 'maskThreshold', labelKey: 'toolbox.f.maskThreshold', type: 'number', defaultValue: 0.2, min: 0.01, max: 0.99, step: 0.05 },
      { name: 'blurSigma', labelKey: 'toolbox.f.blurSigma', type: 'number', defaultValue: 2, min: 0.5, max: 10, step: 0.5 },
      { name: 'maskPath', labelKey: 'toolbox.f.maskPath', type: 'input' },
      { name: 'outputDir', labelKey: 'toolbox.f.outputDir', type: 'input', placeholder: 'storage/watermark/' },
      { name: 'codec', labelKey: 'toolbox.f.codec', type: 'select', options: ['libx264', 'libx265'], defaultValue: 'libx264' },
      { name: 'crf', labelKey: 'toolbox.f.crf', type: 'number', defaultValue: 18, min: 0, max: 51 },
    ],
    actions: [
      {
        key: 'detect',
        labelKey: 'toolbox.a.detect',
        handler: (f) =>
          detectWatermark({
            videoPath: str(f, 'videoPath'),
            keyframes: num(f, 'keyframes', 30),
            gradThreshold: num(f, 'gradThreshold', 8),
            maskThreshold: num(f, 'maskThreshold', 0.2),
            blurSigma: num(f, 'blurSigma', 2),
          }),
      },
      {
        key: 'remove',
        labelKey: 'toolbox.a.remove',
        primary: true,
        handler: (f) =>
          removeWatermark({
            videoPath: str(f, 'videoPath'),
            maskPath: str(f, 'maskPath') || undefined,
            outputDir: str(f, 'outputDir') || undefined,
            codec: str(f, 'codec') || 'libx264',
            crf: num(f, 'crf', 18),
          }),
      },
    ],
  },
]

function parseJsonArray(text: string): unknown[] {
  const parsed = JSON.parse(text)
  if (!Array.isArray(parsed)) {
    throw new Error('data must be a JSON array')
  }
  return parsed
}

const dialogVisible = ref(false)
const currentTool = ref<Tool | null>(null)
const formModel = ref<Record<string, string | number | boolean>>({})
const running = ref('')
const runError = ref('')
const resultJson = ref('')
// 去水印检测预览（采样帧 + 可叠加蒙版）
const wmPreview = ref<{ frame: string; mask: string; coverage: number; showMask: boolean } | null>(
  null,
)

function openTool(tool: Tool) {
  currentTool.value = tool
  formModel.value = {}
  for (const f of tool.fields) {
    formModel.value[f.name] = f.defaultValue ?? (f.type === 'switch' ? false : f.type === 'number' ? 1 : '')
  }
  runError.value = ''
  resultJson.value = ''
  running.value = ''
  wmPreview.value = null
  dialogVisible.value = true
}

async function runAction(action: ToolAction) {
  running.value = action.key
  runError.value = ''
  resultJson.value = ''
  wmPreview.value = null
  try {
    const result = await action.handler(formModel.value)
    resultJson.value = JSON.stringify(result, null, 2)
    // 去水印 detect 结果：展示蒙版预览并自动回填蒙版路径
    const r = result as Record<string, unknown> | null
    if (r && typeof r === 'object' && typeof r.framePreview === 'string' && typeof r.maskPreview === 'string') {
      wmPreview.value = {
        frame: r.framePreview,
        mask: r.maskPreview,
        coverage: Number(r.coverage ?? 0),
        showMask: true,
      }
      if (typeof r.maskPath === 'string' && r.maskPath && currentTool.value?.id === 'watermark') {
        formModel.value.maskPath = r.maskPath
      }
    }
  } catch (e) {
    runError.value = e instanceof Error ? e.message : String(e)
  } finally {
    running.value = ''
  }
}
</script>

<style scoped>
.toolbox-view {
  padding: 20px;
}

.tb-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(268px, 1fr));
  gap: 16px;
}

.tb-card {
  border: 1px solid var(--soma-line);
  border-radius: var(--soma-radius);
  background: var(--soma-glass);
  backdrop-filter: blur(14px);
  padding: 20px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tb-card:hover {
  transform: translateY(-3px);
  border-color: rgba(91, 140, 255, 0.42);
  box-shadow: 0 14px 38px rgba(0, 0, 0, 0.4);
}

.tb-card-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: grid;
  place-items: center;
  margin-bottom: 4px;
}

.tb-card-name {
  font-size: 15.5px;
  font-weight: 650;
}

.tb-card-desc {
  font-size: 12.5px;
  color: var(--soma-text-dim);
  line-height: 1.5;
  min-height: 38px;
}

.tb-card-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 4px;
}

.tb-run-desc {
  font-size: 12.5px;
  color: var(--soma-text-dim);
  margin-bottom: 14px;
}

.tb-field-label {
  font-size: 13px;
  font-weight: 600;
}

.tb-field-key {
  font-family: 'JetBrains Mono', 'Menlo', monospace;
  font-size: 10.5px;
  color: var(--soma-text-faint);
  margin-left: 6px;
}

.tb-actions {
  display: flex;
  gap: 10px;
  margin-top: 6px;
}

.tb-result {
  margin-top: 14px;
}

.tb-result-box {
  margin-top: 14px;
}

.tb-result-title {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--soma-text-dim);
  margin-bottom: 8px;
}

.tb-result-json {
  background: rgba(5, 8, 18, 0.72);
  border: 1px solid var(--soma-line);
  border-radius: var(--soma-radius-sm);
  color: #c9d6f2;
  font-family: 'JetBrains Mono', 'Menlo', monospace;
  font-size: 12px;
  line-height: 1.7;
  padding: 14px;
  max-height: 300px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.tb-wm-frame {
  position: relative;
  border: 1px solid var(--soma-line);
  border-radius: var(--soma-radius-sm);
  overflow: hidden;
}

.tb-wm-img {
  display: block;
  width: 100%;
}

.tb-wm-mask {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0.45;
  mix-blend-mode: screen;
  pointer-events: none;
}

.tb-wm-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 8px;
}

.tb-wm-coverage {
  font-size: 12px;
  color: var(--soma-text-dim);
  font-family: 'JetBrains Mono', 'Menlo', monospace;
}
</style>
