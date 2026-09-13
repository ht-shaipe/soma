<template>
  <div class="app-layout">
    <AppSidebar @open-guide="onOpenGuide" />
    <div class="app-main-area">
      <AppHeader @open-guide="onOpenGuide" />
      <div class="app-content" ref="contentRef">
        <router-view />
      </div>
    </div>
    <GuideDialog ref="guideRef" />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppSidebar from '@/layout/AppSidebar.vue'
import AppHeader from '@/layout/AppHeader.vue'
import GuideDialog from '@/components/GuideDialog.vue'
import { useConfigStore } from '@/stores/config'

const guideRef = ref<InstanceType<typeof GuideDialog> | null>(null)
const configStore = useConfigStore()
const route = useRoute()
const router = useRouter()
const contentRef = ref<HTMLElement | null>(null)

function onOpenGuide() {
  guideRef.value?.open()
}

// 页面切换后滚动复位（滚动容器是 .app-content 而非 window）
watch(() => route.path, async () => {
  await nextTick()
  contentRef.value?.scrollTo({ top: 0 })
})

onMounted(async () => {
  await configStore.loadConfig()
  if (guideRef.value?.shouldShowOnFirstUse()) {
    setTimeout(() => {
      guideRef.value?.open()
    }, 800)
  }

  // 预加载全部路由 chunk：首次点击菜单不再出现"旧页已卸载、新页未到达"的空白闪烁
  const preload = () => {
    for (const r of router.getRoutes()) {
      const c = r.components?.default
      if (typeof c === 'function') (c as () => Promise<unknown>)()
    }
  }
  if ('requestIdleCallback' in window) {
    ;(window as unknown as { requestIdleCallback: (cb: () => void, opts?: { timeout: number }) => void })
      .requestIdleCallback(preload, { timeout: 2000 })
  } else {
    setTimeout(preload, 300)
  }
})
</script>
