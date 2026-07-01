<template>
  <el-dialog
    v-model="visible"
    :title="$t('guide.title')"
    width="720px"
    top="5vh"
    :close-on-click-modal="false"
    destroy-on-close
    class="guide-dialog"
  >
    <div class="guide-content">
      <el-steps :active="activeStep" align-center class="guide-steps">
        <el-step :title="$t('guide.step1.short')" />
        <el-step :title="$t('guide.step2.short')" />
        <el-step :title="$t('guide.step3.short')" />
        <el-step :title="$t('guide.step4.short')" />
        <el-step :title="$t('guide.step5.short')" />
      </el-steps>

      <div class="guide-step-content">
        <!-- Step 1: Welcome -->
        <div v-if="activeStep === 0" class="step-panel">
          <h3>{{ $t('guide.step1.title') }}</h3>
          <p class="step-desc">{{ $t('guide.step1.desc') }}</p>
          <div class="flow-chart">
            <div class="flow-row">
              <div class="flow-item primary">📝 {{ $t('guide.flow.script') }}</div>
              <div class="flow-arrow">→</div>
              <div class="flow-item success">🔊 {{ $t('guide.flow.tts') }}</div>
              <div class="flow-arrow">→</div>
              <div class="flow-item warning">🎬 {{ $t('guide.flow.stock') }}</div>
            </div>
            <div class="flow-row" style="margin-top: 12px">
              <div class="flow-item info">📄 {{ $t('guide.flow.subtitle') }}</div>
              <div class="flow-arrow">+</div>
              <div class="flow-item danger">🎵 {{ $t('guide.flow.bgm') }}</div>
              <div class="flow-arrow">→</div>
              <div class="flow-item primary">🎞️ {{ $t('guide.flow.compose') }}</div>
            </div>
          </div>
          <el-alert type="info" :closable="false" show-icon style="margin-top: 16px">
            <template #title>{{ $t('guide.step1.prereqTitle') }}</template>
            <ul class="prereq-list">
              <li>Rust (Edition 2021)</li>
              <li>Node.js 18+</li>
              <li>FFmpeg — <code>brew install ffmpeg</code> / <code>apt install ffmpeg</code></li>
            </ul>
          </el-alert>
        </div>

        <!-- Step 2: LLM Config -->
        <div v-if="activeStep === 1" class="step-panel">
          <h3>{{ $t('guide.step2.title') }}</h3>
          <p class="step-desc">{{ $t('guide.step2.desc') }}</p>

          <el-alert type="success" :closable="false" show-icon style="margin-bottom: 16px">
            <template #title>{{ $t('guide.step2.freeTitle') }}</template>
          </el-alert>

          <el-table :data="llmProviders" stripe size="small" class="provider-table">
            <el-table-column prop="name" label="Provider" width="130" />
            <el-table-column prop="needKey" :label="$t('guide.col.needKey')" width="80" align="center">
              <template #default="{ row }">
                <el-tag :type="row.needKey ? 'danger' : 'success'" size="small">
                  {{ row.needKey ? $t('guide.yes') : $t('guide.free') }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="defaultModel" :label="$t('guide.col.model')" width="150" />
            <el-table-column :label="$t('guide.col.applyUrl')">
              <template #default="{ row }">
                <a v-if="row.url" :href="row.url" target="_blank" class="guide-link">{{ row.urlText }}</a>
                <span v-else>{{ $t('guide.localInstall') }}</span>
              </template>
            </el-table-column>
          </el-table>

          <div class="step-tip">
            <el-icon><InfoFilled /></el-icon>
            <span>{{ $t('guide.step2.tip') }}</span>
          </div>
        </div>

        <!-- Step 3: TTS & Stock -->
        <div v-if="activeStep === 2" class="step-panel">
          <h3>{{ $t('guide.step3.title') }}</h3>

          <h4 style="margin-bottom: 8px">🔊 TTS</h4>
          <p class="step-desc">{{ $t('guide.step3.ttsDesc') }}</p>
          <el-table :data="ttsProviders" stripe size="small" class="provider-table">
            <el-table-column prop="name" label="Provider" width="150" />
            <el-table-column prop="needKey" :label="$t('guide.col.needKey')" width="80" align="center">
              <template #default="{ row }">
                <el-tag :type="row.needKey ? 'danger' : 'success'" size="small">
                  {{ row.needKey ? $t('guide.yes') : $t('guide.free') }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column :label="$t('guide.col.applyUrl')">
              <template #default="{ row }">
                <a v-if="row.url" :href="row.url" target="_blank" class="guide-link">{{ row.urlText }}</a>
                <span v-else>{{ $t('guide.builtin') }}</span>
              </template>
            </el-table-column>
          </el-table>

          <h4 style="margin: 16px 0 8px">🎬 {{ $t('guide.step3.stockTitle') }}</h4>
          <p class="step-desc">{{ $t('guide.step3.stockDesc') }}</p>
          <el-table :data="stockProviders" stripe size="small" class="provider-table">
            <el-table-column prop="name" label="Provider" width="120" />
            <el-table-column prop="needKey" :label="$t('guide.col.needKey')" width="80" align="center">
              <template #default="{ row }">
                <el-tag :type="row.needKey ? 'danger' : 'success'" size="small">
                  {{ row.needKey ? $t('guide.yes') : $t('guide.free') }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column :label="$t('guide.col.applyUrl')">
              <template #default="{ row }">
                <a v-if="row.url" :href="row.url" target="_blank" class="guide-link">{{ row.urlText }}</a>
                <span v-else>{{ $t('guide.localUpload') }}</span>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <!-- Step 4: Settings walkthrough -->
        <div v-if="activeStep === 3" class="step-panel">
          <h3>{{ $t('guide.step4.title') }}</h3>
          <p class="step-desc">{{ $t('guide.step4.desc') }}</p>

          <el-timeline>
            <el-timeline-item type="primary" :hollow="false">
              <strong>{{ $t('guide.step4.navSettings') }}</strong>
              <p>{{ $t('guide.step4.navSettingsDesc') }}</p>
            </el-timeline-item>
            <el-timeline-item type="primary" :hollow="false">
              <strong>{{ $t('guide.step4.setLlm') }}</strong>
              <p>{{ $t('guide.step4.setLlmDesc') }}</p>
            </el-timeline-item>
            <el-timeline-item type="primary" :hollow="false">
              <strong>{{ $t('guide.step4.setStock') }}</strong>
              <p>{{ $t('guide.step4.setStockDesc') }}</p>
            </el-timeline-item>
            <el-timeline-item type="primary" :hollow="false">
              <strong>{{ $t('guide.step4.setTts') }}</strong>
              <p>{{ $t('guide.step4.setTtsDesc') }}</p>
            </el-timeline-item>
            <el-timeline-item type="success" :hollow="false">
              <strong>{{ $t('guide.step4.save') }}</strong>
              <p>{{ $t('guide.step4.saveDesc') }}</p>
            </el-timeline-item>
          </el-timeline>

          <el-alert type="warning" :closable="false" show-icon style="margin-top: 12px">
            <template #title>{{ $t('guide.step4.homeTipTitle') }}</template>
            {{ $t('guide.step4.homeTip') }}
          </el-alert>
        </div>

        <!-- Step 5: Generate -->
        <div v-if="activeStep === 4" class="step-panel">
          <h3>{{ $t('guide.step5.title') }}</h3>
          <p class="step-desc">{{ $t('guide.step5.desc') }}</p>

          <el-timeline>
            <el-timeline-item type="primary">
              <strong>1. {{ $t('guide.step5.inputTopic') }}</strong>
              <p>{{ $t('guide.step5.inputTopicDesc') }}</p>
            </el-timeline-item>
            <el-timeline-item type="primary">
              <strong>2. {{ $t('guide.step5.genScript') }}</strong>
              <p>{{ $t('guide.step5.genScriptDesc') }}</p>
            </el-timeline-item>
            <el-timeline-item type="primary">
              <strong>3. {{ $t('guide.step5.adjustParams') }}</strong>
              <p>{{ $t('guide.step5.adjustParamsDesc') }}</p>
            </el-timeline-item>
            <el-timeline-item type="primary">
              <strong>4. {{ $t('guide.step5.clickGenerate') }}</strong>
              <p>{{ $t('guide.step5.clickGenerateDesc') }}</p>
            </el-timeline-item>
            <el-timeline-item type="success">
              <strong>5. {{ $t('guide.step5.waitResult') }}</strong>
              <p>{{ $t('guide.step5.waitResultDesc') }}</p>
            </el-timeline-item>
          </el-timeline>

          <el-result icon="success" :title="$t('guide.step5.readyTitle')" :sub-title="$t('guide.step5.readySub')" style="margin-top: 12px" />
        </div>
      </div>
    </div>

    <template #footer>
      <div class="guide-footer">
        <el-checkbox v-if="activeStep === 4" v-model="dontShowAgain">{{ $t('guide.dontShowAgain') }}</el-checkbox>
        <div class="footer-buttons">
          <el-button v-if="activeStep > 0" @click="activeStep--">{{ $t('guide.prev') }}</el-button>
          <el-button v-if="activeStep < 4" type="primary" @click="activeStep++">{{ $t('guide.next') }}</el-button>
          <el-button v-if="activeStep === 4" type="success" @click="onClose">{{ $t('guide.start') }}</el-button>
        </div>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { InfoFilled } from '@element-plus/icons-vue'

const { t } = useI18n()

const visible = ref(false)
const activeStep = ref(0)
const dontShowAgain = ref(false)

const llmProviders = computed(() => [
  { name: 'Ollama', needKey: false, defaultModel: 'llama3', url: 'https://ollama.com', urlText: 'ollama.com' },
  { name: 'Pollinations', needKey: false, defaultModel: 'openai', url: 'https://github.com/pollinations/pollinations', urlText: 'GitHub' },
  { name: 'G4F', needKey: false, defaultModel: 'gpt-4o-mini', url: '', urlText: '' },
  { name: 'DeepSeek', needKey: true, defaultModel: 'deepseek-chat', url: 'https://platform.deepseek.com/signup', urlText: 'platform.deepseek.com' },
  { name: 'Groq', needKey: true, defaultModel: 'llama-3.1-8b', url: 'https://console.groq.com/keys', urlText: 'console.groq.com' },
  { name: 'Qwen', needKey: true, defaultModel: 'qwen-turbo', url: 'https://dashscope.console.aliyun.com/', urlText: 'dashscope.console.aliyun.com' },
  { name: 'Gemini', needKey: true, defaultModel: 'gemini-2.0-flash', url: 'https://aistudio.google.com/apikey', urlText: 'aistudio.google.com' },
  { name: 'OpenAI', needKey: true, defaultModel: 'gpt-4o-mini', url: 'https://platform.openai.com/signup', urlText: 'platform.openai.com' },
  { name: 'Moonshot', needKey: true, defaultModel: 'moonshot-v1-8k', url: 'https://platform.moonshot.cn/', urlText: 'platform.moonshot.cn' },
  { name: 'Zhipu', needKey: true, defaultModel: 'glm-4-flash', url: 'https://bigmodel.cn/', urlText: 'bigmodel.cn' },
  { name: 'Doubao', needKey: true, defaultModel: 'doubao-pro-32k', url: 'https://www.volcengine.com/product/doubao', urlText: 'volcengine.com' },
  { name: 'Hunyuan', needKey: true, defaultModel: 'hunyuan-turbo', url: 'https://hunyuan.tencent.com/', urlText: 'hunyuan.tencent.com' },
  { name: 'Wenxin', needKey: true, defaultModel: 'ernie-4.0-8k', url: 'https://qianfan.baidubce.com/', urlText: 'qianfan.baidubce.com' },
  { name: 'Xunfei', needKey: true, defaultModel: 'generalv3.5', url: 'https://www.xfyun.cn/', urlText: 'xfyun.cn' },
])

const ttsProviders = computed(() => [
  { name: 'Edge TTS', needKey: false, url: '', urlText: '' },
  { name: 'Azure Speech', needKey: true, url: 'https://portal.azure.com', urlText: 'portal.azure.com' },
  { name: 'SiliconFlow', needKey: true, url: 'https://siliconflow.cn/', urlText: 'siliconflow.cn' },
  { name: 'ElevenLabs', needKey: true, url: 'https://elevenlabs.io/app/sign-up', urlText: 'elevenlabs.io' },
  { name: 'MiMo TTS', needKey: true, url: 'https://mimo.xiaomi.com/', urlText: 'mimo.xiaomi.com' },
  { name: 'Gemini TTS', needKey: true, url: 'https://aistudio.google.com/apikey', urlText: 'aistudio.google.com' },
])

const stockProviders = computed(() => [
  { name: 'Pexels', needKey: true, url: 'https://www.pexels.com/api/', urlText: 'pexels.com/api' },
  { name: 'Pixabay', needKey: true, url: 'https://pixabay.com/api/docs/', urlText: 'pixabay.com/api' },
  { name: 'Coverr', needKey: true, url: 'https://coverr.co/api', urlText: 'coverr.co/api' },
  { name: t('video.local'), needKey: false, url: '', urlText: '' },
])

function open() {
  activeStep.value = 0
  dontShowAgain.value = false
  visible.value = true
}

function onClose() {
  if (dontShowAgain.value) {
    localStorage.setItem('soma-guide-dismissed', 'true')
  }
  visible.value = false
}

function shouldShowOnFirstUse(): boolean {
  return localStorage.getItem('soma-guide-dismissed') !== 'true'
}

defineExpose({ open, shouldShowOnFirstUse })
</script>

<style scoped>
.guide-dialog :deep(.el-dialog__body) {
  padding: 16px 20px;
  max-height: 65vh;
  overflow-y: auto;
}

.guide-steps {
  margin-bottom: 20px;
}

.step-panel {
  padding: 0 8px;
}

.step-panel h3 {
  margin: 0 0 8px;
  font-size: 18px;
  color: #303133;
}

.step-panel h4 {
  font-size: 15px;
  color: #303133;
}

.step-desc {
  color: #606266;
  line-height: 1.6;
  margin: 0 0 12px;
  font-size: 14px;
}

.flow-chart {
  background: #f5f7fa;
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.flow-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: center;
}

.flow-item {
  padding: 8px 14px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  color: #fff;
}

.flow-item.primary { background: #409eff; }
.flow-item.success { background: #67c23a; }
.flow-item.warning { background: #e6a23c; }
.flow-item.info    { background: #909399; }
.flow-item.danger  { background: #f56c6c; }

.flow-arrow {
  font-size: 18px;
  color: #909399;
  font-weight: bold;
}

.prereq-list {
  margin: 4px 0 0;
  padding-left: 18px;
  line-height: 1.8;
}

.prereq-list code {
  background: #f0f2f5;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 12px;
}

.provider-table {
  margin-bottom: 12px;
}

.guide-link {
  color: #409eff;
  text-decoration: none;
  font-size: 12px;
  word-break: break-all;
}

.guide-link:hover {
  text-decoration: underline;
}

.step-tip {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #909399;
  font-size: 13px;
  margin-top: 8px;
}

.step-panel :deep(.el-timeline-item__content p) {
  color: #606266;
  font-size: 13px;
  margin: 4px 0 0;
}

.guide-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.footer-buttons {
  display: flex;
  gap: 8px;
}
</style>
