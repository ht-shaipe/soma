<template>
  <div>
    <div class="form-row">
      <div class="form-label">{{ $t('audio.bgm') }}</div>
      <el-select v-model="store.bgmType" style="width: 100%">
        <el-option :label="$t('audio.noBgm')" value="none" />
        <el-option :label="$t('audio.randomBgm')" value="random" />
        <el-option :label="$t('audio.customBgm')" value="custom" />
      </el-select>
    </div>
    <div v-if="store.bgmType === 'custom'" class="form-row">
      <div class="form-label">{{ $t('audio.customBgmFile') }}</div>
      <el-input v-model="store.bgmFile" />
    </div>
    <div class="form-row">
      <div class="form-label">{{ $t('audio.bgmVolume') }}</div>
      <el-select v-model="store.bgmVolume" style="width: 100%">
        <el-option v-for="v in bgmVolumeOptions" :key="v" :label="v.toFixed(1)" :value="v" />
      </el-select>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useVideoParamsStore } from '@/stores/videoParams'

const store = useVideoParamsStore()

const bgmVolumeOptions = Array.from({ length: 11 }, (_, i) => +(i * 0.1).toFixed(1))
</script>
