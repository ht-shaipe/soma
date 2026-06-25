<template>
  <div>
    <div class="form-row">
      <el-checkbox v-model="store.matchMaterialsToScript">
        {{ $t('video.matchMaterialsToScript') }}
      </el-checkbox>
    </div>
    <div class="form-row">
      <div class="form-label">{{ $t('video.videoEncoder') }}</div>
      <el-select v-model="store.videoEncoder" style="width: 100%">
        <el-option :label="$t('video.libx264')" value="libx264" />
        <el-option label="NVIDIA NVENC" value="h264_nvenc" />
        <el-option label="AMD AMF" value="h264_amf" />
        <el-option label="Intel QSV" value="h264_qsv" />
        <el-option label="macOS VideoToolbox" value="h264_videotoolbox" />
      </el-select>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useVideoParamsStore } from '@/stores/videoParams'

const store = useVideoParamsStore()
</script>
