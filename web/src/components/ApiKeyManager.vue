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
                @delete="deletePixabayKey"
                @add="addPixabayKey"
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
import { ref, onMounted, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useI18n } from 'vue-i18n'
import { useConfigStore } from '@/stores/config'
import ApiKeyList from './ApiKeyList.vue'

const { t } = useI18n()
const configStore = useConfigStore()
const activeTab = ref('pexels')

const pexelsKeys = ref<string[]>([])
const pixabayKeys = ref<string[]>([])
const coverrKeys = ref<string[]>([])

function loadKeysFromConfig() {
  const stock = configStore.config.stock
  const pexelsKey = stock.pexels_api_key || ''
  const pixabayKey = stock.pixabay_api_key || ''
  const coverrKey = stock.coverr_api_key || ''
  pexelsKeys.value = pexelsKey ? pexelsKey.split(',').map(k => k.trim()).filter(k => k) : []
  pixabayKeys.value = pixabayKey ? pixabayKey.split(',').map(k => k.trim()).filter(k => k) : []
  coverrKeys.value = coverrKey ? coverrKey.split(',').map(k => k.trim()).filter(k => k) : []
}

function saveKeysToConfig() {
  configStore.updateStockConfig({
    pexels_api_key: pexelsKeys.value.join(','),
    pixabay_api_key: pixabayKeys.value.join(','),
    coverr_api_key: coverrKeys.value.join(','),
  })
  configStore.save()
}

function addPexelsKey(key: string) {
  if (pexelsKeys.value.includes(key)) { ElMessage.warning(t('apiKey.keyExists')); return }
  pexelsKeys.value.push(key)
  saveKeysToConfig()
  ElMessage.success(t('apiKey.addSuccess'))
}
function deletePexelsKey(key: string) {
  pexelsKeys.value = pexelsKeys.value.filter(k => k !== key)
  saveKeysToConfig()
  ElMessage.success(t('apiKey.deleteSuccess'))
}
function addPixabayKey(key: string) {
  if (pixabayKeys.value.includes(key)) { ElMessage.warning(t('apiKey.keyExists')); return }
  pixabayKeys.value.push(key)
  saveKeysToConfig()
  ElMessage.success(t('apiKey.addSuccess'))
}
function deletePixabayKey(key: string) {
  pixabayKeys.value = pixabayKeys.value.filter(k => k !== key)
  saveKeysToConfig()
  ElMessage.success(t('apiKey.deleteSuccess'))
}
function addCoverrKey(key: string) {
  if (coverrKeys.value.includes(key)) { ElMessage.warning(t('apiKey.keyExists')); return }
  coverrKeys.value.push(key)
  saveKeysToConfig()
  ElMessage.success(t('apiKey.addSuccess'))
}
function deleteCoverrKey(key: string) {
  coverrKeys.value = coverrKeys.value.filter(k => k !== key)
  saveKeysToConfig()
  ElMessage.success(t('apiKey.deleteSuccess'))
}

onMounted(() => {
  if (configStore.loaded) loadKeysFromConfig()
})

watch(() => configStore.loaded, (val) => {
  if (val) loadKeysFromConfig()
})
</script>
