<template>
  <el-card class="panel-card" shadow="hover">
    <template #header>
      <span>{{ $t('script.title') }}</span>
    </template>
    <div class="form-row">
      <div class="form-label">{{ $t('script.videoSubject') }}</div>
      <el-input
        v-model="store.videoSubject"
        :placeholder="$t('script.videoSubjectPlaceholder')"
        clearable
      />
    </div>
    <div class="form-row">
      <div class="form-label">{{ $t('script.scriptLanguage') }}</div>
      <el-select v-model="store.videoLanguage" style="width: 100%">
        <el-option :label="$t('script.autoDetect')" value="auto" />
        <el-option label="简体中文" value="zh-CN" />
        <el-option label="繁體中文(香港)" value="zh-HK" />
        <el-option label="繁體中文(台灣)" value="zh-TW" />
        <el-option label="Deutsch" value="de-DE" />
        <el-option label="English" value="en-US" />
        <el-option label="Français" value="fr-FR" />
        <el-option label="Русский" value="ru-RU" />
        <el-option label="Tiếng Việt" value="vi-VN" />
        <el-option label="ภาษาไทย" value="th-TH" />
        <el-option label="Türkçe" value="tr-TR" />
      </el-select>
    </div>
    <el-collapse>
      <el-collapse-item :title="$t('script.advancedSettings')">
        <ScriptAdvancedSettings />
      </el-collapse-item>
    </el-collapse>
    <div style="margin-top: 12px">
      <el-button
        type="primary"
        :loading="generatingScript"
        @click="onGenerateScriptAndKeywords"
        style="width: 100%"
      >
        {{ $t('script.generateScriptAndKeywords') }}
      </el-button>
    </div>
    <div class="form-row" style="margin-top: 12px">
      <div class="form-label">{{ $t('script.videoScript') }}</div>
      <el-input
        v-model="store.videoScript"
        type="textarea"
        :rows="8"
        :placeholder="$t('script.videoScriptPlaceholder')"
      />
    </div>
    <div style="margin-top: 8px">
      <el-button
        :loading="generatingTerms"
        @click="onGenerateKeywords"
        style="width: 100%"
      >
        {{ $t('script.generateKeywords') }}
      </el-button>
    </div>
    <div class="form-row" style="margin-top: 8px">
      <div class="form-label">{{ $t('script.videoKeywords') }}</div>
      <el-input
        v-model="store.videoTerms"
        type="textarea"
        :rows="3"
        :placeholder="$t('script.videoKeywordsPlaceholder')"
      />
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { useVideoParamsStore } from '@/stores/videoParams'
import { useTaskStore } from '@/stores/task'
import { useI18n } from 'vue-i18n'
import ScriptAdvancedSettings from './ScriptAdvancedSettings.vue'
import { generateScript, generateTerms } from '@/api/llm'

const store = useVideoParamsStore()
const taskStore = useTaskStore()
const { t } = useI18n()
const generatingScript = ref(false)
const generatingTerms = ref(false)

async function onGenerateScriptAndKeywords() {
  if (!store.videoSubject) {
    ElMessage.warning(t('generate.validate.subjectRequired'))
    return
  }
  generatingScript.value = true
  try {
    const result = await generateScript(
      store.videoSubject,
      store.videoLanguage !== 'auto' ? store.videoLanguage : undefined,
      store.paragraphNumber,
      store.videoScriptPrompt || undefined,
      store.useCustomSystemPrompt ? store.customSystemPrompt : undefined,
      store.useCustomSystemPrompt,
      {
        intentStyle: store.intentStyle || undefined,
        intentMood: store.intentMood || undefined,
        intentAudience: store.intentAudience || undefined,
      }
    )
    if (result.script) store.videoScript = result.script
    if (result.terms && result.terms.length > 0) store.videoTerms = result.terms.join(',')
    if (taskStore.draftTaskId) {
      const saveData: Record<string, unknown> = {
        video_script: result.script || store.videoScript,
        video_terms: result.terms && result.terms.length > 0 ? result.terms : undefined,
      }
      if (result.script) saveData.script = result.script
      if (result.terms && result.terms.length > 0) saveData.terms = result.terms
      await taskStore.saveStepConfig(saveData as any)
    }
    ElMessage.success(t('common.success'))
  } catch (e) {
    ElMessage.error(`${t('common.error')}: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    generatingScript.value = false
  }
}

async function onGenerateKeywords() {
  if (!store.videoScript) {
    ElMessage.warning(t('generate.validate.subjectRequired'))
    return
  }
  generatingTerms.value = true
  try {
    const result = await generateTerms(
      store.videoScript,
      store.videoSubject || undefined,
      store.videoLanguage !== 'auto' ? store.videoLanguage : undefined
    )
    if (result.terms) store.videoTerms = result.terms.join(',')
    if (taskStore.draftTaskId && result.terms) {
      await taskStore.saveStepConfig({
        video_terms: result.terms,
        terms: result.terms,
      } as any)
    }
    ElMessage.success(t('common.success'))
  } catch (e) {
    ElMessage.error(`${t('common.error')}: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    generatingTerms.value = false
  }
}
</script>
