<template>
  <div class="app-layout">
    <AppSidebar @open-guide="onOpenGuide" />
    <div class="app-main-area">
      <AppHeader @open-guide="onOpenGuide" />
      <div class="app-content">
        <router-view />
      </div>
    </div>
    <GuideDialog ref="guideRef" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import AppSidebar from '@/layout/AppSidebar.vue'
import AppHeader from '@/layout/AppHeader.vue'
import GuideDialog from '@/components/GuideDialog.vue'
import { useConfigStore } from '@/stores/config'

const guideRef = ref<InstanceType<typeof GuideDialog> | null>(null)
const configStore = useConfigStore()

function onOpenGuide() {
  guideRef.value?.open()
}

onMounted(async () => {
  await configStore.loadConfig()
  if (guideRef.value?.shouldShowOnFirstUse()) {
    setTimeout(() => {
      guideRef.value?.open()
    }, 800)
  }
})
</script>
