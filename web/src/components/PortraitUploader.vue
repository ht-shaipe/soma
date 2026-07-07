<template>
  <div class="portrait-uploader">
    <div class="form-label">{{ $t('video.portraitImage') || '人像图片' }}</div>
    <div class="portrait-content">
      <el-upload
        :auto-upload="true"
        :show-file-list="false"
        :accept="'.jpg,.jpeg,.png,.webp'"
        :before-upload="beforeUpload"
        :http-request="handleUpload"
      >
        <div v-if="previewUrl" class="portrait-preview">
          <el-image :src="previewUrl" fit="cover" class="portrait-img" />
          <div class="portrait-overlay">
            <el-icon><Refresh /></el-icon>
          </div>
        </div>
        <div v-else class="portrait-placeholder">
          <el-icon :size="32"><Plus /></el-icon>
          <span>{{ $t('video.uploadPortrait') || '上传人像' }}</span>
        </div>
      </el-upload>
      <el-button
        v-if="previewUrl"
        type="danger"
        size="small"
        text
        @click="clearPortrait"
        style="margin-left: 8px"
      >
        {{ $t('common.remove') || '移除' }}
      </el-button>
    </div>
    <div class="portrait-tip">
      {{ $t('video.portraitTip') || '上传人像图片后，将自动使用AI图生视频模式生成口播视频' }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Plus, Refresh } from '@element-plus/icons-vue'
import { useVideoParamsStore } from '@/stores/videoParams'
import { uploadPortrait } from '@/api/material'
import { useI18n } from 'vue-i18n'

const store = useVideoParamsStore()
const { t } = useI18n()
const previewUrl = ref('')

watch(() => store.portraitImage, (val) => {
  if (val && !previewUrl.value) {
    previewUrl.value = val.startsWith('http') ? val : `/storage/portraits/${val.split('/').pop()}`
  }
})

function beforeUpload(file: File) {
  const isImage = file.type.startsWith('image/')
  if (!isImage) {
    ElMessage.error(t('portrait.imageOnly'))
    return false
  }
  const isLt5M = file.size / 1024 / 1024 < 5
  if (!isLt5M) {
    ElMessage.error(t('portrait.imageSizeLimit'))
    return false
  }
  return true
}

async function handleUpload(options: { file: File }) {
  try {
    const result = await uploadPortrait(options.file)
    store.portraitImage = result.path
    previewUrl.value = `/storage/portraits/${result.name}`
    ElMessage.success(t('common.success'))
  } catch (e) {
    ElMessage.error(`${t('common.error')}: ${e instanceof Error ? e.message : String(e)}`)
  }
}

function clearPortrait() {
  store.portraitImage = ''
  previewUrl.value = ''
}
</script>

<style scoped>
.portrait-uploader {
  margin-top: 8px;
}
.portrait-content {
  display: flex;
  align-items: center;
}
.portrait-preview {
  position: relative;
  width: 80px;
  height: 80px;
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
}
.portrait-img {
  width: 100%;
  height: 100%;
}
.portrait-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0,0,0,0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  opacity: 0;
  transition: opacity 0.2s;
}
.portrait-preview:hover .portrait-overlay {
  opacity: 1;
}
.portrait-placeholder {
  width: 80px;
  height: 80px;
  border: 1px dashed var(--el-border-color);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--el-text-color-secondary);
  cursor: pointer;
  transition: border-color 0.2s;
}
.portrait-placeholder:hover {
  border-color: var(--el-color-primary);
}
.portrait-placeholder span {
  font-size: 12px;
  margin-top: 4px;
}
.portrait-tip {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
  line-height: 1.4;
}
</style>
