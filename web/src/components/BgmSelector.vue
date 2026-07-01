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
      <el-select v-model="store.bgmFile" style="width: 100%" filterable :loading="loadingBgm" :placeholder="$t('audio.selectBgm')">
        <el-option v-for="m in bgmList" :key="m.name" :label="m.name" :value="m.name" />
      </el-select>
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
import { ref, onMounted } from 'vue'
import { useVideoParamsStore } from '@/stores/videoParams'
import { listMusics } from '@/api/music'
import type { MusicInfo } from '@/types'

const store = useVideoParamsStore()
const bgmList = ref<MusicInfo[]>([])
const loadingBgm = ref(false)

const bgmVolumeOptions = Array.from({ length: 11 }, (_, i) => +(i * 0.1).toFixed(1))

async function fetchBgmList() {
  loadingBgm.value = true
  try {
    const data = await listMusics()
    bgmList.value = data.list || data || []
  } catch (e) {
    bgmList.value = []
    console.error('Failed to fetch BGM list:', e)
  } finally {
    loadingBgm.value = false
  }
}

onMounted(() => {
  fetchBgmList()
})
</script>
