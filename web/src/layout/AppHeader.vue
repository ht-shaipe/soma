<template>
  <header class="app-header">
    <div class="header-left">
      <span class="page-title">{{ pageTitle }}</span>
      <span class="page-subtitle">{{ pageSubtitle }}</span>
    </div>
    <div class="header-right">
      <el-tooltip :content="$t('header.theme')" placement="bottom">
        <el-button :icon="isDark ? Moon : Sunny" circle size="small" @click="toggleTheme" />
      </el-tooltip>
      <span class="lang-label">{{ $t('common.language') }}</span>
      <el-select v-model="currentLang" size="small" style="width: 158px" @change="onLangChange">
        <el-option label="简体中文" value="zh-CN" />
        <el-option label="English" value="en-US" />
      </el-select>
      <el-button :icon="QuestionFilled" circle size="small" @click="$emit('open-guide')" />
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { useConfigStore } from '@/stores/config'
import { QuestionFilled, Moon, Sunny } from '@element-plus/icons-vue'

const { t, locale } = useI18n()
const route = useRoute()
const configStore = useConfigStore()

defineEmits(['open-guide'])

const currentLang = ref(configStore.uiLanguage)
const isDark = ref(document.documentElement.classList.contains('dark'))

watch(() => configStore.uiLanguage, (val) => {
  currentLang.value = val
})

function onLangChange(lang: string) {
  locale.value = lang
  configStore.uiLanguage = lang
  localStorage.setItem('ui-language', lang)
}

const pageTitle = computed(() => {
  const key = (route.meta?.titleKey as string) || 'app.title'
  return t(key)
})

const pageSubtitle = computed(() => {
  const key = route.meta?.subtitleKey as string
  return key ? t(key) : ''
})

function toggleTheme() {
  isDark.value = !isDark.value
  document.documentElement.classList.toggle('dark', isDark.value)
  localStorage.setItem('ui-theme', isDark.value ? 'dark' : 'light')
}
</script>
