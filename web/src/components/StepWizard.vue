<template>
  <div class="step-wizard">
    <el-steps :active="currentStep" finish-status="success" align-center class="wizard-steps">
      <el-step :title="$t('wizard.step1.short')" :icon="Aim" />
      <el-step :title="$t('wizard.step2.short')" :icon="Edit" />
      <el-step :title="$t('wizard.step3.short')" :icon="Grid" />
      <el-step title="旁白" :icon="Microphone" />
      <el-step :title="$t('wizard.step5.short')" :icon="Picture" />
      <el-step :title="$t('wizard.step6.short')" :icon="ChatLineSquare" />
      <el-step :title="$t('wizard.step7.short')" :icon="ChatLineSquare" />
      <el-step :title="$t('wizard.step8.short')" :icon="Film" />
      <el-step :title="$t('wizard.step9.short')" :icon="Upload" />
    </el-steps>

    <div class="wizard-body">
      <transition name="wizard-fade" mode="out-in">
        <div :key="currentStep" class="step-content">

          <div v-if="currentStep === 0" class="step-panel">
            <div class="step-header">
              <h3>{{ $t('wizard.step1.title') }}</h3>
              <p class="step-desc">{{ $t('wizard.step1.desc') }}</p>
            </div>
            <el-card class="panel-card" shadow="hover">
              <div class="form-row">
                <div class="form-label">{{ $t('wizard.step1.intent') }}</div>
                <el-input
                  v-model="videoParamsStore.videoSubject"
                  :placeholder="$t('wizard.step1.intentPlaceholder')"
                  clearable
                  size="large"
                />
              </div>
              <el-row :gutter="12">
                <el-col :span="12">
                  <div class="form-row">
                    <div class="form-label">{{ $t('wizard.step1.style') }}</div>
                    <el-select v-model="videoParamsStore.intentStyle" style="width: 100%">
                      <el-option :label="$t('wizard.step1.styleAesthetic')" value="aesthetic" />
                      <el-option :label="$t('wizard.step1.styleDocumentary')" value="documentary" />
                      <el-option :label="$t('wizard.step1.styleVlog')" value="vlog" />
                      <el-option :label="$t('wizard.step1.styleNews')" value="news" />
                      <el-option :label="$t('wizard.step1.styleTutorial')" value="tutorial" />
                    </el-select>
                  </div>
                </el-col>
                <el-col :span="12">
                  <div class="form-row">
                    <div class="form-label">{{ $t('wizard.step1.mood') }}</div>
                    <el-select v-model="videoParamsStore.intentMood" style="width: 100%">
                      <el-option :label="$t('wizard.step1.moodWarm')" value="warm" />
                      <el-option :label="$t('wizard.step1.moodEnergetic')" value="energetic" />
                      <el-option :label="$t('wizard.step1.moodCalm')" value="calm" />
                      <el-option :label="$t('wizard.step1.moodDramatic')" value="dramatic" />
                      <el-option :label="$t('wizard.step1.moodHumorous')" value="humorous" />
                    </el-select>
                  </div>
                </el-col>
              </el-row>
              <el-row :gutter="12">
                <el-col :span="12">
                  <div class="form-row">
                    <div class="form-label">{{ $t('wizard.step1.aspect') }}</div>
                    <el-select v-model="videoParamsStore.videoAspect" style="width: 100%">
                      <el-option :label="$t('video.portrait916')" value="9:16" />
                      <el-option :label="$t('video.landscape169')" value="16:9" />
                      <el-option :label="$t('video.square11')" value="1:1" />
                    </el-select>
                  </div>
                </el-col>
                <el-col :span="12">
                  <div class="form-row">
                    <div class="form-label">{{ $t('script.scriptLanguage') }}</div>
                    <el-select v-model="videoParamsStore.videoLanguage" style="width: 100%">
                      <el-option :label="$t('script.autoDetect')" value="auto" />
                      <el-option label="简体中文" value="zh-CN" />
                      <el-option label="English" value="en-US" />
                    </el-select>
                  </div>
                </el-col>
              </el-row>
              <div class="form-row">
                <div class="form-label">{{ $t('wizard.step1.audience') }}</div>
                <el-input
                  v-model="videoParamsStore.intentAudience"
                  :placeholder="$t('wizard.step1.audiencePlaceholder')"
                  clearable
                />
              </div>
            </el-card>
            <div v-if="parsingIntent" class="intent-parsing-hint">
              <el-icon class="is-loading"><Loading /></el-icon>
              {{ $t('wizard.step1.parsingIntent') }}
            </div>
            <div v-else-if="intentParsed" class="intent-parsed-hint">
              <el-icon><CircleCheck /></el-icon>
              {{ $t('wizard.step1.intentParsed') }}
            </div>
          </div>

          <div v-if="currentStep === 1" class="step-panel">
            <div class="step-header">
              <h3>{{ $t('wizard.step2.title') }}</h3>
              <p class="step-desc">{{ $t('wizard.step2.desc') }}</p>
            </div>
            <div v-if="generatingScript" class="intent-parsing-hint">
              <el-icon class="is-loading"><Loading /></el-icon>
              {{ $t('wizard.step2.generatingScript') }}
            </div>
            <ScriptPanel v-else />
          </div>

          <div v-if="currentStep === 2" class="step-panel">
            <div class="step-header">
              <h3>{{ $t('wizard.step3.title') }}</h3>
              <p class="step-desc">{{ $t('wizard.step3.desc') }}</p>
            </div>
            <el-card class="panel-card" shadow="hover">
              <div class="form-row">
                <div class="form-label">{{ $t('wizard.step3.clipDuration') }}</div>
                <el-select v-model="videoParamsStore.videoClipDuration" style="width: 100%">
                  <el-option v-for="d in [2,3,4,5,6,7,8,9,10]" :key="d" :label="d + 's'" :value="d" />
                </el-select>
              </div>
              <div class="form-row">
                <div class="form-label">{{ $t('video.videoCount') }}</div>
                <el-select v-model="videoParamsStore.videoCount" style="width: 100%">
                  <el-option v-for="c in [1,2,3,4,5]" :key="c" :label="c" :value="c" />
                </el-select>
              </div>
              <div class="form-row">
                <div class="form-label">{{ $t('video.concatMode') }}</div>
                <el-select v-model="videoParamsStore.videoConcatMode" style="width: 100%">
                  <el-option :label="$t('video.sequential')" value="sequential" />
                  <el-option :label="$t('video.random')" value="random" />
                </el-select>
              </div>
              <div class="form-row">
                <div class="form-label">{{ $t('video.transitionMode') }}</div>
                <el-select v-model="videoParamsStore.videoTransitionMode" style="width: 100%">
                  <el-option :label="$t('video.none')" value="none" />
                  <el-option :label="$t('video.shuffle')" value="Shuffle" />
                  <el-option :label="$t('video.fadeIn')" value="FadeIn" />
                  <el-option :label="$t('video.fadeOut')" value="FadeOut" />
                  <el-option :label="$t('video.slideIn')" value="SlideIn" />
                  <el-option :label="$t('video.slideOut')" value="SlideOut" />
                </el-select>
              </div>
              <el-collapse>
                <el-collapse-item :title="$t('script.advancedSettings')">
                  <ScriptAdvancedSettings />
                </el-collapse-item>
              </el-collapse>
            </el-card>
            <div v-if="generatingStoryboard" class="intent-parsing-hint">
              <el-icon class="is-loading"><Loading /></el-icon>
              {{ $t('wizard.step3.generatingStoryboard') }}
            </div>
            <el-card v-else-if="videoParamsStore.storyboard.length > 0" class="panel-card" shadow="hover" style="margin-top: 12px">
              <template #header>
                <span>{{ $t('wizard.step3.storyboardPreview') }} ({{ videoParamsStore.storyboard.length }} {{ $t('wizard.step3.scenes') }})</span>
              </template>
              <div class="storyboard-list">
                <div v-for="scene in videoParamsStore.storyboard" :key="scene.scene_id" class="storyboard-item">
                  <div class="scene-header">
                    <el-tag size="small" type="primary">#{{ scene.scene_id }}</el-tag>
                    <span class="scene-duration">{{ scene.duration || videoParamsStore.videoClipDuration }}s</span>
                    <span v-if="scene.camera_movement" class="scene-camera">{{ scene.camera_movement }}</span>
                    <span v-if="scene.transition" class="scene-transition">→ {{ scene.transition }}</span>
                  </div>
                  <div class="scene-narration">{{ scene.narration }}</div>
                  <div v-if="scene.visual_desc" class="scene-visual-desc">{{ scene.visual_desc }}</div>
                  <div v-if="scene.visual_prompt" class="scene-visual-prompt">{{ scene.visual_prompt }}</div>
                </div>
              </div>
            </el-card>
            <el-card class="panel-card" shadow="hover" style="margin-top: 12px">
              <div class="form-row">
                <el-checkbox v-model="videoParamsStore.matchMaterialsToScript">
                  {{ $t('video.matchMaterialsToScript') }}
                </el-checkbox>
              </div>
              <div class="form-row">
                <div class="form-label">{{ $t('wizard.step5.keywordPreview') }}</div>
                <el-input
                  v-model="videoParamsStore.videoTerms"
                  type="textarea"
                  :rows="3"
                  :placeholder="$t('script.videoKeywordsPlaceholder')"
                />
              </div>
              <div style="margin-top: 8px">
                <el-button
                  type="primary"
                  :loading="generatingTerms"
                  @click="onGenerateTerms"
                  style="width: 100%"
                >
                  {{ generatingTerms ? $t('wizard.step4.generating') : $t('wizard.step4.generate') }}
                </el-button>
              </div>
              <ApiKeyManager />
            </el-card>
          </div>

          <div v-if="currentStep === 3" class="step-panel">
            <div class="step-header">
              <h3>旁白文案</h3>
              <p class="step-desc">将视频脚本转换为适合语音朗读的口语化旁白文案，用于 TTS 语音合成</p>
            </div>
            <el-card class="panel-card" shadow="hover">
              <div style="margin-bottom: 8px">
                <el-button
                  type="primary"
                  :loading="generatingNarration"
                  @click="onGenerateNarration"
                  style="width: 100%"
                >
                  {{ generatingNarration ? '正在生成旁白...' : 'AI 生成旁白文案' }}
                </el-button>
              </div>
              <div class="form-row">
                <div class="form-label">旁白文案 <span style="color:#909399;font-size:12px;font-weight:normal">（可直接编辑，生成语音时使用此文本）</span></div>
                <el-input
                  v-model="videoParamsStore.narration"
                  type="textarea"
                  :rows="10"
                  placeholder="点击上方按钮 AI 自动生成，或手动输入旁白文案..."
                />
              </div>
            </el-card>
          </div>

          <div v-if="currentStep === 4" class="step-panel">
            <div class="step-header">
              <h3>{{ $t('wizard.step5.title') }}</h3>
              <p class="step-desc">{{ $t('wizard.step5.desc') }}</p>
            </div>
            <el-card class="panel-card" shadow="hover">
              <div class="form-row">
                <div class="form-label">{{ $t('video.videoSource') }}</div>
                <el-select v-model="videoParamsStore.videoSource" style="width: 100%">
                  <el-option :label="$t('video.pexels')" value="pexels" />
                  <el-option :label="$t('video.pixabay')" value="pixabay" />
                  <el-option :label="$t('video.coverr')" value="coverr" />
                  <el-option :label="$t('video.cogvideox')" value="cogvideox" />
                  <el-option :label="$t('video.kling')" value="kling" />
                  <el-option :label="$t('video.minimax')" value="minimax" />
                  <el-option :label="$t('video.local')" value="local" />
                </el-select>
              </div>
              <PortraitUploader v-if="videoParamsStore.videoSource !== 'local'" />
              <LocalFileUploader v-if="videoParamsStore.videoSource === 'local'" />
              <el-collapse>
                <el-collapse-item :title="$t('video.advancedSettings')">
                  <VideoAdvancedSettings />
                </el-collapse-item>
              </el-collapse>
            </el-card>
            <div v-if="videoParamsStore.fetchingMaterials" class="intent-parsing-hint">
              <el-icon class="is-loading"><Loading /></el-icon>
              {{ $t('wizard.step5.fetchingMaterials') }}
            </div>
            <div v-else-if="videoParamsStore.materialsFetched" class="intent-parsed-hint">
              <el-icon><CircleCheck /></el-icon>
              {{ $t('wizard.step5.materialsFetched', { count: videoParamsStore.materialsList.length }) }}
            </div>
          </div>

          <div v-if="currentStep === 5" class="step-panel">
            <div class="step-header">
              <h3>{{ $t('wizard.step6.title') }}</h3>
              <p class="step-desc">{{ $t('wizard.step6.desc') }}</p>
            </div>
            <AudioSettingsPanel />
            <div v-if="generatingAudio" class="intent-parsing-hint">
              <el-icon class="is-loading"><Loading /></el-icon>
              {{ $t('wizard.step6.generatingAudio') }}
            </div>
            <div v-else-if="videoParamsStore.audioFile" class="audio-preview-section">
              <el-divider />
              <div class="audio-preview-label">{{ $t('wizard.step6.audioPreview') }}</div>
              <audio controls :src="`/storage/audio/${videoParamsStore.audioFile.split('/').pop()}`" style="width: 100%; max-width: 400px" />
            </div>
          </div>

          <div v-if="currentStep === 6" class="step-panel">
            <div class="step-header">
              <h3>{{ $t('wizard.step7.title') }}</h3>
              <p class="step-desc">{{ $t('wizard.step7.desc') }}</p>
            </div>
            <SubtitleSettingsPanel />
          </div>

          <div v-if="currentStep === 7" class="step-panel">
            <div class="step-header">
              <h3>{{ $t('wizard.step8.title') }}</h3>
              <p class="step-desc">{{ $t('wizard.step8.desc') }}</p>
            </div>
            <div class="review-summary">
              <el-descriptions :column="2" border size="small">
                <el-descriptions-item :label="$t('wizard.step1.intent')">
                  {{ videoParamsStore.videoSubject || '—' }}
                </el-descriptions-item>
                <el-descriptions-item :label="$t('wizard.step1.style')">
                  {{ intentStyleLabel }}
                </el-descriptions-item>
                <el-descriptions-item :label="$t('wizard.step1.aspect')">
                  {{ videoParamsStore.videoAspect }}
                </el-descriptions-item>
                <el-descriptions-item :label="$t('video.videoSource')">
                  {{ videoSourceLabel }}
                </el-descriptions-item>
                <el-descriptions-item :label="$t('wizard.step6.tts')">
                  {{ videoParamsStore.ttsServer }}
                </el-descriptions-item>
                <el-descriptions-item :label="$t('subtitle.enabled')">
                  {{ videoParamsStore.subtitleEnabled ? $t('common.ok') : $t('common.cancel') }}
                </el-descriptions-item>
              </el-descriptions>
            </div>
            <div class="generate-section">
              <el-button
                type="primary"
                size="large"
                :loading="taskStore.isGenerating"
                :disabled="!canGenerate"
                @click="onGenerate"
              >
                <el-icon v-if="!taskStore.isGenerating"><VideoCamera /></el-icon>
                {{ taskStore.isGenerating ? $t('generate.generating') : $t('generate.button') }}
              </el-button>
            </div>
            <TaskProgress v-if="taskStore.currentTask" />
            <AiVideoLogs />
            <LogOutput />
            <VideoPreview />
          </div>

          <div v-if="currentStep === 8" class="step-panel">
            <div class="step-header">
              <h3>{{ $t('wizard.step9.title') }}</h3>
              <p class="step-desc">{{ $t('wizard.step9.desc') }}</p>
            </div>
            <div class="publish-platforms">
              <el-checkbox-group v-model="selectedPlatforms">
                <el-card
                  v-for="p in platformOptions"
                  :key="p.value"
                  class="platform-card"
                  :class="{ 'platform-selected': selectedPlatforms.includes(p.value) }"
                  shadow="hover"
                  @click="togglePlatform(p.value)"
                >
                  <div class="platform-inner">
                    <el-checkbox :label="p.value" :value="p.value" @click.stop>
                      <span class="platform-icon">{{ p.icon }}</span>
                      <span class="platform-name">{{ p.label }}</span>
                    </el-checkbox>
                    <p class="platform-hint">{{ p.hint }}</p>
                  </div>
                </el-card>
              </el-checkbox-group>
            </div>
            <el-card v-if="selectedPlatforms.length > 0" class="panel-card" shadow="hover" style="margin-top: 16px">
              <div class="form-row" style="display:flex;align-items:center;gap:12px">
                <div class="form-label" style="margin-bottom:0">{{ $t('social.platform') }}</div>
                <el-select v-model="socialPlatform" style="width:180px">
                  <el-option v-for="p in selectedPlatforms" :key="p" :label="platformLabel(p)" :value="p" />
                </el-select>
                <el-button type="primary" :loading="generatingSocial" :disabled="!videoParamsStore.videoScript" @click="onGenerateSocial">
                  {{ generatingSocial ? $t('social.generating') : $t('social.generate') }}
                </el-button>
              </div>
              <div class="form-row">
                <div class="form-label">{{ $t('wizard.step9.publishTitle') }}</div>
                <el-input v-model="publishTitle" :placeholder="$t('wizard.step9.publishTitlePlaceholder')" />
              </div>
              <div class="form-row">
                <div class="form-label">{{ $t('wizard.step9.publishDesc') }}</div>
                <el-input v-model="publishDesc" type="textarea" :rows="3" :placeholder="$t('wizard.step9.publishDescPlaceholder')" />
              </div>
              <div class="form-row">
                <div class="form-label">{{ $t('wizard.step9.publishTags') }}</div>
                <el-input v-model="publishTags" :placeholder="$t('wizard.step9.publishTagsPlaceholder')" />
              </div>
              <div class="publish-actions">
                <el-button type="success" size="large" :loading="publishing" @click="onPublish">
                  <el-icon><Upload /></el-icon>
                  {{ publishing ? $t('wizard.step9.publishing') : $t('wizard.step9.publishNow') }}
                </el-button>
              </div>
            </el-card>
          </div>

        </div>
      </transition>
    </div>

    <div class="wizard-footer">
      <el-button v-if="currentStep > 0" @click="onPrev">
        <el-icon><ArrowLeft /></el-icon>
        {{ $t('wizard.prev') }}
      </el-button>
      <div class="spacer" />
      <el-button v-if="currentStep < 8" type="primary" :loading="savingStep" @click="onNext">
        {{ $t('wizard.next') }}
        <el-icon><ArrowRight /></el-icon>
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { VideoCamera, Aim, Edit, Grid, Picture, Microphone, ChatLineSquare, Film, Upload, ArrowLeft, ArrowRight, Loading, CircleCheck } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useVideoParamsStore } from '@/stores/videoParams'
import { useTaskStore } from '@/stores/task'
import { useI18n } from 'vue-i18n'
import { generateSocialMetadata, generateIntent, generateStoryboard, generateTerms, generateScript } from '@/api/llm'
import { getTask, generateAudio, generateNarration } from '@/api/video'
import type { VideoParams, StoryboardScene } from '@/types'
import ScriptPanel from '@/components/ScriptPanel.vue'
import ScriptAdvancedSettings from '@/components/ScriptAdvancedSettings.vue'
import VideoAdvancedSettings from '@/components/VideoAdvancedSettings.vue'
import AudioSettingsPanel from '@/components/AudioSettingsPanel.vue'
import SubtitleSettingsPanel from '@/components/SubtitleSettingsPanel.vue'
import ApiKeyManager from '@/components/ApiKeyManager.vue'
import PortraitUploader from '@/components/PortraitUploader.vue'
import LocalFileUploader from '@/components/LocalFileUploader.vue'
import TaskProgress from '@/components/TaskProgress.vue'
import AiVideoLogs from '@/components/AiVideoLogs.vue'
import LogOutput from '@/components/LogOutput.vue'
import VideoPreview from '@/components/VideoPreview.vue'

const videoParamsStore = useVideoParamsStore()
const taskStore = useTaskStore()
const route = useRoute()
const { t } = useI18n()

const currentStep = ref(0)

onMounted(async () => {
  const resumeId = route.query.resume as string
  if (!resumeId) {
    videoParamsStore.resetAll()
    taskStore.resetAll()
    return
  }
  videoParamsStore.resetAll()
  try {
    const task = await getTask(resumeId)
    if (task.params) {
      videoParamsStore.loadFromTask(task.params, task)
    }
    taskStore.resumeDraft(resumeId)
    intentParsed.value = true
    if (videoParamsStore.narration) {
      currentStep.value = 4
    } else if (videoParamsStore.videoScript) {
      currentStep.value = videoParamsStore.storyboard.length > 0 ? 3 : 3
    } else if (videoParamsStore.videoSubject) {
      currentStep.value = 1
    }
  } catch (e) {
    console.error('Failed to load draft task:', e)
  }
})
const savingStep = ref(false)
const parsingIntent = ref(false)
const intentParsed = ref(false)
const generatingScript = ref(false)
const generatingStoryboard = ref(false)
const generatingTerms = ref(false)
const generatingNarration = ref(false)
const generatingAudio = ref(false)

const selectedPlatforms = ref<string[]>([])
const socialPlatform = ref('')
const generatingSocial = ref(false)
const publishTitle = ref('')
const publishDesc = ref('')
const publishTags = ref('')
const publishing = ref(false)

const canGenerate = computed(() => {
  if (taskStore.isGenerating) return false
  if (!videoParamsStore.videoSubject.trim()) return false
  if (!videoParamsStore.videoScript.trim()) return false
  return true
})

const platformOptions = computed(() => [
  { value: 'douyin', label: '抖音', icon: '🎵', hint: t('wizard.step9.hintDouyin') },
  { value: 'weixin', label: '视频号', icon: '💬', hint: t('wizard.step9.hintWeixin') },
  { value: 'xiaohongshu', label: '小红书', icon: '📕', hint: t('wizard.step9.hintXiaohongshu') },
  { value: 'bilibili', label: 'B站', icon: '📺', hint: t('wizard.step9.hintBilibili') },
  { value: 'tiktok', label: 'TikTok', icon: '🎵', hint: t('wizard.step9.hintTiktok') },
  { value: 'youtube', label: 'YouTube Shorts', icon: '▶️', hint: t('wizard.step9.hintYoutube') },
  { value: 'instagram', label: 'Instagram Reels', icon: '📷', hint: t('wizard.step9.hintInstagram') },
  { value: 'facebook', label: 'Facebook Reels', icon: '👥', hint: t('wizard.step9.hintFacebook') },
])

function platformLabel(value: string): string {
  const p = platformOptions.value.find(o => o.value === value)
  return p ? p.label : value
}

const intentStyleLabel = computed(() => {
  const map: Record<string, string> = {
    aesthetic: t('wizard.step1.styleAesthetic'),
    documentary: t('wizard.step1.styleDocumentary'),
    vlog: t('wizard.step1.styleVlog'),
    news: t('wizard.step1.styleNews'),
    tutorial: t('wizard.step1.styleTutorial'),
  }
  return map[videoParamsStore.intentStyle] || videoParamsStore.intentStyle
})

const videoSourceLabel = computed(() => {
  const map: Record<string, string> = {
    pexels: 'Pexels',
    pixabay: 'Pixabay',
    coverr: 'Coverr',
    cogvideox: 'CogVideoX',
    kling: 'Kling',
    minimax: 'MiniMax',
    local: t('video.local'),
  }
  return map[videoParamsStore.videoSource] || videoParamsStore.videoSource
})

function validateStep(step: number): boolean {
  switch (step) {
    case 0:
      if (!videoParamsStore.videoSubject.trim()) {
        ElMessage.warning(t('wizard.validate.subjectRequired'))
        return false
      }
      break
    case 1:
      if (!videoParamsStore.videoScript.trim()) {
        ElMessage.warning(t('wizard.validate.scriptRequired'))
        return false
      }
      break
    case 4:
      if (videoParamsStore.videoSource === 'local' && videoParamsStore.localVideoMaterials.length === 0) {
        ElMessage.warning(t('wizard.validate.localMaterialsRequired'))
        return false
      }
      break
  }
  return true
}

const STYLE_MAP: Record<string, string> = {
  '唯美': 'aesthetic', '唯美治愈': 'aesthetic', '治愈': 'aesthetic', 'aesthetic': 'aesthetic',
  '纪录': 'documentary', '纪实': 'documentary', '纪录片': 'documentary', 'documentary': 'documentary',
  'vlog': 'vlog', '日常': 'vlog', '生活': 'vlog',
  '新闻': 'news', '资讯': 'news', '新闻体': 'news', 'news': 'news',
  '教程': 'tutorial', '教学': 'tutorial', '知识': 'tutorial', 'tutorial': 'tutorial',
  '科技': 'aesthetic', '科技感': 'aesthetic',
  '幽默': 'humorous', '搞笑': 'humorous', '轻松': 'humorous', 'humorous': 'humorous',
}

const MOOD_MAP: Record<string, string> = {
  '温暖': 'warm', '温馨': 'warm', '希望': 'warm', 'warm': 'warm',
  '活力': 'energetic', '动感': 'energetic', '激情': 'energetic', 'energetic': 'energetic',
  '宁静': 'calm', '平静': 'calm', '淡雅': 'calm', 'calm': 'calm',
  '戏剧': 'dramatic', '紧张': 'dramatic', '悬疑': 'dramatic', 'dramatic': 'dramatic',
  '幽默': 'humorous', '轻松': 'humorous', '欢乐': 'humorous', 'humorous': 'humorous',
}

function mapIntentField(value: string, mapping: Record<string, string>): string {
  if (!value) return ''
  const lower = value.toLowerCase()
  for (const [key, val] of Object.entries(mapping)) {
    if (lower.includes(key.toLowerCase())) return val
  }
  return ''
}

async function parseIntent() {
  if (intentParsed.value || !videoParamsStore.videoSubject.trim()) return
  parsingIntent.value = true
  try {
    const result = await generateIntent(
      videoParamsStore.videoSubject,
      videoParamsStore.videoLanguage !== 'auto' ? videoParamsStore.videoLanguage : undefined,
      videoParamsStore.videoAspect,
    )
    const mappedStyle = mapIntentField(result.style || '', STYLE_MAP)
    const mappedMood = mapIntentField(result.mood || '', MOOD_MAP)
    if (mappedStyle) videoParamsStore.intentStyle = mappedStyle
    if (mappedMood) videoParamsStore.intentMood = mappedMood
    if (result.audience) videoParamsStore.intentAudience = result.audience
    if (result.language && videoParamsStore.videoLanguage === 'auto') {
      const langMap: Record<string, string> = { '中文': 'zh-CN', '英文': 'en-US', 'English': 'en-US' }
      const mapped = langMap[result.language]
      if (mapped) videoParamsStore.videoLanguage = mapped
    }
    if (result.platform) {
      const platformMap: Record<string, string> = {
        '抖音': 'douyin', '视频号': 'weixin', '小红书': 'xiaohongshu',
        'B站': 'bilibili', 'TikTok': 'tiktok', 'YouTube': 'youtube',
        'Instagram': 'instagram', 'Facebook': 'facebook',
      }
      const mapped = platformMap[result.platform]
      if (mapped && !selectedPlatforms.value.includes(mapped)) {
        selectedPlatforms.value.push(mapped)
        if (!socialPlatform.value) socialPlatform.value = mapped
      }
    }
    intentParsed.value = true
  } catch {
    // Intent parsing is best-effort, don't block the user
  } finally {
    parsingIntent.value = false
  }
}

async function generateScriptAuto() {
  if (videoParamsStore.videoScript.trim()) return
  if (!videoParamsStore.videoSubject.trim()) return
  generatingScript.value = true
  try {
    const result = await generateScript(
      videoParamsStore.videoSubject,
      videoParamsStore.videoLanguage !== 'auto' ? videoParamsStore.videoLanguage : undefined,
      videoParamsStore.paragraphNumber,
      videoParamsStore.videoScriptPrompt || undefined,
      videoParamsStore.useCustomSystemPrompt ? videoParamsStore.customSystemPrompt : undefined,
      videoParamsStore.useCustomSystemPrompt,
      {
        intentStyle: videoParamsStore.intentStyle || undefined,
        intentMood: videoParamsStore.intentMood || undefined,
        intentAudience: videoParamsStore.intentAudience || undefined,
      },
    )
    if (result.script) videoParamsStore.videoScript = result.script
    if (result.terms && result.terms.length > 0) {
      videoParamsStore.videoTerms = result.terms.join(',')
    }
    if (taskStore.draftTaskId) {
      const saveData: Record<string, unknown> = {
        video_script: result.script,
      }
      if (result.script) saveData.script = result.script
      if (result.terms && result.terms.length > 0) {
        saveData.video_terms = result.terms
        saveData.terms = result.terms
      }
      await taskStore.saveStepConfig(saveData as any)
    }
  } catch {
    // Script generation is best-effort
  } finally {
    generatingScript.value = false
  }
}

async function generateStoryboardAuto() {
  if (videoParamsStore.storyboardParsed || !videoParamsStore.videoScript.trim()) return
  generatingStoryboard.value = true
  try {
    const scenes = await generateStoryboard(
      videoParamsStore.videoScript,
      videoParamsStore.videoSubject,
      videoParamsStore.videoClipDuration,
      videoParamsStore.intentStyle,
      videoParamsStore.intentMood,
    )
    videoParamsStore.storyboard = scenes
    videoParamsStore.storyboardParsed = true
    if (scenes.length > 0 && !videoParamsStore.videoTerms.trim()) {
      const prompts = scenes
        .map(s => s.visual_prompt)
        .filter(Boolean)
        .join(',')
      if (prompts) videoParamsStore.videoTerms = prompts
    }
    if (taskStore.draftTaskId) {
      const prompts = scenes.map(s => s.visual_prompt).filter(Boolean)
      await taskStore.saveStepConfig({
        storyboard: scenes,
        terms: prompts.length > 0 ? prompts : undefined,
      } as any)
    }
  } catch {
    // Storyboard generation is best-effort
  } finally {
    generatingStoryboard.value = false
  }
}

async function onGenerateTerms() {
  if (!videoParamsStore.videoScript.trim()) {
    ElMessage.warning(t('wizard.validate.scriptRequired'))
    return
  }
  generatingTerms.value = true
  try {
    const result = await generateTerms(
      videoParamsStore.videoScript,
      videoParamsStore.videoSubject || undefined,
      videoParamsStore.videoLanguage !== 'auto' ? videoParamsStore.videoLanguage : undefined,
    )
    if (result.terms && result.terms.length > 0) {
      videoParamsStore.videoTerms = result.terms.join(',')
      if (taskStore.draftTaskId) {
        await taskStore.saveStepConfig({
          video_terms: result.terms,
          terms: result.terms,
        } as any)
      }
      ElMessage.success(t('common.success'))
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    ElMessage.error(msg)
  } finally {
    generatingTerms.value = false
  }
}

async function onGenerateNarration() {
  if (!taskStore.draftTaskId) {
    ElMessage.warning('请先完成前序步骤')
    return
  }
  if (!videoParamsStore.videoScript.trim()) {
    ElMessage.warning(t('wizard.validate.scriptRequired'))
    return
  }
  generatingNarration.value = true
  try {
    const result = await generateNarration(taskStore.draftTaskId)
    if (result.narration) {
      videoParamsStore.narration = result.narration
      ElMessage.success('旁白文案生成成功')
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    ElMessage.error(msg)
  } finally {
    generatingNarration.value = false
  }
}

async function fetchMaterialsAuto() {
  if (videoParamsStore.materialsFetched || videoParamsStore.videoSource === 'local') return
  if (!taskStore.draftTaskId) return
  if (!videoParamsStore.videoTerms.trim()) {
    return
  }
  videoParamsStore.fetchingMaterials = true
  try {
    const result = await taskStore.fetchDraftMaterials()
    if (result && result.materials) {
      videoParamsStore.materialsList = result.materials
      videoParamsStore.materialsFetched = true
    }
  } catch {
    // Material fetch is best-effort, don't block the user
  } finally {
    videoParamsStore.fetchingMaterials = false
  }
}

async function generateAudioAuto() {
  if (!taskStore.draftTaskId) return
  if (videoParamsStore.voiceName === 'no-voice') return
  generatingAudio.value = true
  try {
    const result = await generateAudio(taskStore.draftTaskId)
    if (result.audioFile) {
      videoParamsStore.audioFile = result.audioFile
      videoParamsStore.audioDuration = result.audioDuration
      ElMessage.success(t('wizard.step6.audioGenerated'))
    }
  } catch {
    // Audio generation is best-effort
  } finally {
    generatingAudio.value = false
  }
}

async function onNext() {
  if (!validateStep(currentStep.value)) return
  savingStep.value = true
  try {
    if (currentStep.value === 0) {
      if (!taskStore.draftTaskId) {
        const params = videoParamsStore.toVideoParams()
        await taskStore.initDraftTask(params)
      }
      await parseIntent()
      if (taskStore.draftTaskId) {
        const stepParams = videoParamsStore.collectStepParams(0) as Partial<VideoParams>
        await taskStore.saveStepConfig(stepParams)
      }
      await generateScriptAuto()
    }
    if (currentStep.value === 1) {
      await generateStoryboardAuto()
    }
    if (currentStep.value === 2) {
      if (!videoParamsStore.videoTerms.trim()) {
        await onGenerateTerms()
      }
    }
    if (currentStep.value === 3) {
      if (!videoParamsStore.narration.trim()) {
        await onGenerateNarration()
      }
      if (taskStore.draftTaskId && videoParamsStore.narration.trim()) {
        await taskStore.saveStepConfig({ narration: videoParamsStore.narration } as any)
      }
    }
    if (currentStep.value === 4) {
      if (taskStore.draftTaskId) {
        const stepParams = videoParamsStore.collectStepParams(4) as Partial<VideoParams>
        await taskStore.saveStepConfig(stepParams)
      }
      await fetchMaterialsAuto()
    }
    if (currentStep.value === 5) {
      await generateAudioAuto()
    }
    if (taskStore.draftTaskId && currentStep.value !== 3 && currentStep.value !== 4) {
      const stepParams = videoParamsStore.collectStepParams(currentStep.value) as Partial<VideoParams> & {
        script?: string
        terms?: string[]
        storyboard?: StoryboardScene[]
      }
      if (currentStep.value === 1 && videoParamsStore.videoScript) {
        stepParams.script = videoParamsStore.videoScript
      }
      if (currentStep.value === 2 && videoParamsStore.storyboard.length > 0) {
        stepParams.storyboard = videoParamsStore.storyboard
        const prompts = videoParamsStore.storyboard.map(s => s.visual_prompt).filter(Boolean)
        if (prompts.length > 0) stepParams.terms = prompts
      }
      if (currentStep.value === 2 && videoParamsStore.videoTerms) {
        const termsList = videoParamsStore.videoTerms.split(/[,，]/).map(t => t.trim()).filter(Boolean)
        if (termsList.length > 0) {
          stepParams.terms = termsList
          stepParams.video_terms = termsList
        }
      }
      await taskStore.saveStepConfig(stepParams)
    }
    if (currentStep.value < 8) currentStep.value++
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    ElMessage.error(msg)
  } finally {
    savingStep.value = false
  }
}

function onPrev() {
  if (currentStep.value > 0) currentStep.value--
}

function togglePlatform(value: string) {
  const idx = selectedPlatforms.value.indexOf(value)
  if (idx === -1) {
    selectedPlatforms.value.push(value)
    if (!socialPlatform.value) socialPlatform.value = value
  } else {
    selectedPlatforms.value.splice(idx, 1)
    if (socialPlatform.value === value) {
      socialPlatform.value = selectedPlatforms.value[0] || ''
    }
  }
}

async function onGenerateSocial() {
  if (!videoParamsStore.videoScript) {
    ElMessage.warning(t('social.needScript'))
    return
  }
  if (!socialPlatform.value) {
    ElMessage.warning(t('wizard.step9.selectPlatform'))
    return
  }
  generatingSocial.value = true
  try {
    const data = await generateSocialMetadata(
      videoParamsStore.videoScript,
      videoParamsStore.videoSubject,
      socialPlatform.value,
    )
    publishTitle.value = data.title || ''
    publishDesc.value = data.description || ''
    publishTags.value = (data.tags || []).join(' ')
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    ElMessage.error(msg)
  } finally {
    generatingSocial.value = false
  }
}

async function onPublish() {
  if (selectedPlatforms.value.length === 0) {
    ElMessage.warning(t('wizard.step9.selectPlatform'))
    return
  }
  publishing.value = true
  try {
    ElMessage.info(t('wizard.step9.publishing'))
  } finally {
    publishing.value = false
  }
}

async function onGenerate() {
  try {
    if (taskStore.draftTaskId) {
      await taskStore.launchFromDraft()
    } else {
      const params = videoParamsStore.toVideoParams()
      await taskStore.startGeneration(params)
    }
    ElMessage.success(t('generate.success'))
  } catch {
    ElMessage.error(t('generate.failed'))
  }
}
</script>

<style scoped>
.step-wizard {
  max-width: 900px;
  margin: 0 auto;
}

.wizard-steps {
  margin-bottom: 20px;
  padding: 0 8px;
}

.wizard-body {
  min-height: 360px;
}

.step-content {
  padding: 0 4px;
}

.step-header {
  margin-bottom: 16px;
  text-align: center;
}

.step-header h3 {
  margin: 0 0 6px;
  font-size: 20px;
  color: var(--soma-text);
}

.step-desc {
  color: var(--soma-text-dim);
  font-size: 14px;
  margin: 0;
}

.step-panel {
  padding: 0 8px;
}

.intent-parsing-hint {
  text-align: center;
  margin-top: 12px;
  color: var(--soma-accent);
  font-size: 13px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.intent-parsed-hint {
  text-align: center;
  margin-top: 12px;
  color: var(--soma-success);
  font-size: 13px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.storyboard-list {
  max-height: 400px;
  overflow-y: auto;
}

.storyboard-item {
  padding: 8px 0;
  border-bottom: 1px solid var(--soma-line);
}

.storyboard-item:last-child {
  border-bottom: none;
}

.scene-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.scene-duration {
  color: var(--soma-text-dim);
  font-size: 12px;
}

.scene-camera {
  color: var(--soma-accent);
  font-size: 12px;
}

.scene-transition {
  color: var(--soma-success);
  font-size: 12px;
}

.scene-narration {
  font-size: 13px;
  color: var(--soma-text);
  margin-bottom: 2px;
}

.scene-visual-desc {
  font-size: 12px;
  color: var(--soma-text-dim);
  margin-bottom: 2px;
}

.scene-visual-prompt {
  font-size: 11px;
  color: var(--soma-text-dim);
  font-style: italic;
  word-break: break-all;
}

.review-summary {
  margin-bottom: 20px;
}

.generate-section {
  text-align: center;
  margin: 20px 0;
}

.generate-section .el-button {
  min-width: 220px;
  height: 52px;
  font-size: 16px;
  border-radius: 26px;
}

.publish-platforms {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
}

.platform-card {
  cursor: pointer;
  transition: all 0.2s ease;
  border: 2px solid transparent;
}

.platform-card:hover {
  border-color: var(--soma-accent);
}

.platform-selected {
  border-color: var(--soma-success);
  background: rgba(52, 211, 153, 0.12);
}

.platform-inner {
  padding: 0;
}

.platform-icon {
  font-size: 18px;
  margin-right: 4px;
}

.platform-name {
  font-weight: 500;
}

.platform-hint {
  color: var(--soma-text-dim);
  font-size: 12px;
  margin: 4px 0 0;
}

.publish-actions {
  text-align: center;
  margin-top: 16px;
}

.publish-actions .el-button {
  min-width: 180px;
  height: 44px;
  font-size: 15px;
  border-radius: 22px;
}

.wizard-footer {
  display: flex;
  align-items: center;
  margin-top: 24px;
  padding: 16px 8px;
  border-top: 1px solid var(--soma-line);
}

.spacer {
  flex: 1;
}

.wizard-fade-enter-active,
.wizard-fade-leave-active {
  transition: opacity 0.25s ease, transform 0.25s ease;
}

.wizard-fade-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.wizard-fade-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}
</style>
