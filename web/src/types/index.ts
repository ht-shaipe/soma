export interface VideoParams {
  video_subject: string
  video_script?: string
  video_terms?: string | string[]
  video_aspect?: string
  video_concat_mode?: string
  video_transition_mode?: string
  video_clip_duration?: number
  video_count?: number
  video_source?: string
  voice_name?: string
  voice_volume?: number
  voice_rate?: number
  bgm_type?: string
  bgm_file?: string
  bgm_volume?: number
  subtitle_enabled?: boolean
  subtitle_position?: string
  custom_position?: number
  font_name?: string
  font_size?: number
  text_fore_color?: string
  stroke_color?: string
  stroke_width?: number
  text_background_color?: boolean | string
  rounded_subtitle_background?: boolean
  paragraph_number?: number
  video_script_prompt?: string
  custom_system_prompt?: string
  use_custom_system_prompt?: boolean
  match_materials_to_script?: boolean
  video_language?: string
  n_threads?: number
  video_encoder?: string
  custom_audio_file?: string
  video_materials?: MaterialInfo[]
}

export interface MaterialInfo {
  url: string
  provider: string
  duration?: number
  width?: number
  height?: number
}

export const TaskStatus = {
  Pending: 'Pending',
  Script: 'Script',
  Terms: 'Terms',
  Audio: 'Audio',
  Subtitle: 'Subtitle',
  Materials: 'Materials',
  Video: 'Video',
  Completed: 'Completed',
  Failed: 'Failed',
} as const

export type TaskStatus = typeof TaskStatus[keyof typeof TaskStatus]

export interface TaskInfo {
  task_id: string
  params: VideoParams
  status: TaskStatus
  progress: number
  script?: string
  audio_url?: string
  subtitle_url?: string
  video_url?: string
  created_at: string
  updated_at: string
  error_message?: string
}

export interface SubtitleCue {
  index: number
  start_ms: number
  end_ms: number
  text: string
}

export interface ApiResponse<T = unknown> {
  code: number
  msg: string
  data: T
}

export interface LlmProviderOption {
  label: string
  value: string
  defaultBaseUrl?: string
  defaultModel?: string
  needApiKey?: boolean
  needSecretKey?: boolean
  needAccountId?: boolean
  tip?: string
}

export interface VoiceOption {
  label: string
  value: string
  locale?: string
  gender?: string
}

export interface MusicInfo {
  name: string
  path: string
  size?: number
}

export interface AppConfig {
  app: {
    name: string
    version: string
    host: string
    port: number
    storage_path: string
    concurrent_tasks: number
  }
  llm: {
    provider: string
    model: string
    api_key: string
    base_url: string
    api_version?: string
  }
  tts: {
    provider: string
    voice_name: string
    azure_speech_key?: string
    azure_speech_region?: string
    siliconflow_key?: string
    elevenlabs_key?: string
    elevenlabs_model?: string
    mimo_key?: string
  }
  stock: {
    pexels_api_key: string
    pixabay_api_key: string
    coverr_api_key: string
  }
  ffmpeg: {
    path: string
    threads: number
  }
  whisper: {
    provider: string
    model: string
    endpoint?: string
  }
}
