<template>
  <div class="toolbox-view">
    <!-- 顶部：搜索 + 刷新 -->
    <div class="tb-toolbar">
      <el-input
        v-model="keyword"
        :placeholder="$t('workbench.search')"
        :prefix-icon="Search"
        clearable
        size="large"
        class="tb-search"
      />
      <el-tooltip :content="$t('workbench.refresh')" placement="top">
        <el-button :icon="Refresh" size="large" circle @click="loadFeatures" />
      </el-tooltip>
    </div>

    <!-- 精选工具 -->
    <div class="tb-section">
      <div class="tb-section-head">
        <span class="tb-section-dot" />
        <span class="tb-section-title">{{ $t('toolbox.section.tools') }}</span>
        <span class="tb-section-count">{{ filteredTools.length }}</span>
      </div>
      <div class="tb-grid">
        <div v-for="t in filteredTools" :key="t.id" class="tb-card" @click="openTool(t)">
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
    </div>

    <!-- 功能点分组 -->
    <div class="tb-section">
      <div class="tb-section-head">
        <span class="tb-section-dot dot-feature" />
        <span class="tb-section-title">{{ $t('toolbox.section.features') }}</span>
        <span class="tb-section-count">{{ features.length }}</span>
      </div>
      <div v-for="group in featureGroups" :key="group.kind" class="wb-group">
        <div class="wb-group-head">
          <span class="wb-group-dot" :style="{ background: group.meta.color }" />
          <span class="wb-group-title">{{ $t(group.meta.labelKey) }}</span>
          <span class="wb-group-count">{{ group.features.length }}</span>
        </div>
        <div class="wb-grid">
          <div v-for="f in group.features" :key="f.id" class="wb-card" @click="openFeature(f)">
            <div class="wb-card-icon" :style="{ background: group.meta.chipBg, color: group.meta.color }">
              <el-icon :size="22"><component :is="group.meta.icon" /></el-icon>
            </div>
            <div class="wb-card-name">{{ f.name }}</div>
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
      <el-empty
        v-if="!featureLoading && featureGroups.length === 0"
        :description="loadError || $t('workbench.noFeatures')"
      >
        <template v-if="loadError">
          <div class="wb-load-error">
            <p>isTauri: {{ isTauriEnv }}</p>
            <p>{{ loadError }}</p>
          </div>
        </template>
      </el-empty>
      <el-empty
        v-else-if="keyword && filteredTools.length === 0 && featureGroups.length === 0"
        :description="$t('toolbox.noMatch')"
        :image-size="72"
      />
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

    <!-- 功能点运行对话框 -->
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

      <div v-if="featureFormFields.length" class="wb-form">
        <!-- 必填字段：直接展示 -->
        <template v-if="requiredFields.length">
          <div v-for="field in requiredFields" :key="field.name" class="wb-form-item">
            <el-form-item :required="field.required">
              <template #label>
                <span class="wb-field-label">
                  {{ field.description || field.name }}
                  <code v-if="field.description && field.description !== field.name" class="wb-field-key">{{ field.name }}</code>
                </span>
              </template>
              <!-- 文件选择器：视频素材（多选） -->
              <div v-if="isFileSelectorField(field.name, current?.id || '') === 'video'" class="wb-file-selector">
                <div class="wb-file-list" v-if="featureFormModel[field.name] && (featureFormModel[field.name] as string[]).length">
                  <div v-for="(path, idx) in (featureFormModel[field.name] as string[])" :key="path" class="wb-file-item">
                    <span class="wb-file-name">{{ getFileName(path) }}</span>
                    <el-button type="danger" :icon="Delete" size="small" circle @click="removeFile(field.name, idx)" />
                  </div>
                </div>
                <el-button type="primary" :icon="FolderOpened" @click="handleFilesPick(field.name)">
                  选择视频文件
                </el-button>
              </div>
              <!-- 文件选择器：音频文件（单选） -->
              <div v-else-if="isFileSelectorField(field.name, current?.id || '') === 'audio'" class="wb-file-selector">
                <div v-if="featureFormModel[field.name]" class="wb-file-list">
                  <div class="wb-file-item">
                    <span class="wb-file-name">{{ getFileName(featureFormModel[field.name] as string) }}</span>
                    <el-button type="danger" :icon="Delete" size="small" circle @click="featureFormModel[field.name] = ''" />
                  </div>
                </div>
                <el-button type="primary" :icon="FolderOpened" @click="handleFilePick(field.name)">
                  选择音频文件
                </el-button>
              </div>
              <el-switch v-else-if="field.type === 'boolean'" v-model="featureFormModel[field.name]" />
              <el-select
                v-else-if="field.options"
                v-model="featureFormModel[field.name]"
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
                v-model="featureFormModel[field.name]"
                type="textarea"
                :rows="6"
                class="wb-json-editor"
                :placeholder="$t('workbench.jsonHint')"
              />
              <el-input-number
                v-else-if="field.type === 'number' || field.type === 'integer'"
                v-model="featureFormModel[field.name]"
                :step="field.type === 'integer' ? 1 : 0.1"
                class="!w-full"
              />
              <el-input
                v-else-if="field.longText"
                v-model="featureFormModel[field.name]"
                type="textarea"
                :rows="4"
              />
              <el-input
                v-else
                v-model="featureFormModel[field.name]"
                :placeholder="field.description || field.name"
              />
            </el-form-item>
          </div>
        </template>

        <!-- 可选字段：折叠到「高级设置」 -->
        <el-collapse v-if="optionalFields.length" class="wb-advanced-collapse">
          <el-collapse-item :title="$t('workbench.advancedSettings')" name="advanced">
            <div v-for="field in optionalFields" :key="field.name" class="wb-form-item">
              <el-form-item>
                <template #label>
                  <span class="wb-field-label">
                    {{ field.description || field.name }}
                    <code v-if="field.description && field.description !== field.name" class="wb-field-key">{{ field.name }}</code>
                  </span>
                </template>
                <!-- 文件选择器：视频素材（多选） -->
                <div v-if="isFileSelectorField(field.name, current?.id || '') === 'video'" class="wb-file-selector">
                  <div class="wb-file-list" v-if="featureFormModel[field.name] && (featureFormModel[field.name] as string[]).length">
                    <div v-for="(path, idx) in (featureFormModel[field.name] as string[])" :key="path" class="wb-file-item">
                      <span class="wb-file-name">{{ getFileName(path) }}</span>
                      <el-button type="danger" :icon="Delete" size="small" circle @click="removeFile(field.name, idx)" />
                    </div>
                  </div>
                  <el-button type="primary" :icon="FolderOpened" @click="handleFilesPick(field.name)">
                    选择视频文件
                  </el-button>
                </div>
                <!-- 文件选择器：音频文件（单选） -->
                <div v-else-if="isFileSelectorField(field.name, current?.id || '') === 'audio'" class="wb-file-selector">
                  <div v-if="featureFormModel[field.name]" class="wb-file-list">
                    <div class="wb-file-item">
                      <span class="wb-file-name">{{ getFileName(featureFormModel[field.name] as string) }}</span>
                      <el-button type="danger" :icon="Delete" size="small" circle @click="featureFormModel[field.name] = ''" />
                    </div>
                  </div>
                  <el-button type="primary" :icon="FolderOpened" @click="handleFilePick(field.name)">
                    选择音频文件
                  </el-button>
                </div>
                <el-switch v-else-if="field.type === 'boolean'" v-model="featureFormModel[field.name]" />
                <el-select
                  v-else-if="field.options"
                  v-model="featureFormModel[field.name]"
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
                  v-model="featureFormModel[field.name]"
                  type="textarea"
                  :rows="6"
                  class="wb-json-editor"
                  :placeholder="$t('workbench.jsonHint')"
                />
                <el-input-number
                  v-else-if="field.type === 'number' || field.type === 'integer'"
                  v-model="featureFormModel[field.name]"
                  :step="field.type === 'integer' ? 1 : 0.1"
                  class="!w-full"
                />
                <el-input
                  v-else-if="field.longText"
                  v-model="featureFormModel[field.name]"
                  type="textarea"
                  :rows="4"
                />
                <el-input
                  v-else
                  v-model="featureFormModel[field.name]"
                  :placeholder="field.description || field.name"
                />
              </el-form-item>
            </div>
          </el-collapse-item>
        </el-collapse>
      </div>
      <el-empty v-else :description="$t('workbench.noParams')" :image-size="72" />

      <template #footer>
        <el-button @click="openHistory(current!)">{{ $t('workbench.history') }}</el-button>
        <el-button type="primary" :loading="featureRunning" @click="runFeature">
          {{ featureRunning ? $t('workbench.running') : $t('workbench.run') }}
        </el-button>
      </template>
    </el-dialog>

    <!-- 功能点运行结果对话框 -->
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

    <!-- 功能点历史记录抽屉 -->
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
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Bell, DataAnalysis, Document, Download, Film, Link, MagicStick, Search, Refresh, FolderOpened, Delete } from '@element-plus/icons-vue'
import api, { extractData } from '@/api'
import { open } from '@tauri-apps/plugin-dialog'
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

const { t } = useI18n()

// ══════════════════════ 精选工具 ══════════════════════

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

// ══════════════════════ 功能点（原功能点工作台，schema 驱动） ══════════════════════

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
const TTS_PROVIDERS = ['edge', 'azure', 'siliconflow', 'elevenlabs', 'mimo', 'gemini', 'volcengine', 'xfyun', 'fishspeech'] as const
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
  fishspeech: 'Fish-Speech S2',
  'fishspeech:': 'Fish-Speech S2 (默认音色)',
  // BGM
  none: '无背景音乐',
  random: '随机背景音乐',
}

/** 根据选项值获取中文显示标签 */
function getOptionLabel(value: string): string {
  return OPTION_LABELS[value] || value
}

/** 判断字段是否使用文件选择器 */
function isFileSelectorField(fieldName: string, featureId: string): 'video' | 'audio' | null {
  if (featureId === 'video.compose' || featureId === 'video.concat') {
    if (fieldName === 'materials') return 'video'
    if (fieldName === 'audio_file') return 'audio'
  }
  return null
}

/** 打开文件选择对话框 */
async function pickFile(fieldType: 'video' | 'audio'): Promise<string | null> {
  try {
    const filters = fieldType === 'video'
      ? [{ name: '视频文件', extensions: ['mp4', 'avi', 'mov', 'mkv', 'webm', 'flv', 'wmv'] }]
      : [{ name: '音频文件', extensions: ['mp3', 'wav', 'aac', 'flac', 'ogg', 'm4a', 'wma'] }]

    const selected = await open({
      multiple: false,
      filters,
      title: fieldType === 'video' ? '选择视频文件' : '选择音频文件',
    })

    if (selected && typeof selected === 'string') {
      return selected
    }
    return null
  } catch (e) {
    console.error('[toolbox] pickFile failed:', e)
    return null
  }
}

/** 打开文件选择对话框（多选） */
async function pickFiles(fieldType: 'video' | 'audio'): Promise<string[]> {
  try {
    const filters = fieldType === 'video'
      ? [{ name: '视频文件', extensions: ['mp4', 'avi', 'mov', 'mkv', 'webm', 'flv', 'wmv'] }]
      : [{ name: '音频文件', extensions: ['mp3', 'wav', 'aac', 'flac', 'ogg', 'm4a', 'wma'] }]

    const selected = await open({
      multiple: true,
      filters,
      title: fieldType === 'video' ? '选择视频文件' : '选择音频文件',
    })

    if (selected) {
      if (Array.isArray(selected)) {
        return selected
      }
      return [selected]
    }
    return []
  } catch (e) {
    console.error('[toolbox] pickFiles failed:', e)
    return []
  }
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
const featureLoading = ref(false)
const keyword = ref('')
const loadError = ref('')
const isTauriEnv = typeof window !== 'undefined' && '__TAURI__' in window

const runVisible = ref(false)
const featureRunning = ref(false)
const current = ref<FeatureDescriptor | null>(null)
const featureFormModel = ref<Record<string, unknown>>({})
const resultVisible = ref(false)
const result = ref<FeatureOutput | null>(null)

const historyVisible = ref(false)
const historyLoading = ref(false)
const history = ref<FeatureOutput[]>([])

/** 关键词过滤：同时作用于精选工具与功能点 */
const filteredTools = computed(() => {
  const k = keyword.value.trim().toLowerCase()
  if (!k) return tools
  return tools.filter(
    (t2) =>
      t2.id.toLowerCase().includes(k) ||
      t2.module.toLowerCase().includes(k) ||
      t(t2.titleKey).toLowerCase().includes(k) ||
      t(t2.descKey).toLowerCase().includes(k),
  )
})

onMounted(loadFeatures)

async function loadFeatures() {
  featureLoading.value = true
  loadError.value = ''
  try {
    features.value = (await api.post('/features/list', {}).then(extractData<FeatureDescriptor[]>)) || []
  } catch (e) {
    loadError.value = (e as Error).message || String(e)
    console.error('[toolbox] loadFeatures failed:', e)
    ElMessage.error(loadError.value)
  } finally {
    featureLoading.value = false
  }
}

const featureGroups = computed(() => {
  const k = keyword.value.trim().toLowerCase()
  const matched = features.value.filter((f) => {
    if (!k) return true
    return (
      f.id.toLowerCase().includes(k) ||
      f.name.toLowerCase().includes(k) ||
      f.description.toLowerCase().includes(k)
    )
  })
  const map = new Map<string, { kind: string; meta: KindMeta; features: FeatureDescriptor[] }>()
  for (const f of matched) {
    const kind = KIND_META[f.kind] ? f.kind : 'utility'
    if (!map.has(kind)) {
      map.set(kind, { kind, meta: KIND_META[kind], features: [] })
    }
    map.get(kind)!.features.push(f)
  }
  return Array.from(map.values())
})

// ── Schema 驱动表单 ──
const featureFormFields = computed<FormField[]>(() => {
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

/** 必填字段（直接展示） */
const requiredFields = computed(() => featureFormFields.value.filter(f => f.required))

/** 可选字段（折叠到「高级设置」） */
const optionalFields = computed(() => featureFormFields.value.filter(f => !f.required))

function openFeature(f: FeatureDescriptor) {
  current.value = f
  featureFormModel.value = {}
  const defaults = FIELD_DEFAULTS[f.id] || {}
  for (const field of featureFormFields.value) {
    const d = defaults[field.name]
    if (field.type === 'array') {
      featureFormModel.value[field.name] = field.itemIsObject
        ? JSON.stringify(Array.isArray(d) ? d : [], null, 2)
        : Array.isArray(d) ? [...d] : []
    } else if (field.type === 'object') {
      // 对象字段以 JSON 文本编辑（如 VideoParams）
      featureFormModel.value[field.name] = JSON.stringify(d ?? {}, null, 2)
    } else if (d !== undefined) {
      featureFormModel.value[field.name] = d
    }
  }
  runVisible.value = true
}

/** 从完整路径中提取文件名 */
function getFileName(path: string): string {
  return path.split('/').pop() || path.split('\\').pop() || path
}

/** 处理文件选择（单选） */
async function handleFilePick(fieldName: string) {
  const fieldType = isFileSelectorField(fieldName, current.value?.id || '')
  if (!fieldType) return

  const filePath = await pickFile(fieldType)
  if (filePath) {
    featureFormModel.value[fieldName] = filePath
  }
}

/** 处理文件选择（多选，用于 materials） */
async function handleFilesPick(fieldName: string) {
  const fieldType = isFileSelectorField(fieldName, current.value?.id || '')
  if (!fieldType) return

  const filePaths = await pickFiles(fieldType)
  if (filePaths.length > 0) {
    // 合并到现有列表（去重）
    const existing = Array.isArray(featureFormModel.value[fieldName]) ? featureFormModel.value[fieldName] as string[] : []
    const merged = [...new Set([...existing, ...filePaths])]
    featureFormModel.value[fieldName] = merged
  }
}

/** 移除已选文件 */
function removeFile(fieldName: string, index: number) {
  const existing = Array.isArray(featureFormModel.value[fieldName]) ? featureFormModel.value[fieldName] as string[] : []
  existing.splice(index, 1)
  featureFormModel.value[fieldName] = [...existing]
}

function buildPayload(): Record<string, unknown> {
  const payload: Record<string, unknown> = {}
  for (const field of featureFormFields.value) {
    const v = featureFormModel.value[field.name]
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
  featureRunning.value = true
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
    featureRunning.value = false
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
.toolbox-view {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.tb-toolbar {
  display: flex;
  gap: 12px;
  margin-bottom: 26px;
}

.tb-search {
  max-width: 420px;
}

.tb-section {
  margin-bottom: 30px;
}

.tb-section-head {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-bottom: 14px;
}

.tb-section-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #5b8cff;
  box-shadow: 0 0 10px currentColor;
}

.tb-section-dot.dot-feature {
  background: #34d399;
}

.tb-section-title {
  font-size: 15px;
  font-weight: 650;
}

.tb-section-count {
  font-size: 11.5px;
  color: var(--soma-text-faint);
  border: 1px solid var(--soma-line);
  border-radius: 999px;
  padding: 1px 8px;
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

/* ── 功能点（原工作台样式） ── */

.wb-load-error {
  color: var(--soma-danger);
  font-size: 12.5px;
  line-height: 1.8;
  max-width: 480px;
  word-break: break-all;
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

.wb-advanced-collapse {
  border: none;
  margin-top: 8px;
}

.wb-advanced-collapse :deep(.el-collapse-item__header) {
  font-size: 13px;
  color: var(--soma-text-dim);
  height: 36px;
  line-height: 36px;
  background: transparent;
  border: none;
}

.wb-advanced-collapse :deep(.el-collapse-item__wrap) {
  border: none;
  background: transparent;
}

.wb-advanced-collapse :deep(.el-collapse-item__content) {
  padding-bottom: 0;
}

.wb-form-item {
  margin-bottom: 4px;
}

.wb-file-selector {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.wb-file-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.wb-file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border: 1px solid var(--soma-line);
  border-radius: var(--soma-radius-sm);
  background: var(--soma-glass);
}

.wb-file-name {
  flex: 1;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
