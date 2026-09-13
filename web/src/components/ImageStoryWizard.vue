<template>
  <div class="image-story-wizard">
    <el-steps :active="currentStep" finish-status="success" align-center class="wizard-steps">
      <el-step title="故事主题" :icon="Edit" />
      <el-step title="上传图片" :icon="Picture" />
      <el-step title="添加描述" :icon="ChatLineSquare" />
      <el-step title="生成视频" :icon="Film" />
    </el-steps>

    <div class="wizard-body">
      <transition name="wizard-fade" mode="out-in">
        <div :key="currentStep" class="step-content">

          <!-- 第1步：故事主题 -->
          <div v-if="currentStep === 0" class="step-panel">
            <div class="step-header">
              <h3>故事主题</h3>
              <p class="step-desc">输入图片故事的主题和基本设置</p>
            </div>
            <el-card class="panel-card" shadow="hover">
              <div class="form-row">
                <div class="form-label">故事标题</div>
                <el-input
                  v-model="storySubject"
                  placeholder="输入故事标题，如：我的旅行日记"
                  clearable
                  size="large"
                />
              </div>
              <el-row :gutter="12">
                <el-col :span="12">
                  <div class="form-row">
                    <div class="form-label">画面比例</div>
                    <el-select v-model="videoAspect" style="width: 100%">
                      <el-option label="竖屏 9:16" value="9:16" />
                      <el-option label="横屏 16:9" value="16:9" />
                      <el-option label="方形 1:1" value="1:1" />
                    </el-select>
                  </div>
                </el-col>
                <el-col :span="12">
                  <div class="form-row">
                    <div class="form-label">AI提供商</div>
                    <el-select v-model="aiProvider" style="width: 100%">
                      <el-option label="智谱 CogVideoX" value="cogvideox" />
                      <el-option label="快手可灵" value="kling" />
                      <el-option label="MiniMax海螺" value="minimax" />
                    </el-select>
                  </div>
                </el-col>
              </el-row>
              <div class="form-row">
                <div class="form-label">单个场景时长（秒）</div>
                <el-select v-model="sceneDuration" style="width: 100%">
                  <el-option v-for="d in [2,3,4,5,6,7,8]" :key="d" :label="d + '秒'" :value="d" />
                </el-select>
              </div>
            </el-card>
          </div>

          <!-- 第2步：上传图片 -->
          <div v-if="currentStep === 1" class="step-panel">
            <div class="step-header">
              <h3>上传图片</h3>
              <p class="step-desc">上传故事中的图片，支持多张图片</p>
            </div>
            <el-card class="panel-card" shadow="hover">
              <div class="upload-area">
                <el-upload
                  :auto-upload="false"
                  :show-file-list="false"
                  :on-change="handleImageUpload"
                  multiple
                  accept="image/*"
                  class="image-uploader"
                >
                  <div class="upload-trigger">
                    <el-icon :size="48"><Plus /></el-icon>
                    <div class="upload-text">点击或拖拽上传图片</div>
                    <div class="upload-hint">支持 JPG、PNG 等图片格式，最多上传 20 张</div>
                  </div>
                </el-upload>
              </div>
              
              <div v-if="imageList.length > 0" class="image-list">
                <div class="image-list-header">
                  <span>已上传 {{ imageList.length }} 张图片</span>
                  <el-button type="primary" size="small" @click="clearImages">清空</el-button>
                </div>
                <div class="image-grid">
                  <div v-for="(image, index) in imageList" :key="index" class="image-item">
                    <el-image :src="image.previewUrl" fit="cover" class="image-preview" />
                    <div class="image-actions">
                      <el-button type="danger" size="small" circle @click="removeImage(index)">
                        <el-icon><Delete /></el-icon>
                      </el-button>
                    </div>
                    <div class="image-index">{{ index + 1 }}</div>
                  </div>
                </div>
              </div>
            </el-card>
          </div>

          <!-- 第3步：添加描述 -->
          <div v-if="currentStep === 2" class="step-panel">
            <div class="step-header">
              <h3>添加描述</h3>
              <p class="step-desc">为每张图片添加文字描述，AI将根据描述生成视频</p>
            </div>
            <el-card class="panel-card" shadow="hover">
              <div v-for="(image, index) in imageList" :key="index" class="scene-editor">
                <div class="scene-header">
                  <el-tag size="small" type="primary">场景 {{ index + 1 }}</el-tag>
                  <span class="scene-filename">{{ image.file.name }}</span>
                </div>
                <div class="scene-content">
                  <el-image :src="image.previewUrl" fit="cover" class="scene-image" />
                  <div class="scene-form">
                    <div class="form-row">
                      <div class="form-label">场景描述</div>
                      <el-input
                        v-model="image.description"
                        type="textarea"
                        :rows="3"
                        placeholder="描述这个场景的内容，AI将根据描述生成视频..."
                      />
                    </div>
                    <div class="form-row">
                      <div class="form-label">镜头运动（可选）</div>
                      <el-select v-model="image.cameraMovement" style="width: 100%" clearable>
                        <el-option label="静态" value="static" />
                        <el-option label="推进" value="push_in" />
                        <el-option label="拉远" value="pull_out" />
                        <el-option label="左移" value="pan_left" />
                        <el-option label="右移" value="pan_right" />
                      </el-select>
                    </div>
                  </div>
                </div>
              </div>
            </el-card>
          </div>

          <!-- 第4步：生成视频 -->
          <div v-if="currentStep === 3" class="step-panel">
            <div class="step-header">
              <h3>生成视频</h3>
              <p class="step-desc">确认设置并开始生成图片故事视频</p>
            </div>
            <el-card class="panel-card" shadow="hover">
              <div class="summary">
                <div class="summary-item">
                  <span class="summary-label">故事标题：</span>
                  <span class="summary-value">{{ storySubject }}</span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">图片数量：</span>
                  <span class="summary-value">{{ imageList.length }} 张</span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">画面比例：</span>
                  <span class="summary-value">{{ videoAspect }}</span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">AI提供商：</span>
                  <span class="summary-value">{{ getProviderName(aiProvider) }}</span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">场景时长：</span>
                  <span class="summary-value">{{ sceneDuration }} 秒/场景</span>
                </div>
              </div>
              
            </el-card>
          </div>

        </div>
      </transition>

      <!-- 底部导航：上一步/下一步 -->
      <div class="wizard-footer">
        <el-button v-if="currentStep > 0" :disabled="generating" @click="prevStep">上一步</el-button>
        <div class="footer-spacer" />
        <template v-if="currentStep < 3">
          <el-button type="primary" size="large" :disabled="generating" @click="nextStep">
            {{ currentStep === 2 ? '下一步：确认生成' : '下一步' }}
          </el-button>
        </template>
        <el-button
          v-else
          type="primary"
          size="large"
          :loading="generating"
          :disabled="imageList.length === 0"
          @click="generateVideo"
        >
          {{ generating ? '生成中...' : '开始生成' }}
        </el-button>
      </div>
    </div>

    <!-- 生成进度对话框 -->
    <el-dialog v-model="progressVisible" title="视频生成进度" width="500px" :close-on-click-modal="false">
      <div class="progress-content">
        <el-progress :percentage="progress" :status="progressStatus" />
        <div class="progress-text">{{ progressText }}</div>
      </div>
      <template #footer>
        <el-button @click="progressVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Edit, Picture, ChatLineSquare, Film, Plus, Delete } from '@element-plus/icons-vue'
import type { UploadFile } from 'element-plus'
import api, { extractData } from '@/api'

interface ImageItem {
  file: File
  previewUrl: string
  description: string
  cameraMovement: string
}

const currentStep = ref(0)
const storySubject = ref('')
const videoAspect = ref('9:16')
const aiProvider = ref('cogvideox')
const sceneDuration = ref(4)
const imageList = ref<ImageItem[]>([])
const generating = ref(false)
const progressVisible = ref(false)
const progress = ref(0)
const progressText = ref('')
const progressStatus = ref<'success' | 'exception' | ''>('')

function handleImageUpload(uploadFile: UploadFile) {
  if (imageList.value.length >= 20) {
    ElMessage.error('最多上传20张图片')
    return
  }
  // el-upload on-change 传的是 UploadFile 包装对象，真实文件在 .raw 中
  const raw = uploadFile.raw
  if (!raw) {
    ElMessage.error('读取图片文件失败')
    return
  }
  const previewUrl = URL.createObjectURL(raw)
  imageList.value.push({
    file: raw,
    previewUrl,
    description: '',
    cameraMovement: 'static',
  })
}

function removeImage(index: number) {
  URL.revokeObjectURL(imageList.value[index].previewUrl)
  imageList.value.splice(index, 1)
}

function clearImages() {
  imageList.value.forEach(img => URL.revokeObjectURL(img.previewUrl))
  imageList.value = []
}

function getProviderName(provider: string): string {
  const names: Record<string, string> = {
    cogvideox: '智谱 CogVideoX',
    kling: '快手可灵',
    minimax: 'MiniMax海螺',
  }
  return names[provider] || provider
}

/** 分步校验：通过后才允许进入下一步 */
function validateStep(step: number): boolean {
  if (step === 0 && !storySubject.value.trim()) {
    ElMessage.error('请输入故事标题')
    return false
  }
  if (step === 1 && imageList.value.length === 0) {
    ElMessage.error('请至少上传一张图片')
    return false
  }
  if (step === 2 && imageList.value.some(img => !img.description.trim())) {
    ElMessage.error('请为所有图片添加场景描述')
    return false
  }
  return true
}

function nextStep() {
  if (validateStep(currentStep.value)) currentStep.value++
}

function prevStep() {
  if (currentStep.value > 0) currentStep.value--
}

async function generateVideo() {
  if (!storySubject.value.trim()) {
    ElMessage.error('请输入故事标题')
    return
  }
  
  if (imageList.value.length === 0) {
    ElMessage.error('请至少上传一张图片')
    return
  }
  
  // 检查所有图片是否有描述
  const missingDescriptions = imageList.value.filter(img => !img.description.trim())
  if (missingDescriptions.length > 0) {
    ElMessage.error('请为所有图片添加描述')
    return
  }
  
  generating.value = true
  progressVisible.value = true
  progress.value = 0
  progressText.value = '准备生成...'
  progressStatus.value = ''
  
  try {
    // 上传图片并获取路径
    const uploadedImages: Array<{ scene_id: number; image_path: string; description: string; camera_movement?: string }> = []
    
    for (let i = 0; i < imageList.value.length; i++) {
      const image = imageList.value[i]
      progress.value = Math.round((i / imageList.value.length) * 30)
      progressText.value = `上传图片 ${i + 1}/${imageList.value.length}...`
      
      // 上传图片
      const formData = new FormData()
      formData.append('file', image.file)
      const uploadResult = await api.post('/materials/upload', formData, {
        headers: { 'Content-Type': 'multipart/form-data' }
      }).then(extractData<{ path: string }>)
      
      uploadedImages.push({
        scene_id: i + 1,
        image_path: uploadResult.path,
        description: image.description,
        camera_movement: image.cameraMovement || undefined,
      })
    }
    
    progress.value = 30
    progressText.value = '创建视频生成任务...'
    
    // 创建图片故事任务
    const taskResult = await api.post('/image_story/create', {
      story_subject: storySubject.value,
      scenes: uploadedImages,
      video_aspect: videoAspect.value,
      scene_duration: sceneDuration.value,
      ai_provider: aiProvider.value,
    }).then(extractData<{ taskId: string }>)
    
    progress.value = 40
    progressText.value = '任务已创建，开始生成视频...'
    
    // 轮询任务状态
    await pollTaskStatus(taskResult.taskId)
    
  } catch (e) {
    progressStatus.value = 'exception'
    progressText.value = `生成失败: ${(e as Error).message || String(e)}`
    ElMessage.error(`生成失败: ${(e as Error).message || String(e)}`)
  } finally {
    generating.value = false
  }
}

async function pollTaskStatus(taskId: string) {
  const maxAttempts = 60 // 最多轮询5分钟
  let attempts = 0
  
  while (attempts < maxAttempts) {
    try {
      const task = await api.post('/image_story/get', { taskId }).then(extractData<any>)
      
      progress.value = 40 + Math.round((task.progress / 100) * 60)
      progressText.value = `生成进度: ${task.progress}%`
      
      if (task.state === 1) { // Completed
        progress.value = 100
        progressStatus.value = 'success'
        progressText.value = '视频生成完成！'
        ElMessage.success('视频生成完成！')
        break
      } else if (task.state === -1) { // Failed
        progressStatus.value = 'exception'
        progressText.value = `生成失败: ${task.errorMessage || '未知错误'}`
        ElMessage.error(`生成失败: ${task.errorMessage || '未知错误'}`)
        break
      }
      
      // 等待5秒后再次轮询
      await new Promise(resolve => setTimeout(resolve, 5000))
      attempts++
    } catch (e) {
      console.error('轮询任务状态失败:', e)
      await new Promise(resolve => setTimeout(resolve, 5000))
      attempts++
    }
  }
  
  if (attempts >= maxAttempts) {
    progressStatus.value = 'exception'
    progressText.value = '轮询超时，请稍后查看任务列表'
    ElMessage.warning('轮询超时，请稍后查看任务列表')
  }
}
</script>

<style scoped>
.image-story-wizard {
  max-width: 900px;
  margin: 0 auto;
}

.wizard-steps {
  margin-bottom: 30px;
}

.step-content {
  min-height: 400px;
}

.step-panel {
  padding: 20px;
}

.step-header {
  margin-bottom: 20px;
}

.step-header h3 {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
}

.step-desc {
  margin: 0;
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.panel-card {
  margin-bottom: 16px;
}

.form-row {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  font-size: 14px;
}

.upload-area {
  margin-bottom: 20px;
}

.image-uploader {
  width: 100%;
}

.upload-trigger {
  border: 2px dashed var(--el-border-color);
  border-radius: 8px;
  padding: 40px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.3s;
}

.upload-trigger:hover {
  border-color: var(--el-color-primary);
}

.upload-text {
  margin-top: 12px;
  font-size: 16px;
  color: var(--el-text-color-primary);
}

.upload-hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.image-list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.image-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}

.image-item {
  position: relative;
  border-radius: 8px;
  overflow: hidden;
  aspect-ratio: 1;
}

.image-preview {
  width: 100%;
  height: 100%;
}

.image-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  opacity: 0;
  transition: opacity 0.3s;
}

.image-item:hover .image-actions {
  opacity: 1;
}

.image-index {
  position: absolute;
  bottom: 8px;
  left: 8px;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
}

.scene-editor {
  margin-bottom: 20px;
  padding: 16px;
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
}

.scene-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.scene-filename {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.scene-content {
  display: flex;
  gap: 16px;
}

.scene-image {
  width: 120px;
  height: 120px;
  border-radius: 8px;
  flex-shrink: 0;
}

.scene-form {
  flex: 1;
}

.summary {
  margin-bottom: 20px;
}

.summary-item {
  display: flex;
  margin-bottom: 8px;
}

.summary-label {
  width: 100px;
  color: var(--el-text-color-secondary);
}

.summary-value {
  font-weight: 500;
}

.action-buttons {
  display: flex;
  justify-content: center;
  gap: 12px;
  margin-top: 20px;
}

.wizard-footer {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 24px;
  padding: 16px 4px 0;
  border-top: 1px solid var(--el-border-color);
}

.footer-spacer {
  flex: 1;
}

.progress-content {
  text-align: center;
}

.progress-text {
  margin-top: 16px;
  color: var(--el-text-color-secondary);
}
</style>