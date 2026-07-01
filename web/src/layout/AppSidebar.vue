<template>
  <aside class="app-sidebar">
    <div class="sidebar-logo">
      <span class="logo-icon">🤖</span>
      <span>Soma</span>
    </div>
    <el-menu
      :default-active="activeRoute"
      :router="true"
      background-color="#304156"
      text-color="#bfcbd9"
      active-text-color="#409eff"
    >
      <el-menu-item index="/">
        <el-icon><VideoCamera /></el-icon>
        <span>{{ $t('nav.home') }}</span>
      </el-menu-item>
      <el-menu-item index="/tasks">
        <el-icon><List /></el-icon>
        <span>{{ $t('nav.tasks') }}</span>
      </el-menu-item>
      <el-menu-item index="/settings">
        <el-icon><Setting /></el-icon>
        <span>{{ $t('nav.settings') }}</span>
      </el-menu-item>
      <el-menu-item @click="$emit('open-guide')">
        <el-icon><QuestionFilled /></el-icon>
        <span>{{ $t('nav.guide') }}</span>
      </el-menu-item>
    </el-menu>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { VideoCamera, List, Setting, QuestionFilled } from '@element-plus/icons-vue'

const route = useRoute()

defineEmits(['open-guide'])
const activeRoute = computed(() => route.path)
</script>
