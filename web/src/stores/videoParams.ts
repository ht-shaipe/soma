import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { VideoParams } from '@/types'

export const useVideoParamsStore = defineStore('videoParams', () => {
  const videoSubject = ref('')
  const videoScript = ref('')
  const videoTerms = ref('')
  const videoLanguage = ref('auto')
  const paragraphNumber = ref(1)
  const videoScriptPrompt = ref('')
  const useCustomSystemPrompt = ref(false)
  const customSystemPrompt = ref('')
  const matchMaterialsToScript = ref(false)

  const videoSource = ref('pexels')
  const videoConcatMode = ref('random')
  const videoTransitionMode = ref('none')
  const videoAspect = ref('9:16')
  const videoClipDuration = ref(3)
  const videoCount = ref(1)
  const videoEncoder = ref('libx264')
  const localVideoMaterials = ref<File[]>([])

  const ttsServer = ref('edge-tts')
  const voiceName = ref('zh-CN-XiaoxiaoNeural-Female')
  const voiceVolume = ref(1.0)
  const voiceRate = ref(1.0)
  const customAudioFile = ref<File | null>(null)
  const bgmType = ref('none')
  const bgmFile = ref('')
  const bgmVolume = ref(0.2)

  const subtitleEnabled = ref(true)
  const subtitlePosition = ref('bottom')
  const customSubtitlePosition = ref(70)
  const fontName = ref('STHeitiMedium.ttc')
  const fontSize = ref(60)
  const textForeColor = ref('#FFFFFF')
  const strokeColor = ref('#000000')
  const strokeWidth = ref(1.5)
  const subtitleBackgroundEnabled = ref(true)
  const subtitleBackgroundColor = ref('#000000')
  const roundedSubtitleBackground = ref(false)

  function toVideoParams(): VideoParams {
    return {
      video_subject: videoSubject.value,
      video_script: videoScript.value || undefined,
      video_terms: videoTerms.value || undefined,
      video_language: videoLanguage.value !== 'auto' ? videoLanguage.value : undefined,
      paragraph_number: paragraphNumber.value,
      video_script_prompt: videoScriptPrompt.value || undefined,
      use_custom_system_prompt: useCustomSystemPrompt.value,
      custom_system_prompt: useCustomSystemPrompt.value ? customSystemPrompt.value : undefined,
      match_materials_to_script: matchMaterialsToScript.value,
      video_source: videoSource.value,
      video_concat_mode: videoConcatMode.value,
      video_transition_mode: videoTransitionMode.value,
      video_aspect: videoAspect.value,
      video_clip_duration: videoClipDuration.value,
      video_count: videoCount.value,
      video_encoder: videoEncoder.value,
      voice_name: voiceName.value,
      voice_volume: voiceVolume.value,
      voice_rate: voiceRate.value,
      custom_audio_file: customAudioFile.value ? customAudioFile.value.name : undefined,
      bgm_type: bgmType.value,
      bgm_file: bgmFile.value || undefined,
      bgm_volume: bgmVolume.value,
      subtitle_enabled: subtitleEnabled.value,
      subtitle_position: subtitlePosition.value,
      custom_position: subtitlePosition.value === 'custom' ? customSubtitlePosition.value : undefined,
      font_name: fontName.value,
      font_size: fontSize.value,
      text_fore_color: textForeColor.value,
      stroke_color: strokeColor.value,
      stroke_width: strokeWidth.value,
      text_background_color: subtitleBackgroundEnabled.value ? subtitleBackgroundColor.value : false,
      rounded_subtitle_background: roundedSubtitleBackground.value,
    }
  }

  function resetScript() {
    videoScript.value = ''
    videoTerms.value = ''
  }

  return {
    videoSubject, videoScript, videoTerms, videoLanguage,
    paragraphNumber, videoScriptPrompt, useCustomSystemPrompt, customSystemPrompt,
    matchMaterialsToScript,
    videoSource, videoConcatMode, videoTransitionMode, videoAspect,
    videoClipDuration, videoCount, videoEncoder, localVideoMaterials,
    ttsServer, voiceName, voiceVolume, voiceRate, customAudioFile,
    bgmType, bgmFile, bgmVolume,
    subtitleEnabled, subtitlePosition, customSubtitlePosition,
    fontName, fontSize, textForeColor, strokeColor, strokeWidth,
    subtitleBackgroundEnabled, subtitleBackgroundColor, roundedSubtitleBackground,
    toVideoParams, resetScript,
  }
})
