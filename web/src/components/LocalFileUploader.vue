<template>
  <div class="form-row">
    <div class="form-label">{{ $t('video.uploadLocalFiles') }}</div>
    <el-upload
      v-model:file-list="fileList"
      :auto-upload="false"
      :accept="'.mp4,.mov,.avi,.flv,.mkv,.jpg,.jpeg,.png'"
      multiple
      :tip="$t('video.uploadTip')"
    >
      <el-button type="primary" size="small">
        <el-icon><Upload /></el-icon>
        {{ $t('common.upload') }}
      </el-button>
      <template #tip>
        <div class="el-upload__tip">{{ $t('video.uploadTip') }}</div>
      </template>
    </el-upload>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { UploadUserFile } from 'element-plus'
import { Upload } from '@element-plus/icons-vue'
import { useVideoParamsStore } from '@/stores/videoParams'

const store = useVideoParamsStore()
const fileList = ref<UploadUserFile[]>([])

watch(fileList, (val) => {
  store.localVideoMaterials = val.map((f) => f.raw!).filter(Boolean)
})
</script>
