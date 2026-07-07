<template>
  <el-card class="panel-card" shadow="hover">
    <template #header>
      <span>{{ $t('video.title') }}</span>
    </template>
    <div class="form-row">
      <div class="form-label">{{ $t('video.videoSource') }}</div>
      <el-select v-model="store.videoSource" style="width: 100%">
        <el-option :label="$t('video.pexels')" value="pexels" />
        <el-option :label="$t('video.pixabay')" value="pixabay" />
        <el-option :label="$t('video.coverr')" value="coverr" />
        <el-option :label="$t('video.cogvideox')" value="cogvideox" />
        <el-option :label="$t('video.kling')" value="kling" />
        <el-option :label="$t('video.minimax')" value="minimax" />
        <el-option :label="$t('video.local')" value="local" />
      </el-select>
    </div>
    <PortraitUploader v-if="store.videoSource !== 'local'" />
    <LocalFileUploader v-if="store.videoSource === 'local'" />
    <div class="form-row">
      <div class="form-label">{{ $t('video.concatMode') }}</div>
      <el-select v-model="store.videoConcatMode" style="width: 100%">
        <el-option :label="$t('video.sequential')" value="sequential" />
        <el-option :label="$t('video.random')" value="random" />
      </el-select>
    </div>
    <div class="form-row">
      <div class="form-label">{{ $t('video.transitionMode') }}</div>
      <el-select v-model="store.videoTransitionMode" style="width: 100%">
        <el-option :label="$t('video.none')" value="none" />
        <el-option :label="$t('video.shuffle')" value="Shuffle" />
        <el-option :label="$t('video.fadeIn')" value="FadeIn" />
        <el-option :label="$t('video.fadeOut')" value="FadeOut" />
        <el-option :label="$t('video.slideIn')" value="SlideIn" />
        <el-option :label="$t('video.slideOut')" value="SlideOut" />
      </el-select>
    </div>
    <div class="form-row">
      <div class="form-label">{{ $t('video.aspectRatio') }}</div>
      <el-select v-model="store.videoAspect" style="width: 100%">
        <el-option :label="$t('video.portrait916')" value="9:16" />
        <el-option :label="$t('video.landscape169')" value="16:9" />
        <el-option :label="$t('video.square11')" value="1:1" />
      </el-select>
    </div>
    <div class="form-row">
      <div class="form-label">{{ $t('video.clipDuration') }}</div>
      <el-select v-model="store.videoClipDuration" style="width: 100%">
        <el-option v-for="d in [2,3,4,5,6,7,8,9,10]" :key="d" :label="d" :value="d" />
      </el-select>
    </div>
    <div class="form-row">
      <div class="form-label">{{ $t('video.videoCount') }}</div>
      <el-select v-model="store.videoCount" style="width: 100%">
        <el-option v-for="c in [1,2,3,4,5]" :key="c" :label="c" :value="c" />
      </el-select>
    </div>
    <el-collapse>
      <el-collapse-item :title="$t('video.advancedSettings')">
        <VideoAdvancedSettings />
      </el-collapse-item>
    </el-collapse>
  </el-card>
</template>

<script setup lang="ts">
import { useVideoParamsStore } from '@/stores/videoParams'
import LocalFileUploader from './LocalFileUploader.vue'
import VideoAdvancedSettings from './VideoAdvancedSettings.vue'
import PortraitUploader from './PortraitUploader.vue'

const store = useVideoParamsStore()
</script>
