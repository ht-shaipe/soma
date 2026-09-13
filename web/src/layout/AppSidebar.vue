<template>
  <aside class="app-sidebar">
    <div class="sidebar-logo">
      <span class="logo-mark">S</span>
      <span class="logo-name">Soma</span>
      <span class="logo-version">v0.1.2</span>
    </div>

    <div class="sidebar-group">{{ $t('nav.groupCreate') }}</div>
    <nav class="sidebar-nav">
      <router-link to="/" class="nav-item" :class="{ 'is-active': isActive('/') }">
        <el-icon><VideoCamera /></el-icon>
        <span>{{ $t('nav.home') }}</span>
      </router-link>
      <router-link to="/image-story" class="nav-item" :class="{ 'is-active': isActive('/image-story') }">
        <el-icon><Picture /></el-icon>
        <span>{{ $t('nav.imageStory') }}</span>
      </router-link>
      <router-link to="/workbench" class="nav-item" :class="{ 'is-active': isActive('/workbench') }">
        <el-icon><Grid /></el-icon>
        <span>{{ $t('nav.workbench') }}</span>
      </router-link>
    </nav>

    <div class="sidebar-group">{{ $t('nav.groupManage') }}</div>
    <nav class="sidebar-nav">
      <router-link to="/tasks" class="nav-item" :class="{ 'is-active': isActive('/tasks') }">
        <el-icon><List /></el-icon>
        <span>{{ $t('nav.tasks') }}</span>
      </router-link>
      <router-link to="/settings" class="nav-item" :class="{ 'is-active': isActive('/settings') }">
        <el-icon><Setting /></el-icon>
        <span>{{ $t('nav.settings') }}</span>
      </router-link>
    </nav>

    <div class="sidebar-footer">
      <div class="nav-item" @click="$emit('open-guide')">
        <el-icon><QuestionFilled /></el-icon>
        <span>{{ $t('nav.guide') }}</span>
      </div>
      <div class="footer-hint">{{ $t('app.subtitle') }}</div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router'
import { VideoCamera, List, Setting, QuestionFilled, Grid, Picture } from '@element-plus/icons-vue'

const route = useRoute()

defineEmits(['open-guide'])

function isActive(path: string) {
  return route.path === path
}
</script>
