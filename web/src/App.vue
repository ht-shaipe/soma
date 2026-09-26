<template>
  <div class="app-layout">
    <AppSidebar @open-guide="onOpenGuide" />
    <div class="app-main-area">
      <AppHeader @open-guide="onOpenGuide" />
      <!-- M2.5 启动环境体检：关键依赖缺失时全局横幅引导（可关闭） -->
      <el-alert
        v-if="!envDismissed && envIssues.length"
        class="env-banner"
        type="warning"
        show-icon
        :title="$t('envBanner.title', { names: envIssueNames })"
        :closable="true"
        @close="envDismissed = true"
      >
        <div class="env-banner-body">
          <div v-for="issue in envIssues" :key="issue.name" class="env-banner-item">
            <span class="env-banner-name">{{ issue.name }}</span>
            <span class="env-banner-hint">{{ issue.hint }}</span>
          </div>
          <el-button size="small" type="warning" plain @click="router.push('/settings')">
            {{ $t('envBanner.goto') }}
          </el-button>
        </div>
      </el-alert>
      <div class="app-content" ref="contentRef">
        <router-view />
      </div>
    </div>
    <GuideDialog ref="guideRef" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppSidebar from '@/layout/AppSidebar.vue'
import AppHeader from '@/layout/AppHeader.vue'
import GuideDialog from '@/components/GuideDialog.vue'
import { useConfigStore } from '@/stores/config'
import api, { extractData } from '@/api'

interface PreflightCheck {
  name: string
  ok: boolean
  detail: string
  hint: string
  optional?: boolean
}

const guideRef = ref<InstanceType<typeof GuideDialog> | null>(null)
const configStore = useConfigStore()
const route = useRoute()
const router = useRouter()
const contentRef = ref<HTMLElement | null>(null)

// ── M2.5 启动环境体检 ──
const envIssues = ref<PreflightCheck[]>([])
const envDismissed = ref(false)
const envIssueNames = computed(() => envIssues.value.map((c) => c.name).join('、'))

async function runStartupPreflight() {
  try {
    const res = await api.post('/system/preflight', {}).then(extractData<{ checks: PreflightCheck[] }>)
    // 只横幅关键项（optional 缺失不拦截，设置页可见详情）
    envIssues.value = (res?.checks || []).filter((c) => !c.ok && !c.optional)
  } catch {
    // 检测失败静默：不打扰启动流程，设置页可手动重测
  }
}

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
  runStartupPreflight()
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

<style scoped>
.env-banner {
  margin: 0 20px;
  border-radius: 12px;
}

.env-banner-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
}

.env-banner-item {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 13px;
}

.env-banner-name {
  font-weight: 600;
  flex-shrink: 0;
}

.env-banner-hint {
  opacity: 0.85;
}
</style>
