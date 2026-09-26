// 创作参数 Pinia store：向导分步草稿参数与任务提交载荷组装
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import type { VideoParams, StoryboardScene, TaskInfo } from '@/types'

const TTS_DEFAULT_VOICES: Record<string, string> = {
  'edge-tts': 'zh-CN-XiaoxiaoNeural-Female',
  'azure-v2': 'azure:zh-CN-XiaoxiaoNeural',
  'siliconflow': 'siliconflow:FunAudioLLM/CosyVoice2-0.5B:alice',
  'gemini': 'gemini:Zephyr',
  'mimo': 'mimo:almara',
  'elevenlabs': 'elevenlabs:21m00Tcm4TlvDq8ikWAM:Rachel',
  'fishspeech': 'fishspeech:',
  'none': 'no-voice',
}

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
  const intentStyle = ref('aesthetic')
  const intentMood = ref('warm')
  const intentAudience = ref('')

  const videoSource = ref('pexels')
  const videoConcatMode = ref('random')
  const videoTransitionMode = ref('none')
  const videoAspect = ref('9:16')
  const videoClipDuration = ref(3)
  const videoCount = ref(1)
  const videoEncoder = ref('libx264')
  const localVideoMaterials = ref<File[]>([])

  // 切换素材源时清除已缓存的素材，强制重新获取
  watch(videoSource, () => {
    materialsFetched.value = false
    materialsList.value = []
  })

  const ttsServer = ref('edge-tts')
  const voiceName = ref('zh-CN-XiaoxiaoNeural-Female')

  watch(ttsServer, (newServer) => {
    const defaultVoice = TTS_DEFAULT_VOICES[newServer]
    if (defaultVoice) {
      voiceName.value = defaultVoice
    }
  })
  const voiceVolume = ref(1.0)
  const voiceRate = ref(1.0)
  const customAudioFile = ref<File | null>(null)
  const customAudioFileName = ref('')
  const bgmType = ref('none')
  const bgmFile = ref('')
  const bgmVolume = ref(0.2)

  const subtitleEnabled = ref(true)
  const subtitlePosition = ref('bottom')
  const customSubtitlePosition = ref(70)
  const fontName = ref('STHeitiMedium.ttc')
  const fontSize = ref(16)
  const textForeColor = ref('#FFFFFF')
  const strokeColor = ref('#000000')
  const strokeWidth = ref(1.5)
  const subtitleBackgroundEnabled = ref(false)
  const subtitleBackgroundColor = ref('#000000')
  const roundedSubtitleBackground = ref(false)
  const videoWatermark = ref('')
  const videoIntro = ref('')
  const videoOutro = ref('')
  const portraitImage = ref('')
  const storyboard = ref<StoryboardScene[]>([])
  const storyboardParsed = ref(false)
  const materialsFetched = ref(false)
  const materialsList = ref<string[]>([])
  const fetchingMaterials = ref(false)
  const narration = ref('')
  const audioFile = ref('')
  const audioDuration = ref(0)

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
      intent_style: intentStyle.value || undefined,
      intent_mood: intentMood.value || undefined,
      intent_audience: intentAudience.value || undefined,
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
      custom_audio_file: customAudioFileName.value || (customAudioFile.value ? customAudioFile.value.name : undefined),
      video_materials: videoSource.value === 'local' && localVideoMaterials.value.length > 0
        ? localVideoMaterials.value.map(f => ({ name: f.name, path: f.name, provider: 'local' }))
        : undefined,
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
      video_watermark: videoWatermark.value || undefined,
      video_intro: videoIntro.value || undefined,
      video_outro: videoOutro.value || undefined,
      portrait_image: portraitImage.value || undefined,
    }
  }

  function loadFromTask(params: VideoParams, task?: TaskInfo) {
    if (params.video_subject) videoSubject.value = params.video_subject
    if (params.video_script) videoScript.value = params.video_script
    if (params.video_terms) {
      const t = typeof params.video_terms === 'string' ? params.video_terms : (params.video_terms as string[]).join(',')
      videoTerms.value = t
    }
    if (params.video_language) videoLanguage.value = params.video_language
    if (params.paragraph_number) paragraphNumber.value = params.paragraph_number
    if (params.video_script_prompt) videoScriptPrompt.value = params.video_script_prompt
    if (params.use_custom_system_prompt !== undefined) useCustomSystemPrompt.value = params.use_custom_system_prompt
    if (params.custom_system_prompt) customSystemPrompt.value = params.custom_system_prompt
    if (params.match_materials_to_script !== undefined) matchMaterialsToScript.value = params.match_materials_to_script
    if (params.intent_style) intentStyle.value = params.intent_style
    if (params.intent_mood) intentMood.value = params.intent_mood
    if (params.intent_audience) intentAudience.value = params.intent_audience
    if (params.video_source) videoSource.value = params.video_source
    if (params.video_concat_mode) videoConcatMode.value = params.video_concat_mode
    if (params.video_transition_mode) videoTransitionMode.value = params.video_transition_mode
    if (params.video_aspect) videoAspect.value = params.video_aspect
    if (params.video_clip_duration) videoClipDuration.value = params.video_clip_duration
    if (params.video_count) videoCount.value = params.video_count
    if (params.video_encoder) videoEncoder.value = params.video_encoder
    if (params.voice_name) {
      voiceName.value = params.voice_name
      const serverHint = params.voice_name.split(':')[0]
      if (serverHint && TTS_DEFAULT_VOICES[serverHint]) ttsServer.value = serverHint
    }
    if (params.voice_volume !== undefined) voiceVolume.value = params.voice_volume
    if (params.voice_rate !== undefined) voiceRate.value = params.voice_rate
    if (params.custom_audio_file) customAudioFileName.value = params.custom_audio_file
    if (params.bgm_type) bgmType.value = params.bgm_type
    if (params.bgm_file) bgmFile.value = params.bgm_file
    if (params.bgm_volume !== undefined) bgmVolume.value = params.bgm_volume
    if (params.subtitle_enabled !== undefined) subtitleEnabled.value = params.subtitle_enabled
    if (params.subtitle_position) subtitlePosition.value = params.subtitle_position
    if (params.custom_position !== undefined) customSubtitlePosition.value = params.custom_position
    if (params.font_name) fontName.value = params.font_name
    if (params.font_size) fontSize.value = params.font_size
    if (params.text_fore_color) textForeColor.value = params.text_fore_color
    if (params.stroke_color) strokeColor.value = params.stroke_color
    if (params.stroke_width !== undefined) strokeWidth.value = params.stroke_width
    if (params.text_background_color !== undefined && params.text_background_color !== false) {
      subtitleBackgroundEnabled.value = true
      if (typeof params.text_background_color === 'string') subtitleBackgroundColor.value = params.text_background_color
    }
    if (params.rounded_subtitle_background !== undefined) roundedSubtitleBackground.value = params.rounded_subtitle_background
    if (params.video_watermark) videoWatermark.value = params.video_watermark
    if (params.video_intro) videoIntro.value = params.video_intro
    if (params.video_outro) videoOutro.value = params.video_outro
    if (params.portrait_image) portraitImage.value = params.portrait_image

    if (task) {
      if (task.script) {
        videoScript.value = task.script
      }
      if (task.terms && task.terms.length > 0) {
        videoTerms.value = task.terms.join(',')
      }
      if (task.storyboard && task.storyboard.length > 0) {
        storyboard.value = task.storyboard
        storyboardParsed.value = true
      }
      if (task.materials && task.materials.length > 0) {
        materialsList.value = task.materials
        materialsFetched.value = true
      }
      if (task.narration) {
        narration.value = task.narration
      }
      if (task.audioFile) {
        audioFile.value = task.audioFile
      }
      if (task.audioDuration) {
        audioDuration.value = task.audioDuration
      }
    }
  }

  function resetScript() {
    videoScript.value = ''
    videoTerms.value = ''
  }

  function resetAll() {
    videoSubject.value = ''
    videoScript.value = ''
    videoTerms.value = ''
    videoLanguage.value = 'auto'
    paragraphNumber.value = 1
    videoScriptPrompt.value = ''
    useCustomSystemPrompt.value = false
    customSystemPrompt.value = ''
    matchMaterialsToScript.value = false
    intentStyle.value = 'aesthetic'
    intentMood.value = 'warm'
    intentAudience.value = ''
    videoSource.value = 'pexels'
    videoConcatMode.value = 'random'
    videoTransitionMode.value = 'none'
    videoAspect.value = '9:16'
    videoClipDuration.value = 3
    videoCount.value = 1
    videoEncoder.value = 'libx264'
    localVideoMaterials.value = []
    ttsServer.value = 'edge-tts'
    voiceName.value = 'zh-CN-XiaoxiaoNeural-Female'
    voiceVolume.value = 1.0
    voiceRate.value = 1.0
    customAudioFile.value = null
    customAudioFileName.value = ''
    bgmType.value = 'none'
    bgmFile.value = ''
    bgmVolume.value = 0.2
    subtitleEnabled.value = true
    subtitlePosition.value = 'bottom'
    customSubtitlePosition.value = 70
    fontName.value = 'STHeitiMedium.ttc'
    fontSize.value = 16
    textForeColor.value = '#FFFFFF'
    strokeColor.value = '#000000'
    strokeWidth.value = 1.5
    subtitleBackgroundEnabled.value = false
    subtitleBackgroundColor.value = '#000000'
    roundedSubtitleBackground.value = false
    videoWatermark.value = ''
    videoIntro.value = ''
    videoOutro.value = ''
    portraitImage.value = ''
    storyboard.value = []
    storyboardParsed.value = false
    materialsFetched.value = false
    materialsList.value = []
    narration.value = ''
  }

  function collectStepParams(step: number): Partial<VideoParams> {
    switch (step) {
      case 0:
        return {
          video_subject: videoSubject.value,
          intent_style: intentStyle.value || undefined,
          intent_mood: intentMood.value || undefined,
          intent_audience: intentAudience.value || undefined,
          video_aspect: videoAspect.value,
          video_language: videoLanguage.value !== 'auto' ? videoLanguage.value : undefined,
          paragraph_number: paragraphNumber.value,
          video_script_prompt: videoScriptPrompt.value || undefined,
          use_custom_system_prompt: useCustomSystemPrompt.value,
          custom_system_prompt: useCustomSystemPrompt.value ? customSystemPrompt.value : undefined,
        }
      case 1:
        return {
          video_script: videoScript.value || undefined,
          video_terms: videoTerms.value || undefined,
        }
      case 2:
        return {
          video_clip_duration: videoClipDuration.value,
          video_count: videoCount.value,
          video_concat_mode: videoConcatMode.value,
          video_transition_mode: videoTransitionMode.value,
          video_terms: videoTerms.value || undefined,
          match_materials_to_script: matchMaterialsToScript.value,
        }
      case 3:
        return {}
      case 4:
        return {
          video_source: videoSource.value,
          video_materials: videoSource.value === 'local' && localVideoMaterials.value.length > 0
            ? localVideoMaterials.value.map(f => ({ name: f.name, path: f.name, provider: 'local' }))
            : undefined,
          portrait_image: portraitImage.value || undefined,
        }
      case 5:
        return {
          voice_name: voiceName.value,
          voice_volume: voiceVolume.value,
          voice_rate: voiceRate.value,
          custom_audio_file: customAudioFileName.value || (customAudioFile.value ? customAudioFile.value.name : undefined),
          bgm_type: bgmType.value,
          bgm_file: bgmFile.value || undefined,
          bgm_volume: bgmVolume.value,
        }
      case 6:
        return {
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
      case 7:
        return {
          video_encoder: videoEncoder.value,
          video_watermark: videoWatermark.value || undefined,
          video_intro: videoIntro.value || undefined,
          video_outro: videoOutro.value || undefined,
        }
      default:
        return {}
    }
  }

  return {
    videoSubject, videoScript, videoTerms, videoLanguage,
    paragraphNumber, videoScriptPrompt, useCustomSystemPrompt, customSystemPrompt,
    matchMaterialsToScript, intentStyle, intentMood, intentAudience,
    videoSource, videoConcatMode, videoTransitionMode, videoAspect,
    videoClipDuration, videoCount, videoEncoder, localVideoMaterials,
    ttsServer, voiceName, voiceVolume, voiceRate, customAudioFile, customAudioFileName,
    bgmType, bgmFile, bgmVolume,
    subtitleEnabled, subtitlePosition, customSubtitlePosition,
    fontName, fontSize, textForeColor, strokeColor, strokeWidth,
    subtitleBackgroundEnabled, subtitleBackgroundColor, roundedSubtitleBackground,
    videoWatermark, videoIntro, videoOutro, portraitImage,
    storyboard, storyboardParsed,
    materialsFetched, materialsList, fetchingMaterials,
    narration, audioFile, audioDuration,
    toVideoParams, loadFromTask, resetAll, resetScript, collectStepParams,
  }
})
