<template>
  <div>
    <div class="form-row">
      <div class="form-label">{{ $t('script.paragraphNumber') }}</div>
      <el-slider v-model="store.paragraphNumber" :min="1" :max="10" show-input size="small" />
    </div>
    <div class="form-row">
      <div class="form-label">{{ $t('script.customRequirements') }}</div>
      <el-input
        v-model="store.videoScriptPrompt"
        type="textarea"
        :rows="3"
        :placeholder="$t('script.customRequirementsPlaceholder')"
        maxlength="2000"
        show-word-limit
      />
    </div>
    <div class="form-row">
      <el-checkbox v-model="store.useCustomSystemPrompt">
        {{ $t('script.useCustomSystemPrompt') }}
      </el-checkbox>
    </div>
    <div v-if="store.useCustomSystemPrompt" class="form-row">
      <div class="form-label">{{ $t('script.customSystemPrompt') }}</div>
      <el-input
        v-model="store.customSystemPrompt"
        type="textarea"
        :rows="6"
        maxlength="8000"
        show-word-limit
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useVideoParamsStore } from '@/stores/videoParams'

const store = useVideoParamsStore()
</script>
