<template>
  <div class="app-layout">
    <AppSidebar />
    <div class="app-main-area">
      <AppHeader />
      <div class="app-content">
        <router-view />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import AppSidebar from '@/layout/AppSidebar.vue'
import AppHeader from '@/layout/AppHeader.vue'
</script>
