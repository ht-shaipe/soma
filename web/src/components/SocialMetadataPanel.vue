<template>
  <el-card class="panel-card" shadow="hover">
    <template #header>
      <span>{{ $t('social.title') }}</span>
    </template>
    <div class="form-row">
      <div class="form-label">{{ $t('social.platform') }}</div>
      <el-select v-model="platform" style="width: 100%">
        <el-option label="TikTok" value="tiktok" />
        <el-option label="YouTube Shorts" value="youtube" />
        <el-option label="Instagram Reels" value="instagram" />
        <el-option label="Facebook Reels" value="facebook" />
        <el-option :label="$t('social.xiaohongshu')" value="xiaohongshu" />
      </el-select>
    </div>
    <div class="form-row">
      <el-button type="primary" :loading="generating" @click="onGenerate" :disabled="!store.videoScript">
        {{ generating ? $t('social.generating') : $t('social.generate') }}
      </el-button>
    </div>
    <template v-if="result">
      <div class="form-row">
        <div class="form-label">{{ $t('social.videoTitle') }}</div>
        <el-input v-model="result.title" type="textarea" :rows="2" />
      </div>
      <div class="form-row">
        <div class="form-label">{{ $t('social.description') }}</div>
        <el-input v-model="result.description" type="textarea" :rows="3" />
      </div>
      <div class="form-row">
        <div class="form-label">{{ $t('social.tags') }}</div>
        <el-input v-model="tagsText" type="textarea" :rows="2" />
      </div>
      <div class="form-row">
        <el-button size="small" @click="onCopy">
          {{ $t('common.copy') }}
        </el-button>
      </div>
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { useVideoParamsStore } from '@/stores/videoParams'
import { generateSocialMetadata } from '@/api/llm'
import { useI18n } from 'vue-i18n'

const store = useVideoParamsStore()
const { t } = useI18n()

const platform = ref('tiktok')
const generating = ref(false)
const result = ref<{ title: string; description: string; tags: string[] } | null>(null)

const tagsText = computed({
  get: () => result.value?.tags?.join(', ') || '',
  set: (val: string) => { if (result.value) result.value.tags = val.split(',').map(s => s.trim()).filter(Boolean) },
})

async function onGenerate() {
  if (!store.videoScript) {
    ElMessage.warning(t('social.needScript'))
    return
  }
  generating.value = true
  try {
    const data = await generateSocialMetadata(store.videoScript, store.videoSubject, platform.value)
    result.value = data
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    ElMessage.error(msg)
  } finally {
    generating.value = false
  }
}

function onCopy() {
  if (!result.value) return
  const text = `${result.value.title}\n\n${result.value.description}\n\n${result.value.tags.join(' ')}`
  navigator.clipboard.writeText(text).then(() => {
    ElMessage.success(t('common.copied'))
  })
}
</script>
