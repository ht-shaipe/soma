<template>
  <el-card class="panel-card" shadow="hover">
    <template #header>
      <span>{{ $t('subtitle.title') }}</span>
    </template>
    <div class="form-row">
      <el-checkbox v-model="store.subtitleEnabled">
        {{ $t('subtitle.enabled') }}
      </el-checkbox>
    </div>
    <template v-if="store.subtitleEnabled">
      <div class="form-row">
        <div class="form-label">{{ $t('subtitle.font') }}</div>
        <el-select v-model="store.fontName" style="width: 100%" filterable>
          <el-option v-for="f in fontOptions" :key="f" :label="f" :value="f" />
        </el-select>
      </div>
      <div class="form-row">
        <div class="form-label">{{ $t('subtitle.position') }}</div>
        <el-select v-model="store.subtitlePosition" style="width: 100%">
          <el-option :label="$t('subtitle.top')" value="top" />
          <el-option :label="$t('subtitle.center')" value="center" />
          <el-option :label="$t('subtitle.bottom')" value="bottom" />
          <el-option :label="$t('subtitle.customPosition')" value="custom" />
        </el-select>
      </div>
      <div v-if="store.subtitlePosition === 'custom'" class="form-row">
        <div class="form-label">{{ $t('subtitle.customPosition') }}</div>
        <el-input-number
          v-model="store.customSubtitlePosition"
          :min="0"
          :max="100"
          :placeholder="$t('subtitle.customPositionPlaceholder')"
          style="width: 100%"
        />
      </div>
      <div class="form-row">
        <div class="form-label">{{ $t('subtitle.fontSize') }}</div>
        <el-slider v-model="store.fontSize" :min="30" :max="100" show-input size="small" />
      </div>
      <div class="form-row">
        <div class="form-label">{{ $t('subtitle.fontColor') }}</div>
        <el-color-picker v-model="store.textForeColor" />
      </div>
      <div class="form-row">
        <div class="form-label">{{ $t('subtitle.strokeColor') }}</div>
        <el-color-picker v-model="store.strokeColor" />
      </div>
      <div class="form-row">
        <div class="form-label">{{ $t('subtitle.strokeWidth') }}</div>
        <el-slider v-model="store.strokeWidth" :min="0" :max="10" :step="0.5" show-input size="small" />
      </div>
      <div class="form-row">
        <el-checkbox v-model="store.subtitleBackgroundEnabled">
          {{ $t('subtitle.backgroundEnabled') }}
        </el-checkbox>
      </div>
      <div v-if="store.subtitleBackgroundEnabled" class="form-row">
        <div class="form-label">{{ $t('subtitle.backgroundColor') }}</div>
        <el-color-picker v-model="store.subtitleBackgroundColor" />
      </div>
      <div v-if="store.subtitleBackgroundEnabled" class="form-row">
        <el-checkbox v-model="store.roundedSubtitleBackground" :disabled="!store.subtitleBackgroundEnabled">
          {{ $t('subtitle.roundedBackground') }}
        </el-checkbox>
      </div>
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useVideoParamsStore } from '@/stores/videoParams'

const store = useVideoParamsStore()

const fontOptions = ref([
  'STHeitiMedium.ttc',
  'STHeitiLight.ttc',
  'PingFang.ttc',
  'Songti.ttc',
  'Heiti.ttc',
  'Arial.ttf',
  'Verdana.ttf',
  'NotoSansSC-Regular.ttf',
  'NotoSansSC-Bold.ttf',
])
</script>
