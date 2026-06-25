<template>
  <el-card class="panel-card" shadow="hover">
    <template #header>
      <el-collapse>
        <el-collapse-item :title="$t('apiKey.title')">
          <el-tabs v-model="activeTab">
            <el-tab-pane :label="$t('apiKey.pexels')" name="pexels">
              <ApiKeyList
                :keys="pexelsKeys"
                @add="addPexelsKey"
                @delete="deletePexelsKey"
              />
            </el-tab-pane>
            <el-tab-pane :label="$t('apiKey.pixabay')" name="pixabay">
              <ApiKeyList
                :keys="pixabayKeys"
                @add="addPixabayKey"
                @delete="deletePixabayKey"
              />
            </el-tab-pane>
            <el-tab-pane :label="$t('apiKey.coverr')" name="coverr">
              <ApiKeyList
                :keys="coverrKeys"
                @add="addCoverrKey"
                @delete="deleteCoverrKey"
              />
            </el-tab-pane>
          </el-tabs>
        </el-collapse-item>
      </el-collapse>
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { useI18n } from 'vue-i18n'
import ApiKeyList from './ApiKeyList.vue'

const { t } = useI18n()
const activeTab = ref('pexels')

const pexelsKeys = ref<string[]>([])
const pixabayKeys = ref<string[]>([])
const coverrKeys = ref<string[]>([])

function addPexelsKey(key: string) {
  if (pexelsKeys.value.includes(key)) { ElMessage.warning(t('apiKey.keyExists')); return }
  pexelsKeys.value.push(key)
  ElMessage.success(t('apiKey.addSuccess'))
}
function deletePexelsKey(key: string) {
  pexelsKeys.value = pexelsKeys.value.filter(k => k !== key)
  ElMessage.success(t('apiKey.deleteSuccess'))
}
function addPixabayKey(key: string) {
  if (pixabayKeys.value.includes(key)) { ElMessage.warning(t('apiKey.keyExists')); return }
  pixabayKeys.value.push(key)
  ElMessage.success(t('apiKey.addSuccess'))
}
function deletePixabayKey(key: string) {
  pixabayKeys.value = pixabayKeys.value.filter(k => k !== key)
  ElMessage.success(t('apiKey.deleteSuccess'))
}
function addCoverrKey(key: string) {
  if (coverrKeys.value.includes(key)) { ElMessage.warning(t('apiKey.keyExists')); return }
  coverrKeys.value.push(key)
  ElMessage.success(t('apiKey.addSuccess'))
}
function deleteCoverrKey(key: string) {
  coverrKeys.value = coverrKeys.value.filter(k => k !== key)
  ElMessage.success(t('apiKey.deleteSuccess'))
}
</script>
