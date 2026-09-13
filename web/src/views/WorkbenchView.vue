<template>
  <div class="workbench-view">
    <!-- 顶部：说明 + 搜索 -->
    <div class="wb-toolbar">
      <el-input
        v-model="keyword"
        :placeholder="$t('workbench.search')"
        :prefix-icon="Search"
        clearable
        size="large"
        class="wb-search"
      />
      <el-tooltip :content="$t('workbench.refresh')" placement="top">
        <el-button :icon="Refresh" size="large" circle @click="loadFeatures" />
      </el-tooltip>
    </div>

    <!-- 功能点分组卡片 -->
    <div v-for="group in groups" :key="group.kind" class="wb-group">
      <div class="wb-group-head">
        <span class="wb-group-dot" :style="{ background: group.meta.color }" />
        <span class="wb-group-title">{{ $t(group.meta.labelKey) }}</span>
        <span class="wb-group-count">{{ group.features.length }}</span>
      </div>
      <div class="wb-grid">
        <div
          v-for="f in group.features"
          :key="f.id"
          class="wb-card"
          @click="openRun(f)"
        >
          <div class="wb-card-icon" :style="{ background: group.meta.chipBg, color: group.meta.color }">
            <el-icon :size="22"><component :is="group.meta.icon" /></el-icon>
          </div>
          <div class="wb-card-name">{{ f.name }}</div>
          <el-tooltip :content="f.id" placement="top" :show-after="500">
            <div class="wb-card-id">{{ f.id }}</div>
          </el-tooltip>
          <div class="wb-card-desc">{{ f.description }}</div>
          <div class="wb-card-foot">
            <el-tag size="small" type="info" effect="plain" round>
              {{ $t('workbench.schemaReady') }}
            </el-tag>
            <el-button type="primary" size="small" round>
              {{ $t('workbench.use') }}
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <el-empty v-if="!loading && groups.length === 0" :description="loadError || $t('workbench.noFeatures')">
      <template v-if="loadError">
        <div class="wb-load-error">
          <p>isTauri: {{ isTauriEnv }}</p>
          <p>{{ loadError }}</p>
        </div>
      </template>
    </el-empty>

    <!-- 运行对话框 -->
    <el-dialog
      v-model="runVisible"
      :title="current ? current.name : ''"
      width="640px"
      destroy-on-close
      append-to-body
    >
      <div class="wb-run-id">
        <span>{{ current?.description }}</span>
        <el-tooltip :content="current?.id" placement="top">
          <el-tag size="small" type="info" effect="plain" class="wb-run-id-tag">{{ current?.id }}</el-tag>
        </el-tooltip>
      </div>

      <div v-if="formFields.length" class="wb-form">
        <div class="section-title">{{ $t('workbench.inputParams') }}</div>
        <el-form label-position="top" size="large">
          <el-form-item
            v-for="field in formFields"
            :key="field.name"
            :required="field.required"
          >
            <template #label>
              <span class="wb-field-label">
                {{ field.description || field.name }}
                <code v-if="field.description && field.description !== field.name" class="wb-field-key">{{ field.name }}</code>
                <code class="wb-field-type">{{ field.typeLabel }}</code>
              </span>
            </template>

            <el-switch v-if="field.type === 'boolean'" v-model="formModel[field.name]" />
            <el-select
              v-else-if="field.options"
              v-model="formModel[field.name]"
              filterable
              allow-create
              default-first-option
              :placeholder="$t('workbench.selectHint')"
              class="!w-full"
            >
              <el-option v-for="o in field.options" :key="o" :label="getOptionLabel(o)" :value="o" />
            </el-select>
            <el-input
              v-else-if="field.type === 'object'"
              v-model="formModel[field.name]"
              type="textarea"
              :rows="6"
              class="wb-json-editor"
              :placeholder="$t('workbench.jsonHint')"
            />
            <el-input-number
              v-else-if="field.type === 'number' || field.type === 'integer'"
              v-model="formModel[field.name]"
              :step="field.type === 'integer' ? 1 : 0.1"
              class="!w-full"
            />
            <el-input
              v-else-if="field.type === 'array' && field.itemIsObject"
              v-model="formModel[field.name]"
              type="textarea"
              :rows="6"
              class="wb-json-editor"
              :placeholder="$t('workbench.jsonArrayHint')"
            />
            <el-select
              v-else-if="field.type === 'array'"
              v-model="formModel[field.name]"
              multiple
              filterable
              allow-create
              default-first-option
              :placeholder="$t('workbench.arrayHint')"
              class="!w-full"
            />
            <el-input
              v-else-if="field.longText"
              v-model="formModel[field.name]"
              type="textarea"
              :rows="4"
            />
            <el-input
              v-else
              v-model="formModel[field.name]"
              :placeholder="field.description || field.name"
            />
          </el-form-item>
        </el-form>
      </div>
      <el-empty v-else :description="$t('workbench.noParams')" :image-size="72" />

      <template #footer>
        <el-button @click="openHistory(current!)">{{ $t('workbench.history') }}</el-button>
        <el-button type="primary" :loading="running" @click="runFeature">
          {{ running ? $t('workbench.running') : $t('workbench.run') }}
        </el-button>
      </template>
    </el-dialog>

    <!-- 运行结果对话框 -->
    <el-dialog v-model="resultVisible" :title="$t('workbench.result')" width="680px" append-to-body>
      <template v-if="result">
        <el-alert
          v-if="resultOk"
          :title="$t('workbench.runSuccess')"
          type="success"
          show-icon
          :closable="false"
          class="!mb-4"
        />
        <el-alert
          v-else
          :title="$t('workbench.runFailed')"
          :description="result.error || ''"
          type="error"
          show-icon
          :closable="false"
          class="!mb-4"
        />

        <div class="wb-meta-row">
          <el-tooltip :content="result.feature_id" placement="top">
            <el-tag effect="plain" round size="small" class="wb-result-id">{{ result.feature_id }}</el-tag>
          </el-tooltip>
          <el-tag effect="plain" round type="info" size="small">{{ result.run_id.slice(0, 8) }}</el-tag>
          <el-tag effect="plain" round type="info" size="small">{{ result.duration_ms }} ms</el-tag>
        </div>

        <template v-if="result.artifacts && result.artifacts.length">
          <div class="section-title !mt-5">{{ $t('workbench.artifacts') }}</div>
          <div v-for="a in result.artifacts" :key="a.path" class="wb-artifact">
            <el-icon :size="18"><Document /></el-icon>
            <div class="wb-artifact-info">
              <div class="wb-artifact-name">{{ a.name }}</div>
              <div class="wb-artifact-path">{{ a.path }}</div>
            </div>
            <span class="wb-artifact-size">{{ formatSize(a.size) }}</span>
            <el-button size="small" text @click="copyText(a.path)">
              {{ $t('workbench.copyPath') }}
            </el-button>
          </div>
        </template>

        <div v-if="result.data && result.data !== null" class="section-title !mt-5">
          {{ $t('workbench.outputData') }}
        </div>
        <pre v-if="result.data && result.data !== null" class="wb-json">{{ prettyJson(result.data) }}</pre>
      </template>
      <template #footer>
        <el-button @click="resultVisible = false">{{ $t('common.close') }}</el-button>
        <el-button v-if="resultOk && current" type="primary" @click="openHistory(current)">
          {{ $t('workbench.viewHistory') }}
        </el-button>
      </template>
    </el-dialog>

    <!-- 历史记录抽屉 -->
    <el-drawer v-model="historyVisible" :title="$t('workbench.history')" size="560px">
      <div v-if="historyLoading" class="wb-history-loading">
        <el-skeleton :rows="5" animated />
      </div>
      <el-empty v-else-if="history.length === 0" :description="$t('workbench.noHistory')" />
      <div v-else class="wb-history-list">
        <div v-for="r in history" :key="r.run_id" class="wb-history-item">
          <div class="wb-history-head">
            <el-tag size="small" :type="r.status === 'success' ? 'success' : 'danger'" effect="light" round>
              {{ r.status }}
            </el-tag>
            <span class="wb-history-time">{{ formatTime(r.finished_at) }}</span>
            <span class="wb-history-duration">{{ r.duration_ms }} ms</span>
          </div>
          <pre class="wb-json wb-json-small">{{ prettyJson(r.data) }}</pre>
          <div v-for="a in r.artifacts" :key="a.path" class="wb-history-artifact">
            <el-icon><Document /></el-icon>
            <span class="wb-artifact-name">{{ a.name }}</span>
            <span class="wb-artifact-size">{{ formatSize(a.size) }}</span>
            <el-button size="small" text @click="copyText(a.path)">
              {{ $t('workbench.copyPath') }}
            </el-button>
          </div>
        </div>
      </div>
    </el-drawer>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { Search, Refresh, Document } from '@element-plus/icons-vue'
import api, { extractData } from '@/api'

/** 功能点描述（与后端 FeatureDescriptor 对应） */
interface FeatureDescriptor {
  id: string
  name: string
  description: string
  kind: string
  input_schema?: Record<string, unknown> | null
  output_schema?: Record<string, unknown> | null
}

/** 功能点运行结果（与后端 FeatureOutput 对应） */
interface FeatureOutput {
  feature_id: string
  run_id: string
  status: 'success' | 'failed'
  data: unknown
  artifacts: Array<{ name: string; path: string; kind: string; size: number }>
  error?: string
  input: unknown
  started_at: string
  finished_at: string
  duration_ms: number
}

interface KindMeta {
  icon: unknown
  color: string
  chipBg: string
  labelKey: string
}

// ── 共享选项常量（消除重复数组） ──
const SOURCES_STOCK = ['pexels', 'pixabay', 'coverr'] as const
const SOURCES_AI = ['cogvideox', 'kling', 'minimax'] as const
const SOURCES_ALL = [...SOURCES_STOCK, 'local', ...SOURCES_AI] as const
const LANGUAGES = ['zh-CN', 'en-US', 'ja-JP'] as const
const TRANSITIONS = ['FadeIn', 'FadeOut', 'None', 'Shuffle', 'SlideIn', 'SlideOut'] as const
const PLATFORMS = ['douyin', 'xiaohongshu', 'tiktok', 'youtube_shorts', 'instagram_reels', 'x'] as const
const VOICES = ['zh-CN-XiaoxiaoNeural', 'zh-CN-YunxiNeural', 'zh-CN-YunjianNeural', 'en-US-JennyNeural', 'en-US-GuyNeural', 'no-voice'] as const
const TTS_PROVIDERS = ['edge', 'azure', 'siliconflow', 'elevenlabs', 'mimo', 'gemini', 'volcengine', 'xfyun'] as const
const ASPECT_RATIOS = ['9:16', '16:9', '1:1'] as const

/** 可枚举字段：优先下拉选择，仍可手动输入其它值（allow-create） */
const FIELD_OPTIONS: Record<string, Record<string, string[]>> = {
  'material.search': { source: [...SOURCES_STOCK] },
  'material.generate': { source: [...SOURCES_ALL] },
  'aivideo.generate': { source: [...SOURCES_AI] },
  'video.transition': { transition: ['FadeIn', 'FadeOut'] },
  'video.concat': { transition_mode: [...TRANSITIONS] },
  'video.compose': { 'params.bgm_type': ['none', 'random'] },
  'llm.intent': { aspect_ratio: [...ASPECT_RATIOS] },
  'llm.script': { language: [...LANGUAGES] },
  'llm.storyboard': { language: [...LANGUAGES] },
  'llm.narration': { language: [...LANGUAGES] },
  'llm.social': { platform: [...PLATFORMS] },
  'tts.synthesize': { voice_name: [...VOICES], tts_provider: [...TTS_PROVIDERS] },
  'subtitle.generate': {},
}

/** 选项值 → 中文显示标签 */
const OPTION_LABELS: Record<string, string> = {
  // 素材来源
  pexels: 'Pexels (免费图库)',
  pixabay: 'Pixabay (免费图库)',
  coverr: 'Coverr (免费视频)',
  local: '本地上传',
  // AI 视频供应商
  cogvideox: '智谱 CogVideoX',
  kling: '快影 Kling',
  minimax: 'MiniMax',
  // 转场
  FadeIn: '淡入',
  FadeOut: '淡出',
  None: '无',
  Shuffle: '随机',
  SlideIn: '滑入',
  SlideOut: '滑出',
  // 画幅比例
  '9:16': '9:16 (竖屏)',
  '16:9': '16:9 (横屏)',
  '1:1': '1:1 (方形)',
  // 语言
  'zh-CN': '中文',
  'en-US': '英文',
  'ja-JP': '日文',
  // 平台
  douyin: '抖音',
  xiaohongshu: '小红书',
  tiktok: 'TikTok',
  youtube_shorts: 'YouTube Shorts',
  instagram_reels: 'Instagram Reels',
  x: 'X (Twitter)',
  // 语音
  'zh-CN-XiaoxiaoNeural': '晓晓 (女声, 中文)',
  'zh-CN-YunxiNeural': '云希 (男声, 中文)',
  'zh-CN-YunjianNeural': '云健 (男声, 中文)',
  'en-US-JennyNeural': 'Jenny (女声, 英文)',
  'en-US-GuyNeural': 'Guy (男声, 英文)',
  'no-voice': '无语音',
  // TTS 供应商
  edge: 'Edge TTS (免费)',
  azure: 'Azure Speech',
  siliconflow: 'SiliconFlow',
  elevenlabs: 'ElevenLabs',
  mimo: 'Mimo TTS',
  gemini: 'Gemini TTS',
  volcengine: '火山引擎',
  xfyun: '讯飞语音',
  // BGM
  none: '无背景音乐',
  random: '随机背景音乐',
}

/** 根据选项值获取中文显示标签 */
function getOptionLabel(value: string): string {
  return OPTION_LABELS[value] || value
}

/** 常用默认值：打开表单即预填，减少手动输入 */
const FIELD_DEFAULTS: Record<string, Record<string, unknown>> = {
  'llm.intent': { aspect_ratio: '9:16' },
  'llm.script': { language: 'zh-CN', paragraph_number: 1 },
  'llm.storyboard': { clip_duration: 4 },
  'llm.terms': { amount: 5 },
  'llm.social': { platform: 'douyin' },
  'tts.synthesize': { voice_name: 'zh-CN-XiaoxiaoNeural', voice_rate: 1, tts_provider: 'edge' },
  'subtitle.generate': { subtitle_enabled: true },
  'material.generate': { source: 'pexels', aspect: '9:16', clip_duration: 4 },
  'material.search': { source: 'pexels' },
  'aivideo.generate': { source: 'cogvideox', aspect: '9:16', clip_duration: 4 },
  'video.concat': { aspect: '9:16', clip_duration: 4, transition_mode: 'FadeIn' },
  'video.transition': { transition: 'FadeIn', duration: 1 },
  'video.clip_resize': { aspect: '9:16' },
  'video.render': {
    subtitle_path: '',
    params: { video_subject: '', subtitle_enabled: true, bgm_type: 'none', video_count: 1 },
  },
  'video.compose': {
    subtitle_path: '',
    params: { video_subject: '', subtitle_enabled: true, bgm_type: 'none', video_count: 1 },
  },
  'digitalhuman.compose': {
    subtitle_path: '',
    params: { video_subject: '', subtitle_enabled: true, bgm_type: 'none', video_count: 1 },
  },
}

const KIND_META: Record<string, KindMeta> = {
  llm: { icon: 'MagicStick', color: '#7ba3ff', chipBg: 'rgba(91,140,255,.14)', labelKey: 'workbench.kind.llm' },
  tts: { icon: 'Microphone', color: '#5eead4', chipBg: 'rgba(94,234,212,.12)', labelKey: 'workbench.kind.tts' },
  audio: { icon: 'Headset', color: '#38bdf8', chipBg: 'rgba(56,189,248,.12)', labelKey: 'workbench.kind.audio' },
  subtitle: { icon: 'ChatDotRound', color: '#fbbf24', chipBg: 'rgba(251,191,36,.12)', labelKey: 'workbench.kind.subtitle' },
  material: { icon: 'PictureFilled', color: '#f472b6', chipBg: 'rgba(244,114,182,.12)', labelKey: 'workbench.kind.material' },
  video_compose: { icon: 'Film', color: '#a78bfa', chipBg: 'rgba(167,139,250,.12)', labelKey: 'workbench.kind.videoCompose' },
  video_tool: { icon: 'Scissor', color: '#fb923c', chipBg: 'rgba(251,146,60,.12)', labelKey: 'workbench.kind.videoTool' },
  digital_human: { icon: 'User', color: '#34d399', chipBg: 'rgba(52,211,153,.12)', labelKey: 'workbench.kind.digitalHuman' },
  utility: { icon: 'Tools', color: '#94a3b8', chipBg: 'rgba(148,163,184,.12)', labelKey: 'workbench.kind.utility' },
}

interface FormField {
  name: string
  type: 'string' | 'number' | 'integer' | 'boolean' | 'array' | 'object' | 'unknown'
  typeLabel: string
  description: string
  required: boolean
  longText: boolean
  /** 可枚举字段：渲染为下拉（可搜索、可自填） */
  options?: string[]
  /** 数组元素是否为对象（如 MaterialInfo[]）：用 JSON 编辑器而非多选框 */
  itemIsObject?: boolean
}

const features = ref<FeatureDescriptor[]>([])
const loading = ref(false)
const keyword = ref('')
const loadError = ref('')
const isTauriEnv = typeof window !== 'undefined' && '__TAURI__' in window

const runVisible = ref(false)
const running = ref(false)
const current = ref<FeatureDescriptor | null>(null)
const formModel = ref<Record<string, unknown>>({})
const resultVisible = ref(false)
const result = ref<FeatureOutput | null>(null)

const historyVisible = ref(false)
const historyLoading = ref(false)
const history = ref<FeatureOutput[]>([])

onMounted(loadFeatures)

async function loadFeatures() {
  loading.value = true
  loadError.value = ''
  try {
    features.value = (await api.post('/features/list', {}).then(extractData<FeatureDescriptor[]>)) || []
  } catch (e) {
    loadError.value = (e as Error).message || String(e)
    console.error('[workbench] loadFeatures failed:', e)
    ElMessage.error(loadError.value)
  } finally {
    loading.value = false
  }
}

const filtered = computed(() =>
  features.value.filter((f) => {
    const k = keyword.value.trim().toLowerCase()
    if (!k) return true
    return (
      f.id.toLowerCase().includes(k) ||
      f.name.toLowerCase().includes(k) ||
      f.description.toLowerCase().includes(k)
    )
  })
)

const groups = computed(() => {
  const map = new Map<string, { kind: string; meta: KindMeta; features: FeatureDescriptor[] }>()
  for (const f of filtered.value) {
    const kind = KIND_META[f.kind] ? f.kind : 'utility'
    if (!map.has(kind)) {
      map.set(kind, { kind, meta: KIND_META[kind], features: [] })
    }
    map.get(kind)!.features.push(f)
  }
  return Array.from(map.values())
})

// ── Schema 驱动表单 ──
const formFields = computed<FormField[]>(() => {
  const schema = current.value?.input_schema as
    | { properties?: Record<string, { type?: string | string[]; description?: string; enum?: string[] }> }
    | null
  if (!schema?.properties) return []
  const required: string[] =
    ((current.value?.input_schema as { required?: string[] }).required as string[]) || []
  const fid = current.value?.id || ''
  const defs = (current.value?.input_schema as { $defs?: Record<string, { type?: string; enum?: string[] }> })?.$defs || {}
  const resolve = (node: { type?: string | string[]; $ref?: string; enum?: string[] }) => {
    // schemars 1.x 对命名结构体生成 $ref 引用，需解引用后读取真实类型
    if (node.$ref) return defs[node.$ref.split('/').pop() || ''] || {}
    return node
  }
  return Object.entries(schema.properties).map(([name, prop]) => {
    const real = resolve(prop as { $ref?: string })
    let type = Array.isArray(real.type) ? real.type[0] : (real.type as string) || 'string'
    if (type === 'array') type = 'array'
    if (type !== 'string' && type !== 'number' && type !== 'integer' && type !== 'boolean' && type !== 'array' && type !== 'object') {
      type = 'unknown'
    }
    // 选项来源：字段选项表 > Schema enum
    const options = FIELD_OPTIONS[fid]?.[name] ?? (real.enum as string[] | undefined)
    // 数组元素是否为对象：items 为 $ref 或 object 类型
    const items = (prop as { items?: { $ref?: string; type?: string } }).items
    const itemIsObject = type === 'array' && (!!items?.$ref || resolve(items as { $ref?: string }).type === 'object')
    return {
      name,
      type: type as FormField['type'],
      typeLabel: type,
      description: prop.description || '',
      required: required.includes(name),
      longText: type === 'string' && !options && /text|prompt|script|narration|描述|文案|文本/i.test(name + (prop.description || '')),
      options: type === 'string' ? options : undefined,
      itemIsObject,
    }
  })
})

function openRun(f: FeatureDescriptor) {
  current.value = f
  formModel.value = {}
  const defaults = FIELD_DEFAULTS[f.id] || {}
  for (const field of formFields.value) {
    const d = defaults[field.name]
    if (field.type === 'array') {
      formModel.value[field.name] = field.itemIsObject
        ? JSON.stringify(Array.isArray(d) ? d : [], null, 2)
        : Array.isArray(d) ? [...d] : []
    } else if (field.type === 'object') {
      // 对象字段以 JSON 文本编辑（如 VideoParams）
      formModel.value[field.name] = JSON.stringify(d ?? {}, null, 2)
    } else if (d !== undefined) {
      formModel.value[field.name] = d
    }
  }
  runVisible.value = true
}

function buildPayload(): Record<string, unknown> {
  const payload: Record<string, unknown> = {}
  for (const field of formFields.value) {
    const v = formModel.value[field.name]
    if (v === undefined || v === null || v === '') continue
    if (Array.isArray(v) && v.length === 0) continue
    if (field.type === 'object') {
      // JSON 文本编辑框：解析后提交，非法 JSON 直接抛错由调用方提示
      payload[field.name] = JSON.parse(String(v))
      continue
    }
    if (field.type === 'array' && field.itemIsObject) {
      payload[field.name] = JSON.parse(String(v))
      continue
    }
    payload[field.name] = v
  }
  return payload
}

async function runFeature() {
  if (!current.value) return
  let input: Record<string, unknown>
  try {
    input = buildPayload()
  } catch (e) {
    ElMessage.error((e as Error).message.includes('JSON') ? 'JSON 格式错误，请检查对象类型字段' : (e as Error).message)
    return
  }
  running.value = true
  try {
    result.value = await api
      .post('/features/run', {
        featureId: current.value.id,
        input,
      })
      .then(extractData<FeatureOutput>)
    resultVisible.value = true
    runVisible.value = false
  } catch (e) {
    ElMessage.error((e as Error).message || 'run failed')
  } finally {
    running.value = false
  }
}

const resultOk = computed(() => result.value?.status === 'success')

// ── 历史 ──
async function openHistory(f: FeatureDescriptor) {
  current.value = f
  historyVisible.value = true
  historyLoading.value = true
  try {
    history.value =
      (await api.post('/features/history', { featureId: f.id }).then(extractData<FeatureOutput[]>)) ||
      []
  } catch (e) {
    ElMessage.error((e as Error).message || 'history failed')
  } finally {
    historyLoading.value = false
  }
}

// ── 工具 ──
function prettyJson(v: unknown): string {
  try {
    return typeof v === 'string' ? v : JSON.stringify(v, null, 2)
  } catch {
    return String(v)
  }
}

function formatSize(bytes: number): string {
  if (!bytes) return '0 B'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`
}

function formatTime(iso: string): string {
  if (!iso) return ''
  return new Date(iso).toLocaleString()
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success(text)
  } catch {
    ElMessage.warning(text)
  }
}
</script>

<style scoped>
.wb-load-error {
  color: var(--soma-danger);
  font-size: 12.5px;
  line-height: 1.8;
  max-width: 480px;
  word-break: break-all;
}

.workbench-view {
  max-width: 1200px;
  margin: 0 auto;
}

.wb-toolbar {
  display: flex;
  gap: 12px;
  margin-bottom: 26px;

  .wb-search {
    max-width: 420px;
  }
}

.wb-group {
  margin-bottom: 30px;
}

.wb-group-head {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-bottom: 14px;
}

.wb-group-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  box-shadow: 0 0 10px currentColor;
}

.wb-group-title {
  font-size: 15px;
  font-weight: 650;
}

.wb-group-count {
  font-size: 11.5px;
  color: var(--soma-text-faint);
  border: 1px solid var(--soma-line);
  border-radius: 999px;
  padding: 1px 8px;
}

.wb-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(268px, 1fr));
  gap: 16px;
}

.wb-card {
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

.wb-card:hover {
  transform: translateY(-3px);
  border-color: rgba(91, 140, 255, 0.42);
  box-shadow: 0 14px 38px rgba(0, 0, 0, 0.4);
}

.wb-card-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: grid;
  place-items: center;
  margin-bottom: 4px;
}

.wb-card-name {
  font-size: 15.5px;
  font-weight: 650;
}

.wb-card-id {
  font-family: 'JetBrains Mono', 'Menlo', monospace;
  font-size: 10px;
  color: var(--soma-text-faint);
  opacity: 0;
  cursor: default;
  transition: opacity 0.25s;
  max-height: 0;
  overflow: hidden;
}

.wb-card:hover .wb-card-id {
  opacity: 0.7;
  max-height: 20px;
}

.wb-card-desc {
  font-size: 12.5px;
  color: var(--soma-text-dim);
  line-height: 1.6;
  flex: 1;
}

.wb-card-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 8px;
}

.wb-run-id {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--soma-text-dim);
  font-size: 13px;
  margin-bottom: 18px;
}

.wb-run-id-tag {
  font-family: 'JetBrains Mono', 'Menlo', monospace;
  font-size: 10px;
  opacity: 0.6;
  cursor: default;
}

.wb-field-label {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.wb-field-key {
  font-size: 10.5px;
  color: var(--soma-text-faint);
  font-weight: 400;
  opacity: 0.7;
}

.wb-field-type {
  font-size: 10px;
  color: var(--soma-text-faint);
  border: 1px solid var(--soma-line);
  border-radius: 5px;
  padding: 0 5px;
  opacity: 0.7;
}

.wb-meta-row {
  display: flex;
  gap: 8px;
  margin-top: 14px;
}

.wb-result-id {
  font-family: 'JetBrains Mono', 'Menlo', monospace;
  font-size: 10px;
  opacity: 0.7;
}

.wb-artifact,
.wb-history-artifact {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border: 1px solid var(--soma-line);
  border-radius: var(--soma-radius-sm);
  background: var(--soma-glass);
  margin-bottom: 8px;
}

.wb-artifact-info {
  flex: 1;
  min-width: 0;
}

.wb-artifact-name {
  font-size: 13px;
  font-weight: 600;
}

.wb-artifact-path {
  font-size: 11px;
  color: var(--soma-text-faint);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.wb-artifact-size {
  font-size: 11.5px;
  color: var(--soma-text-dim);
}

.wb-json-editor :deep(.el-textarea__inner) {
  font-family: 'JetBrains Mono', 'Menlo', monospace;
  font-size: 12px;
}

.wb-json {
  background: rgba(5, 8, 18, 0.72);
  border: 1px solid var(--soma-line);
  border-radius: var(--soma-radius-sm);
  color: #c9d6f2;
  font-family: 'JetBrains Mono', 'Menlo', monospace;
  font-size: 12px;
  line-height: 1.7;
  padding: 14px;
  max-height: 260px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.wb-json-small {
  max-height: 140px;
  margin-top: 8px;
}

.wb-history-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.wb-history-item {
  border: 1px solid var(--soma-line);
  border-radius: var(--soma-radius);
  padding: 14px 16px;
  background: var(--soma-glass);
}

.wb-history-head {
  display: flex;
  align-items: center;
  gap: 10px;
}

.wb-history-time {
  font-size: 12.5px;
  color: var(--soma-text-dim);
}

.wb-history-duration {
  margin-left: auto;
  font-size: 11.5px;
  color: var(--soma-text-faint);
  font-family: 'JetBrains Mono', monospace;
}
</style>
