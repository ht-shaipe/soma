<template>
  <header class="app-header">
    <div class="header-left">
      <span>{{ $t('app.title') }} v0.1.0</span>
    </div>
    <div class="header-right">
      <el-button :icon="QuestionFilled" circle size="small" @click="$emit('open-guide')" />
      <span class="lang-label">{{ $t('common.language') }}</span>
      <el-select v-model="currentLang" size="small" style="width: 160px" @change="onLangChange">
        <el-option label="zh-CN - 简体中文" value="zh-CN" />
        <el-option label="en-US - English" value="en-US" />
      </el-select>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useConfigStore } from '@/stores/config'
import { QuestionFilled } from '@element-plus/icons-vue'

const { locale } = useI18n()
const configStore = useConfigStore()

defineEmits(['open-guide'])

const currentLang = ref(configStore.uiLanguage)

watch(() => configStore.uiLanguage, (val) => {
  currentLang.value = val
})

function onLangChange(lang: string) {
  locale.value = lang
  configStore.uiLanguage = lang
  localStorage.setItem('ui-language', lang)
}
</script>
